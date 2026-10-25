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

use serde::Serialize;

use crate::paths;
use crate::system::runtime_abi;

pub const MODE_AUTO: &str = "auto";
/// Follow kernel fork/exec/exit events.
pub const MODE_KERNEL: &str = "kernel";
/// Trace init and follow every fork.
pub const MODE_PTRACE: &str = "ptrace";
/// Poll `/proc` and never trace init.
pub const MODE_PROC: &str = "proc";
/// Never trace init: seize targets late and defer to another tracer.
pub const MODE_LATE: &str = "late";

pub const DEBUGDATA_LZMA: &str = "lzma";
pub const DEBUGDATA_XZ: &str = "xz";

pub const DISABLE_ZN_TRUE: &str = "true";
pub const DISABLE_ZN_FALSE: &str = "false";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HookConfig {
    pub inline_hook: String,
    pub plt_hook: String,
    pub mode: String,
    pub debugdata: String,
    pub disable_zn: String,
}

#[derive(Debug, Serialize)]
pub struct HookOption {
    value: String,
    options: Vec<&'static str>,
}

#[derive(Debug, Serialize)]
pub struct HookConfigReport {
    #[serde(rename = "inlineHook")]
    inline_hook: HookOption,
    #[serde(rename = "pltHook")]
    plt_hook: HookOption,
    mode: HookOption,
    debugdata: HookOption,
    #[serde(rename = "disableZn")]
    disable_zn: HookOption,
}

pub fn inline_hook_options() -> Vec<&'static str> {
    match runtime_abi() {
        "arm64-v8a" => vec![
            "dobby",
            "shadowhook",
            "and64inlinehook",
            "frida-gum",
            "hookzz",
            "disable",
        ],
        "armeabi-v7a" => vec!["dobby", "shadowhook", "frida-gum", "hookzz", "disable"],
        "x86_64" => vec!["dobby", "frida-gum", "hookzz", "disable"],
        "riscv64" => vec!["rv64hook", "disable"],
        _ => vec!["dobby", "frida-gum", "disable"],
    }
}

pub fn plt_hook_options() -> Vec<&'static str> {
    if runtime_abi() == "riscv64" {
        vec!["lsplt", "plti", "disable"]
    } else {
        vec!["lsplt", "plti", "bytehook", "xhook", "plthook", "disable"]
    }
}

pub fn tracking_mode_options() -> Vec<&'static str> {
    vec![MODE_AUTO, MODE_KERNEL, MODE_PTRACE, MODE_PROC, MODE_LATE]
}

pub fn debugdata_options() -> Vec<&'static str> {
    vec![DEBUGDATA_LZMA, DEBUGDATA_XZ]
}

pub fn disable_zn_options() -> Vec<&'static str> {
    vec![DISABLE_ZN_TRUE, DISABLE_ZN_FALSE]
}

pub fn option_in(options: &[&str], value: &str) -> bool {
    options.contains(&value)
}

impl HookConfig {
    pub fn options(kind: &str) -> Option<Vec<&'static str>> {
        match kind {
            "inline" => Some(inline_hook_options()),
            "plt" => Some(plt_hook_options()),
            "mode" => Some(tracking_mode_options()),
            "debugdata" => Some(debugdata_options()),
            "disable-zn" => Some(disable_zn_options()),
            _ => None,
        }
    }

    pub fn set(&mut self, kind: &str, value: &str) {
        match kind {
            "inline" => self.inline_hook = value.to_owned(),
            "plt" => self.plt_hook = value.to_owned(),
            "mode" => self.mode = value.to_owned(),
            "debugdata" => self.debugdata = value.to_owned(),
            "disable-zn" => self.disable_zn = value.to_owned(),
            _ => {}
        }
    }

    pub fn report(&self) -> HookConfigReport {
        HookConfigReport {
            inline_hook: HookOption {
                value: self.inline_hook.clone(),
                options: inline_hook_options(),
            },
            plt_hook: HookOption {
                value: self.plt_hook.clone(),
                options: plt_hook_options(),
            },
            mode: HookOption {
                value: self.mode.clone(),
                options: tracking_mode_options(),
            },
            debugdata: HookOption {
                value: self.debugdata.clone(),
                options: debugdata_options(),
            },
            disable_zn: HookOption {
                value: self.disable_zn.clone(),
                options: disable_zn_options(),
            },
        }
    }
}

pub fn effective() -> HookConfig {
    let inline_options = inline_hook_options();
    let plt_options = plt_hook_options();
    let mode_options = tracking_mode_options();
    let debugdata_choices = debugdata_options();
    let disable_zn_choices = disable_zn_options();
    let mut config = HookConfig {
        inline_hook: inline_options
            .first()
            .copied()
            .unwrap_or_default()
            .to_owned(),
        plt_hook: plt_options.first().copied().unwrap_or_default().to_owned(),
        mode: MODE_AUTO.to_owned(),
        debugdata: debugdata_choices
            .first()
            .copied()
            .unwrap_or_default()
            .to_owned(),
        disable_zn: DISABLE_ZN_TRUE.to_owned(),
    };

    let Some(stored) = read_file() else {
        return config;
    };
    if option_in(&inline_options, &stored.inline_hook) {
        config.inline_hook = stored.inline_hook;
    }
    if option_in(&plt_options, &stored.plt_hook) {
        config.plt_hook = stored.plt_hook;
    }
    if option_in(&mode_options, &stored.mode) {
        config.mode = stored.mode;
    }
    if option_in(&debugdata_choices, &stored.debugdata) {
        config.debugdata = stored.debugdata;
    }
    if option_in(&disable_zn_choices, &stored.disable_zn) {
        config.disable_zn = stored.disable_zn;
    }
    config
}

pub fn write(config: &HookConfig) -> bool {
    let _ = fs::create_dir_all(paths::STATE_DIR);
    let contents = format!(
        "inline_hook={}\nplt_hook={}\nmode={}\ndebugdata={}\ndisable_zn={}\n",
        config.inline_hook, config.plt_hook, config.mode, config.debugdata, config.disable_zn
    );
    crate::fileio::replace_file(paths::CONFIG_FILE.as_ref(), contents.as_bytes()).is_ok()
}

struct StoredConfig {
    inline_hook: String,
    plt_hook: String,
    mode: String,
    debugdata: String,
    disable_zn: String,
}

fn read_file() -> Option<StoredConfig> {
    let contents = fs::read_to_string(paths::CONFIG_FILE).ok()?;
    let mut stored = StoredConfig {
        inline_hook: String::new(),
        plt_hook: String::new(),
        mode: String::new(),
        debugdata: String::new(),
        disable_zn: String::new(),
    };

    for line in contents.lines() {
        let line = line.trim_end_matches(['\n', '\r']);
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        if key.is_empty() {
            continue;
        }
        match key {
            "inline_hook" => stored.inline_hook = value.to_owned(),
            "plt_hook" => stored.plt_hook = value.to_owned(),
            "mode" => stored.mode = value.to_owned(),
            "debugdata" => stored.debugdata = value.to_owned(),
            "disable_zn" => stored.disable_zn = value.to_owned(),
            _ => {}
        }
    }
    Some(stored)
}
