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

use std::collections::BTreeMap;
use std::{fs, io, path::Path};

use serde_json::{Value, json, value::RawValue};

use crate::targets::{declarations, read_prop};

enum ModuleIssue {
    Disabled,
    NoDeclarations,
}

impl ModuleIssue {
    fn mark(self, module: &mut Value) {
        match self {
            Self::Disabled => module["disabled"] = Value::Bool(true),
            Self::NoDeclarations => module["declarationIssue"] = json!("noDeclarations"),
        }
    }
}

pub fn include_diagnostics(live: &RawValue, root: &Path) -> io::Result<Option<Vec<Value>>> {
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
        let issue = if dir.join("disable").exists() {
            ModuleIssue::Disabled
        } else if declarations(&dir).is_ok_and(|entries| entries.is_empty()) {
            ModuleIssue::NoDeclarations
        } else {
            continue;
        };
        issues.insert(id, (dir, issue));
    }
    if issues.is_empty() {
        return Ok(None);
    }

    let mut modules: Vec<Value> = serde_json::from_str(live.get())?;
    for module in &mut modules {
        if let Some(id) = module.get("id").and_then(Value::as_str)
            && let Some((_, issue)) = issues.remove(id)
        {
            issue.mark(module);
        }
    }
    for (id, (dir, issue)) in issues {
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
