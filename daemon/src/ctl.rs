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

use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::thread;
use std::time::{Duration, Instant};
use std::{fs, io};

use clap::builder::ValueParser;
use clap::{Arg, ArgMatches, Command as ClapCommand};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;
use serde_json::value::RawValue;

use crate::config;
use crate::daemon::VERSION;
use crate::paths;
use crate::procfs;
use crate::state::SystemReport;
use crate::state_socket;
use crate::sys::Pid;
use crate::sys::process;
use crate::sys::signal;

/// Name of the argument that carries the control command, which may consume up
/// to three values: the command itself plus the two `config-set` operands.
const CTL_ARGUMENT: &str = "ctl";

const CTL_VALUE_NAME: &str = "COMMAND";
const MODULE_DIR_ARGUMENT: &str = "module-dir";

/// How long `--ctl restart` waits for a running daemon to go away before it
/// escalates, and after that before it reports a failure.
const STOP_INTERVAL: Duration = Duration::from_millis(25);
const STOP_ATTEMPTS: u32 = 80;

const START_TIMEOUT: Duration = Duration::from_secs(4);

const USAGE: &str = "usage: znnd --ctl <status|system|modules|config|config-export|config-diff [JSON]|config-import [JSON]|config-reset|config-undo|config-set <inline|inline32|plt|plt32|mode|debugdata|disable-zn|hot-update> <value>|apply-zn [wait]|start|restart|rescan|exit>";

const CONFIG_SET_USAGE: &str = "usage: znnd --ctl config-set <inline|inline32|plt|plt32|mode|debugdata|disable-zn|hot-update> <value>";

pub fn command() -> ClapCommand {
    ClapCommand::new("znnd")
        .about("Serves the Zygisk Next module API, or answers a control command.")
        .version(VERSION)
        .arg(
            Arg::new(MODULE_DIR_ARGUMENT)
                .value_name("MODULE_DIR")
                .value_parser(ValueParser::path_buf())
                .help("Installed module directory; defaults to the parent of this binary."),
        )
        .arg(
            Arg::new(CTL_ARGUMENT)
                .long(CTL_ARGUMENT)
                .value_name(CTL_VALUE_NAME)
                .num_args(1..=3)
                .conflicts_with(MODULE_DIR_ARGUMENT)
                .help(
                    "Control command: status, system, modules, config, config-export, \
                       config-diff [JSON] and config-import [JSON] (default to stdin), \
                       config-reset (restore device defaults), config-undo (undo the last change), \
                       config-set <inline|inline32|plt|plt32|mode|debugdata|disable-zn|hot-update> \
                       <value>, apply-zn [wait], start, restart, rescan or exit.",
                ),
        )
}

/// The module directory to serve, when the caller named one.
pub fn module_dir(matches: &ArgMatches) -> Option<PathBuf> {
    matches.get_one::<PathBuf>(MODULE_DIR_ARGUMENT).cloned()
}

/// The control command and its operands, when `--ctl` was given.
pub fn ctl_arguments(matches: &ArgMatches) -> Option<Vec<String>> {
    let arguments = matches.get_many::<String>(CTL_ARGUMENT)?;
    Some(arguments.cloned().collect())
}

pub fn ctl_main(arguments: &[String]) -> i32 {
    let Some((command, arguments)) = arguments.split_first() else {
        return usage();
    };

    match command.as_str() {
        "rescan" => signal_daemon(signal::SIGHUP, |pid| {
            format!("daemon ({pid}) requested to rescan modules")
        }),
        "exit" => signal_daemon(signal::SIGTERM, |pid| format!("daemon ({pid}) exited")),
        "start" => start_daemon(false),
        "restart" => start_daemon(true),
        "config" => print_json(&config::effective().report()),
        "config-export" if arguments.is_empty() => {
            config_result(read_config().map(|config| print_json(&config)))
        }
        "config-set" => config_result(config_set(arguments)),
        "config-undo" => config_result(config_undo(arguments)),
        "config-diff" | "config-import" | "config-reset" => {
            config_result(config_snapshot(command, arguments))
        }
        "apply-zn" => apply_zn(arguments),
        "status" | "system" | "modules" => query_state(command),
        _ => usage(),
    }
}

fn config_result(result: Result<i32, String>) -> i32 {
    result.unwrap_or_else(|error| {
        eprintln!("{error}");
        1
    })
}

/// Echo the part of the snapshot the WebUI renders as the device information.
fn system(snapshot: Option<&StateSnapshot>) -> i32 {
    match snapshot.and_then(|state| state.system.as_deref()) {
        Some(system) => print_json(system),
        None => print_json(&SystemReport::empty()),
    }
}

/// Echo the module list of the snapshot, empty while no daemon published one.
fn modules(snapshot: Option<&StateSnapshot>) -> i32 {
    match snapshot.and_then(|state| state.modules.as_deref()) {
        Some(modules) => print_json(modules),
        None => print_json(&Vec::<Value>::new()),
    }
}

/// The snapshot the daemon last published. Only the fields the control channel
/// reads are decoded; `system` and `modules` are kept as raw text so that they
/// are handed back exactly as they were written.
#[derive(Debug, Deserialize)]
struct StateSnapshot {
    #[serde(default, deserialize_with = "lenient")]
    mode: Option<String>,
    #[serde(default, deserialize_with = "lenient")]
    version: Option<String>,
    #[serde(default)]
    system: Option<Box<RawValue>>,
    #[serde(default)]
    modules: Option<Box<RawValue>>,
}

/// Read a scalar the way the WebUI does: a value that arrived as a JSON string
/// must not turn a malformed snapshot into a hard failure.
fn lenient<'de, D, T>(deserializer: D) -> Result<Option<T>, D::Error>
where
    D: Deserializer<'de>,
    T: DeserializeOwned + FromStr,
{
    let Ok(value) = Option::<Value>::deserialize(deserializer) else {
        return Ok(None);
    };
    Ok(match value {
        None | Some(Value::Null) => None,
        Some(Value::String(text)) => text.parse().ok(),
        Some(value) => T::deserialize(value).ok(),
    })
}

fn live_state(pid: Pid) -> io::Result<StateSnapshot> {
    serde_json::from_slice(&state_socket::read(pid)?).map_err(Into::into)
}

fn query_state(command: &str) -> i32 {
    let pid = procfs::find_daemon_pid();
    let snapshot = match pid {
        Some(pid) => match live_state(pid) {
            Ok(snapshot) => Some(snapshot),
            Err(_) if command == "status" => {
                return status(procfs::is_daemon_alive(pid).then_some(pid), None);
            }
            Err(error) if procfs::is_daemon_alive(pid) => {
                eprintln!("cannot query daemon ({pid}): {error}");
                return 1;
            }
            Err(_) => None,
        },
        None => None,
    };
    match command {
        "status" => status(pid, snapshot.as_ref()),
        "system" => system(snapshot.as_ref()),
        "modules" => modules(snapshot.as_ref()),
        _ => usage(),
    }
}

/// Bring the daemon up from this process, which is how the WebUI recovers after
/// a userspace restart that skipped the module's boot scripts. The child gets
/// its own session and stdio on /dev/null, so it outlives the caller.
fn start_daemon(restart: bool) -> i32 {
    let mut previous_state = None;
    if let Some(pid) = procfs::find_daemon_pid() {
        if !restart {
            return status(Some(pid), live_state(pid).ok().as_ref());
        }
        match state_socket::read(pid) {
            Ok(bytes) => previous_state = Some(bytes),
            Err(error) => eprintln!("previous daemon state unavailable: {error}"),
        }
        if let Err(error) = stop_daemon(pid) {
            eprintln!("{error}");
            return 1;
        }
    }

    let Ok(executable) = fs::read_link("/proc/self/exe") else {
        eprintln!("cannot resolve the daemon path");
        return 1;
    };
    let module_dir = module_dir_of_self();
    if module_dir.as_os_str().is_empty() {
        eprintln!("cannot resolve the module directory");
        return 1;
    }
    match process::spawn_detached(&executable, &module_dir, previous_state.as_deref()) {
        Ok(pid) => {
            let deadline = Instant::now() + START_TIMEOUT;
            while Instant::now() < deadline && !procfs::process_gone(pid) {
                if let Ok(snapshot) = live_state(pid) {
                    return status(Some(pid), Some(&snapshot));
                }
                thread::sleep(STOP_INTERVAL);
            }
            eprintln!("daemon ({pid}) did not answer a status query during startup");
            1
        }
        Err(error) => {
            eprintln!("cannot start the daemon: {error}");
            1
        }
    }
}

fn stop_daemon(pid: Pid) -> Result<(), String> {
    let _ = process::kill(pid, signal::SIGTERM);
    for _ in 0..STOP_ATTEMPTS {
        if !procfs::is_daemon_alive(pid) {
            return Ok(());
        }
        thread::sleep(STOP_INTERVAL);
    }
    Err(format!(
        "daemon ({pid}) is still running; it may be holding an armed breakpoint, so it is not killed"
    ))
}

pub fn module_dir_of_self() -> PathBuf {
    let Ok(executable) = fs::read_link("/proc/self/exe") else {
        return PathBuf::new();
    };
    match executable.parent().and_then(Path::parent) {
        Some(module_dir) => module_dir.to_path_buf(),
        None => PathBuf::new(),
    }
}

fn signal_daemon(requested: i32, message: fn(Pid) -> String) -> i32 {
    let Some(pid) = procfs::find_daemon_pid() else {
        println!("daemon is not running");
        return 1;
    };
    if let Err(error) = process::kill(pid, requested) {
        eprintln!("cannot signal daemon ({pid}): {error}");
        return 1;
    }
    println!("{}", message(pid));
    0
}

fn read_config() -> Result<config::HookConfig, String> {
    config::for_export().map_err(|error| format!("cannot read {}: {error}", paths::CONFIG_FILE))
}

fn lock_config() -> Result<fs::File, String> {
    config::lock_for_update().map_err(|error| format!("cannot lock {}: {error}", paths::STATE_DIR))
}

fn config_snapshot(command: &str, arguments: &[String]) -> Result<i32, String> {
    let proposed = match command {
        "config-reset" if arguments.is_empty() => config::defaults(),
        "config-reset" => return Err("usage: znnd --ctl config-reset".into()),
        _ => crate::config_diff::read_snapshot(command, arguments)?,
    };
    let write = matches!(command, "config-import" | "config-reset");
    let guard = if write { Some(lock_config()?) } else { None };
    let current = read_config()?;
    let diff =
        crate::config_diff::compare(current.clone(), &proposed, config::HookConfig::options)?;
    let status = if write && !diff.changes.is_empty() {
        persist_config(
            &proposed,
            Some(&current),
            diff.changes.contains_key("disable-zn"),
        )?
    } else {
        0
    };
    drop(guard);
    Ok(print_json(&diff).max(status))
}

fn config_undo(arguments: &[String]) -> Result<i32, String> {
    if !arguments.is_empty() {
        return Err("usage: znnd --ctl config-undo".into());
    }
    let guard = lock_config()?;
    let (current, previous) = config::undo::read()?;
    let diff = crate::config_diff::compare(current, &previous, config::HookConfig::stored_options)?;
    let status = persist_config(&previous, None, diff.changes.contains_key("disable-zn"))?;
    drop(guard);
    Ok(print_json(&diff).max(status))
}

fn config_set(arguments: &[String]) -> Result<i32, String> {
    let [kind, value] = arguments else {
        return Err(CONFIG_SET_USAGE.into());
    };

    let Some(options) = config::HookConfig::options(kind) else {
        if kind == "inline32" || kind == "plt32" {
            return Err("this device has no secondary 32-bit ABI".into());
        }
        return Err(CONFIG_SET_USAGE.into());
    };
    if !config::option_in(&options, value) {
        return Err(format!("invalid {kind} value \"{value}\" for this device"));
    }

    let guard = lock_config()?;
    let current = read_config()?;
    let mut updated = current.clone();
    updated.set(kind, value);
    let status = persist_config(&updated, Some(&current), kind == "disable-zn")?;
    drop(guard);
    Ok(print_json(&updated.report()).max(status))
}

fn persist_config(
    updated: &config::HookConfig,
    previous: Option<&config::HookConfig>,
    apply_zn: bool,
) -> Result<i32, String> {
    if previous != Some(updated) {
        config::write(updated, previous)
            .map_err(|error| format!("cannot write {}: {error}", paths::CONFIG_FILE))?;
    }
    let applied = !apply_zn || crate::zn::apply_now(updated.disable_zn == config::DISABLE_ZN_TRUE);
    if !applied {
        eprintln!(
            "the setting was saved, but Zygisk Next could not be updated; see the module log"
        );
    }

    if let Some(pid) = procfs::find_daemon_pid() {
        let _ = process::kill(pid, signal::SIGHUP);
    }

    Ok(i32::from(!applied))
}

fn apply_zn(arguments: &[String]) -> i32 {
    if arguments.iter().any(|argument| argument != "wait") {
        eprintln!("usage: znnd --ctl apply-zn [wait]");
        return 1;
    }

    let wait = arguments.iter().any(|argument| argument == "wait");
    let disable = config::effective().disable_zn == config::DISABLE_ZN_TRUE;
    let ok = if wait {
        crate::zn::wait_and_apply(disable)
    } else {
        crate::zn::apply_now(disable)
    };
    if ok { 0 } else { 1 }
}

fn status(pid: Option<Pid>, snapshot: Option<&StateSnapshot>) -> i32 {
    let recorded_mode = snapshot
        .and_then(|state| state.mode.as_deref())
        .filter(|mode| {
            *mode == config::MODE_KERNEL
                || *mode == config::MODE_PTRACE
                || *mode == config::MODE_PROC
                || *mode == config::MODE_LATE
        });
    let pid = pid.unwrap_or(0);
    let running = pid > 0;
    let mode = match recorded_mode {
        Some(mode) if running => mode,
        _ => "unknown",
    };
    let version = snapshot
        .and_then(|state| state.version.as_deref())
        .filter(|_| running);
    print_json(&StatusReport {
        running,
        pid: if running { pid } else { 0 },
        mode,
        version,
    })
}

#[derive(Debug, Serialize)]
struct StatusReport<'a> {
    running: bool,
    pid: Pid,
    mode: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    version: Option<&'a str>,
}

fn print_json<T: Serialize + ?Sized>(value: &T) -> i32 {
    match serde_json::to_string(value) {
        Ok(json) => {
            println!("{json}");
            0
        }
        Err(error) => {
            eprintln!("cannot encode the answer: {error}");
            1
        }
    }
}

fn usage() -> i32 {
    eprintln!("{USAGE}");
    1
}
