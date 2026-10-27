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

use std::fs::File;
use std::io;
use std::io::Read;
use std::path::Path;
use std::thread;
use std::time::Duration;

use crate::arch::Arch;
use crate::config;
use crate::daemon::{Daemon, Tracking};
use crate::elf;
use crate::log::{logi, logw};
use crate::maps;
use crate::procfs;
use crate::ptrace_ops::read_registers;
use crate::sys::Pid;
use crate::sys::ptrace;
use crate::sys::signal;
use crate::sys::wait::{self, Status};
use crate::trace::{State, Tracee, release_tracee, set_entry_breakpoint};

/// How long a target is retried before it is written off. Waiting longer has no
/// value: a process another tracer holds is held for its whole life.
const HOLD_TIMEOUT_MS: u64 = 5000;
const INTERRUPT_TIMEOUT_MS: u64 = 200;
const LATE_DEFER_MS: u64 = 10_000;
const LATE_SAFE_AGE_MS: u64 = 5000;

#[derive(Debug, Clone)]
pub struct Candidate {
    pub exe: String,
    pub target_since_ms: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Seizure {
    Seized,
    Retry,
    GiveUp,
}

struct StoppedTarget {
    arch: Arch,
    is_64: bool,
    entry: usize,
    thumb: bool,
    pc: usize,
    resume_signal: Option<i32>,
}

impl Daemon {
    pub fn select_mode(&mut self) {
        self.requested_mode = config::effective().mode;
        if self.requested_mode == config::MODE_PROC {
            logi!("tracking mode forced to proc (poll /proc); init is never traced");
            self.mode = Tracking::Proc;
            return;
        }
        if self.requested_mode == config::MODE_PTRACE {
            logi!("tracking mode forced to ptrace (trace init)");
            self.mode = if self.seize_init() {
                Tracking::Ptrace
            } else {
                Tracking::Proc
            };
            return;
        }

        if self.requested_mode == config::MODE_LATE {
            if self.start_kernel_tracker() {
                logi!(
                    "tracking mode forced to late: kernel events are only an accelerator, init is never traced"
                );
            } else {
                logi!("tracking mode forced to late (poll /proc); init is never traced");
            }
            self.mode = Tracking::Late;
            return;
        }

        if self.requested_mode == config::MODE_KERNEL {
            if self.start_kernel_tracker() {
                self.mode = Tracking::Kernel;
                return;
            }
            logw!("kernel tracking is unavailable on this kernel; using proc (poll /proc)");
            self.mode = Tracking::Proc;
            return;
        }

        if !monitor_present() && self.seize_init() {
            self.stop_kernel_tracker();
            self.mode = Tracking::Ptrace;
            return;
        }
        if monitor_present() {
            self.start_kernel_tracker();
            logi!(
                "another Zygisk-family loader is running; using late mode (never trace init, defer to it)"
            );
            self.mode = Tracking::Late;
            return;
        }
        if self.start_kernel_tracker() {
            self.mode = Tracking::Kernel;
            return;
        }
        self.mode = Tracking::Proc;
    }

    pub fn apply_requested_mode(&mut self) {
        let wanted = config::effective().mode;
        if wanted == self.requested_mode {
            return;
        }
        self.requested_mode = wanted.clone();

        if wanted == config::MODE_LATE {
            if self.mode == Tracking::Ptrace && !self.hand_over_init() {
                logw!("tracking mode late needs init to be handed over; staying in ptrace mode");
                return;
            }
            if !self.start_kernel_tracker() {
                logw!("kernel tracking is unavailable on this kernel; late mode polls /proc");
            }
            self.mode = Tracking::Late;
            self.state.dirty = true;
            logw!(
                "tracking mode changed to late; init is never traced and another tracer is deferred to"
            );
            return;
        }

        if wanted == config::MODE_KERNEL || wanted == config::MODE_AUTO {
            if self.mode == Tracking::Kernel {
                return;
            }
            if self.start_kernel_tracker() {
                if self.mode == Tracking::Ptrace && !self.hand_over_init() {
                    self.stop_kernel_tracker();
                    logw!(
                        "tracking mode kernel needs init to be handed over; staying in ptrace mode"
                    );
                    return;
                }
                self.mode = Tracking::Kernel;
                self.state.dirty = true;
                logw!("tracking mode changed to {wanted}; detached init and left /proc polling");
                return;
            }
            logw!("kernel tracking is unavailable on this kernel; falling back");
        }

        if self.mode == Tracking::Late && wanted == config::MODE_PROC {
            self.stop_kernel_tracker();
            self.mode = Tracking::Proc;
            self.state.dirty = true;
            logw!(
                "tracking mode changed to proc; stopped the kernel event sources and switched to polling /proc"
            );
            return;
        }

        if self.mode == Tracking::Kernel {
            self.stop_kernel_tracker();
            self.mode = Tracking::Proc;
            self.state.dirty = true;
            logw!(
                "tracking mode changed to {wanted}; stopped the kernel event sources and switched to polling /proc (ptrace needs a restart)"
            );
            return;
        }

        if wanted == config::MODE_PROC && self.mode == Tracking::Ptrace {
            if !self.hand_over_init() {
                logw!("tracking mode proc needs init to be handed over; staying in ptrace mode");
                return;
            }
            self.mode = Tracking::Proc;
            self.state.dirty = true;
            logw!("tracking mode changed to proc; detached init and switched to polling /proc");
        } else if wanted != config::MODE_PROC && self.mode == Tracking::Proc {
            logw!(
                "tracking mode {wanted} only applies on the next daemon start; staying in proc mode"
            );
        } else if wanted != config::MODE_PROC && self.mode == Tracking::Late {
            logw!(
                "tracking mode {wanted} only applies on the next daemon start; staying in late mode"
            );
        }
    }

    pub fn hand_over_init(&mut self) -> bool {
        if stop_tracee(1, INTERRUPT_TIMEOUT_MS).is_none() {
            logw!("init (pid 1) does not stop; keeping it traced");
            return false;
        }
        match ptrace::detach_result(1, None) {
            Ok(()) => {
                self.tracees.remove(&1);
                true
            }
            Err(error) => {
                logw!("cannot detach from init: {error}");
                ptrace::cont(1, None);
                false
            }
        }
    }

    fn seize_init(&mut self) -> bool {
        logi!("tracing init (pid 1)");
        if let Err(error) = ptrace::seize(1, ptrace::TRACING_OPTIONS) {
            logw!(
                "cannot seize init: {error} — another tracer holds pid 1; using proc mode (poll /proc) instead"
            );
            return false;
        }
        self.tracees
            .insert(1, Tracee::new(State::Traced, Arch::Unknown));
        logi!("successfully seized init");
        true
    }

    pub fn expire_pending(&mut self) {
        let now = procfs::now_ms();
        let expired: Vec<Pid> = self
            .tracees
            .iter()
            .filter(|(_, tracee)| {
                tracee.state == State::Entry && tracee.deadline_ms != 0 && now >= tracee.deadline_ms
            })
            .map(|(pid, _)| *pid)
            .collect();

        for pid in expired {
            let Some(mut tracee) = self.tracees.remove(&pid) else {
                continue;
            };
            let Some(status) = stop_tracee(pid, INTERRUPT_TIMEOUT_MS) else {
                logw!(
                    "entry breakpoint for pid {pid} never hit, and it does not stop; retrying later"
                );
                tracee.deadline_ms = procfs::now_ms() + HOLD_TIMEOUT_MS;
                self.tracees.insert(pid, tracee);
                continue;
            };

            logw!("entry breakpoint for pid {pid} never hit (already past entry?), detaching");
            if status.is_stopped() {
                release_tracee(pid, &tracee);
                ptrace::detach(pid, None);
            }
            self.done.insert(pid);
        }
    }

    pub fn recheck_pending(&mut self) {
        if self.recheck.is_empty() {
            return;
        }
        let now = procfs::now_ms();
        let pending: Vec<Pid> = self.recheck.keys().copied().collect();
        for pid in pending {
            let deadline = self.recheck.get(&pid).copied().unwrap_or(0);
            if now >= deadline
                || !procfs::is_alive(pid)
                || self.tracees.contains_key(&pid)
                || self.done.contains(&pid)
                || self.ignored.contains(&pid)
            {
                self.recheck.remove(&pid);
                continue;
            }
            let exe = procfs::read_exe_path(pid);
            if exe.is_empty() || !crate::targets::matches(&self.targets, &exe) {
                continue;
            }
            self.recheck.remove(&pid);
            self.observe_target(pid, &exe);
        }
    }

    pub fn release_tracees(&mut self) -> bool {
        let pending: Vec<Pid> = self
            .tracees
            .iter()
            .filter(|(_, tracee)| tracee.state != State::Traced)
            .map(|(pid, _)| *pid)
            .collect();

        let mut released = true;
        for pid in pending {
            let Some(tracee) = self.tracees.remove(&pid) else {
                continue;
            };
            if stop_tracee(pid, INTERRUPT_TIMEOUT_MS).is_none() {
                self.tracees.insert(pid, tracee);
                released = false;
                continue;
            }
            release_tracee(pid, &tracee);
            ptrace::detach(pid, None);
        }
        released
    }

    pub fn poll_processes(&mut self) {
        let last_pid = procfs::last_allocated_pid();
        if last_pid.is_some() && last_pid == self.last_allocated_pid && self.candidates.is_empty() {
            self.proc_scan_active = false;
            return;
        }
        self.last_allocated_pid = last_pid;

        let mut active = false;
        let mut seen = std::collections::HashSet::new();
        let Ok(entries) = std::fs::read_dir("/proc") else {
            return;
        };

        for entry in entries.flatten() {
            let file_name = entry.file_name();
            let Ok(pid) = file_name.to_string_lossy().parse::<Pid>() else {
                continue;
            };
            if pid <= 1 {
                continue;
            }
            seen.insert(pid);

            if self.tracees.contains_key(&pid)
                || self.done.contains(&pid)
                || self.ignored.contains(&pid)
            {
                continue;
            }

            let exe = procfs::read_exe_path(pid);
            if exe.is_empty() {
                continue; // zombie or gone; pruned below
            }
            let is_target = crate::targets::matches(&self.targets, &exe);
            active |= is_target;

            match self.candidates.get(&pid).cloned() {
                None => {
                    if is_target {
                        self.candidates.insert(
                            pid,
                            Candidate {
                                exe: exe.clone(),
                                target_since_ms: procfs::now_ms(),
                            },
                        );
                        self.observe_target(pid, &exe);
                    } else if procfs::is_pre_exec_fork(&exe) {
                        self.candidates.insert(
                            pid,
                            Candidate {
                                exe,
                                target_since_ms: 0,
                            },
                        );
                    } else {
                        self.ignored.insert(pid);
                    }
                    continue;
                }
                Some(candidate) => {
                    if exe == candidate.exe {
                        if is_target {
                            // Late mode has its own deadline for a target another
                            // tracer holds, so the proc hold timeout does not apply.
                            if self.mode != Tracking::Late
                                && procfs::now_ms() - candidate.target_since_ms > HOLD_TIMEOUT_MS
                            {
                                self.abandon(pid, &exe);
                            } else {
                                self.observe_target(pid, &exe);
                            }
                        }
                        continue;
                    }
                    let since = if is_target { procfs::now_ms() } else { 0 };
                    if let Some(tracked) = self.candidates.get_mut(&pid) {
                        tracked.exe = exe.clone();
                        tracked.target_since_ms = since;
                    }
                    if is_target {
                        self.observe_target(pid, &exe);
                    } else {
                        self.candidates.remove(&pid);
                        self.ignored.insert(pid);
                    }
                }
            }
        }

        self.candidates.retain(|pid, _| seen.contains(pid));
        self.ignored.retain(|pid| seen.contains(pid));
        self.done.retain(|pid| seen.contains(pid));
        self.yielded.retain(|pid| seen.contains(pid));
        self.proc_scan_active = active;
        self.ignored_rescans += 1;
        if self.ignored_rescans >= 15000 {
            self.ignored_rescans = 0;
            self.ignored.clear();
        }
    }

    /// Give up on a target and say why. A foreign tracer that never lets go is
    /// the reason we cannot inject into a process at all — a process has exactly
    /// one tracer — so it is reported instead of being dropped in silence.
    fn abandon(&mut self, pid: Pid, exe: &str) {
        let holder = procfs::foreign_tracer(pid)
            .map(procfs::process_exe)
            .unwrap_or_default();
        if holder.is_empty() {
            logw!("{exe} (pid {pid}) was never injectable; giving up");
            self.record_failure(pid, exe, "no injection window");
        } else {
            logw!("{exe} (pid {pid}) is still held by {holder}; giving up");
            self.record_failure(pid, exe, &format!("held by {holder}"));
        }
        self.done.insert(pid);
        self.candidates.remove(&pid);
    }

    pub(crate) fn observe_target(&mut self, pid: Pid, exe: &str) {
        if self.mode == Tracking::Late {
            self.observe_late_target(pid, exe);
            return;
        }
        match self.seize_target(pid, exe) {
            Seizure::Retry => {}
            Seizure::Seized | Seizure::GiveUp => {
                self.done.insert(pid);
                self.candidates.remove(&pid);
            }
        }
    }

    pub(crate) fn observe_late_target(&mut self, pid: Pid, exe: &str) {
        if let Some(tracer) = procfs::foreign_tracer(pid) {
            if !self.deferred.contains_key(&pid) {
                logi!(
                    "{exe} (pid {pid}) is traced by {} (pid {tracer}); deferring",
                    procfs::process_exe(tracer)
                );
            }
            self.deferred
                .entry(pid)
                .or_insert(procfs::now_ms() + LATE_DEFER_MS);
            return;
        }

        match self.seize_late_target(pid, exe) {
            Seizure::Retry => {
                self.deferred
                    .entry(pid)
                    .or_insert(procfs::now_ms() + LATE_DEFER_MS);
            }
            Seizure::Seized | Seizure::GiveUp => {
                self.deferred.remove(&pid);
                self.done.insert(pid);
                self.candidates.remove(&pid);
            }
        }
    }

    pub fn retry_deferred(&mut self) {
        if self.deferred.is_empty() {
            return;
        }
        let now = procfs::now_ms();
        let pending: Vec<Pid> = self.deferred.keys().copied().collect();
        for pid in pending {
            let deadline = self.deferred.get(&pid).copied().unwrap_or(0);
            let Some(tracer) = procfs::foreign_tracer(pid) else {
                self.deferred.remove(&pid);
                if !procfs::is_alive(pid)
                    || self.tracees.contains_key(&pid)
                    || self.done.contains(&pid)
                    || self.ignored.contains(&pid)
                {
                    continue;
                }
                let exe = procfs::read_exe_path(pid);
                if exe.is_empty() || !crate::targets::matches(&self.targets, &exe) {
                    continue;
                }
                logi!("{exe} (pid {pid}): the foreign tracer went away; retrying the injection");
                self.observe_late_target(pid, &exe);
                continue;
            };
            if now < deadline {
                continue;
            }

            let exe = procfs::read_exe_path(pid);
            let holder = procfs::process_exe(tracer);
            if holder.is_empty() {
                logw!("{exe} (pid {pid}) is still held by pid {tracer}; giving up");
                self.record_failure(pid, &exe, "no injection window");
            } else {
                logw!("{exe} (pid {pid}) is still held by {holder}; giving up");
                self.record_failure(pid, &exe, &format!("held by {holder}"));
            }
            self.deferred.remove(&pid);
            self.done.insert(pid);
        }
    }

    pub fn prune_deferred(&mut self) {
        self.deferred.retain(|pid, _| procfs::is_alive(*pid));
    }

    fn seize_late_target(&mut self, pid: Pid, exe: &str) -> Seizure {
        let stopped = match self.seize_and_stop(pid, exe) {
            Ok(stopped) => stopped,
            Err(seizure) => return seizure,
        };

        if !procfs::pc_in_exe_text(pid, exe, stopped.pc) {
            return self.arm_entry_injection(pid, exe, stopped);
        }

        let single_threaded = procfs::is_single_threaded(pid);
        let young = procfs::process_age_ms(pid).is_some_and(|age| age < LATE_SAFE_AGE_MS);
        if !single_threaded && !young {
            logw!(
                "{exe} (pid {pid}) is a running multithreaded process; it is unsafe to inject late"
            );
            self.record_failure(
                pid,
                exe,
                "unsafe to inject late into a running multithreaded process",
            );
            ptrace::detach(pid, None);
            return Seizure::GiveUp;
        }

        let mut tracee = Tracee::new(State::Late, stopped.arch);
        tracee.exe = exe.to_owned();
        tracee.loader = if stopped.is_64 {
            self.loader64.to_string_lossy().into_owned()
        } else {
            self.loader32.to_string_lossy().into_owned()
        };
        tracee.entry = if stopped.arch == Arch::Arm32 {
            stopped.pc & !1
        } else {
            stopped.pc
        };
        tracee.thumb = stopped.arch == Arch::Arm32 && stopped.pc & 1 != 0;
        let thumb = tracee.thumb;
        if !set_entry_breakpoint(pid, &mut tracee, thumb) {
            logw!("{exe} (pid {pid}): cannot arm the late breakpoint; skipping");
            ptrace::detach(pid, None);
            return Seizure::GiveUp;
        }
        if !self.begin_late_injection(pid, &mut tracee) {
            return Seizure::GiveUp;
        }
        logi!(
            "seized {exe} (pid {pid}) past its entry (pc {:#x}); injecting late ({})",
            stopped.pc,
            if stopped.is_64 { "64-bit" } else { "32-bit" }
        );
        Seizure::Seized
    }

    fn seize_target(&mut self, pid: Pid, exe: &str) -> Seizure {
        let stopped = match self.seize_and_stop(pid, exe) {
            Ok(stopped) => stopped,
            Err(seizure) => return seizure,
        };
        if procfs::pc_in_exe_text(pid, exe, stopped.pc) {
            logw!(
                "{exe} (pid {pid}) is already past its entry (pc {:#x}); instance missed",
                stopped.pc
            );
            self.record_failure(pid, exe, "injection window missed (already past its entry)");
            ptrace::detach(pid, None);
            return Seizure::GiveUp;
        }
        self.arm_entry_injection(pid, exe, stopped)
    }

    fn seize_and_stop(&mut self, pid: Pid, exe: &str) -> Result<StoppedTarget, Seizure> {
        // A process has exactly one tracer, so a foreign one means another
        // loader already reached this process. Wait for it to finish and take
        // the process afterwards: this is the same outcome as a failed
        // PTRACE_SEIZE, but decided from the tracer instead of from an errno.
        if let Some(tracer) = procfs::foreign_tracer(pid) {
            if self.yielded.insert(tracer) {
                logi!(
                    "{exe} (pid {pid}) is traced by {} (pid {tracer}); waiting for it",
                    procfs::process_exe(tracer)
                );
            }
            return Err(Seizure::Retry);
        }

        let Some(header) = elf::header_of_file(Path::new(&format!("/proc/{pid}/exe"))) else {
            logw!("{exe} (pid {pid}): cannot read its ELF header; skipping");
            return Err(Seizure::GiveUp);
        };
        let arch = elf::arch_of(&header);
        if arch == Arch::Unknown {
            logw!(
                "skipping {exe} (pid {pid}): unsupported machine {}",
                header.machine
            );
            return Err(Seizure::GiveUp);
        }

        let mut base = 0;
        let mut has_loader = false;
        for entry in maps::parse_for_pid(pid) {
            if entry.offset == 0 && procfs::map_path_equals_exe(&entry.path, exe) {
                base = entry.start;
            }
            if entry.path.contains("libloader.so")
                || entry.path.contains("memfd:loader")
                || entry.path.contains("memfd:znn-loader")
            {
                has_loader = true;
            }
        }
        if base == 0 {
            logw!(
                "{exe} (pid {pid}): no load base in /proc/{pid}/maps ({} entries read); skipping",
                maps::parse_for_pid(pid).len()
            );
            return Err(Seizure::GiveUp);
        }
        if has_loader {
            logw!(
                "{exe} (pid {pid}) already carries the ZNN loader (fork of an injected process); skipping"
            );
            return Err(Seizure::GiveUp);
        }

        let (entry, thumb) = crate::trace::entry_point(arch, &header, base);
        if entry == 0 {
            logw!("{exe} (pid {pid}): no usable entry point; skipping");
            return Err(Seizure::GiveUp);
        }

        if let Err(error) = ptrace::seize(pid, 0) {
            if error.kind() == std::io::ErrorKind::PermissionDenied {
                return Err(Seizure::Retry);
            }
            logw!("{exe} (pid {pid}): cannot seize: {error}; skipping");
            return Err(Seizure::GiveUp);
        }

        if let Err(error) = ptrace::interrupt(pid) {
            logw!("{exe} (pid {pid}): cannot interrupt: {error}; skipping");
            ptrace::detach(pid, None);
            return Err(Seizure::GiveUp);
        }

        let deadline = procfs::now_ms() + INTERRUPT_TIMEOUT_MS;
        let Some(status) = wait_for_stop(pid, deadline) else {
            logw!("{exe} (pid {pid}): never stopped after being interrupted; skipping");
            ptrace::detach(pid, None);
            return Err(Seizure::GiveUp);
        };
        if status.is_exited() || status.is_signaled() {
            ptrace::detach(pid, None);
            return Err(Seizure::GiveUp);
        }
        let mut resume_signal = None;
        if status.is_stopped() {
            let stopped_by = status.stop_signal();
            if stopped_by != signal::SIGTRAP {
                resume_signal = Some(stopped_by);
            }
        }

        let Ok(regs) = read_registers(pid, arch) else {
            logw!("{exe} (pid {pid}): cannot read its registers; skipping");
            ptrace::detach(pid, None);
            return Err(Seizure::GiveUp);
        };
        Ok(StoppedTarget {
            arch,
            is_64: header.is_64,
            entry,
            thumb,
            pc: regs.pc(),
            resume_signal,
        })
    }

    fn arm_entry_injection(&mut self, pid: Pid, exe: &str, stopped: StoppedTarget) -> Seizure {
        let mut tracee = Tracee::new(State::Entry, stopped.arch);
        tracee.entry = stopped.entry;
        tracee.thumb = stopped.thumb;
        tracee.exe = exe.to_owned();
        tracee.loader = if stopped.is_64 {
            self.loader64.to_string_lossy().into_owned()
        } else {
            self.loader32.to_string_lossy().into_owned()
        };
        tracee.deadline_ms = procfs::now_ms() + 3000;
        if !set_entry_breakpoint(pid, &mut tracee, stopped.thumb) {
            logw!("{exe} (pid {pid}): cannot arm the entry breakpoint; skipping");
            ptrace::detach(pid, None);
            return Seizure::GiveUp;
        }
        self.tracees.insert(pid, tracee);

        logi!(
            "seized {exe} (pid {pid}) mid-linker, entry breakpoint at {:#x} ({})",
            stopped.entry,
            if stopped.is_64 { "64-bit" } else { "32-bit" }
        );
        ptrace::cont(pid, stopped.resume_signal);
        Seizure::Seized
    }
}

/// The tracer that holds init, when it is not us. Exact, and independent of any
/// process name: whoever traces pid 1 owns the spawn path.
pub(crate) fn init_holder() -> Option<Pid> {
    procfs::foreign_tracer(1)
}

fn wait_for_stop(pid: Pid, deadline: u64) -> Option<Status> {
    loop {
        match wait::waitpid(pid, wait::WALL | wait::WNOHANG) {
            Ok(Some((waited, status))) if waited == pid => return Some(status),
            Ok(_) => {}
            Err(error) if error.kind() == io::ErrorKind::Interrupted => continue,
            Err(_) => return None,
        }
        if procfs::now_ms() >= deadline {
            return None;
        }
        thread::sleep(Duration::from_micros(500));
    }
}

fn stop_tracee(pid: Pid, timeout_ms: u64) -> Option<Status> {
    ptrace::interrupt(pid).ok()?;
    wait_for_stop(pid, procfs::now_ms() + timeout_ms)
}

/// Whether another Zygisk-family loader is around. The exact signal is a
/// foreign tracer, but it is blind while we hold init ourselves, so the process
/// name scan stays as a complement. Neither is used to decide a single process:
/// that is decided by that process's own tracer.
pub(crate) fn monitor_present() -> bool {
    init_holder().is_some() || zygisk_named_present()
}

fn zygisk_named_present() -> bool {
    let Ok(entries) = std::fs::read_dir("/proc") else {
        return false;
    };
    for entry in entries.flatten() {
        let file_name = entry.file_name();
        let name = file_name.to_string_lossy();
        if !name.as_bytes().first().is_some_and(u8::is_ascii_digit) {
            continue;
        }
        let Ok(mut file) = File::open(format!("/proc/{name}/comm")) else {
            continue;
        };
        let mut comm = [0u8; 63];
        let Ok(read) = file.read(&mut comm) else {
            continue;
        };
        if comm[..read].windows(6).any(|window| window == b"zygisk") {
            return true;
        }
    }
    false
}
