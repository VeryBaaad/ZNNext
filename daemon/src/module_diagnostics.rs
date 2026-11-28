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

mod presence;

use std::cell::LazyCell;
use std::collections::{BTreeMap, BTreeSet};
use std::{fs, io, path::Path};

use serde_json::{Value, json, value::RawValue};

use crate::targets::{declarations, module_library_path, read_prop};
use presence::ProcessSnapshot;
pub(crate) use presence::declaration_fingerprint;

enum ModuleDiagnostic {
    Disabled,
    NoDeclarations,
    MissingLibraries(Vec<String>),
    WaitingForTarget,
}

impl ModuleDiagnostic {
    fn mark(self, module: &mut Value) {
        match self {
            Self::Disabled => module["disabled"] = Value::Bool(true),
            Self::WaitingForTarget => module["waitingForTarget"] = Value::Bool(true),
            Self::NoDeclarations => module["declarationIssue"] = json!("noDeclarations"),
            Self::MissingLibraries(paths) => {
                module["declarationIssue"] = json!("missingLibraries");
                module["missingLibraries"] = json!(paths);
            }
        }
    }
}

fn library_present(dir: &Path, library: &str) -> io::Result<bool> {
    let file =
        module_library_path(dir, library).and_then(|path| path.map(fs::metadata).transpose());
    match file {
        Ok(metadata) => Ok(metadata.is_some_and(|metadata| metadata.is_file())),
        Err(error)
            if matches!(
                error.kind(),
                io::ErrorKind::NotFound | io::ErrorKind::NotADirectory
            ) =>
        {
            Ok(false)
        }
        Err(error) => Err(error),
    }
}

fn module_diagnostic<'a>(
    dir: &Path,
    fingerprint: Option<&u64>,
    snapshot: impl FnOnce() -> Option<&'a ProcessSnapshot>,
) -> Option<ModuleDiagnostic> {
    if dir.join("disable").try_exists().ok()? {
        return Some(ModuleDiagnostic::Disabled);
    }
    let entries = declarations(dir).ok()?;
    if entries.is_empty() {
        return Some(ModuleDiagnostic::NoDeclarations);
    }
    let libraries: BTreeSet<_> = entries.iter().map(|entry| &entry.library).collect();
    let mut verified = true;
    let mut missing = Vec::new();
    for library in libraries {
        match library_present(dir, library) {
            Ok(true) => (),
            Ok(false) => missing.push(library.clone()),
            Err(_) => verified = false,
        }
    }
    if !missing.is_empty() {
        return Some(ModuleDiagnostic::MissingLibraries(missing));
    }
    if !verified || *fingerprint? != declaration_fingerprint(&entries) {
        return None;
    }
    let snapshot = snapshot()?;
    entries
        .iter()
        .all(|entry| snapshot.absent(&entry.target))
        .then_some(ModuleDiagnostic::WaitingForTarget)
}

pub fn include_diagnostics(
    live: &RawValue,
    idle_declarations: &BTreeMap<String, u64>,
    root: &Path,
    proc_root: &Path,
) -> io::Result<Option<Vec<Value>>> {
    let snapshot = LazyCell::new(|| ProcessSnapshot::read(proc_root).ok());
    let mut issues = BTreeMap::new();
    for entry in fs::read_dir(root)? {
        let entry = entry?;
        let Ok(id) = entry.file_name().into_string() else {
            continue;
        };
        let dir = entry.path();
        if id.starts_with('.')
            || dir.join("remove").exists()
            || !dir.join("zn_modules.txt").is_file()
        {
            continue;
        }
        if let Some(issue) =
            module_diagnostic(&dir, idle_declarations.get(&id), || snapshot.as_ref())
        {
            issues.insert(id, (dir, issue));
        }
    }
    if issues.is_empty() {
        return Ok(None);
    }

    let mut modules: Vec<Value> = serde_json::from_str(live.get())?;
    for module in &mut modules {
        if let Some(id) = module.get("id").and_then(Value::as_str)
            && let Some((_, issue)) = issues.remove(id)
            && (!matches!(issue, ModuleDiagnostic::WaitingForTarget)
                || ["processes", "failed"]
                    .iter()
                    .all(|key| module[*key].as_array().is_some_and(Vec::is_empty)))
        {
            issue.mark(module);
        }
    }
    for (id, (dir, issue)) in issues {
        if matches!(issue, ModuleDiagnostic::WaitingForTarget) {
            continue;
        }
        let (name, version) = if dir.join("module.prop").is_file() {
            (read_prop(&dir, "name"), read_prop(&dir, "version"))
        } else {
            (String::new(), String::new())
        };
        let mut module = json!({
            "id": id,
            "name": if name.is_empty() { &id } else { &name },
            "version": version,
            "processes": [],
            "failed": [],
        });
        issue.mark(&mut module);
        modules.push(module);
    }
    Ok(Some(modules))
}
