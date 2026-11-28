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

use std::ffi::CString;
use std::io;
use std::os::fd::OwnedFd;
use std::os::unix::process::CommandExt;
use std::path::Path;
use std::process::{Command, Stdio};

use nix::sys::signal::kill as nix_kill;
use nix::unistd::{self, ForkResult, Pid as NixPid};

use super::signal;
use crate::handoff;
use crate::log::logw;

pub type Pid = libc::pid_t;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Forked {
    Parent(Pid),
    Child,
}

pub fn fork() -> io::Result<Forked> {
    // SAFETY: the daemon forks while it is still single-threaded, so the
    // child cannot inherit a half-locked allocator or another thread's state.
    match unsafe { unistd::fork() } {
        Ok(ForkResult::Parent { child }) => Ok(Forked::Parent(child.as_raw())),
        Ok(ForkResult::Child) => Ok(Forked::Child),
        Err(error) => Err(io::Error::from(error)),
    }
}

pub fn spawn_detached(executable: &Path, argument: &Path, state: Option<&[u8]>) -> io::Result<Pid> {
    let mut command = Command::new(executable);
    command
        .arg(argument)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .env_remove(handoff::ENV);
    unsafe {
        command.pre_exec(|| unistd::setsid().map(|_| ()).map_err(io::Error::from));
    }

    let sender = if let Some(payload) = state {
        let (sender, receiver) = handoff::channel(payload)?;
        command
            .env(handoff::ENV, "1")
            .stdin(Stdio::from(OwnedFd::from(receiver)));
        Some((sender, payload))
    } else {
        None
    };

    let child = command.spawn()?;
    drop(command);
    if let Some((sender, payload)) = sender
        && let Err(error) = handoff::send(sender, payload)
    {
        logw!("cannot send daemon state handoff: {error}");
    }
    Ok(child.id() as Pid)
}

pub fn exit_now(code: i32) -> ! {
    unsafe { libc::_exit(code) }
}

pub fn set_process_name(name: &str) {
    let Ok(name) = CString::new(name) else {
        return;
    };
    unsafe {
        libc::prctl(libc::PR_SET_NAME, name.as_ptr(), 0, 0, 0);
    }
}

pub fn kill(pid: Pid, signal_number: i32) -> io::Result<()> {
    let Some(signal) = signal::from_number(signal_number) else {
        return Err(io::Error::from(io::ErrorKind::InvalidInput));
    };
    nix_kill(NixPid::from_raw(pid), signal).map_err(io::Error::from)
}
