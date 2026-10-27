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
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

use crate::log::{loge, logi, logw};
use crate::paths;
use crate::targets;

const ZN_MODULE_DIR: &str = "/data/adb/modules/zygisksu";

const ZN_CLI: [&str; 3] = ["bin/zygiskd", "bin/zygiskd64", "bin/zygiskd32"];

const DAEMON_WAIT: Duration = Duration::from_secs(30);
const RETRY_INTERVAL: Duration = Duration::from_millis(200);
const SETTLE_AFTER_DAEMON: Duration = Duration::from_millis(500);
const PROBE_TIMEOUT: Duration = Duration::from_secs(2);

pub fn cli_path() -> Option<PathBuf> {
    let root = Path::new(ZN_MODULE_DIR);
    for relative in ZN_CLI {
        let candidate = root.join(relative);
        if candidate.is_file() {
            return Some(candidate);
        }
    }
    None
}

struct ZnIdentity {
    id: String,
    name: String,
    version: String,
    author: String,
    update_json: String,
}

fn installed_identity() -> ZnIdentity {
    let directory = Path::new(ZN_MODULE_DIR);
    ZnIdentity {
        id: targets::read_prop(directory, "id"),
        name: targets::read_prop(directory, "name"),
        version: targets::read_prop(directory, "version"),
        author: targets::read_prop(directory, "author"),
        update_json: targets::read_prop(directory, "updateJson"),
    }
}

fn is_original_zygisk_next(identity: &ZnIdentity) -> bool {
    identity.id == "zygisksu"
        && identity.name == "Zygisk Next"
        && (identity.author.contains("5ec1cff")
            || identity.author.contains("Nullptr")
            || identity.author.contains("LSPosed"))
        && identity.update_json.contains("zygisk-next")
}

fn identity_is_original() -> bool {
    let identity = installed_identity();
    logi!(
        "zn modules: zygisksu is {} {} by {} (update {})",
        identity.name,
        identity.version,
        identity.author,
        identity.update_json
    );
    if is_original_zygisk_next(&identity) {
        return true;
    }
    logw!(
        "zn modules: {ZN_MODULE_DIR} is not the original Zygisk Next \
         (possibly a maintained fork); refusing to run znmod"
    );
    false
}

fn supports_znmod(cli: &Path) -> bool {
    let mut child = match Command::new(cli)
        .arg("znmod")
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
    {
        Ok(child) => child,
        Err(error) => {
            logw!("zn modules: cannot probe {}: {error}", cli.display());
            return false;
        }
    };

    let deadline = Instant::now() + PROBE_TIMEOUT;
    loop {
        match child.try_wait() {
            Ok(Some(_)) => break,
            Ok(None) => {
                if Instant::now() >= deadline {
                    let _ = child.kill();
                    let _ = child.wait();
                    logw!("zn modules: znmod probe timed out; treating it as unsupported");
                    return false;
                }
                thread::sleep(Duration::from_millis(50));
            }
            Err(error) => {
                let _ = child.kill();
                let _ = child.wait();
                logw!("zn modules: znmod probe failed: {error}");
                return false;
            }
        }
    }

    let output = match child.wait_with_output() {
        Ok(output) => output,
        Err(error) => {
            logw!("zn modules: cannot read the znmod probe result: {error}");
            return false;
        }
    };
    let mut text = String::from_utf8_lossy(&output.stdout).to_lowercase();
    text.push_str(&String::from_utf8_lossy(&output.stderr).to_lowercase());
    let supported = text.contains("znmod") && (text.contains("disable") || text.contains("reload"));
    if supported {
        logi!("zn modules: znmod support confirmed");
    } else {
        logw!("zn modules: installed zygisksu does not advertise znmod; skipping");
    }
    supported
}

struct DeclaredModule {
    id: String,
    path: String,
    name: Option<String>,
}

fn declared_modules() -> Vec<DeclaredModule> {
    let mut modules = Vec::new();
    let Ok(entries) = fs::read_dir(paths::MODULES_DIR) else {
        return modules;
    };
    for entry in entries.flatten() {
        let id = entry.file_name().to_string_lossy().into_owned();
        if id.starts_with('.') {
            continue;
        }
        let directory = entry.path();
        if !targets::is_enabled(&directory) {
            continue;
        }
        if !directory.join("zn_modules.txt").is_file() {
            continue;
        }
        let name = targets::read_prop(&directory, "name");
        modules.push(DeclaredModule {
            path: directory.to_string_lossy().into_owned(),
            name: (!name.is_empty()).then_some(name),
            id,
        });
    }
    modules.sort_by(|left, right| left.id.cmp(&right.id));
    modules
}

fn controller_operands(module: &DeclaredModule) -> Vec<&str> {
    let mut operands = vec![module.id.as_str(), module.path.as_str()];
    if let Some(name) = module.name.as_deref() {
        operands.push(name);
    }
    operands
}

fn daemon_present() -> bool {
    let Ok(entries) = fs::read_dir("/proc") else {
        return false;
    };
    for entry in entries.flatten() {
        let name = entry.file_name();
        let Ok(pid) = name.to_string_lossy().parse::<u32>() else {
            continue;
        };
        let exe = match fs::read_link(format!("/proc/{pid}/exe")) {
            Ok(exe) => exe,
            Err(_) => continue,
        };
        if exe.starts_with(ZN_MODULE_DIR) && exe.to_string_lossy().contains("/bin/zygiskd") {
            return true;
        }
        let Ok(comm) = fs::read_to_string(format!("/proc/{pid}/comm")) else {
            continue;
        };
        if comm.trim_start_matches('/').starts_with("zygiskd") {
            return true;
        }
    }
    false
}

fn run_action(cli: &Path, action: &str, module: &str) -> Result<(), String> {
    let output = Command::new(cli)
        .arg("znmod")
        .arg(action)
        .arg(module)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .map_err(|error| format!("cannot run {}: {error}", cli.display()))?;

    if output.status.success() {
        return Ok(());
    }

    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);
    let detail = if stderr.trim().is_empty() {
        stdout
    } else {
        stderr
    };
    Err(format!("{}: {}", output.status, detail.trim()))
}

fn apply(disable: bool) -> bool {
    let Some(cli) = cli_path() else {
        logi!("zn modules: Zygisk Next is not installed");
        return true;
    };
    let modules = declared_modules();
    if modules.is_empty() {
        logi!("zn modules: no module declares zn_modules.txt");
        return true;
    }

    let action = if disable { "disable" } else { "enable" };
    let mut all_ok = true;
    for module in &modules {
        let mut last_error = String::new();
        let mut applied = false;
        for operand in controller_operands(module) {
            match run_action(&cli, action, operand) {
                Ok(()) => {
                    logi!("zn modules: {action} {}: ok", module.id);
                    applied = true;
                    break;
                }
                Err(error) => last_error = error,
            }
        }
        if !applied {
            logw!("zn modules: {action} {}: failed: {last_error}", module.id);
        }
        all_ok &= applied;
    }
    all_ok
}

pub fn apply_now(disable: bool) -> bool {
    let Some(cli) = cli_path() else {
        logi!("zn modules: Zygisk Next is not installed");
        return true;
    };
    if !targets::is_enabled(Path::new(ZN_MODULE_DIR)) {
        logi!("zn modules: Zygisk Next is disabled");
        return true;
    }
    if !identity_is_original() {
        return false;
    }
    if !daemon_present() {
        logw!("zn modules: zygiskd is not running; the toggle will apply on the next boot");
        return false;
    }
    if !supports_znmod(&cli) {
        return false;
    }
    apply(disable)
}

pub fn wait_and_apply(disable: bool) -> bool {
    if !disable {
        logi!("zn modules: leaving Zygisk Next's loader enabled");
        return true;
    }
    let Some(cli) = cli_path() else {
        logi!("zn modules: Zygisk Next is not installed");
        return true;
    };
    if !targets::is_enabled(Path::new(ZN_MODULE_DIR)) {
        logi!("zn modules: Zygisk Next is disabled");
        return true;
    }
    if !identity_is_original() {
        return false;
    }

    let deadline = Instant::now() + DAEMON_WAIT;
    while Instant::now() < deadline {
        if daemon_present() {
            if !supports_znmod(&cli) {
                return false;
            }
            for _ in 0..3 {
                thread::sleep(SETTLE_AFTER_DAEMON);
                if apply(disable) {
                    return true;
                }
            }
            logw!("zn modules: some module updates failed");
            return false;
        }
        thread::sleep(RETRY_INTERVAL);
    }

    loge!(
        "zn modules: zygiskd did not start within {} seconds",
        DAEMON_WAIT.as_secs()
    );
    false
}
