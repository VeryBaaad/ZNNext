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

use std::env;
use std::fs::File;
use std::io::{self, Write};
use std::os::unix::net::UnixStream;
use std::time::{Duration, Instant};

use nix::{poll::PollFlags, unistd};

use crate::state_socket::{MAX_REPORT_BYTES, read_report, remaining, wait};

pub(crate) const ENV: &str = "ZNN_DAEMON_HANDOFF_V1";
const TRANSFER_TIMEOUT: Duration = Duration::from_millis(500);

pub(crate) fn channel(payload: &[u8]) -> io::Result<(UnixStream, UnixStream)> {
    if payload.len() > MAX_REPORT_BYTES {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "daemon state exceeds the handoff limit",
        ));
    }
    UnixStream::pair()
}

pub(crate) fn send(mut socket: UnixStream, payload: &[u8]) -> io::Result<()> {
    socket.set_nonblocking(true)?;
    let deadline = Instant::now() + TRANSFER_TIMEOUT;
    let length = (payload.len() as u32).to_be_bytes();
    for mut bytes in [length.as_slice(), payload] {
        while !bytes.is_empty() {
            remaining(deadline)?;
            match socket.write(bytes) {
                Ok(0) => return Err(io::Error::from(io::ErrorKind::WriteZero)),
                Ok(written) => bytes = &bytes[written..],
                Err(error) if error.kind() == io::ErrorKind::Interrupted => continue,
                Err(error) if error.kind() == io::ErrorKind::WouldBlock => {
                    wait(&socket, PollFlags::POLLOUT, deadline)?;
                }
                Err(error) => return Err(error),
            }
        }
    }
    Ok(())
}

pub(crate) fn receive() -> io::Result<Option<Vec<u8>>> {
    if env::var_os(ENV).as_deref() != Some("1".as_ref()) {
        return Ok(None);
    }
    let descriptor = unistd::dup(io::stdin());
    unistd::dup2_stdin(File::open("/dev/null")?)?;
    let mut stream = UnixStream::from(descriptor?);
    stream.set_nonblocking(true)?;
    read_report(&mut stream, Instant::now() + TRANSFER_TIMEOUT).map(Some)
}
