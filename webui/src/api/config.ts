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

import { apiBinary, execJson, shellQuote } from "./ksu";

export interface HookEngineEntry {
  value: string;
  options: string[];
}

export interface HookConfig {
  inlineHook: HookEngineEntry;
  pltHook: HookEngineEntry;
  mode: HookEngineEntry;
  debugdata: HookEngineEntry;
  disableZn: HookEngineEntry;
}

export type HookKind = "inline" | "plt" | "mode" | "debugdata" | "disable-zn";

function isHookEngineEntry(value: unknown): value is HookEngineEntry {
  if (typeof value !== "object" || value === null) return false;
  const entry = value as { value?: unknown; options?: unknown };
  return (
    typeof entry.value === "string" &&
    Array.isArray(entry.options) &&
    entry.options.every((option) => typeof option === "string")
  );
}

interface LegacyHookConfig {
  inlineHook: HookEngineEntry;
  pltHook: HookEngineEntry;
  mode: HookEngineEntry;
  debugdata: HookEngineEntry;
  disableZn?: HookEngineEntry;
}

const DEFAULT_DISABLE_ZN: HookEngineEntry = {
  value: "true",
  options: ["true", "false"],
};

function isHookConfig(value: unknown): value is LegacyHookConfig {
  if (typeof value !== "object" || value === null) return false;
  const config = value as Record<string, unknown>;
  return (
    isHookEngineEntry(config.inlineHook) &&
    isHookEngineEntry(config.pltHook) &&
    isHookEngineEntry(config.mode) &&
    isHookEngineEntry(config.debugdata) &&
    (config.disableZn === undefined || isHookEngineEntry(config.disableZn))
  );
}

function parseHookConfig(raw: unknown): HookConfig {
  if (!isHookConfig(raw)) throw new Error("unexpected config response");
  return {
    inlineHook: raw.inlineHook,
    pltHook: raw.pltHook,
    mode: raw.mode,
    debugdata: raw.debugdata,
    disableZn: raw.disableZn ?? DEFAULT_DISABLE_ZN,
  };
}

export async function getHookConfig(): Promise<HookConfig> {
  return parseHookConfig(
    await execJson<unknown>(`${apiBinary()} --ctl config`),
  );
}

export async function setHookConfig(
  kind: HookKind,
  value: string,
): Promise<HookConfig> {
  return parseHookConfig(
    await execJson<unknown>(
      `${apiBinary()} --ctl config-set ${shellQuote(kind)} ${shellQuote(value)}`,
    ),
  );
}
