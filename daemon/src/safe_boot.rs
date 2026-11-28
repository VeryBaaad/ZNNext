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
use std::os::unix::fs::OpenOptionsExt;
use std::path::Path;
use std::{fs, io};

use serde::{Deserialize, Serialize};

use crate::{config, fileio, paths, targets};

const BOOT_ID: &str = "/proc/sys/kernel/random/boot_id";

#[derive(Default, PartialEq, Eq, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct State {
    boot: String,
    active: bool,
    armed: bool,
    enabled: BTreeSet<String>,
}

#[derive(Serialize)]
pub struct Report {
    active: bool,
    armed: bool,
    enabled: BTreeSet<String>,
    paused: BTreeSet<String>,
}

enum Action<'a> {
    Status,
    Arm,
    Cancel,
    Enable(&'a str),
}

fn valid_id(id: &str) -> bool {
    !id.is_empty() && !id.starts_with('.') && !id.contains(['/', '\0'])
}

fn valid_boot(id: &str) -> bool {
    id.len() == 36
        && id.bytes().enumerate().all(|(index, ch)| {
            if matches!(index, 8 | 13 | 18 | 23) {
                ch == b'-'
            } else {
                ch.is_ascii_digit() || (b'a'..=b'f').contains(&ch)
            }
        })
}

fn eligible(root: &Path, id: &str) -> io::Result<bool> {
    if !valid_id(id) {
        return Ok(false);
    }
    let dir = root.join(id);
    match fs::metadata(dir.join("zn_modules.txt")) {
        Ok(file) if file.is_file() => Ok(!dir.join("disable").try_exists()?
            && !dir.join("remove").try_exists()?
            && !targets::declarations(&dir)?.is_empty()),
        Ok(_) => Ok(false),
        Err(error)
            if matches!(
                error.kind(),
                io::ErrorKind::NotFound | io::ErrorKind::NotADirectory
            ) =>
        {
            Ok(false)
        }
        Err(error) => Err(error),
    }
}

fn load(path: &Path) -> io::Result<State> {
    let file = match fs::OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK)
        .open(path)
    {
        Ok(file) => file,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(State::default()),
        Err(error) => return Err(error),
    };
    if !file.metadata()?.is_file() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "invalid safe boot state file",
        ));
    }
    let state: State = serde_json::from_reader(io::BufReader::new(file))?;
    if !valid_boot(&state.boot)
        || (!state.active && (!state.armed || !state.enabled.is_empty()))
        || !state.enabled.iter().all(|id| valid_id(id))
    {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "invalid safe boot state",
        ));
    }
    Ok(state)
}

fn update(action: Action<'_>) -> io::Result<State> {
    let path = Path::new(paths::STATE_DIR).join("safe-boot.json");
    let observed = load(&path)?;
    if !matches!(action, Action::Arm) && observed.boot.is_empty() {
        return match action {
            Action::Enable(_) => Err(io::Error::other("safe boot is not active")),
            _ => Ok(State::default()),
        };
    }
    let boot = fs::read_to_string(BOOT_ID)?;
    let boot = boot.trim();
    if !valid_boot(boot) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "invalid boot ID",
        ));
    }
    if matches!(action, Action::Status) && observed.boot == boot {
        return Ok(observed);
    }
    let directory = config::lock_for_update()?;
    let previous = load(&path)?;
    let mut state = if previous.boot == boot {
        previous.clone()
    } else {
        State {
            boot: boot.to_owned(),
            active: previous.armed,
            ..State::default()
        }
    };
    let result = match action {
        Action::Status => Ok(()),
        Action::Arm => {
            state.armed = true;
            Ok(())
        }
        Action::Cancel => {
            state.armed = false;
            Ok(())
        }
        Action::Enable(_) if !state.active => Err(io::Error::other("safe boot is not active")),
        Action::Enable(id) => eligible(Path::new(paths::MODULES_DIR), id).and_then(|installed| {
            if !installed {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    format!("unavailable ZN module: {id}"),
                ));
            }
            state.enabled.insert(id.to_owned());
            Ok(())
        }),
    };
    if !state.active && !state.armed {
        state = State::default();
    }
    if previous != state {
        if state.active || state.armed {
            fileio::replace_file(&path, &serde_json::to_vec(&state)?)?;
        } else {
            fs::remove_file(&path)?;
        }
        directory.sync_all()?;
    }
    result.map(|()| state)
}

pub fn allowed_modules() -> io::Result<Option<BTreeSet<String>>> {
    let state = update(Action::Status)?;
    Ok(state.active.then_some(state.enabled))
}

pub fn command(args: &[String]) -> io::Result<Report> {
    let action = match args {
        [cmd] if cmd == "status" => Action::Status,
        [cmd] if cmd == "arm" => Action::Arm,
        [cmd] if cmd == "cancel" => Action::Cancel,
        [cmd, id] if cmd == "enable" && valid_id(id) => Action::Enable(id),
        _ => {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "usage: znnd --ctl safe-boot <status|arm|cancel|enable MODULE_ID>",
            ));
        }
    };
    let state = update(action)?;
    let mut paused = BTreeSet::new();
    if state.active {
        let root = Path::new(paths::MODULES_DIR);
        for entry in fs::read_dir(root)? {
            let entry = entry?;
            if let Some(id) = entry.file_name().to_str()
                && !state.enabled.contains(id)
                && eligible(root, id).unwrap_or(false)
            {
                paused.insert(id.to_owned());
            }
        }
    }
    Ok(Report {
        active: state.active,
        armed: state.armed,
        enabled: state.enabled,
        paused,
    })
}
