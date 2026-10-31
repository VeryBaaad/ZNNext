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

import type { HookConfig, HookEngineEntry, HookKind } from "./api/config";
import { t } from "./i18n";

export const ENGINE_LABELS: Record<string, string> = {
  dobby: "Dobby",
  shadowhook: "ShadowHook",
  rv64hook: "rv64hook",
  and64inlinehook: "And64InlineHook",
  "frida-gum": "Frida Gum",
  hookzz: "HookZz",
  whale: "Whale",
  lsplt: "LSPlt",
  bytehook: "ByteHook",
  xhook: "xHook",
  plti: "PLTI",
  plthook: "PLTHook",
  yukiplt: "YukiPLT",
  urhook: "UrHook",
  baaadhook: "BaaadHook",
  disable: t(`config.disable`),
};

export interface HookRow {
  kind: HookKind;
  field: keyof HookConfig;
  labelKey: string;
}

export const HOOK_ROWS: HookRow[] = [
  { kind: "inline", field: "inlineHook", labelKey: "config.inlineHook" },
  { kind: "inline32", field: "inlineHook32", labelKey: "config.inlineHook32" },
  { kind: "plt", field: "pltHook", labelKey: "config.pltHook" },
  { kind: "plt32", field: "pltHook32", labelKey: "config.pltHook32" },
  { kind: "mode", field: "mode", labelKey: "config.mode" },
  { kind: "debugdata", field: "debugdata", labelKey: "config.debugdata" },
  { kind: "disable-zn", field: "disableZn", labelKey: "config.disableZn" },
];

export const UI_ROW_LABEL_KEY = "config.uiStyle";

export function configRows<T>(
  config: HookConfig,
  map: (row: HookRow, entry: HookEngineEntry) => T,
  uiRow: T,
): T[] {
  const upper: T[] = [];
  const lower: T[] = [];
  for (const row of HOOK_ROWS) {
    const entry = config[row.field];
    if (!entry) continue;
    (row.kind === "disable-zn" ? lower : upper).push(map(row, entry));
  }
  upper.push(uiRow);
  upper.push(...lower);
  return upper;
}

export function hookOptionLabel(kind: HookKind, id: string): string {
  if (kind === "mode") return t(`config.mode.${id}`);
  if (kind === "debugdata") return t(`config.debugdata.${id}`);
  if (kind === "disable-zn") return t(`config.disableZn.${id}`);
  return ENGINE_LABELS[id] ?? id;
}

export function uiModeLabel(mode: string): string {
  if (mode === "miuix") return t("ui.miuix");
  if (mode === "zn") return t("ui.zn");
  return t("ui.material");
}
