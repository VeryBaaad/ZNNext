/*
 * SPDX-License-Identifier: GPL-3.0-or-later AND LicenseRef-DCLPL-2.0
 *
 * This file is part of ZNNext.
 *
 * ZNNext is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * ZNNext is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with ZNNext. If not, see <https://www.gnu.org/licenses/>.
 *
 * Copyright (C) 2026 VeryBaaad <verybaaad@outlook.com>
 */

use std::io::{self, Read};
use std::os::fd::{AsFd, AsRawFd, OwnedFd};
use std::os::unix::net::{UnixListener, UnixStream};
use std::rc::Rc;
use std::time::{Duration, Instant};

use nix::errno::Errno;
use nix::poll::{PollFd, PollFlags, PollTimeout, poll};
use nix::sys::socket::{
    self, AddressFamily, Backlog, MsgFlags, SockFlag, SockType, UnixAddr, sockopt,
};

use crate::sys::Pid;

const TIMEOUT: Duration = Duration::from_secs(2);
const MAX_CLIENTS: usize = 16;
const ACCEPT_BUDGET: usize = 8;
pub(crate) const MAX_REPORT_BYTES: usize = 8 * 1024 * 1024;
const WRITE_BUDGET: usize = 256 * 1024;
const HEADER_BYTES: usize = size_of::<u32>();
const SOCKET_FLAGS: SockFlag = SockFlag::SOCK_NONBLOCK.union(SockFlag::SOCK_CLOEXEC);

#[derive(Debug)]
pub struct Server {
    listener: UnixListener,
    clients: Vec<Client>,
}

#[derive(Debug)]
struct Client {
    stream: UnixStream,
    report: Rc<Vec<u8>>,
    sent: usize,
    deadline: Instant,
}

impl Server {
    pub fn bind() -> io::Result<Self> {
        let listener = new_socket()?;
        let address = address(std::process::id() as Pid)?;
        socket::bind(listener.as_raw_fd(), &address)?;
        socket::listen(&listener, Backlog::new(MAX_CLIENTS as i32)?)?;
        Ok(Self {
            listener: UnixListener::from(listener),
            clients: Vec::new(),
        })
    }

    pub fn poll(&mut self, snapshot: impl FnOnce() -> io::Result<Vec<u8>>) -> io::Result<()> {
        let now = Instant::now();
        self.clients
            .retain_mut(|client| now < client.deadline && client.write());

        let mut accepted = Vec::new();
        for _ in 0..ACCEPT_BUDGET {
            let stream = match self.listener.accept() {
                Ok((stream, _)) => stream,
                Err(error) if error.kind() == io::ErrorKind::Interrupted => continue,
                Err(error) if error.kind() == io::ErrorKind::WouldBlock => break,
                Err(error) => return Err(error),
            };
            if self.clients.len() + accepted.len() >= MAX_CLIENTS
                || check_peer(&stream, None).is_err()
                || stream.set_nonblocking(true).is_err()
            {
                continue;
            }
            accepted.push(stream);
        }
        if accepted.is_empty() {
            return Ok(());
        }

        let report = Rc::new(encode_report(snapshot()?)?);
        self.clients
            .extend(accepted.into_iter().filter_map(|stream| {
                let mut client = Client {
                    stream,
                    report: Rc::clone(&report),
                    sent: 0,
                    deadline: now + TIMEOUT,
                };
                (Instant::now() < client.deadline && client.write()).then_some(client)
            }));
        Ok(())
    }
}

impl Client {
    fn write(&mut self) -> bool {
        let end = (self.sent + WRITE_BUDGET).min(self.report.len());
        let chunk = &self.report[self.sent..end];
        match socket::send(self.stream.as_raw_fd(), chunk, MsgFlags::MSG_NOSIGNAL) {
            Ok(0) => false,
            Ok(written) => {
                self.sent += written;
                self.sent < self.report.len()
            }
            Err(Errno::EAGAIN | Errno::EINTR) => true,
            Err(_) => false,
        }
    }
}

pub fn read(pid: Pid) -> io::Result<Vec<u8>> {
    if pid <= 0 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "invalid daemon PID",
        ));
    }

    let deadline = Instant::now() + TIMEOUT;
    let fd = new_socket()?;
    let address = address(pid)?;
    let connected = socket::connect(fd.as_raw_fd(), &address);
    let mut stream = UnixStream::from(fd);
    match connected {
        Ok(()) => (),
        Err(Errno::EINPROGRESS | Errno::EINTR) => {
            wait(&stream, PollFlags::POLLOUT, deadline)?;
            if let Some(error) = stream.take_error()? {
                return Err(error);
            }
        }
        Err(error) => return Err(error.into()),
    }
    check_peer(&stream, Some(pid))?;
    read_report(&mut stream, deadline)
}

pub(crate) fn read_report(stream: &mut UnixStream, deadline: Instant) -> io::Result<Vec<u8>> {
    let mut header = [0; HEADER_BYTES];
    read_exact(stream, &mut header, deadline)?;
    let length = u32::from_be_bytes(header) as usize;
    if length > MAX_REPORT_BYTES {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "state report exceeds size limit",
        ));
    }
    let mut report = vec![0; length];
    read_exact(stream, &mut report, deadline)?;
    Ok(report)
}

fn encode_report(payload: Vec<u8>) -> io::Result<Vec<u8>> {
    if payload.len() > MAX_REPORT_BYTES {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "state report exceeds size limit",
        ));
    }
    let mut report = Vec::with_capacity(HEADER_BYTES + payload.len());
    report.extend_from_slice(&(payload.len() as u32).to_be_bytes());
    report.extend(payload);
    Ok(report)
}

fn read_exact(stream: &mut UnixStream, mut buffer: &mut [u8], deadline: Instant) -> io::Result<()> {
    while !buffer.is_empty() {
        remaining(deadline)?;
        match stream.read(buffer) {
            Ok(0) => return Err(io::ErrorKind::UnexpectedEof.into()),
            Ok(length) => buffer = &mut buffer[length..],
            Err(error) if error.kind() == io::ErrorKind::Interrupted => continue,
            Err(error) if error.kind() == io::ErrorKind::WouldBlock => {
                wait(stream, PollFlags::POLLIN, deadline)?;
            }
            Err(error) => return Err(error),
        }
    }
    Ok(())
}

pub(crate) fn wait(stream: &UnixStream, events: PollFlags, deadline: Instant) -> io::Result<()> {
    let mut descriptors = [PollFd::new(stream.as_fd(), events)];
    loop {
        let timeout = remaining(deadline)?.max(Duration::from_millis(1));
        let timeout = PollTimeout::try_from(timeout).unwrap_or(PollTimeout::MAX);
        match poll(&mut descriptors, timeout) {
            Ok(0) | Err(Errno::EINTR) => continue,
            Ok(_) => return Ok(()),
            Err(error) => return Err(error.into()),
        }
    }
}

pub(crate) fn remaining(deadline: Instant) -> io::Result<Duration> {
    let now = Instant::now();
    if now >= deadline {
        return Err(io::ErrorKind::TimedOut.into());
    }
    Ok(deadline - now)
}

fn check_peer(stream: &UnixStream, pid: Option<Pid>) -> io::Result<()> {
    let peer = socket::getsockopt(stream, sockopt::PeerCredentials)?;
    if peer.uid() != 0 || pid.is_some_and(|pid| peer.pid() != pid) {
        return Err(io::ErrorKind::PermissionDenied.into());
    }
    Ok(())
}

fn new_socket() -> io::Result<OwnedFd> {
    Ok(socket::socket(
        AddressFamily::Unix,
        SockType::Stream,
        SOCKET_FLAGS,
        None,
    )?)
}

fn address(pid: Pid) -> io::Result<UnixAddr> {
    Ok(UnixAddr::new_abstract(
        format!("znnext-state.v1.{pid}").as_bytes(),
    )?)
}

#[cfg(test)]
mod tests {
    use std::io::Write;
    use std::sync::mpsc;
    use std::thread;

    use super::*;

    #[test]
    fn handles_partial_writes() {
        let (writer, mut reader) = UnixStream::pair().unwrap();
        writer.set_nonblocking(true).unwrap();
        reader.set_nonblocking(true).unwrap();
        let payload = vec![b'x'; WRITE_BUDGET * 3];
        let deadline = Instant::now() + TIMEOUT;
        let reader = thread::spawn(move || read_report(&mut reader, deadline).unwrap());
        let mut client = Client {
            stream: writer,
            report: Rc::new(encode_report(payload.clone()).unwrap()),
            sent: 0,
            deadline,
        };
        assert!(client.write());
        while client.write() {
            assert!(Instant::now() < deadline);
            thread::yield_now();
        }
        assert_eq!(reader.join().unwrap(), payload);
        assert_eq!(client.sent, client.report.len());
    }

    #[test]
    fn rejects_oversized_and_truncated_frames() {
        let error = encode_report(vec![0; MAX_REPORT_BYTES + 1]).unwrap_err();
        assert_eq!(error.kind(), io::ErrorKind::InvalidData);
        for (length, expected) in [
            ((MAX_REPORT_BYTES + 1) as u32, io::ErrorKind::InvalidData),
            (1, io::ErrorKind::UnexpectedEof),
        ] {
            let (mut writer, mut reader) = UnixStream::pair().unwrap();
            reader.set_nonblocking(true).unwrap();
            writer.write_all(&length.to_be_bytes()).unwrap();
            drop(writer);
            let error = read_report(&mut reader, Instant::now() + TIMEOUT).unwrap_err();
            assert_eq!(error.kind(), expected);
        }
    }

    #[test]
    fn times_out_a_stalled_peer() {
        let (_writer, mut reader) = UnixStream::pair().unwrap();
        reader.set_nonblocking(true).unwrap();
        let deadline = Instant::now() + Duration::from_millis(10);
        let error = read_report(&mut reader, deadline).unwrap_err();
        assert_eq!(error.kind(), io::ErrorKind::TimedOut);
    }

    #[test]
    fn authenticates_the_live_connection() {
        let pid = std::process::id() as Pid;
        let (stream, _peer) = UnixStream::pair().unwrap();
        let peer = socket::getsockopt(&stream, sockopt::PeerCredentials).unwrap();
        assert_eq!(
            check_peer(&stream, Some(peer.pid() + 1))
                .unwrap_err()
                .kind(),
            io::ErrorKind::PermissionDenied
        );

        let (ready, listening) = mpsc::sync_channel(1);
        let (done, stop) = mpsc::sync_channel::<()>(1);
        let server = thread::spawn(move || {
            let mut server = Server::bind().unwrap();
            server.poll(|| panic!("rendered without clients")).unwrap();
            ready.send(()).unwrap();
            let mut byte = 42;
            while let Err(mpsc::TryRecvError::Empty) = stop.try_recv() {
                server
                    .poll(|| {
                        let payload = vec![byte];
                        byte += 1;
                        Ok(payload)
                    })
                    .unwrap();
                thread::sleep(Duration::from_millis(1));
            }
            byte
        });
        listening.recv().unwrap();

        let result = read(pid);
        if peer.uid() == 0 {
            assert_eq!(result.unwrap(), vec![42]);
            assert_eq!(read(pid).unwrap(), vec![43]);
        } else {
            assert_eq!(result.unwrap_err().kind(), io::ErrorKind::PermissionDenied);
            let fd = new_socket().unwrap();
            socket::connect(fd.as_raw_fd(), &address(pid).unwrap()).unwrap();
            let mut stream = UnixStream::from(fd);
            let error = read_report(&mut stream, Instant::now() + TIMEOUT).unwrap_err();
            assert_eq!(error.kind(), io::ErrorKind::UnexpectedEof);
        }

        drop(done);
        assert_eq!(
            server.join().unwrap(),
            if peer.uid() == 0 { 44 } else { 42 }
        );
    }
}
