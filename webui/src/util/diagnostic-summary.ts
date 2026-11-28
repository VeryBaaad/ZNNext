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

import { cleanVersion, type RootImplementation } from "../api/system";
import type { AppData } from "../composables/use-app-data";

const knownAbis = new Set([
  "arm64-v8a",
  "armeabi-v7a",
  "armeabi",
  "x86",
  "x86_64",
  "riscv64",
  "mips",
  "mips64",
]);
const activeModes = ["kernel", "ptrace", "proc", "late"];
const unknown = "unknown";

function rootSummary(root: RootImplementation | undefined): string {
  const managers = [
    ["KernelSU", root?.kernelSU],
    ["Magisk", root?.magisk],
    ["APatch", root?.apatch],
  ] as const;
  const versions = [];
  for (const [name, raw] of managers) {
    if (!raw?.trim()) continue;
    const version = cleanVersion(raw)?.match(/^\d+(?:\.\d+){0,3}/)?.[0];
    versions.push(version ? `${name} ${version}` : name);
  }
  return versions.join(", ") || unknown;
}

export function formatDiagnosticSummary(data: AppData): string {
  const status = data.status.value;
  const system = data.system.value;
  const modules = status?.running === false ? null : data.modules.value;
  const version = status?.running
    ? status.version?.match(
        /^v?\d+(?:\.\d+){1,3}(?:-\d+-(?:[a-f\d]{4,40}|unknown)-(?:debug|release))?/i,
      )?.[0]
    : undefined;
  const active = status?.running ? status.mode : undefined;
  const configured = data.config.value?.mode.value;
  const sdk = system?.sdk ?? 0;
  const lines = [
    "ZNNext diagnostics",
    `znnd version: ${version ?? unknown}`,
    `Daemon: ${status ? (status.running ? "running" : "stopped") : unknown}`,
    `Active mode: ${active && activeModes.includes(active) ? active : unknown}`,
    `Configured mode: ${configured && [...activeModes, "auto"].includes(configured) ? configured : unknown}`,
    `Android SDK: ${Number.isSafeInteger(sdk) && sdk > 0 ? sdk : unknown}`,
    `Kernel: ${system?.kernel.match(/^\d+\.\d+(?:\.\d+)?/)?.[0] ?? unknown}`,
    `ABI: ${system && knownAbis.has(system.abi) ? system.abi : unknown}`,
    `Root: ${rootSummary(system?.root)}`,
    `ZN modules: ${modules?.length ?? unknown}`,
    `Load failure records: ${modules?.reduce((sum, module) => sum + (module.failed?.length ?? 0), 0) ?? unknown}`,
  ];
  const unavailable = [];
  const stale = [];
  for (const [name, section] of Object.entries(data.sections)) {
    if (!section.failed) continue;
    if (section.lastSuccess === null) unavailable.push(name);
    else stale.push(name);
  }
  if (unavailable.length)
    lines.push(`Unavailable data: ${unavailable.join(", ")}`);
  if (stale.length) lines.push(`Stale data: ${stale.join(", ")}`);
  return lines.join("\n");
}
