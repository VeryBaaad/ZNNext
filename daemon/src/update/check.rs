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

use std::fs;
use std::io;
use std::path::Path;

use nix::unistd::{AccessFlags, access};
use serde::Serialize;

use super::{DAEMON_FILE, MODULE_ID};
use crate::{config, machikado, paths};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Report {
    hot_update_enabled: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    installed_version: Option<String>,
    #[serde(flatten)]
    stage: Stage,
}

#[derive(Serialize)]
#[serde(
    tag = "status",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
enum Stage {
    NotStaged,
    Disabled,
    Removed,
    Incomplete {
        detail: String,
    },
    Unreadable {
        detail: String,
    },
    VerificationFailed {
        detail: String,
    },
    NotExecutable {
        #[serde(skip_serializing_if = "Option::is_none")]
        staged_version: Option<String>,
        detail: String,
    },
    Ready {
        #[serde(skip_serializing_if = "Option::is_none")]
        staged_version: Option<String>,
    },
}

impl Stage {
    fn file_error(error: io::Error) -> Self {
        let detail = error.to_string();
        match error.kind() {
            io::ErrorKind::NotFound
            | io::ErrorKind::NotADirectory
            | io::ErrorKind::IsADirectory
            | io::ErrorKind::InvalidInput => Self::Incomplete { detail },
            _ => Self::Unreadable { detail },
        }
    }
}

fn staged_package(dir: &Path) -> io::Result<Stage> {
    match fs::metadata(dir) {
        Ok(metadata) if metadata.is_dir() => (),
        Ok(_) => {
            return Ok(Stage::Incomplete {
                detail: "staging path is not a directory".to_owned(),
            });
        }
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(Stage::NotStaged),
        Err(error) => return Err(error),
    }
    if dir.join("remove").try_exists()? {
        return Ok(Stage::Removed);
    }
    if dir.join("disable").try_exists()? {
        return Ok(Stage::Disabled);
    }

    let staged_version = match machikado::verify_module(dir) {
        Ok(version) => version,
        Err(error) if error.kind() == io::ErrorKind::InvalidData => {
            return Ok(Stage::VerificationFailed {
                detail: error.to_string(),
            });
        }
        Err(error) => return Err(error),
    };
    match access(dir.join(DAEMON_FILE).as_path(), AccessFlags::X_OK) {
        Ok(()) => Ok(Stage::Ready { staged_version }),
        Err(nix::errno::Errno::EACCES) => Ok(Stage::NotExecutable {
            staged_version,
            detail: format!("{DAEMON_FILE} lacks execute permission"),
        }),
        Err(error) => Err(io::Error::new(
            io::Error::from(error).kind(),
            format!("{DAEMON_FILE}: {error}"),
        )),
    }
}

pub(super) fn inspect(installed: &Path, staged: &Path, hot_update_enabled: bool) -> Report {
    Report {
        hot_update_enabled,
        installed_version: machikado::installed_version(installed),
        stage: staged_package(staged).unwrap_or_else(Stage::file_error),
    }
}

pub(crate) fn report() -> Report {
    inspect(
        &Path::new(paths::MODULES_DIR).join(MODULE_ID),
        &Path::new(paths::MODULES_UPDATE_DIR).join(MODULE_ID),
        config::effective().hot_update == config::HOT_UPDATE_TRUE,
    )
}
