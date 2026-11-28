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
use std::io::{self, BufRead, BufReader};
use std::process::{Command, Stdio};

use crate::config::{inline_hook_options_for, plt_hook_options_for};

type Selection = BTreeMap<&'static str, String>;
type Report = BTreeMap<String, Selection>;

fn selection(line: &str) -> Option<(&str, Selection)> {
    let record = line.strip_prefix("hook engines: selected ")?;
    let fields = record.split_once("; ").map_or(record, |(fields, _)| fields);
    let (abi, backends) = fields.split_once(' ')?;
    if !matches!(
        abi,
        "arm64-v8a" | "armeabi-v7a" | "armeabi" | "x86_64" | "x86" | "riscv64" | "mips" | "mips64"
    ) {
        return None;
    }
    let mut selection = Selection::new();
    for field in backends.split(' ') {
        let (kind, value) = field.split_once('=')?;
        let (key, choices) = match kind {
            "inline" => ("inline", inline_hook_options_for(abi)),
            "plt" => ("plt", plt_hook_options_for(abi)),
            _ => return None,
        };
        if !choices.contains(&value) || selection.insert(key, value.to_owned()).is_some() {
            return None;
        }
    }
    Some((abi, selection))
}

fn collect(reader: impl BufRead) -> io::Result<Report> {
    let mut report = Report::new();
    for line in reader.split(b'\n') {
        let bytes = line?;
        if let Ok(line) = std::str::from_utf8(&bytes)
            && let Some((abi, selection)) = selection(line)
        {
            report.entry(abi.to_owned()).or_default().extend(selection);
        }
    }
    Ok(report)
}

pub(crate) fn report() -> io::Result<Report> {
    let mut child = Command::new("/system/bin/logcat")
        .args(["-b", "main", "-d", "-v", "raw", "ZNNloader:I", "*:S"])
        .stdout(Stdio::piped())
        .spawn()
        .map_err(|error| io::Error::other(format!("cannot start logcat: {error}")))?;
    let stdout = child.stdout.take().expect("logcat stdout is piped");
    let report = collect(BufReader::new(stdout));
    let status = child.wait()?;
    if !status.success() {
        return Err(io::Error::other(format!("logcat failed: {status}")));
    }
    report
}

#[cfg(test)]
mod tests {
    use super::collect;

    #[test]
    fn fallbacks_only_replace_the_reported_engine_for_that_abi() {
        let log = concat!(
            "hook engines: selected arm64-v8a inline=shadowhook plt=lsplt\n",
            "hook engines: selected armeabi-v7a inline=disable plt=xhook\n",
            "hook engines: selected arm64-v8a inline=dobby; initialization failed\n",
            "hook engines: selected x86 plt=xhook; initialization failed\n",
        );
        let report = collect(log.as_bytes()).unwrap();
        assert_eq!(report["arm64-v8a"]["inline"], "dobby");
        assert_eq!(report["arm64-v8a"]["plt"], "lsplt");
        assert_eq!(report["armeabi-v7a"]["inline"], "disable");
        assert_eq!(report["armeabi-v7a"]["plt"], "xhook");
        assert_eq!(report["x86"].len(), 1);
        assert_eq!(report["x86"]["plt"], "xhook");
    }

    #[test]
    fn ignores_unattributed_and_malformed_selections() {
        let log = concat!(
            "hook engines: inline=dobby plt=lsplt\n",
            "hook engines: selected unknown inline=dobby\n",
            "hook engines: selected mips plt=xhook\n",
            "hook engines: selected x86 inline=dobby plt=/private/path\n",
        );
        assert!(collect(log.as_bytes()).unwrap().is_empty());
    }
}
