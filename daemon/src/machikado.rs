/*
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
use std::path::Path;

use machikado_rs::{FileEntry, verify};

use crate::daemon::MODULE_ID;

const ORG_PUBLIC_KEY: [u8; 32] = [
    123, 192, 97, 178, 44, 86, 95, 77, 138, 95, 46, 239, 113, 227, 151, 200, 0, 120, 148, 124, 95,
    104, 121, 88, 173, 152, 219, 116, 71, 200, 239, 190,
];

const SIGNED_FILES: [&str; 9] = [
    "bin/znnd",
    "emulated-soft-reboot.sh",
    "lib/libloader.so",
    "lib64/libloader.so",
    "mazoku",
    "module.prop.orig",
    "post-fs-data.sh",
    "late-load.sh",
    "sepolicy.rule",
];

fn load_entry(path: &Path, relative_path: &str) -> Result<FileEntry, String> {
    let content = fs::read(path).map_err(|error| error.to_string())?;
    Ok(FileEntry {
        relative_path: relative_path.to_owned(),
        content,
    })
}

fn load_webroot(dir: &Path, prefix: &str, entries: &mut Vec<FileEntry>) -> Result<(), String> {
    let mut children = Vec::new();
    for child in fs::read_dir(dir).map_err(|error| error.to_string())? {
        children.push(child.map_err(|error| error.to_string())?);
    }
    children.sort_by_key(|child| child.file_name());

    for child in children {
        let file_type = child.file_type().map_err(|error| error.to_string())?;
        let relative_path = format!("{prefix}/{}", child.file_name().to_string_lossy());
        if file_type.is_dir() {
            load_webroot(&child.path(), &relative_path, entries)?;
        } else if file_type.is_file() {
            entries.push(load_entry(&child.path(), &relative_path)?);
        }
    }
    Ok(())
}

pub(crate) fn verify_module(module_dir: &Path) -> Result<(), String> {
    let mut entries = Vec::new();
    for relative_path in SIGNED_FILES {
        entries.push(load_entry(&module_dir.join(relative_path), relative_path)?);
    }
    load_webroot(&module_dir.join("webroot"), "webroot", &mut entries)?;
    entries.sort_by(|a, b| a.relative_path.cmp(&b.relative_path));

    let machikado = fs::read(module_dir.join("machikado")).map_err(|error| error.to_string())?;
    let mazoku = fs::read(module_dir.join("mazoku")).map_err(|error| error.to_string())?;

    let (ok, error) = verify(&machikado, &mazoku, &entries, MODULE_ID, &ORG_PUBLIC_KEY);
    if ok {
        Ok(())
    } else {
        Err(error.map_or_else(
            || "signature verification failed".to_owned(),
            |error| error.to_string(),
        ))
    }
}

pub(crate) fn restore_prop(module_dir: &Path) {
    let original = module_dir.join("module.prop.orig");
    if original.is_file() {
        let _ = fs::copy(&original, module_dir.join("module.prop"));
    }
}
