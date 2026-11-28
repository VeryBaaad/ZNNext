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
use std::io::{self, Read};

use serde::Serialize;

use crate::config::{self, HookConfig};
use crate::paths::CONFIG_FILE;

#[derive(Serialize)]
pub struct ConfigDiff {
    changes: BTreeMap<&'static str, Change>,
}

#[derive(Serialize)]
struct Change {
    from: Option<String>,
    to: Option<String>,
}

pub fn preview(arguments: &[String]) -> Result<ConfigDiff, String> {
    let mut stdin = String::new();
    let json = match arguments {
        [] => {
            io::stdin()
                .read_to_string(&mut stdin)
                .map_err(|error| format!("cannot read configuration from stdin: {error}"))?;
            &stdin
        }
        [json] => json,
        _ => return Err("usage: znnd --ctl config-diff [JSON]".into()),
    };
    if !json.trim_start().starts_with('{') {
        return Err("expected a configuration JSON object".into());
    }
    let proposed: HookConfig =
        serde_json::from_str(json).map_err(|error| format!("invalid configuration: {error}"))?;
    let current =
        config::for_export().map_err(|error| format!("cannot read {CONFIG_FILE}: {error}"))?;
    let fields = [
        (
            "inline",
            Some(current.inline_hook),
            Some(proposed.inline_hook),
        ),
        ("inline32", current.inline_hook32, proposed.inline_hook32),
        ("plt", Some(current.plt_hook), Some(proposed.plt_hook)),
        ("plt32", current.plt_hook32, proposed.plt_hook32),
        ("mode", Some(current.mode), Some(proposed.mode)),
        (
            "debugdata",
            Some(current.debugdata),
            Some(proposed.debugdata),
        ),
        (
            "disable-zn",
            Some(current.disable_zn),
            Some(proposed.disable_zn),
        ),
        (
            "hot-update",
            Some(current.hot_update),
            Some(proposed.hot_update),
        ),
    ];
    let mut changes = BTreeMap::new();
    for (key, from, to) in fields {
        if from == to {
            continue;
        }
        if let Some(value) = &to {
            let options = HookConfig::options(key)
                .ok_or_else(|| format!("this device has no secondary 32-bit ABI for {key}"))?;
            if !config::option_in(&options, value) {
                return Err(format!("invalid {key} value {value:?} for this device"));
            }
        }
        changes.insert(key, Change { from, to });
    }
    Ok(ConfigDiff { changes })
}
