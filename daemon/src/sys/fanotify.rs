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

use std::ffi::CString;
use std::io;
use std::os::fd::RawFd;
use std::path::Path;

use super::Pid;

const FAN_CLASS_NOTIF: libc::c_uint = 0x0000_0000;
const FAN_CLOEXEC: libc::c_uint = 0x0000_0001;
const FAN_NONBLOCK: libc::c_uint = 0x0000_0002;
const FAN_MARK_ADD: libc::c_uint = 0x0000_0001;

pub const FAN_OPEN_EXEC: u64 = 0x0000_1000;

const METADATA_SIZE: usize = 24;
const EVENT_LEN_OFFSET: usize = 0;
const MASK_OFFSET: usize = 8;
const FD_OFFSET: usize = 16;
const PID_OFFSET: usize = 20;

pub struct Fanotify {
    fd: RawFd,
}

impl Fanotify {
    pub fn init() -> io::Result<Self> {
        let flags = FAN_CLASS_NOTIF | FAN_CLOEXEC | FAN_NONBLOCK;
        let event_flags = libc::O_RDONLY | libc::O_LARGEFILE;
        // SAFETY: syscall
        let fd =
            unsafe { libc::syscall(libc::SYS_fanotify_init, flags, event_flags as libc::c_uint) };
        if fd < 0 {
            return Err(io::Error::last_os_error());
        }
        Ok(Self { fd: fd as RawFd })
    }

    pub fn add_path_mark(&self, path: &Path, mask: u64) -> io::Result<()> {
        let path = path_cstring(path)?;
        // SAFETY: syscall
        let result = mark_syscall(self.fd, FAN_MARK_ADD, mask, &path);
        if result < 0 {
            return Err(io::Error::last_os_error());
        }
        Ok(())
    }

    pub fn read_event(&self, buffer: &mut [u8]) -> io::Result<usize> {
        loop {
            // SAFETY: read buffer
            let read = unsafe { libc::read(self.fd, buffer.as_mut_ptr().cast(), buffer.len()) };
            if read >= 0 {
                return Ok(read as usize);
            }
            let error = io::Error::last_os_error();
            match error.kind() {
                io::ErrorKind::Interrupted => continue,
                io::ErrorKind::WouldBlock => return Ok(0),
                _ => return Err(error),
            }
        }
    }

    pub fn descriptor(&self) -> RawFd {
        self.fd
    }
}

impl Drop for Fanotify {
    fn drop(&mut self) {
        super::fs::close(self.fd);
    }
}

#[cfg(target_pointer_width = "64")]
fn mark_syscall(fd: RawFd, flags: libc::c_uint, mask: u64, path: &CString) -> libc::c_long {
    // SAFETY: syscall
    unsafe {
        libc::syscall(
            libc::SYS_fanotify_mark,
            fd,
            flags,
            mask,
            libc::AT_FDCWD,
            path.as_ptr(),
        )
    }
}

#[cfg(target_pointer_width = "32")]
fn mark_syscall(fd: RawFd, flags: libc::c_uint, mask: u64, path: &CString) -> libc::c_long {
    // SAFETY: syscall
    unsafe {
        libc::syscall(
            libc::SYS_fanotify_mark,
            fd,
            flags,
            mask as u32,
            (mask >> 32) as u32,
            libc::AT_FDCWD,
            path.as_ptr(),
        )
    }
}

pub struct Events<'a> {
    remaining: &'a [u8],
    consumed: usize,
}

impl<'a> Events<'a> {
    pub fn new(buffer: &'a [u8]) -> Self {
        Self {
            remaining: buffer,
            consumed: 0,
        }
    }

    pub fn consumed(&self) -> usize {
        self.consumed
    }
}

impl Iterator for Events<'_> {
    type Item = (Pid, RawFd, u64);

    fn next(&mut self) -> Option<Self::Item> {
        if self.remaining.len() < METADATA_SIZE {
            return None;
        }
        let event_len = read_u32(self.remaining, EVENT_LEN_OFFSET) as usize;
        if event_len < METADATA_SIZE || event_len > self.remaining.len() {
            return None;
        }
        let mask = read_u64(self.remaining, MASK_OFFSET);
        let fd = read_i32(self.remaining, FD_OFFSET) as RawFd;
        let pid = read_i32(self.remaining, PID_OFFSET);
        self.remaining = &self.remaining[event_len..];
        self.consumed += event_len;
        Some((pid, fd, mask))
    }
}

fn path_cstring(path: &Path) -> io::Result<CString> {
    CString::new(path.as_os_str().as_encoded_bytes())
        .map_err(|_| io::Error::from(io::ErrorKind::InvalidInput))
}

fn read_u32(bytes: &[u8], offset: usize) -> u32 {
    u32::from_ne_bytes([
        bytes[offset],
        bytes[offset + 1],
        bytes[offset + 2],
        bytes[offset + 3],
    ])
}

fn read_i32(bytes: &[u8], offset: usize) -> i32 {
    read_u32(bytes, offset) as i32
}

fn read_u64(bytes: &[u8], offset: usize) -> u64 {
    u64::from_ne_bytes([
        bytes[offset],
        bytes[offset + 1],
        bytes[offset + 2],
        bytes[offset + 3],
        bytes[offset + 4],
        bytes[offset + 5],
        bytes[offset + 6],
        bytes[offset + 7],
    ])
}

#[cfg(test)]
mod tests {
    use super::*;

    fn record(event_len: u32, mask: u64, fd: i32, pid: i32) -> Vec<u8> {
        let mut bytes = vec![0u8; METADATA_SIZE];
        bytes[0..4].copy_from_slice(&event_len.to_ne_bytes());
        bytes[4] = 3;
        bytes[6..8].copy_from_slice(&(METADATA_SIZE as u16).to_ne_bytes());
        bytes[8..16].copy_from_slice(&mask.to_ne_bytes());
        bytes[16..20].copy_from_slice(&fd.to_ne_bytes());
        bytes[20..24].copy_from_slice(&pid.to_ne_bytes());
        bytes
    }

    #[test]
    fn walks_complete_records() {
        let mut buffer = record(24, FAN_OPEN_EXEC, 7, 42);
        buffer.extend_from_slice(&record(24, FAN_OPEN_EXEC, 8, 43));
        let mut events = Events::new(&buffer);
        assert_eq!(events.next(), Some((42, 7, FAN_OPEN_EXEC)));
        assert_eq!(events.next(), Some((43, 8, FAN_OPEN_EXEC)));
        assert_eq!(events.next(), None);
        assert_eq!(events.consumed(), 48);
    }

    #[test]
    fn stops_at_a_truncated_record() {
        let buffer = record(24, FAN_OPEN_EXEC, 7, 42);
        let mut events = Events::new(&buffer[..20]);
        assert_eq!(events.next(), None);
        assert_eq!(events.consumed(), 0);
    }

    #[test]
    fn rejects_a_record_longer_than_the_buffer() {
        let buffer = record(48, FAN_OPEN_EXEC, 7, 42);
        let mut events = Events::new(&buffer);
        assert_eq!(events.next(), None);
        assert_eq!(events.consumed(), 0);
    }
}
