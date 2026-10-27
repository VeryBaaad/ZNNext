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

use crate::daemon::Daemon;
use crate::kernel_bpf::BpfTracker;
use crate::kernel_connector::ConnectorTracker;
use crate::kernel_fanotify::FanotifyTracker;
use crate::kernel_kpm::KpmTracker;
use crate::kernel_tracefs::TracefsTracker;
use crate::log::{logi, logw};
use crate::procfs;
use crate::sys::Pid;
use crate::sys::poll;
use crate::targets::Target;

pub const POLL_TIMEOUT_MS: i32 = 20;
const RECHECK_TIMEOUT_MS: u64 = 150;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventKind {
    Exec,
    Fork,
    Exit,
}

impl EventKind {
    #[cfg(target_pointer_width = "64")]
    pub fn from_raw(raw: u32) -> Option<Self> {
        match raw {
            0 => Some(EventKind::Exec),
            1 => Some(EventKind::Fork),
            2 => Some(EventKind::Exit),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            EventKind::Exec => "exec",
            EventKind::Fork => "fork",
            EventKind::Exit => "exit",
        }
    }
}

#[derive(Debug, Clone)]
pub struct KernelEvent {
    pub kind: EventKind,
    pub pid: Pid,
    pub uid: u32,
    pub ppid: Pid,
    pub comm: [u8; 16],
    pub filename: Option<String>,
    pub pre_exec: bool,
}

#[derive(Debug)]
pub struct KernelTracker {
    tracefs: Option<TracefsTracker>,
    connector: Option<ConnectorTracker>,
    bpf: Option<BpfTracker>,
    fanotify: Option<FanotifyTracker>,
    kpm: Option<KpmTracker>,
    bpf_pending: bool,
}

impl KernelTracker {
    pub fn start(targets: &[Target]) -> Option<Self> {
        let tracefs = match TracefsTracker::start() {
            Ok(tracker) => {
                logi!(
                    "kernel tracking: tracefs source ready at {} (sched exec/exit events)",
                    tracker.instance_path().display()
                );
                Some(tracker)
            }
            Err(error) => {
                logw!("kernel tracking: tracefs source unavailable: {error}");
                None
            }
        };
        let connector = match ConnectorTracker::start() {
            Ok(tracker) => {
                logi!("kernel tracking: proc connector source ready (fork/exec/exit)");
                Some(tracker)
            }
            Err(error) => {
                logw!("kernel tracking: proc connector unavailable: {error}");
                None
            }
        };
        let booted =
            crate::sys::prop::system_property("sys.boot_completed").as_deref() == Some("1");
        let (bpf, bpf_pending) = if booted {
            (start_bpf(), false)
        } else {
            logi!("kernel tracking: eBPF source held back until boot completes");
            (None, true)
        };
        let fanotify = match FanotifyTracker::start(targets) {
            Ok(tracker) => Some(tracker),
            Err(error) => {
                logw!("kernel tracking: fanotify source unavailable: {error}");
                None
            }
        };
        let kpm = match KpmTracker::start() {
            Ok(tracker) => Some(tracker),
            Err(error) => {
                logw!("kernel tracking: KernelPatch znn source unavailable: {error}");
                None
            }
        };

        if tracefs.is_none()
            && connector.is_none()
            && bpf.is_none()
            && fanotify.is_none()
            && kpm.is_none()
        {
            return None;
        }
        Some(Self {
            tracefs,
            connector,
            bpf,
            fanotify,
            kpm,
            bpf_pending,
        })
    }

    pub fn upgrade(&mut self) {
        if !self.bpf_pending {
            return;
        }
        if crate::sys::prop::system_property("sys.boot_completed").as_deref() != Some("1") {
            return;
        }
        if let Some(bpf) = start_bpf() {
            self.bpf = Some(bpf);
            self.bpf_pending = false;
        }
    }

    pub fn description(&self) -> String {
        let mut sources = Vec::new();
        if self.tracefs.is_some() {
            sources.push("tracefs sched exec/exit events");
        }
        if self.connector.is_some() {
            sources.push("proc connector fork/exec/exit events");
        }
        if self.bpf.is_some() {
            sources.push("eBPF exec events");
        } else if self.bpf_pending {
            sources.push("eBPF source pending boot completion");
        }
        if self.fanotify.is_some() {
            sources.push("fanotify exec events");
        }
        if self.kpm.is_some() {
            sources.push("KernelPatch znn exec records");
        }
        if sources.is_empty() {
            return "no kernel event source".to_owned();
        }
        sources.join(" and ")
    }

    pub fn poll(&mut self, events: &mut Vec<KernelEvent>) {
        let mut descriptors = Vec::new();
        if let Some(tracefs) = &self.tracefs {
            descriptors.push(tracefs.descriptor());
        }
        if let Some(bpf) = &self.bpf {
            descriptors.push(bpf.descriptor());
        }
        if let Some(connector) = &self.connector {
            descriptors.push(connector.descriptor());
        }
        if let Some(fanotify) = &self.fanotify {
            descriptors.push(fanotify.descriptor());
        }
        let _ = poll::poll_readable(&descriptors, POLL_TIMEOUT_MS);

        if let Some(tracefs) = &mut self.tracefs {
            tracefs.drain(events);
        }
        if let Some(bpf) = &mut self.bpf {
            bpf.drain(events);
        }
        if let Some(connector) = &mut self.connector {
            connector.drain(events);
        }
        if let Some(fanotify) = &mut self.fanotify {
            fanotify.drain(events);
        }
        if let Some(kpm) = &mut self.kpm {
            kpm.drain(events);
        }
    }
}

fn start_bpf() -> Option<BpfTracker> {
    match BpfTracker::start() {
        Ok(tracker) => {
            logi!("kernel tracking: eBPF exec source ready (raw tracepoint sched_process_exec)");
            Some(tracker)
        }
        Err(error) => {
            logw!("kernel tracking: eBPF source unavailable: {error}");
            None
        }
    }
}

impl Daemon {
    pub fn start_kernel_tracker(&mut self) -> bool {
        if self.kernel.is_some() {
            return true;
        }
        let Some(tracker) = KernelTracker::start(&self.targets) else {
            return false;
        };
        logi!("tracking mode kernel: {}", tracker.description());
        self.kernel = Some(tracker);
        true
    }

    pub fn stop_kernel_tracker(&mut self) {
        if self.kernel.take().is_some() {
            logi!("kernel tracking: stopped");
        }
    }

    pub fn poll_kernel(&mut self) {
        let mut events = Vec::new();
        if let Some(tracker) = self.kernel.as_mut() {
            tracker.poll(&mut events);
        }
        for event in events {
            self.consider_kernel_event(event);
        }
    }

    fn consider_kernel_event(&mut self, event: KernelEvent) {
        let pid = event.pid;
        if pid <= 1 {
            return;
        }

        match event.kind {
            EventKind::Fork => return,
            EventKind::Exit => {
                self.candidates.remove(&pid);
                self.done.remove(&pid);
                self.ignored.remove(&pid);
                self.deferred.remove(&pid);
                self.recheck.remove(&pid);
                return;
            }
            EventKind::Exec => {}
        }

        if self.tracees.contains_key(&pid)
            || self.done.contains(&pid)
            || self.ignored.contains(&pid)
        {
            return;
        }

        if event.pre_exec {
            if procfs::foreign_tracer(pid).is_none() {
                self.defer_recheck(pid, event.filename.as_deref());
            }
            return;
        }

        let exe = procfs::read_exe_path(pid);
        if exe.is_empty() {
            if procfs::process_gone(pid) {
                self.ignored.insert(pid);
            }
            return;
        }
        let exec_path = event.filename.as_deref().filter(|name| !name.is_empty());
        let is_target = crate::targets::matches(&self.targets, &exe)
            || exec_path.is_some_and(|name| crate::targets::matches(&self.targets, name));
        if !is_target {
            self.ignored.insert(pid);
            return;
        }

        let comm = String::from_utf8_lossy(&event.comm);
        let exec_path_note = exec_path
            .map(|name| format!(" (exec path {name})"))
            .unwrap_or_default();
        logi!(
            "{exe} (pid {pid}, uid {}, ppid {}, comm {}) appeared via a {} event{exec_path_note}",
            event.uid,
            event.ppid,
            comm.trim_end_matches('\0'),
            event.kind.as_str()
        );
        self.observe_target(pid, &exe);
    }

    fn defer_recheck(&mut self, pid: Pid, exec_path: Option<&str>) {
        let exec_path = exec_path.filter(|name| !name.is_empty());
        if let Some(path) = exec_path
            && !crate::targets::matches(&self.targets, path)
        {
            self.ignored.insert(pid);
            return;
        }
        if self.recheck.contains_key(&pid) {
            return;
        }
        self.recheck
            .insert(pid, procfs::now_ms() + RECHECK_TIMEOUT_MS);
        let note = exec_path
            .map(|name| format!(" (exec path {name})"))
            .unwrap_or_default();
        logi!("pid {pid}: exec in flight{note}; rechecking in {RECHECK_TIMEOUT_MS} ms");
    }

    pub fn upgrade_kernel_sources(&mut self) {
        if let Some(tracker) = self.kernel.as_mut() {
            tracker.upgrade();
        }
    }

    pub fn prune_kernel_bookkeeping(&mut self) {
        self.done.retain(|pid| procfs::is_alive(*pid));
        self.ignored.retain(|pid| procfs::is_alive(*pid));
        self.candidates.retain(|pid, _| procfs::is_alive(*pid));
    }
}

impl fmt::Display for EventKind {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}
