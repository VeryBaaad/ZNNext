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

import { execText, getModuleDir, shellQuote } from "./ksu";

const events: [RegExp, string][] = [
  [/^ZNNext \S+ starting$/, "Daemon starting"],
  [/^shutting down:/, "Daemon shutting down"],
  [/^module verification failed:/, "Module verification failed"],
  [
    /^starting without previous daemon state:/,
    "Previous daemon state unavailable",
  ],
  [
    /^cannot decode state from previous daemon$/,
    "Previous daemon state unreadable",
  ],
  [/^cannot open daemon state socket:/, "State socket unavailable"],
  [/^cannot serve daemon state:/, "State query failed"],
  [/^collected \d+ zn modules$/, "Module scan completed"],
  [/^rescanned znn targets \(\d+\)$/, "Module list refreshed"],
  [
    /^kernel tracking is unavailable on this kernel/,
    "Kernel tracking unavailable",
  ],
  [
    /^memfd_create failed while preparing the loader\b/,
    "Loader memory file creation failed",
  ],
  [
    /^memfd_create failed while preparing a module library\b/,
    "Module memory file creation failed",
  ],
  [/^hot update: refusing /, "Staged update rejected"],
  [/^hot update: cannot prepare daemon state:/, "Update state transfer failed"],
  [/^hot update: cannot start /, "Updated daemon failed to start"],
  [
    /^hot update: the staged daemon \(\d+\) exited during startup/,
    "Updated daemon exited during startup",
  ],
];

export async function readDiagnosticLogs(): Promise<string> {
  const raw = await execText(
    "set -o pipefail; /system/bin/logcat -b main -d -v brief znndaemon:I '*:S' | /system/bin/tail -n 100",
    15000,
  );
  const lines = ["Recent znnd log events (redacted)"];
  for (const line of raw.split("\n")) {
    const entry = /^([IWEF])\/znndaemon\s*\(\s*\d+\):\s*(.*)$/.exec(line);
    if (!entry) continue;
    const event = events.find(([pattern]) => pattern.test(entry[2]));
    lines.push(
      `[${entry[1]}] ${event?.[1] ?? "Other daemon event (details omitted)"}`,
    );
  }
  if (lines.length === 1) lines.push("No recent entries.");
  return lines.join("\n");
}

export async function saveDiagnosticReport(
  summary: string,
  logs?: string,
): Promise<string> {
  const args = [`${getModuleDir()}/bugreport.sh`, summary];
  if (logs !== undefined) args.push(logs);
  const output = await execText(
    `/system/bin/sh ${args.map(shellQuote).join(" ")}`,
    30000,
  );
  const path = output.trim();
  if (
    !/^(?:\/sdcard\/Download\/|mock:\/\/Download\/)ZNNext-bugreport-[\w.-]+\.tar\.gz$/.test(
      path,
    )
  ) {
    throw new Error("Unexpected bugreport path");
  }
  return path;
}
