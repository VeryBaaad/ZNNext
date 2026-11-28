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
use std::io::{self, Read};
use std::os::unix::fs::OpenOptionsExt;
use std::path::Path;

use machikado_rs::{FileEntry, verify};

use crate::daemon::MODULE_ID;
use crate::mazoku::ORG_PUBLIC_KEY;

const SIGNED_FILES: [&str; 10] = [
    "bin/znnd",
    "emulated-soft-reboot.sh",
    "lib/libzn_loader.so",
    "lib64/libzn_loader.so",
    "mazoku",
    "module.prop.orig",
    "post-fs-data.sh",
    "late-load.sh",
    "sepolicy.rule",
    "uninstall.sh",
];

fn with_path(path: &str, error: io::Error) -> io::Error {
    io::Error::new(error.kind(), format!("{path}: {error}"))
}

fn read_file(path: &Path, relative_path: &str) -> io::Result<Vec<u8>> {
    let context = |error| with_path(relative_path, error);
    let mut file = fs::OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_NONBLOCK)
        .open(path)
        .map_err(context)?;
    if !file.metadata().map_err(context)?.is_file() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("{relative_path}: expected a regular file"),
        ));
    }
    let mut bytes = Vec::new();
    file.read_to_end(&mut bytes).map_err(context)?;
    Ok(bytes)
}

fn load_entry(path: &Path, relative_path: &str) -> io::Result<FileEntry> {
    Ok(FileEntry {
        relative_path: relative_path.to_owned(),
        content: read_file(path, relative_path)?,
    })
}

fn load_webroot(dir: &Path, prefix: &str, entries: &mut Vec<FileEntry>) -> io::Result<()> {
    let mut children = fs::read_dir(dir)
        .and_then(|directory| directory.collect::<io::Result<Vec<_>>>())
        .map_err(|error| with_path(prefix, error))?;
    children.sort_by_key(|child| child.file_name());

    for child in children {
        let relative_path = format!("{prefix}/{}", child.file_name().to_string_lossy());
        let file_type = child
            .file_type()
            .map_err(|error| with_path(&relative_path, error))?;
        if file_type.is_dir() {
            load_webroot(&child.path(), &relative_path, entries)?;
        } else if file_type.is_file() {
            entries.push(load_entry(&child.path(), &relative_path)?);
        }
    }
    Ok(())
}

fn version(contents: &[u8]) -> Option<String> {
    let value = std::str::from_utf8(contents)
        .ok()?
        .lines()
        .find_map(|line| line.strip_prefix("version="))?
        .trim();
    (!value.is_empty()).then(|| value.to_owned())
}

pub(crate) fn installed_version(module_dir: &Path) -> Option<String> {
    let contents = match read_file(&module_dir.join("module.prop.orig"), "module.prop.orig") {
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            read_file(&module_dir.join("module.prop"), "module.prop").ok()?
        }
        result => result.ok()?,
    };
    version(&contents)
}

pub(crate) fn verify_module(module_dir: &Path) -> io::Result<Option<String>> {
    let mut entries = Vec::new();
    for relative_path in SIGNED_FILES {
        entries.push(load_entry(&module_dir.join(relative_path), relative_path)?);
    }
    load_webroot(&module_dir.join("webroot"), "webroot", &mut entries)?;
    entries.sort_by(|a, b| a.relative_path.cmp(&b.relative_path));

    let contents = |path: &str| {
        entries
            .iter()
            .find(|entry| entry.relative_path == path)
            .map(|entry| entry.content.as_slice())
    };
    let machikado = read_file(&module_dir.join("machikado"), "machikado")?;
    let mazoku =
        contents("mazoku").ok_or_else(|| io::Error::other("missing signed mazoku entry"))?;
    let (ok, error) = verify(&machikado, mazoku, &entries, MODULE_ID, &ORG_PUBLIC_KEY);
    if !ok {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            error.map_or_else(
                || "signature verification failed".to_owned(),
                |error| error.to_string(),
            ),
        ));
    }
    Ok(contents("module.prop.orig").and_then(version))
}

pub(crate) fn restore_prop(module_dir: &Path) {
    let original = module_dir.join("module.prop.orig");
    if original.is_file() {
        let _ = fs::copy(&original, module_dir.join("module.prop"));
    }
}
