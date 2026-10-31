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
use std::fs::{self, File};
use std::io::{self, Read};
use std::os::fd::{AsRawFd, RawFd};
use std::os::unix::fs::OpenOptionsExt;
use std::path::{Path, PathBuf};

use crate::kernel::{EventKind, KernelEvent};
use crate::log::{logi, logw};
use crate::sys::Pid;

const TRACEFS_ROOTS: [&str; 2] = ["/sys/kernel/tracing", "/sys/kernel/debug/tracing"];
const INSTANCE: &str = "znn";
const EVENTS: [&str; 2] = ["sched_process_exec", "sched_process_exit"];
const BUFFER_SIZE_KB: u32 = 64;
const READ_BUFFER_SIZE: usize = 8192;
const MAX_TAIL_SIZE: usize = 1 << 16;

pub struct TracefsTracker {
    pipe: File,
    instance: PathBuf,
    tail: Vec<u8>,
    reported_error: bool,
    reported_loss: bool,
}

impl fmt::Debug for TracefsTracker {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("TracefsTracker")
    }
}

impl TracefsTracker {
    pub fn start() -> Result<Self, String> {
        let mut errors = Vec::new();
        for root in TRACEFS_ROOTS {
            let root = Path::new(root);
            if !root.is_dir() {
                errors.push(format!("{} is absent", root.display()));
                continue;
            }
            match Self::open(root) {
                Ok(tracker) => return Ok(tracker),
                Err(error) => errors.push(format!("{}: {error}", root.display())),
            }
        }
        Err(errors.join("; "))
    }

    fn open(root: &Path) -> io::Result<Self> {
        let instance = root.join("instances").join(INSTANCE);
        if !instance.is_dir() {
            fs::create_dir_all(&instance)?;
        }

        let events = instance.join("events").join("sched");
        for event in EVENTS {
            if let Err(error) = fs::write(events.join(event).join("enable"), "1") {
                disable(&instance);
                return Err(error);
            }
        }
        let _ = fs::write(instance.join("buffer_size_kb"), BUFFER_SIZE_KB.to_string());

        let pipe = match fs::OpenOptions::new()
            .read(true)
            .custom_flags(libc::O_NONBLOCK)
            .open(instance.join("trace_pipe"))
        {
            Ok(pipe) => pipe,
            Err(error) => {
                disable(&instance);
                return Err(error);
            }
        };
        logi!(
            "kernel tracking: tracefs instance {} listening on sched exec/exit events",
            instance.display()
        );
        Ok(Self {
            pipe,
            instance,
            tail: Vec::new(),
            reported_error: false,
            reported_loss: false,
        })
    }

    pub fn instance_path(&self) -> &Path {
        &self.instance
    }

    pub fn descriptor(&self) -> RawFd {
        self.pipe.as_raw_fd()
    }

    pub fn drain(&mut self, events: &mut Vec<KernelEvent>) {
        let mut buffer = [0u8; READ_BUFFER_SIZE];
        loop {
            match self.pipe.read(&mut buffer) {
                Ok(0) => break,
                Ok(read) => {
                    self.tail.extend_from_slice(&buffer[..read]);
                    self.emit_lines(events);
                }
                Err(error) if error.kind() == io::ErrorKind::WouldBlock => break,
                Err(error) if error.kind() == io::ErrorKind::Interrupted => continue,
                Err(error) => {
                    if !self.reported_error {
                        self.reported_error = true;
                        logw!("kernel tracking: tracefs trace_pipe read failed: {error}");
                    }
                    break;
                }
            }
        }
    }

    fn emit_lines(&mut self, events: &mut Vec<KernelEvent>) {
        let mut line_start = 0;
        let mut lost = false;
        for (index, byte) in self.tail.iter().enumerate() {
            if *byte != b'\n' {
                continue;
            }
            let line = String::from_utf8_lossy(&self.tail[line_start..index]);
            if is_lost_events_line(&line) {
                lost = true;
            } else if let Some(event) = parse_line(&line) {
                events.push(event);
            }
            line_start = index + 1;
        }
        self.tail.drain(..line_start);
        if self.tail.len() > MAX_TAIL_SIZE {
            self.tail.clear();
        }
        if lost {
            self.report_loss();
        }
    }

    fn report_loss(&mut self) {
        if self.reported_loss {
            return;
        }
        self.reported_loss = true;
        logw!(
            "kernel tracking: the tracefs ring buffer dropped events; a target exec may not have been seen"
        );
    }
}

impl Drop for TracefsTracker {
    fn drop(&mut self) {
        disable(&self.instance);
    }
}

fn disable(instance: &Path) {
    let events = instance.join("events").join("sched");
    for event in EVENTS {
        let _ = fs::write(events.join(event).join("enable"), "0");
    }
}

fn parse_line(line: &str) -> Option<KernelEvent> {
    let (kind, fields) = match line.split_once("sched_process_exec:") {
        Some((_, fields)) => (EventKind::Exec, fields),
        None => {
            let (_, fields) = line.split_once("sched_process_exit:")?;
            (EventKind::Exit, fields)
        }
    };

    let pid = field(fields, "pid")?.parse::<Pid>().ok()?;
    let mut comm = [0u8; 16];
    if let Some(name) = field(fields, "comm").or_else(|| prefix_comm(line)) {
        let bytes = name.as_bytes();
        let length = bytes.len().min(comm.len());
        comm[..length].copy_from_slice(&bytes[..length]);
    }

    Some(KernelEvent {
        kind,
        pid,
        uid: 0,
        ppid: 0,
        comm,
        filename: field(fields, "filename").map(str::to_owned),
        pre_exec: false,
    })
}

fn field<'a>(fields: &'a str, key: &str) -> Option<&'a str> {
    fields.split_whitespace().find_map(|token| {
        let (name, value) = token.split_once('=')?;
        (name == key).then_some(value)
    })
}

fn prefix_comm(line: &str) -> Option<&str> {
    let (prefix, _) = line.trim_start().split_once(' ')?;
    let (comm, _) = prefix.rsplit_once('-')?;
    (!comm.is_empty()).then_some(comm)
}

fn is_lost_events_line(line: &str) -> bool {
    line.contains("[LOST ") && line.contains(" EVENTS]")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_an_exec_line() {
        let line = "sh-1234  [002] ....  4567.890123: sched_process_exec: filename=/system/bin/app_process64 pid=1234 old_pid=1234";
        let event = parse_line(line).expect("exec line parses");
        assert_eq!(event.kind, EventKind::Exec);
        assert_eq!(event.pid, 1234);
        assert_eq!(event.filename.as_deref(), Some("/system/bin/app_process64"));
        assert_eq!(&event.comm[..2], b"sh");
    }

    #[test]
    fn parses_an_exit_line() {
        let line = "          <idle>-0     [000] d.h.  4567.890123: sched_process_exit: comm=swapper/0 pid=42 prio=120";
        let event = parse_line(line).expect("exit line parses");
        assert_eq!(event.kind, EventKind::Exit);
        assert_eq!(event.pid, 42);
        assert_eq!(event.filename, None);
        assert_eq!(&event.comm[..7], b"swapper");
    }

    #[test]
    fn ignores_other_lines() {
        assert!(parse_line("sh-1234 [002] .... 4567.890123: sched_switch: prev_comm=sh").is_none());
        assert!(parse_line("").is_none());
    }

    #[test]
    fn recognizes_lost_event_markers() {
        assert!(is_lost_events_line("CPU:1 [LOST 8 EVENTS]"));
        assert!(is_lost_events_line("CPU:0 [LOST 123 EVENTS]"));
        assert!(!is_lost_events_line(
            "sh-1234 [002] .... 4567.890123: sched_process_exec: pid=1234"
        ));
    }
}
