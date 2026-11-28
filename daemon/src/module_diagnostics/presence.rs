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

use std::collections::BTreeSet;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::path::Path;
use std::{fs, io};

use crate::procfs::{basename, strip_deleted_suffix};
use crate::targets::{Declaration, Target};

pub(crate) fn declaration_fingerprint(declarations: &[Declaration]) -> u64 {
    let mut hasher = DefaultHasher::new();
    declarations.hash(&mut hasher);
    hasher.finish()
}

#[derive(Default)]
pub(super) struct ProcessSnapshot {
    paths: BTreeSet<String>,
    names: BTreeSet<String>,
}

impl ProcessSnapshot {
    pub(super) fn read(proc_root: &Path) -> io::Result<Self> {
        let mut snapshot = Self::default();
        for entry in fs::read_dir(proc_root)? {
            let entry = entry?;
            if entry
                .file_name()
                .as_encoded_bytes()
                .iter()
                .any(|ch| !ch.is_ascii_digit())
            {
                continue;
            }
            let path = match fs::read_link(entry.path().join("exe")) {
                Ok(path) => strip_deleted_suffix(path.to_string_lossy().into_owned()),
                Err(error) if matches!(error.raw_os_error(), Some(libc::ENOENT | libc::ESRCH)) => {
                    if has_other_threads(&entry)? {
                        return Err(error);
                    }
                    continue;
                }
                Err(error) => return Err(error),
            };
            snapshot.names.insert(basename(&path).to_owned());
            snapshot.paths.insert(path);
        }
        if snapshot.paths.is_empty() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "no visible process executables",
            ));
        }
        Ok(snapshot)
    }

    pub(super) fn absent(&self, target: &Target) -> bool {
        if target.by_name {
            !target.value.is_empty()
                && !target.value.contains('/')
                && !self.names.contains(&target.value)
        } else {
            Path::new(&target.value).is_absolute() && !self.paths.contains(&target.value)
        }
    }
}

fn has_other_threads(process: &fs::DirEntry) -> io::Result<bool> {
    let tasks = match fs::read_dir(process.path().join("task")) {
        Ok(tasks) => tasks,
        Err(_) if !process.path().try_exists()? => return Ok(false),
        Err(error) => return Err(error),
    };
    let leader = process.file_name();
    for task in tasks {
        if task?.file_name() != leader {
            return Ok(true);
        }
    }
    Ok(false)
}
