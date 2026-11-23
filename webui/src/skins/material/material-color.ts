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

import {
  DynamicScheme,
  Hct,
  MaterialDynamicColors,
  Variant,
  argbFromHex,
  hexFromArgb,
} from "@material/material-color-utilities";
import { computed, ref, type ComputedRef } from "vue";

export type MaterialSpec = "SPEC_2021" | "SPEC_2025";

export const MATERIAL_SPEC_STORAGE_KEY = "znn_material_spec";

export const MATERIAL_SEED_STORAGE_KEY = "znn_material_seed";

export const DEFAULT_MATERIAL_SPEC: MaterialSpec = "SPEC_2025";

export const DEFAULT_MATERIAL_SEED = "#6750a4";

export const MATERIAL_SPECS: MaterialSpec[] = ["SPEC_2021", "SPEC_2025"];

export const MATERIAL_SEED_PRESETS: string[] = [
  "#6750a4",
  "#006a6a",
  "#0061a4",
  "#4c662b",
  "#8b5000",
  "#b3261e",
  "#7d5260",
  "#43474e",
];

const CACHE_LIMIT = 64;

const tokenCache = new Map<string, Record<string, string>>();

const seedRef = ref(detectSeed());

const specRef = ref(detectSpec());

let appliedProperties = new Set<string>();

export const materialSeed: ComputedRef<string> = computed(() => seedRef.value);

export const materialSpec: ComputedRef<MaterialSpec> = computed(
  () => specRef.value,
);

export function normalizeSeed(value: string): string | null {
  const match = /^#?([0-9a-f]{3}|[0-9a-f]{6})$/i.exec(value.trim());
  if (!match) return null;
  let digits = match[1].toLowerCase();
  if (digits.length === 3) {
    digits = digits
      .split("")
      .map((digit) => digit + digit)
      .join("");
  }
  return `#${digits}`;
}

export function isMaterialSpec(value: string): value is MaterialSpec {
  return (MATERIAL_SPECS as string[]).includes(value);
}

export function specLabel(spec: MaterialSpec): string {
  return spec === "SPEC_2021" ? "SPEC 2021" : "SPEC 2025";
}

export function setMaterialSeed(next: string): void {
  const normalized = normalizeSeed(next);
  if (!normalized || normalized === seedRef.value) return;
  seedRef.value = normalized;
  try {
    localStorage.setItem(MATERIAL_SEED_STORAGE_KEY, normalized);
  } catch {}
}

export function setMaterialSpec(next: MaterialSpec): void {
  if (next === specRef.value) return;
  specRef.value = next;
  try {
    localStorage.setItem(MATERIAL_SPEC_STORAGE_KEY, next);
  } catch {}
}

function createMaterialScheme(
  seed: string,
  spec: MaterialSpec,
  dark: boolean,
): DynamicScheme {
  return new DynamicScheme({
    sourceColorHct: Hct.fromInt(
      argbFromHex(normalizeSeed(seed) ?? DEFAULT_MATERIAL_SEED),
    ),
    variant: Variant.TONAL_SPOT,
    contrastLevel: 0,
    isDark: dark,
    specVersion: spec === "SPEC_2021" ? "2021" : "2025",
  });
}

export function materialTokens(
  seed: string,
  spec: MaterialSpec,
  dark: boolean,
): Record<string, string> {
  const key = `${normalizeSeed(seed) ?? DEFAULT_MATERIAL_SEED}|${spec}|${dark ? 1 : 0}`;
  return cached(tokenCache, key, () => {
    const scheme = createMaterialScheme(seed, spec, dark);
    const colors = new MaterialDynamicColors();
    const list = [
      ...colors.allColors,
      colors.surfaceVariant(),
      colors.shadow(),
      colors.scrim(),
      colors.surfaceTint(),
    ];
    const tokens: Record<string, string> = {};
    for (const color of list) {
      const token = `--md-sys-color-${color.name.replace(/_/g, "-").toLowerCase()}`;
      tokens[token] = hexFromArgb(color.getArgb(scheme));
    }
    tokens["--m3e-scrollbar-thumb-color"] = hexFromArgb(
      scheme.neutralPalette.tone(60),
    );
    return tokens;
  });
}

export function applyMaterialTheme(
  seed: string,
  spec: MaterialSpec,
  dark: boolean,
): void {
  const tokens = materialTokens(seed, spec, dark);
  const root = document.documentElement;
  for (const property of appliedProperties) {
    if (!(property in tokens)) root.style.removeProperty(property);
  }
  appliedProperties = new Set(Object.keys(tokens));
  for (const [property, value] of Object.entries(tokens)) {
    root.style.setProperty(property, value);
  }
}

function cached<T>(cache: Map<string, T>, key: string, create: () => T): T {
  const existing = cache.get(key);
  if (existing !== undefined) return existing;
  const value = create();
  if (cache.size >= CACHE_LIMIT) {
    const oldest = cache.keys().next().value;
    if (oldest !== undefined) cache.delete(oldest);
  }
  cache.set(key, value);
  return value;
}

function detectSeed(): string {
  try {
    const saved = localStorage.getItem(MATERIAL_SEED_STORAGE_KEY);
    if (saved) {
      const normalized = normalizeSeed(saved);
      if (normalized) return normalized;
    }
  } catch {}
  return DEFAULT_MATERIAL_SEED;
}

function detectSpec(): MaterialSpec {
  try {
    const saved = localStorage.getItem(MATERIAL_SPEC_STORAGE_KEY);
    if (saved && isMaterialSpec(saved)) return saved;
  } catch {}
  return DEFAULT_MATERIAL_SPEC;
}
