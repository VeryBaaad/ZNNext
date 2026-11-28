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

use crate::targets::read_prop;

pub fn include_disabled(live: &RawValue, root: &Path) -> io::Result<Option<Vec<Value>>> {
    let mut disabled = BTreeMap::new();
    for entry in fs::read_dir(root)? {
        let entry = entry?;
        let Ok(id) = entry.file_name().into_string() else {
            continue;
        };
        let dir = entry.path();
        if !id.starts_with('.')
            && dir.join("disable").exists()
            && !dir.join("remove").exists()
            && dir.join("zn_modules.txt").is_file()
        {
            disabled.insert(id, dir);
        }
    }
    if disabled.is_empty() {
        return Ok(None);
    }

    let mut modules: Vec<Value> = serde_json::from_str(live.get())?;
    for module in &mut modules {
        if let Some(id) = module.get("id").and_then(Value::as_str)
            && disabled.remove(id).is_some()
        {
            module["disabled"] = Value::Bool(true);
        }
    }
    for (id, dir) in disabled {
        let (name, version) = if dir.join("module.prop").is_file() {
            (read_prop(&dir, "name"), read_prop(&dir, "version"))
        } else {
            (String::new(), String::new())
        };
        modules.push(json!({
            "id": id,
            "name": if name.is_empty() { &id } else { &name },
            "version": version,
            "disabled": true,
            "processes": [],
            "failed": [],
        }));
    }
    Ok(Some(modules))
}
