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

export interface InjectedProcess {
  pid: number;
  name: string;
}

export interface FailedProcess {
  name: string;
  reason: string;
  pid?: number;
  lastSeenMs?: number;
}

export interface ZnnModule {
  id: string;
  name: string;
  version: string;
  disabled?: boolean;
  declarationIssue?: "noDeclarations" | "missingLibraries";
  missingLibraries?: string[];
  processes: InjectedProcess[];
  failed?: FailedProcess[];
}

export function moduleStatusKey(
  module: ZnnModule,
):
  | "modules.disabled"
  | "modules.noDeclarations"
  | "modules.missingLibraries"
  | undefined {
  if (module.disabled === true) return "modules.disabled";
  if (module.declarationIssue === "noDeclarations")
    return "modules.noDeclarations";
  if (module.declarationIssue === "missingLibraries")
    return "modules.missingLibraries";
}

function normaliseProcesses(raw: unknown): InjectedProcess[] {
  if (!Array.isArray(raw)) return [];
  const processes: InjectedProcess[] = [];
  for (const item of raw) {
    if (typeof item !== "object" || item === null) continue;
    const process = item as { pid?: unknown; name?: unknown };
    if (typeof process.pid !== "number" || typeof process.name !== "string")
      continue;
    processes.push({ pid: process.pid, name: process.name });
  }
  return processes;
}

function normaliseFailed(raw: unknown): FailedProcess[] {
  if (!Array.isArray(raw)) return [];
  const failed: FailedProcess[] = [];
  for (const item of raw) {
    if (typeof item !== "object" || item === null) continue;
    const { name, reason, pid, lastSeenMs } = item as Record<string, unknown>;
    if (typeof name !== "string" || typeof reason !== "string") continue;

    const failure: FailedProcess = { name, reason };
    if (typeof pid === "number" && Number.isSafeInteger(pid) && pid > 0)
      failure.pid = pid;
    if (
      typeof lastSeenMs === "number" &&
      Number.isSafeInteger(lastSeenMs) &&
      lastSeenMs >= 0 &&
      lastSeenMs <= 8_640_000_000_000_000
    )
      failure.lastSeenMs = lastSeenMs;
    failed.push(failure);
  }
  return failed;
}

function normaliseModule(raw: unknown): ZnnModule | null {
  if (typeof raw !== "object" || raw === null) return null;
  const module = raw as Record<string, unknown>;
  if (typeof module.id !== "string" || typeof module.name !== "string")
    return null;
  const issue = module.declarationIssue;
  return {
    id: module.id,
    name: module.name,
    version: typeof module.version === "string" ? module.version.trim() : "",
    disabled: module.disabled === true,
    declarationIssue:
      issue === "noDeclarations" || issue === "missingLibraries"
        ? issue
        : undefined,
    missingLibraries:
      module.disabled !== true &&
      issue === "missingLibraries" &&
      Array.isArray(module.missingLibraries)
        ? module.missingLibraries.filter(
            (path): path is string =>
              typeof path === "string" && path.length > 0,
          )
        : undefined,
    processes: normaliseProcesses(module.processes),
    failed: normaliseFailed(module.failed),
  };
}

export async function getModules(): Promise<ZnnModule[]> {
  const raw = await execJson<unknown>(`${apiBinary()} --ctl modules`);
  if (!Array.isArray(raw)) throw new Error("unexpected modules response");
  const modules: ZnnModule[] = [];
  for (const item of raw) {
    const module = normaliseModule(item);
    if (module !== null) modules.push(module);
  }
  return modules;
}
