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

use std::{fs, io};

use serde::{Deserialize, Serialize};

use super::{HookConfig, effective_from, file_contents, parse_file};
use crate::paths;

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Record<T> {
    before: T,
    after: T,
}

pub(super) fn append(
    contents: &mut String,
    before: &HookConfig,
    after: &HookConfig,
) -> io::Result<()> {
    let record = serde_json::to_string(&Record { before, after })?;
    contents.push_str("undo=");
    contents.push_str(&record);
    contents.push('\n');
    Ok(())
}

pub fn read() -> Result<(HookConfig, HookConfig), String> {
    let no_change = "no configuration change to undo";
    let contents = match fs::read_to_string(paths::CONFIG_FILE) {
        Ok(contents) => contents,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Err(no_change.into()),
        Err(error) => return Err(format!("cannot read {}: {error}", paths::CONFIG_FILE)),
    };
    let (settings, history) = contents.split_once("\nundo=").ok_or(no_change)?;
    let record: Record<HookConfig> = serde_json::from_str(history)
        .map_err(|error| format!("invalid configuration undo record: {error}"))?;
    let current = effective_from(Some(parse_file(settings)));
    if current != record.after || file_contents(&record.after).trim_end_matches('\n') != settings {
        return Err("configuration has changed since the undo record was saved".into());
    }
    Ok((current, record.before))
}
