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

import { apiBinary, execJson } from "./ksu";

export interface InjectedProcess {
  pid: number;
  name: string;
}

export interface FailedProcess {
  name: string;
  reason: string;
}

export interface ZnnModule {
  id: string;
  name: string;
  version: string;
  processes: InjectedProcess[];
  failed?: FailedProcess[];
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
    const failure = item as { name?: unknown; reason?: unknown };
    if (typeof failure.name !== "string" || typeof failure.reason !== "string")
      continue;
    failed.push({ name: failure.name, reason: failure.reason });
  }
  return failed;
}

function normaliseModule(raw: unknown): ZnnModule | null {
  if (typeof raw !== "object" || raw === null) return null;
  const module = raw as Record<string, unknown>;
  if (
    typeof module.id !== "string" ||
    typeof module.name !== "string" ||
    typeof module.version !== "string"
  ) {
    return null;
  }
  return {
    id: module.id,
    name: module.name,
    version: module.version,
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
