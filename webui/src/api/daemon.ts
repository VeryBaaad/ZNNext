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

import { apiBinary, execJson } from "./ksu";

const activeModes = ["kernel", "proc", "ptrace", "late"] as const;
const modeReasons = [
  "kernelUnavailable",
  "ptraceUnavailable",
  "otherLoaderActive",
  "switchDeferred",
  "restartRequired",
] as const;

export interface DaemonStatus {
  running: boolean;
  pid: number;
  mode?: string;
  requestedMode?: (typeof activeModes)[number] | "auto";
  modeReason?: (typeof modeReasons)[number];
  version?: string;
}

export function isActiveMode(
  mode: unknown,
): mode is (typeof activeModes)[number] {
  return activeModes.some((activeMode) => activeMode === mode);
}

export async function getDaemonStatus(): Promise<DaemonStatus> {
  const raw = await execJson<{
    running?: unknown;
    pid?: number;
    mode?: unknown;
    requestedMode?: unknown;
    modeReason?: unknown;
    version?: unknown;
  }>(`${apiBinary()} --ctl status`);
  const reportedRunning = raw?.running;
  if (
    typeof reportedRunning !== "boolean" &&
    reportedRunning !== "true" &&
    reportedRunning !== "false"
  )
    throw new Error("unexpected daemon status response");
  const running = reportedRunning === true || reportedRunning === "true";
  const mode = typeof raw.mode === "string" ? raw.mode : undefined;
  const active = running && isActiveMode(mode);
  const requested = raw.requestedMode;
  return {
    pid: typeof raw.pid === "number" ? raw.pid : 0,
    mode,
    requestedMode:
      active && (requested === "auto" || isActiveMode(requested))
        ? requested
        : undefined,
    modeReason: active
      ? modeReasons.find((reason) => reason === raw.modeReason)
      : undefined,
    version: typeof raw.version === "string" ? raw.version : undefined,
    running,
  };
}
