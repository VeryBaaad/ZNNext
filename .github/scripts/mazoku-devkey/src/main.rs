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

use std::env;
use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};

use machikado_rs::{generate_keypair, sign_mazoku};

const MODULE_ID: &str = "zygisknextsu";

fn main() -> Result<(), Box<dyn Error>> {
    let root = repository_root()?;
    let module_dir = root.join("module");

    let org_key_pair = generate_keypair();
    let member_key_pair = generate_keypair();

    let private_key_path = module_dir.join("private_key");
    fs::write(&private_key_path, member_key_pair.private_key)?;
    println!("wrote {}", private_key_path.display());

    let mazoku = sign_mazoku(
        MODULE_ID,
        &member_key_pair.public_key,
        &org_key_pair.private_key,
    )?;
    let mazoku_path = module_dir.join("src/mazoku");
    fs::write(&mazoku_path, mazoku.as_bytes())?;
    println!("wrote {}", mazoku_path.display());
    Ok(())
}

fn repository_root() -> Result<PathBuf, Box<dyn Error>> {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    manifest_dir
        .ancestors()
        .find(|dir| dir.join("module/src").is_dir() && dir.join("daemon/Cargo.toml").is_file())
        .map(Path::to_path_buf)
        .ok_or_else(|| {
            format!(
                "cannot locate the repository root from {}",
                manifest_dir.display()
            )
            .into()
        })
}
