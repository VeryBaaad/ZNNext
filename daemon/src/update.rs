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

use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};

use crate::daemon::MODULE_ID;
use crate::log::{logi, logw};
use crate::paths;
use crate::sys::fs as raw_fs;
use crate::targets;

const LOADER_FILE: &str = "libzn_loader.so";
const DAEMON_FILE: &str = "bin/znnd";

#[derive(Debug, Clone)]
pub struct PendingUpdate {
    pub dir: PathBuf,
    pub version: String,
    pub installed_version: String,
}

fn pending_dir() -> Option<PathBuf> {
    let dir = Path::new(paths::MODULES_UPDATE_DIR).join(MODULE_ID);
    if !dir.is_dir() || !raw_fs::access(&dir, raw_fs::F_OK) {
        return None;
    }
    if raw_fs::access(dir.join("disable"), raw_fs::F_OK)
        || raw_fs::access(dir.join("remove"), raw_fs::F_OK)
    {
        return None;
    }
    Some(dir)
}

pub fn executable(pending: &PendingUpdate) -> PathBuf {
    pending.dir.join(DAEMON_FILE)
}

fn installed_version() -> String {
    targets::read_prop(&Path::new(paths::MODULES_DIR).join(MODULE_ID), "version")
}

pub fn pending() -> Option<PendingUpdate> {
    let dir = pending_dir()?;
    if !raw_fs::access(dir.join(DAEMON_FILE), raw_fs::R_OK) {
        return None;
    }
    let loader32 = raw_fs::access(dir.join("lib").join(LOADER_FILE), raw_fs::R_OK);
    let loader64 = raw_fs::access(dir.join("lib64").join(LOADER_FILE), raw_fs::R_OK);
    if !loader32 && !loader64 {
        return None;
    }
    Some(PendingUpdate {
        version: targets::read_prop(&dir, "version"),
        installed_version: installed_version(),
        dir,
    })
}

pub fn running_from(dir: &Path) -> bool {
    let Ok(current) = std::fs::metadata("/proc/self/exe") else {
        return false;
    };
    let Ok(staged) = std::fs::metadata(dir.join(DAEMON_FILE)) else {
        return false;
    };
    current.dev() == staged.dev() && current.ino() == staged.ino()
}

pub fn fingerprint(dir: &Path) -> String {
    let path = dir.join("machikado");
    let Ok(metadata) = std::fs::metadata(&path) else {
        return String::new();
    };
    let modified = metadata
        .modified()
        .ok()
        .and_then(|time| time.duration_since(std::time::UNIX_EPOCH).ok())
        .map_or(0, |elapsed| elapsed.as_nanos());
    format!("{modified}:{}", metadata.len())
}

pub fn log_status() {
    let Some(dir) = pending_dir() else {
        return;
    };
    match pending() {
        Some(pending) => logi!(
            "hot update: {} holds version \"{}\" (installed \"{}\")",
            dir.display(),
            pending.version,
            pending.installed_version
        ),
        None => logw!(
            "hot update: {} exists but is incomplete; it is ignored",
            dir.display()
        ),
    }
}
