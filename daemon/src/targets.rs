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

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::daemon::Daemon;
use crate::log::logw;
use crate::paths;
use crate::procfs::{self, basename};
use crate::sys::Pid;
use crate::sys::fs as raw_fs;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Target {
    pub by_name: bool,
    pub value: String,
}

impl Target {
    pub fn matches(&self, exe: &str) -> bool {
        if self.by_name {
            basename(exe) == self.value
        } else {
            exe == self.value
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct ModuleInfo {
    pub id: String,
    pub name: String,
    pub version: String,
    pub targets: Vec<Target>,
    pub declarations: Vec<Declaration>,
    pub processes: Vec<(Pid, String)>,
    pub failed: Vec<ModuleFailure>,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct ModuleFailure {
    pub name: String,
    pub reason: String,
    pub pid: Pid,
    #[serde(rename = "lastSeenMs", skip_serializing_if = "Option::is_none")]
    pub last_seen_ms: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Declaration {
    pub target: Target,
    pub companion: bool,
    pub library: String,
}

pub fn parse_declaration(line: &str) -> Option<Declaration> {
    if line.contains('\0') {
        return None;
    }
    let mut fields = line.split_whitespace();
    let (kind, value) = fields.next()?.split_once('=')?;
    let by_name = match kind {
        "name" if !value.is_empty() && !value.contains('/') => true,
        "path" if Path::new(value).is_absolute() => false,
        _ => return None,
    };
    let library = fields.next_back()?;

    Some(Declaration {
        target: Target {
            by_name,
            value: value.to_owned(),
        },
        companion: fields.any(|field| field == "companion"),
        library: library.to_owned(),
    })
}

pub fn declarations(module_dir: &Path) -> io::Result<Vec<Declaration>> {
    let contents = fs::read_to_string(module_dir.join("zn_modules.txt"))?;
    Ok(contents.lines().filter_map(parse_declaration).collect())
}

pub fn is_enabled(module_dir: &Path) -> bool {
    !raw_fs::access(module_dir.join("disable"), raw_fs::F_OK)
        && !raw_fs::access(module_dir.join("remove"), raw_fs::F_OK)
}

pub fn read_prop(module_dir: &Path, key: &str) -> String {
    let Ok(contents) = fs::read_to_string(module_dir.join("module.prop")) else {
        return String::new();
    };
    for line in contents.lines() {
        let line = line.trim_end_matches(['\n', '\r']);
        if let Some(value) = line
            .strip_prefix(key)
            .and_then(|rest| rest.strip_prefix('='))
        {
            return value.to_owned();
        }
    }
    String::new()
}

fn collect(
    previous: &BTreeMap<String, ModuleInfo>,
    allowed: Option<&BTreeSet<String>>,
) -> BTreeMap<String, ModuleInfo> {
    let mut modules = BTreeMap::new();
    if allowed.is_some_and(BTreeSet::is_empty) {
        return modules;
    }
    let permits = |id: &str| allowed.is_none_or(|ids| ids.contains(id));
    let entries = match fs::read_dir(paths::MODULES_DIR) {
        Ok(entries) => entries,
        Err(error) => {
            logw!(
                "cannot list {}: {error}; keeping the previous module table",
                paths::MODULES_DIR
            );
            let mut retained = previous.clone();
            retained.retain(|id, _| permits(id));
            return retained;
        }
    };

    for entry in entries.flatten() {
        let file_name = entry.file_name();
        let Some(id) = file_name.to_str() else {
            continue;
        };
        if id.starts_with('.') || !permits(id) {
            continue;
        }
        let module_dir = PathBuf::from(paths::MODULES_DIR).join(id);
        if !is_enabled(&module_dir) {
            continue;
        }
        let declarations = match declarations(&module_dir) {
            Ok(declarations) => declarations,
            Err(error) if error.kind() == io::ErrorKind::NotFound => continue,
            Err(error) => {
                logw!(
                    "cannot read {}: {error}; keeping the previous record",
                    module_dir.join("zn_modules.txt").display()
                );
                if let Some(previous) = previous.get(id) {
                    modules.insert(previous.id.clone(), previous.clone());
                }
                continue;
            }
        };

        let mut module = ModuleInfo {
            id: id.to_owned(),
            ..ModuleInfo::default()
        };
        module.name = read_prop(&module_dir, "name");
        if module.name.is_empty() {
            module.name = module.id.clone();
        }
        module.version = read_prop(&module_dir, "version");
        module.declarations = declarations;
        module.targets = module
            .declarations
            .iter()
            .map(|declaration| declaration.target.clone())
            .collect();
        if module.targets.is_empty() {
            continue;
        }

        if let Some(previous) = previous.get(id) {
            module.processes = previous.processes.clone();
            module.processes.retain(|(pid, _)| procfs::is_alive(*pid));
            module.failed = previous.failed.clone();
        }
        modules.insert(module.id.clone(), module);
    }
    modules
}

pub fn flatten(modules: &BTreeMap<String, ModuleInfo>) -> Vec<Target> {
    let mut seen = BTreeSet::new();
    let mut targets = Vec::new();
    for module in modules.values() {
        for target in &module.targets {
            if seen.insert(target.clone()) {
                targets.push(target.clone());
            }
        }
    }
    targets
}

pub fn matches(targets: &[Target], exe: &str) -> bool {
    targets.iter().any(|target| target.matches(exe))
}

pub fn module_matches(module: &ModuleInfo, exe: &str) -> bool {
    !exe.is_empty() && matches(&module.targets, exe)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModuleLibrary {
    pub path: String,
    pub companion: bool,
}

pub fn module_library_path(module_dir: &Path, library: &str) -> io::Result<Option<PathBuf>> {
    let library = fs::canonicalize(module_dir.join(library))?;
    let module = fs::canonicalize(module_dir)?;
    Ok((library != module && library.starts_with(module)).then_some(library))
}

pub fn resolve_library(module_dir: &Path, library: &str) -> Option<String> {
    module_library_path(module_dir, library)
        .ok()
        .flatten()
        .map(|path| path.to_string_lossy().into_owned())
}

pub fn libraries_for(modules: &BTreeMap<String, ModuleInfo>, exe: &str) -> Vec<ModuleLibrary> {
    let mut libraries: Vec<ModuleLibrary> = Vec::new();
    let mut seen: BTreeMap<String, usize> = BTreeMap::new();
    for module in modules.values() {
        let module_dir = PathBuf::from(paths::MODULES_DIR).join(&module.id);
        for declaration in &module.declarations {
            if !declaration.target.matches(exe) {
                continue;
            }
            let Some(path) = resolve_library(&module_dir, &declaration.library) else {
                continue;
            };
            if let Some(index) = seen.get(&path).copied() {
                if declaration.companion {
                    libraries[index].companion = true;
                }
                continue;
            }
            seen.insert(path.clone(), libraries.len());
            libraries.push(ModuleLibrary {
                path,
                companion: declaration.companion,
            });
        }
    }
    libraries
}

impl Daemon {
    pub fn collect_targets(&mut self) {
        self.modules = match crate::safe_boot::allowed_modules() {
            Ok(allowed) => collect(&self.modules, allowed.as_ref()),
            Err(error) => {
                logw!("cannot read safe boot state: {error}; pausing ZNNext modules");
                BTreeMap::new()
            }
        };
        self.targets = flatten(&self.modules);
    }

    pub fn record_success(&mut self, pid: Pid, exe: &str) {
        let name = procfs::process_name(pid, exe);
        let mut changed = false;
        for module in self.modules.values_mut() {
            if !module_matches(module, exe) {
                continue;
            }
            let failures = module.failed.len();
            module.failed.retain(|failure| failure.name != name);
            if module.failed.len() != failures {
                changed = true;
            }
            if module.processes.iter().any(|(known, _)| *known == pid) {
                continue;
            }
            module.processes.push((pid, name.clone()));
            changed = true;
        }
        if changed {
            self.state.dirty = true;
        }
    }

    pub fn record_failure(&mut self, pid: Pid, exe: &str, reason: &str) {
        let name = procfs::process_name(pid, exe);
        let last_seen_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .ok()
            .and_then(|duration| u64::try_from(duration.as_millis()).ok());
        let mut changed = false;
        for module in self.modules.values_mut() {
            if !module_matches(module, exe) {
                continue;
            }
            if let Some(failure) = module
                .failed
                .iter_mut()
                .find(|failure| failure.name == name && failure.reason == reason)
            {
                if failure.pid == pid && failure.last_seen_ms == last_seen_ms {
                    continue;
                }
                failure.pid = pid;
                failure.last_seen_ms = last_seen_ms;
            } else {
                module.failed.push(ModuleFailure {
                    name: name.clone(),
                    reason: reason.to_owned(),
                    pid,
                    last_seen_ms,
                });
            }
            changed = true;
        }
        if changed {
            self.state.dirty = true;
        }
    }
}
