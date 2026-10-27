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

use std::io;
use std::os::fd::{AsRawFd, RawFd};

use netlink_sys::protocols::NETLINK_CONNECTOR;
use netlink_sys::{Socket, SocketAddr};

use crate::sys::Pid;

const CN_IDX_PROC: u32 = 1;
const CN_VAL_PROC: u32 = 1;
const PROC_CN_MCAST_LISTEN: u32 = 1;
const PROC_EVENT_FORK: u32 = 0x0000_0001;
const PROC_EVENT_EXEC: u32 = 0x0000_0002;
const PROC_EVENT_EXIT: u32 = 0x8000_0000;

const NLMSG_ERROR: u16 = 2;
const NLMSG_DONE: u16 = 3;
const NLMSG_HEADER_SIZE: usize = 16;
const CN_MESSAGE_SIZE: usize = 20;
const PROC_EVENT_HEADER_SIZE: usize = 16;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProcEvent {
    Fork { parent: Pid, child: Pid },
    Exec { pid: Pid },
    Exit { pid: Pid },
}

pub struct Connector {
    socket: Socket,
    buffer: Vec<u8>,
}

impl Connector {
    pub fn open() -> io::Result<Self> {
        let mut socket = Socket::new(NETLINK_CONNECTOR)?;
        socket.bind(&SocketAddr::new(std::process::id(), CN_IDX_PROC))?;

        let connector = Self {
            socket,
            buffer: vec![0u8; 4096],
        };
        connector.listen()?;
        connector.socket.set_non_blocking(true)?;
        Ok(connector)
    }

    pub fn descriptor(&self) -> RawFd {
        self.socket.as_raw_fd()
    }

    fn listen(&self) -> io::Result<()> {
        let mut message = [0u8; NLMSG_HEADER_SIZE + CN_MESSAGE_SIZE + 4];
        let length = message.len() as u32;
        message[0..4].copy_from_slice(&length.to_ne_bytes());
        message[4..6].copy_from_slice(&NLMSG_DONE.to_ne_bytes());
        message[12..16].copy_from_slice(&std::process::id().to_ne_bytes());
        message[16..20].copy_from_slice(&CN_IDX_PROC.to_ne_bytes());
        message[20..24].copy_from_slice(&CN_VAL_PROC.to_ne_bytes());
        message[32..34].copy_from_slice(&4u16.to_ne_bytes());
        message[36..40].copy_from_slice(&PROC_CN_MCAST_LISTEN.to_ne_bytes());

        self.socket.send_to(&message, &SocketAddr::new(0, 0), 0)?;
        Ok(())
    }

    pub fn drain(&mut self, events: &mut Vec<ProcEvent>) -> io::Result<()> {
        loop {
            let mut buffer = &mut self.buffer[..];
            match self.socket.recv(&mut buffer, 0) {
                Ok(0) => return Ok(()),
                Ok(received) => self.decode(received, events)?,
                Err(error) if error.kind() == io::ErrorKind::WouldBlock => return Ok(()),
                Err(error) if error.kind() == io::ErrorKind::Interrupted => continue,
                Err(error) => return Err(error),
            }
        }
    }

    fn decode(&self, length: usize, events: &mut Vec<ProcEvent>) -> io::Result<()> {
        let length = length.min(self.buffer.len());
        let mut offset = 0usize;
        while offset + NLMSG_HEADER_SIZE <= length {
            let header = &self.buffer[offset..offset + NLMSG_HEADER_SIZE];
            let message_length = read_u32(header, 0) as usize;
            let message_type = read_u16(header, 4);
            if message_length < NLMSG_HEADER_SIZE || offset + message_length > length {
                break;
            }

            let payload = &self.buffer[offset + NLMSG_HEADER_SIZE..offset + message_length];
            if message_type == NLMSG_DONE {
                decode_payload(payload, events);
            } else if message_type == NLMSG_ERROR && payload.len() >= 4 {
                let error = read_u32(payload, 0) as i32;
                if error != 0 {
                    return Err(io::Error::from_raw_os_error(-error));
                }
            }
            offset += (message_length + 3) & !3;
        }
        Ok(())
    }
}

fn decode_payload(payload: &[u8], events: &mut Vec<ProcEvent>) {
    if payload.len() < CN_MESSAGE_SIZE + PROC_EVENT_HEADER_SIZE {
        return;
    }
    let body_length = read_u16(payload, 16) as usize;
    let body = &payload[CN_MESSAGE_SIZE..];
    if body_length < PROC_EVENT_HEADER_SIZE || body_length > body.len() {
        return;
    }

    let what = read_u32(body, 0);
    match what {
        PROC_EVENT_FORK if body.len() >= 32 => {
            let parent = read_pid(body, 16);
            let child = read_pid(body, 24);
            let child_tgid = read_pid(body, 28);
            if child == child_tgid && child_tgid > 1 {
                events.push(ProcEvent::Fork {
                    parent,
                    child: child_tgid,
                });
            }
        }
        PROC_EVENT_EXEC if body.len() >= 24 => {
            let pid = read_pid(body, 16);
            let tgid = read_pid(body, 20);
            if pid == tgid && tgid > 1 {
                events.push(ProcEvent::Exec { pid: tgid });
            }
        }
        PROC_EVENT_EXIT if body.len() >= 24 => {
            let pid = read_pid(body, 16);
            let tgid = read_pid(body, 20);
            if pid == tgid && tgid > 1 {
                events.push(ProcEvent::Exit { pid: tgid });
            }
        }
        _ => {}
    }
}

fn read_u16(bytes: &[u8], offset: usize) -> u16 {
    u16::from_ne_bytes([bytes[offset], bytes[offset + 1]])
}

fn read_u32(bytes: &[u8], offset: usize) -> u32 {
    u32::from_ne_bytes([
        bytes[offset],
        bytes[offset + 1],
        bytes[offset + 2],
        bytes[offset + 3],
    ])
}

fn read_pid(bytes: &[u8], offset: usize) -> Pid {
    read_u32(bytes, offset) as Pid
}
