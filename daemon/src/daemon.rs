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

use std::collections::{BTreeMap, HashMap, HashSet};
use std::fs;
use std::io;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Duration;

use crate::config;
use crate::log::{loge, logi, logw};
use crate::mode::Candidate;
use crate::procfs;
use crate::state::StateWriter;
use crate::sys::fs as raw_fs;
use crate::sys::page;
use crate::sys::wait;
use crate::sys::{Pid, process, signal};
use crate::system::{self, SystemInfo};
use crate::targets::{ModuleInfo, Target};
use crate::trace::{State, Tracee};

pub const MODULE_ID: &str = "zygisknextsu";
pub const VERSION: &str = env!("ZNN_VERSION");

const SHUTDOWN_RELEASE_BUDGET_MS: u64 = 800;
const PROC_IDLE_SLEEP_MS: u64 = 25;
const STATE_POLL_INTERVAL_MS: u64 = 15;
const HOT_UPDATE_START_PROBES: u32 = 20;
const HOT_UPDATE_RETRY_MS: u64 = 15_000;

static RESCAN: AtomicBool = AtomicBool::new(false);
static SHUTDOWN: AtomicBool = AtomicBool::new(false);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tracking {
    Kernel,
    Ptrace,
    Proc,
    Late,
}

#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ModeReason {
    KernelUnavailable,
    PtraceUnavailable,
    OtherLoaderActive,
    SwitchDeferred,
    RestartRequired,
}

impl Tracking {
    pub fn as_str(self) -> &'static str {
        match self {
            Tracking::Kernel => "kernel",
            Tracking::Ptrace => "ptrace",
            Tracking::Proc => "proc",
            Tracking::Late => "late",
        }
    }
}

#[derive(Debug)]
pub struct Daemon {
    pub targets: Vec<Target>,
    pub modules: BTreeMap<String, ModuleInfo>,
    pub system: SystemInfo,
    pub started_at_ticks: u64,
    pub loader64: PathBuf,
    pub loader32: PathBuf,
    pub module_dir: PathBuf,
    pub state: StateWriter,
    pub kernel: Option<crate::kernel::KernelTracker>,
    pub mode: Tracking,
    pub requested_mode: String,
    pub mode_reason: Option<ModeReason>,
    pub hot_update: bool,
    pub hot_update_checked: Option<(String, u64)>,
    pub tracees: HashMap<Pid, Tracee>,
    pub done: HashSet<Pid>,
    pub ignored: HashSet<Pid>,
    pub candidates: HashMap<Pid, Candidate>,
    pub yielded: HashSet<Pid>,
    pub deferred: HashMap<Pid, u64>,
    pub recheck: HashMap<Pid, u64>,
    pub ignored_rescans: u32,
    pub last_allocated_pid: Option<Pid>,
    pub proc_scan_active: bool,
    pub last_proc_scan_ms: u64,
    pub last_rescan_ms: Option<u64>,
    pub last_zygisk_check_ms: u64,
    pub last_sweep_ms: u64,
    pub last_prop_update_ms: u64,
}

impl Daemon {
    pub fn new(module_dir: PathBuf, handoff: Option<&[u8]>) -> Self {
        // Log before anything that can fail or block: a daemon that dies or
        // stalls during startup must not do so silently.
        logi!("ZNNext {VERSION} starting");

        let settings = config::effective();
        let requested_mode = settings.mode.clone();
        let hot_update = settings.hot_update == config::HOT_UPDATE_TRUE;

        let mut daemon = Self {
            targets: Vec::new(),
            modules: BTreeMap::new(),
            system: SystemInfo::default(),
            started_at_ticks: procfs::start_time_ticks(std::process::id() as Pid).unwrap_or(0),
            loader64: module_dir.join("lib64").join("libzn_loader.so"),
            loader32: module_dir.join("lib").join("libzn_loader.so"),
            module_dir,
            state: StateWriter::default(),
            kernel: None,
            mode: Tracking::Ptrace,
            requested_mode: requested_mode.clone(),
            mode_reason: None,
            hot_update,
            hot_update_checked: None,
            tracees: HashMap::new(),
            done: HashSet::new(),
            ignored: HashSet::new(),
            candidates: HashMap::new(),
            yielded: HashSet::new(),
            deferred: HashMap::new(),
            recheck: HashMap::new(),
            ignored_rescans: 0,
            last_allocated_pid: None,
            proc_scan_active: false,
            last_proc_scan_ms: 0,
            last_rescan_ms: None,
            last_zygisk_check_ms: 0,
            last_sweep_ms: 0,
            last_prop_update_ms: 0,
        };

        daemon.system = system::collect();
        daemon.check_loaders();
        crate::update::log_status();
        signal::set_handler(signal::SIGHUP, on_sighup);
        //`--ctl exit` and `--ctl restart` end us with SIGTERM, and a tracee that
        //is parked on an entry breakpoint must not be left holding it
        signal::set_handler(signal::SIGTERM, on_sigterm);

        daemon.collect_targets();
        if let Some(bytes) = handoff {
            daemon.restore_state(bytes);
        }
        logi!("collected {} zn modules", daemon.targets.len());

        let kernel_early = requested_mode == config::MODE_KERNEL
            || (requested_mode == config::MODE_AUTO && crate::mode::monitor_present());
        if kernel_early {
            daemon.start_kernel_tracker();
        }

        daemon.select_mode();
        daemon
    }

    pub fn run(&mut self) -> ! {
        self.open_state_socket();
        self.apply_prop_text();
        let mut last_query_poll_ms = 0;
        self.last_prop_update_ms = procfs::now_ms();

        loop {
            if SHUTDOWN.swap(false, Ordering::SeqCst) {
                logi!("shutting down: releasing the tracees we are holding");
                let deadline = procfs::now_ms() + SHUTDOWN_RELEASE_BUDGET_MS;
                while !self.release_tracees() {
                    if procfs::now_ms() >= deadline {
                        loge!(
                            "cannot release every tracee; aborting shutdown to avoid leaving a trap instruction in a target"
                        );
                        self.state.dirty = true;
                        break;
                    }
                    thread::sleep(Duration::from_millis(25));
                }
                if self
                    .tracees
                    .values()
                    .all(|tracee| tracee.state == State::Traced)
                {
                    break;
                }
            }

            if RESCAN.swap(false, Ordering::SeqCst) {
                self.reload_settings();
                self.collect_targets();
                self.apply_requested_mode();
                self.last_rescan_ms = Some(procfs::now_ms());
                self.state.dirty = true;
                self.maybe_hot_update();
                logi!("rescanned znn targets ({})", self.targets.len());
            }

            match self.mode {
                Tracking::Kernel => {
                    self.poll_kernel();
                    self.reconcile_targets();
                    self.retry_deferred();
                }
                Tracking::Proc => self.poll_processes(),
                Tracking::Late => {
                    if self.kernel.is_some() {
                        self.poll_kernel();
                    } else {
                        self.poll_processes();
                    }
                    self.reconcile_targets();
                    self.retry_deferred();
                }
                Tracking::Ptrace => {
                    let now = procfs::now_ms();
                    if self.requested_mode == config::MODE_AUTO
                        && now - self.last_zygisk_check_ms >= 5000
                    {
                        self.last_zygisk_check_ms = now;
                        if crate::mode::monitor_present() && self.hand_over_init() {
                            self.start_kernel_tracker();
                            logw!(
                                "another Zygisk-family loader is running; yielded init and switched to late mode (deferring to it)"
                            );
                            self.mode = Tracking::Late;
                            self.mode_reason = Some(ModeReason::OtherLoaderActive);
                            self.state.dirty = true;
                        }
                    }
                }
            }

            self.reap_traces();
            self.expire_pending();
            self.recheck_pending();

            let now = procfs::now_ms();
            if now - last_query_poll_ms >= STATE_POLL_INTERVAL_MS {
                self.serve_state();
                last_query_poll_ms = now;
            }
            if self
                .last_rescan_ms
                .is_none_or(|last_rescan| now - last_rescan >= 5000)
            {
                self.collect_targets();
                self.last_rescan_ms = Some(now);
                self.state.dirty = true;
                if self.mode == Tracking::Kernel
                    || (self.mode == Tracking::Late && self.kernel.is_some())
                {
                    self.prune_kernel_bookkeeping();
                    self.upgrade_kernel_sources();
                }
                if self.mode == Tracking::Kernel || self.mode == Tracking::Late {
                    self.prune_deferred();
                }
                self.recheck.retain(|pid, _| procfs::is_alive(*pid));
                self.maybe_hot_update();
            }
            if self.state.dirty
                && (now - self.last_prop_update_ms >= 2000 || self.last_prop_update_ms == 0)
            {
                self.open_state_socket();
                self.apply_prop_text();
                self.state.dirty = false;
                self.last_prop_update_ms = now;
            }

            thread::sleep(match self.mode {
                Tracking::Proc if self.proc_scan_active => Duration::from_millis(2),
                Tracking::Proc => Duration::from_millis(PROC_IDLE_SLEEP_MS),
                Tracking::Late if self.kernel.is_some() => Duration::from_millis(1),
                Tracking::Late if self.proc_scan_active => Duration::from_millis(2),
                Tracking::Late => Duration::from_millis(PROC_IDLE_SLEEP_MS),
                Tracking::Kernel | Tracking::Ptrace => Duration::from_millis(1),
            });
        }

        process::exit_now(0)
    }

    fn reload_settings(&mut self) {
        self.hot_update = config::effective().hot_update == config::HOT_UPDATE_TRUE;
    }

    fn maybe_hot_update(&mut self) {
        if !self.hot_update {
            return;
        }
        let Some(pending) = crate::update::pending() else {
            return;
        };
        if crate::update::running_from(&pending.dir) {
            return;
        }
        let now = procfs::now_ms();
        let fingerprint = crate::update::fingerprint(&pending.dir);
        if let Some((checked, retry_at)) = &self.hot_update_checked
            && *checked == fingerprint
            && now < *retry_at
        {
            return;
        }

        if let Err(error) = crate::machikado::verify_module(&pending.dir) {
            let first_attempt = self
                .hot_update_checked
                .as_ref()
                .is_none_or(|(checked, _)| *checked != fingerprint);
            if first_attempt {
                logw!("hot update: refusing {}: {error}", pending.dir.display());
            }
            // when module is being install
            self.hot_update_checked = Some((fingerprint, now + HOT_UPDATE_RETRY_MS));
            return;
        }

        let executable = crate::update::executable(&pending);
        let state = match serde_json::to_vec(&self.state_report()) {
            Ok(state) => state,
            Err(error) => {
                logw!("hot update: cannot prepare daemon state: {error}");
                self.hot_update_checked = Some((fingerprint, now + HOT_UPDATE_RETRY_MS));
                return;
            }
        };
        self.hot_update_checked = Some((fingerprint, u64::MAX));
        let pid = match process::spawn_detached(&executable, &pending.dir, Some(&state)) {
            Ok(pid) => pid,
            Err(error) => {
                logw!("hot update: cannot start {}: {error}", executable.display());
                self.hot_update_checked = None;
                return;
            }
        };

        let mut alive = false;
        for _ in 0..HOT_UPDATE_START_PROBES {
            if !procfs::is_daemon_alive(pid) {
                break;
            }
            alive = true;
            thread::sleep(Duration::from_millis(25));
        }
        if alive && procfs::is_daemon_alive(pid) {
            logi!(
                "hot update: staged version \"{}\" verified; started daemon ({pid}) from {}",
                pending.version,
                pending.dir.display()
            );
            SHUTDOWN.store(true, Ordering::SeqCst);
        } else {
            logw!("hot update: the staged daemon ({pid}) exited during startup; staying up");
            self.hot_update_checked = None;
        }
    }

    fn check_loaders(&mut self) {
        let mut unreadable = 0;
        for path in [&self.loader64, &self.loader32] {
            if !raw_fs::access(path, raw_fs::R_OK) {
                match fs::metadata(path) {
                    Ok(_) => loge!("loader {} not readable", path.display()),
                    Err(error) => loge!("loader {} not readable: {error}", path.display()),
                }
                unreadable += 1;
                continue;
            }
            match crate::sys::fs::lgetxattr(path, "security.selinux") {
                Ok(label) => logi!(
                    "loader {} label {}",
                    path.display(),
                    String::from_utf8_lossy(&label)
                ),
                Err(error) => loge!(
                    "loader {}: cannot read selinux label: {error}",
                    path.display()
                ),
            }
        }
        if unreadable > 0 {
            self.state.status_reason = "cannot read the loader".to_owned();
        }

        let page_size = page::size();
        for path in [&self.loader64, &self.loader32] {
            let Some(align) = crate::elf::min_load_align(path) else {
                continue;
            };
            if align < page_size {
                loge!(
                    "loader {} is only {align}-byte aligned, but this device uses {page_size}-byte pages",
                    path.display()
                );
                self.state.status_reason = format!("loader not {} KB aligned", page_size / 1024);
            }
        }

        for (path, is_64) in [(&self.loader64, true), (&self.loader32, false)] {
            let Some(header) = crate::elf::header_of_file(path) else {
                continue;
            };
            if header.is_64 != is_64 {
                loge!(
                    "loader {} is {}-bit, but the daemon expects {}-bit",
                    path.display(),
                    if header.is_64 { 64 } else { 32 },
                    if is_64 { 64 } else { 32 }
                );
                self.state.status_reason = "loader architecture mismatch".to_owned();
            }
        }
    }

    fn reap_traces(&mut self) {
        loop {
            match wait::waitpid(-1, wait::WALL | wait::WNOHANG) {
                Ok(Some((pid, status))) => self.handle_event(pid, status),
                Ok(None) => break,
                Err(error) => {
                    if error.kind() == io::ErrorKind::Interrupted {
                        continue;
                    }
                    if error.raw_os_error() == Some(wait::ECHILD) {
                        break;
                    }
                    loge!("waitpid failed: {error}");
                    break;
                }
            }
        }
    }
}

extern "C" fn on_sighup(_signal: i32) {
    RESCAN.store(true, Ordering::SeqCst);
}

extern "C" fn on_sigterm(_signal: i32) {
    SHUTDOWN.store(true, Ordering::SeqCst);
}
