/*
 * This file is part of Zygisk Next Next.
 *
 * Zygisk Next Next is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * Zygisk Next Next is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with Zygisk Next Next. If not, see <https://www.gnu.org/licenses/>.
 *
 * Copyright (C) 2026 VeryBaaad <verybaaad@outlook.com>
 */

#![deny(unsafe_code)]
mod arch;
mod companion;
mod config;
mod ctl;
mod daemon;
mod elf;
mod fileio;
mod kernel;
mod kernel_bpf;
mod kernel_connector;
mod kernel_fanotify;
mod kernel_kpm;
mod kernel_tracefs;
mod log;
mod machikado;
mod maps;
mod mode;
mod paths;
mod plan;
mod procfs;
mod ptrace_ops;
mod state;
mod sys;
mod system;
mod targets;
mod trace;

fn main() {
    let matches = ctl::command().get_matches();

    if let Some(arguments) = ctl::ctl_arguments(&matches) {
        std::process::exit(ctl::ctl_main(&arguments));
    }

    let module_dir = ctl::module_dir(&matches).unwrap_or_else(ctl::module_dir_of_self);

    if let Err(error) = machikado::verify_module(&module_dir) {
        crate::log::loge!("module verification failed: {error}");
        machikado::restore_prop(&module_dir);
        std::process::exit(1);
    }

    let deadline = std::time::Instant::now() + std::time::Duration::from_millis(3000);
    let mut reported = false;
    while let Some(pid) = procfs::find_daemon_pid(None) {
        if !reported {
            crate::log::logi!("waiting for the previous injector daemon ({pid}) to exit");
            reported = true;
        }
        if std::time::Instant::now() >= deadline {
            crate::log::loge!(
                "an injector daemon ({pid}) is already running, refusing to start a second instance"
            );
            std::process::exit(1);
        }
        std::thread::sleep(std::time::Duration::from_millis(100));
    }

    daemon::Daemon::new(module_dir).run();
}
