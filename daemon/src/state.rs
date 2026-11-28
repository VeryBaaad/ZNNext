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

use std::fmt::Write as _;
use std::fs;

use serde::{Deserialize, Serialize};

use crate::config;
use crate::daemon::{Daemon, MODULE_ID, ModeReason, Tracking, VERSION};
use crate::log::{loge, logi, logw};
use crate::procfs;
use crate::state_socket::Server;
use crate::sys::Pid;
use crate::sys::fs as raw_fs;
use crate::system::{self, RootImpl};
use crate::targets::{self, ModuleFailure, ModuleInfo};

#[derive(Debug, Serialize)]
pub struct StateReport<'a> {
    running: bool,
    pid: u32,
    mode: &'a str,
    #[serde(rename = "requestedMode")]
    requested_mode: &'a str,
    #[serde(rename = "modeReason", skip_serializing_if = "Option::is_none")]
    mode_reason: Option<ModeReason>,
    version: &'a str,
    system: SystemReport<'a>,
    config: config::HookConfigReport,
    modules: Vec<ModuleReport<'a>>,
    #[serde(rename = "idleDeclarationFingerprints")]
    idle_declarations: std::collections::BTreeMap<&'a str, u64>,
}

#[derive(Debug, Serialize)]
pub struct SystemReport<'a> {
    kernel: &'a str,
    sdk: i32,
    #[serde(rename = "pageSize", skip_serializing_if = "Option::is_none")]
    page_size: Option<u64>,
    abi: &'a str,
    abilist: &'a str,
    root: RootReport<'a>,
}

impl SystemReport<'_> {
    pub fn empty() -> SystemReport<'static> {
        SystemReport {
            kernel: "",
            sdk: 0,
            page_size: None,
            abi: "",
            abilist: "",
            root: RootReport {
                kernel_su: "",
                magisk: "",
                apatch: "",
            },
        }
    }
}

#[derive(Debug, Serialize)]
struct RootReport<'a> {
    #[serde(rename = "kernelSU")]
    kernel_su: &'a str,
    magisk: &'a str,
    apatch: &'a str,
}

#[derive(Debug, Serialize)]
struct ModuleReport<'a> {
    id: &'a str,
    name: &'a str,
    version: &'a str,
    processes: Vec<ProcessReport<'a>>,
    failed: &'a [ModuleFailure],
}

#[derive(Debug, Serialize)]
struct ProcessReport<'a> {
    pid: u32,
    name: &'a str,
}

#[derive(Debug, Deserialize)]
struct Snapshot {
    #[serde(default)]
    modules: Vec<SnapshotModule>,
}

#[derive(Debug, Deserialize)]
struct SnapshotModule {
    #[serde(default)]
    id: String,
    #[serde(default)]
    processes: Vec<SnapshotProcess>,
    #[serde(default)]
    failed: Vec<ModuleFailure>,
}

#[derive(Debug, Deserialize)]
struct SnapshotProcess {
    #[serde(default)]
    pid: i64,
    #[serde(default)]
    name: String,
}

#[derive(Debug, Default)]
pub struct StateWriter {
    pub dirty: bool,
    pub server: Option<Server>,
    pub status_reason: String,
    last_prop_text: String,
}

impl Daemon {
    pub fn restore_state(&mut self, bytes: &[u8]) {
        let Ok(snapshot) = serde_json::from_slice::<Snapshot>(bytes) else {
            logw!("cannot decode state from previous daemon");
            return;
        };
        let mut records = 0;
        let mut modules_restored = 0;
        for recorded in snapshot.modules {
            let Some(module) = self.modules.get_mut(&recorded.id) else {
                continue;
            };
            module.failed = recorded.failed;
            let before = module.processes.len();
            for recorded_process in recorded.processes {
                let Ok(pid) = Pid::try_from(recorded_process.pid) else {
                    continue;
                };
                if module.processes.iter().any(|(known, _)| *known == pid) {
                    continue;
                }
                if procfs::process_gone(pid) {
                    continue;
                }
                let exe = procfs::read_exe_path(pid);
                if !exe.is_empty() && !targets::module_matches(module, &exe) {
                    continue;
                }
                module.processes.push((pid, recorded_process.name));
            }
            let restored = module.processes.len() - before;
            records += restored;
            if restored > 0 {
                modules_restored += 1;
            }
        }
        if records > 0 {
            self.state.dirty = true;
            logi!(
                "restored {records} injected process record(s) across {modules_restored} module(s) \
                 from the state snapshot"
            );
        }
    }

    pub fn state_report(&self) -> StateReport<'_> {
        StateReport {
            running: true,
            pid: std::process::id(),
            mode: self.mode.as_str(),
            requested_mode: &self.requested_mode,
            mode_reason: self.mode_reason,
            version: VERSION,
            system: SystemReport {
                kernel: &self.system.kernel,
                sdk: self.system.sdk,
                page_size: self.system.page_size,
                abi: &self.system.abi,
                abilist: &self.system.abilist,
                root: RootReport {
                    kernel_su: &self.system.root.kernel_su,
                    magisk: &self.system.root.magisk,
                    apatch: &self.system.root.apatch,
                },
            },
            config: config::effective().report(),
            modules: self.modules.values().map(module_report).collect::<Vec<_>>(),
            idle_declarations: self
                .modules
                .values()
                .filter(|module| module.processes.is_empty() && module.failed.is_empty())
                .map(|module| {
                    (
                        module.id.as_str(),
                        crate::module_diagnostics::declaration_fingerprint(&module.declarations),
                    )
                })
                .collect(),
        }
    }

    pub fn open_state_socket(&mut self) {
        if self.state.server.is_none() {
            match Server::bind() {
                Ok(server) => self.state.server = Some(server),
                Err(error) => logw!("cannot open daemon state socket: {error}"),
            }
        }
    }

    pub fn serve_state(&mut self) {
        if let Some(mut server) = self.state.server.take() {
            if let Err(error) =
                server.poll(|| serde_json::to_vec(&self.state_report()).map_err(Into::into))
            {
                loge!("cannot serve daemon state: {error}");
            }
            self.state.server = Some(server);
        }
    }

    pub fn apply_prop_text(&mut self) {
        if self.module_dir.as_os_str().is_empty() {
            return;
        }
        let text = self.prop_status_text();
        if text == self.state.last_prop_text {
            return;
        }

        let mut applied = false;
        if raw_fs::access("/data/adb/ksud", raw_fs::X_OK) {
            applied = set_prop_override("/data/adb/ksud", &text);
        }
        if !applied && raw_fs::access("/data/adb/apd", raw_fs::X_OK) {
            applied = set_prop_override("/data/adb/apd", &text);
        }
        if !applied {
            applied = self.write_prop_file(&text);
        }
        if applied {
            self.state.last_prop_text = text;
        }
    }

    fn prop_status_text(&self) -> String {
        let base = self.base_prop_description();
        let text = if !self.state.status_reason.is_empty() {
            format!("[❌ {}]", self.state.status_reason)
        } else {
            let mut status = String::from("✅init");
            let (implementation, version) = root_implementation(&self.system.root);
            if !implementation.is_empty() {
                let _ = write!(status, ", Root: ✅{implementation} ({version})");
            }
            if self.mode == Tracking::Proc {
                status.push_str(", PL");
            }
            let _ = write!(status, ", {} module(s) loaded", self.modules.len());
            format!("[{status}]")
        };
        if base.is_empty() {
            text
        } else {
            format!("{text} {base}")
        }
    }

    fn base_prop_description(&self) -> String {
        let mut path = self.module_dir.join("module.prop.orig");
        if !raw_fs::access(&path, raw_fs::R_OK) {
            path = self.module_dir.join("module.prop");
        }
        let Ok(contents) = fs::read_to_string(&path) else {
            return String::new();
        };
        for line in contents.lines() {
            let line = line.trim_end_matches(['\n', '\r']);
            if line.len() > 12 && line.starts_with("description=") {
                return line[12..].to_owned();
            }
        }
        String::new()
    }

    fn write_prop_file(&self, text: &str) -> bool {
        let path = self.module_dir.join("module.prop");
        let Ok(contents) = fs::read_to_string(&path) else {
            return false;
        };

        let mut out = String::with_capacity(contents.len() + text.len());
        let mut replaced = false;
        for line in contents.lines() {
            let mut line = line.trim_end_matches(['\n', '\r']).to_owned();
            if !replaced && line.starts_with("description=") {
                line = format!("description={text}");
                replaced = true;
            }
            out.push_str(&line);
            out.push('\n');
        }
        if !replaced {
            return false;
        }
        crate::fileio::replace_file(&path, out.as_bytes()).is_ok()
    }
}

fn module_report(module: &ModuleInfo) -> ModuleReport<'_> {
    ModuleReport {
        id: &module.id,
        name: &module.name,
        version: &module.version,
        processes: module
            .processes
            .iter()
            .map(|(pid, name)| ProcessReport {
                pid: *pid as u32,
                name,
            })
            .collect(),
        failed: &module.failed,
    }
}

fn root_implementation(root: &RootImpl) -> (&str, &str) {
    if !root.kernel_su.is_empty() {
        ("KernelSU", root.kernel_su.as_str())
    } else if !root.magisk.is_empty() {
        ("Magisk", root.magisk.as_str())
    } else if !root.apatch.is_empty() {
        ("APatch", root.apatch.as_str())
    } else {
        ("", "")
    }
}

fn set_prop_override(binary: &str, text: &str) -> bool {
    system::run_helper(
        binary,
        &[
            "module",
            "config",
            "set",
            "override.description",
            text,
            "--temp",
        ],
        &[("KSU_MODULE", MODULE_ID), ("AP_MODULE", MODULE_ID)],
    )
}
