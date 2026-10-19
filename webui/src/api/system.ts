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

export interface RootImplementation {
  magisk: string | null;
  kernelSU: string | null;
  apatch: string | null;
}

export interface SystemInfo {
  kernel: string;
  sdk: number;
  abi: string;
  abilist: string;
  root: RootImplementation;
}

function asString(value: unknown): string {
  return typeof value === "string" ? value : "";
}

function asRootString(value: unknown): string | null {
  return typeof value === "string" ? value : null;
}

export async function getSystemInfo(): Promise<SystemInfo> {
  const raw = await execJson<unknown>(`${apiBinary()} --ctl system`);
  const info =
    typeof raw === "object" && raw !== null
      ? (raw as Record<string, unknown>)
      : {};
  const root =
    typeof info.root === "object" && info.root !== null
      ? (info.root as Record<string, unknown>)
      : {};
  return {
    kernel: asString(info.kernel),
    sdk: typeof info.sdk === "number" ? info.sdk : 0,
    abi: asString(info.abi),
    abilist: asString(info.abilist),
    root: {
      magisk: asRootString(root.magisk),
      kernelSU: asRootString(root.kernelSU),
      apatch: asRootString(root.apatch),
    },
  };
}

export function cleanVersion(raw: string | null): string | null {
  if (!raw) return null;
  return (
    raw
      .replace(/^(ksud|apd|magisk)\s+/i, "")
      .replace(/^v/i, "")
      .trim() || null
  );
}
