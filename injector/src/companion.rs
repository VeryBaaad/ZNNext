/*
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

use std::io::{Read, Write};
#[cfg(target_os = "android")]
use std::os::android::net::SocketAddrExt;
use std::os::fd::{AsFd, AsRawFd, IntoRawFd, RawFd};
#[cfg(not(target_os = "android"))]
use std::os::linux::net::SocketAddrExt;
use std::os::unix::net::{SocketAddr, UnixListener, UnixStream};
use std::path::Path;
use std::time::{Duration, Instant};

use nix::poll::{PollFd, PollFlags, poll};

use crate::log::{loge, logi};
use crate::plan;
use crate::procfs;
use crate::sys;
use crate::sys::process::Forked;

const CONNECT: u8 = 1;
const CONNECT_TIMEOUT: Duration = Duration::from_secs(5);
const RETRY_INTERVAL: Duration = Duration::from_millis(10);
const PROOF_TIMEOUT: Duration = Duration::from_secs(2);

pub fn spawn(library: &str, nonce: u32, index: u32, token: [u8; 16]) {
    match sys::process::fork() {
        Ok(Forked::Child) => run_helper(library, nonce, index, token),
        Ok(Forked::Parent(pid)) => {
            logi!("companion for {library} (module {index}) started as pid {pid}");
        }
        Err(error) => loge!("companion for {library}: cannot fork: {error}"),
    }
}

fn connect_channel(nonce: u32, index: u32) -> Option<UnixStream> {
    let name = plan::channel_name(nonce, index);
    let address = SocketAddr::from_abstract_name(name.as_bytes()).ok()?;
    let deadline = Instant::now() + CONNECT_TIMEOUT;

    loop {
        match UnixStream::connect_addr(&address) {
            Ok(stream) => return Some(stream),
            Err(error) => {
                if Instant::now() >= deadline {
                    loge!("companion: cannot reach {name}: {error}");
                    return None;
                }
                std::thread::sleep(RETRY_INTERVAL);
            }
        }
    }
}

fn bind_direct(nonce: u32, index: u32) -> Option<UnixListener> {
    let name = plan::direct_channel_name(nonce, index);
    let address = SocketAddr::from_abstract_name(name.as_bytes()).ok()?;
    let listener = match UnixListener::bind_addr(&address) {
        Ok(listener) => listener,
        Err(error) => {
            loge!("companion: cannot listen on {name}: {error}");
            return None;
        }
    };
    if let Err(error) = listener.set_nonblocking(true) {
        loge!("companion: cannot make {name} nonblocking: {error}");
        return None;
    }
    Some(listener)
}

fn accept_direct(listener: &UnixListener, token: &[u8; 16], companion: &sys::dlfcn::Companion<'_>) {
    loop {
        let mut stream = match listener.accept() {
            Ok((stream, _)) => stream,
            Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => return,
            Err(error) => {
                loge!("companion: direct accept failed: {error}");
                return;
            }
        };
        let _ = stream.set_nonblocking(false);
        let _ = stream.set_read_timeout(Some(PROOF_TIMEOUT));
        let mut proof = [0u8; 16];
        if stream.read_exact(&mut proof).is_err() {
            loge!("companion: a direct connection did not prove itself");
            continue;
        }
        if &proof != token {
            loge!("companion: rejected a direct connection with a bad proof");
            continue;
        }
        let _ = stream.set_read_timeout(None);
        logi!("companion: serving a direct connection");
        companion.notify_connected(stream.into_raw_fd());
    }
}

fn memfd_from_file(path: &str) -> Option<RawFd> {
    let contents = procfs::read_file(Path::new(path))?;
    let descriptor = sys::memfd::memfd_create("znn-companion", sys::memfd::MFD_CLOEXEC).ok()?;
    if sys::fs::write_all(descriptor, &contents).is_err() {
        sys::fs::close(descriptor);
        return None;
    }
    Some(descriptor)
}

fn run_helper(library: &str, nonce: u32, index: u32, token: [u8; 16]) -> ! {
    sys::process::set_process_name("znn-companion");
    sys::signal::reset_to_default(sys::signal::SIGHUP);
    sys::signal::reset_to_default(sys::signal::SIGTERM);

    let module = sys::dlfcn::Library::open(library).or_else(|| {
        let descriptor = memfd_from_file(library)?;
        sys::dlfcn::Library::open_from_fd(library, descriptor)
    });
    let Some(module) = module else {
        loge!(
            "companion: dlopen {library} failed: {}",
            sys::dlfcn::last_error()
        );
        sys::process::exit_now(1);
    };
    let Some(companion) = module.companion() else {
        loge!("companion: {library} does not export zn_companion_module");
        sys::process::exit_now(1);
    };

    logi!("companion: serving {library}");
    let direct = bind_direct(nonce, index);
    companion.notify_loaded();

    let mut channel = connect_channel(nonce, index);
    let nonce_sent = channel
        .as_mut()
        .is_some_and(|stream| stream.write_all(&nonce.to_ne_bytes()).is_ok());
    if !nonce_sent {
        if channel.is_some() {
            loge!("companion: cannot send the channel nonce");
            channel = None;
        } else if direct.is_none() {
            sys::process::exit_now(1);
        }
    }

    let mut command = [0u8; 1];
    loop {
        let mut descriptors = Vec::with_capacity(2);
        if let Some(stream) = channel.as_ref() {
            descriptors.push(PollFd::new(stream.as_fd(), PollFlags::POLLIN));
        }
        if let Some(listener) = direct.as_ref() {
            descriptors.push(PollFd::new(listener.as_fd(), PollFlags::POLLIN));
        }
        match poll(&mut descriptors, 1000u16) {
            Ok(_) => {}
            Err(nix::errno::Errno::EINTR) => continue,
            Err(error) => {
                loge!("companion: poll failed: {error}");
                sys::process::exit_now(1);
            }
        }

        let control_ready = channel.is_some()
            && descriptors
                .first()
                .is_some_and(|fd| fd.revents().is_some_and(|r| r.contains(PollFlags::POLLIN)));
        let direct_ready = direct.is_some()
            && descriptors
                .last()
                .is_some_and(|fd| fd.revents().is_some_and(|r| r.contains(PollFlags::POLLIN)));

        if control_ready {
            match sys::socket::recv_with_fd(channel.as_ref().unwrap().as_raw_fd(), &mut command) {
                Ok((0, descriptor)) => {
                    if let Some(descriptor) = descriptor {
                        sys::fs::close(descriptor);
                    }
                    logi!("companion: the control channel closed, serving direct connections");
                    channel = None;
                }
                Ok((_, descriptor)) => {
                    if command[0] == CONNECT {
                        if let Some(descriptor) = descriptor {
                            companion.notify_connected(descriptor);
                        }
                    } else if let Some(descriptor) = descriptor {
                        sys::fs::close(descriptor);
                    }
                }
                Err(error) => {
                    loge!("companion: control channel read failed: {error}");
                    channel = None;
                }
            }
        }

        if direct_ready && let Some(listener) = direct.as_ref() {
            accept_direct(listener, &token, &companion);
        }

        if channel.is_none() && direct.is_none() {
            break;
        }
    }
    sys::process::exit_now(0)
}
