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

use std::fmt;
use std::fs;
use std::os::fd::RawFd;
use std::path::Path;

use crate::kernel::{EventKind, KernelEvent};
use crate::log::{logi, logw};
use crate::sys::fanotify::{Events, FAN_OPEN_EXEC, Fanotify};
use crate::sys::fs as raw_fs;
use crate::targets::Target;

const READ_BUFFER_SIZE: usize = 8192;
const MAX_PENDING: usize = 1 << 16;

pub struct FanotifyTracker {
    fanotify: Fanotify,
    pending: Vec<u8>,
    reported_error: bool,
}

impl fmt::Debug for FanotifyTracker {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("FanotifyTracker")
    }
}

impl FanotifyTracker {
    pub fn start(targets: &[Target]) -> Result<Self, String> {
        let fanotify =
            Fanotify::init().map_err(|error| format!("fanotify_init failed: {error}"))?;

        let mut marked = 0usize;
        for target in targets {
            if target.by_name {
                continue;
            }
            match fanotify.add_path_mark(Path::new(&target.value), FAN_OPEN_EXEC) {
                Ok(()) => marked += 1,
                Err(error) => logw!(
                    "kernel tracking: cannot mark {} for fanotify exec events: {error}",
                    target.value
                ),
            }
        }
        if marked == 0 {
            return Err("no fanotify path targets".to_owned());
        }
        logi!("kernel tracking: fanotify exec source ready on {marked} path target(s)");
        Ok(Self {
            fanotify,
            pending: Vec::new(),
            reported_error: false,
        })
    }

    pub fn descriptor(&self) -> RawFd {
        self.fanotify.descriptor()
    }

    pub fn drain(&mut self, events: &mut Vec<KernelEvent>) {
        let mut buffer = [0u8; READ_BUFFER_SIZE];
        loop {
            match self.fanotify.read_event(&mut buffer) {
                Ok(0) => break,
                Ok(read) => {
                    self.pending.extend_from_slice(&buffer[..read]);
                    let consumed = emit_records(&self.pending, events);
                    self.pending.drain(..consumed);
                }
                Err(error) => {
                    if !self.reported_error {
                        self.reported_error = true;
                        logw!("kernel tracking: fanotify event read failed: {error}");
                    }
                    break;
                }
            }
        }
        if self.pending.len() > MAX_PENDING {
            self.pending.clear();
        }
    }
}

fn emit_records(pending: &[u8], events: &mut Vec<KernelEvent>) -> usize {
    let mut records = Events::new(pending);
    for (pid, fd, _mask) in &mut records {
        if pid > 0 {
            let filename = fs::read_link(format!("/proc/self/fd/{fd}"))
                .ok()
                .map(|path| path.to_string_lossy().into_owned());
            events.push(KernelEvent {
                kind: EventKind::Exec,
                pid,
                uid: 0,
                ppid: 0,
                comm: [0; 16],
                filename,
                pre_exec: true,
            });
        }
        if fd >= 0 {
            raw_fs::close(fd);
        }
    }
    records.consumed()
}
