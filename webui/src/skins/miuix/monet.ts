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
 * Copyright (C) 2026 VeryBaaad <verybaad@outlook.com>
 */

import { computed, ref, type ComputedRef } from "vue";

import { materialTokens, type MaterialSpec } from "../../color/material-color";

export const MIUIX_MONET_STORAGE_KEY = "znn_miuix_monet";

interface Rgb {
  red: number;
  green: number;
  blue: number;
}

interface Rgba extends Rgb {
  alpha: number;
}

const enabledRef = ref(detectMonet());

let appliedProperties = new Set<string>();

export const miuixMonetEnabled: ComputedRef<boolean> = computed(
  () => enabledRef.value,
);

export function setMiuixMonetEnabled(next: boolean): void {
  if (next === enabledRef.value) return;
  enabledRef.value = next;
  try {
    localStorage.setItem(MIUIX_MONET_STORAGE_KEY, next ? "true" : "false");
  } catch {}
}

export function applyMiuixMonet(
  seed: string,
  spec: MaterialSpec,
  dark: boolean,
): void {
  const vars = monetVariables(seed, spec, dark);
  const root = document.documentElement;
  for (const property of appliedProperties) {
    if (!(property in vars)) root.style.removeProperty(property);
  }
  appliedProperties = new Set(Object.keys(vars));
  root.dataset.znnMonet = "true";
  for (const [property, value] of Object.entries(vars)) {
    root.style.setProperty(property, value);
  }
}

export function clearMiuixMonet(): void {
  const root = document.documentElement;
  for (const property of appliedProperties) {
    root.style.removeProperty(property);
  }
  delete root.dataset.znnMonet;
  appliedProperties = new Set();
}

function monetVariables(
  seed: string,
  spec: MaterialSpec,
  dark: boolean,
): Record<string, string> {
  const tokens = materialTokens(seed, spec, dark);
  const role = (name: string): Rgb =>
    parseHex(tokens[`--md-sys-color-${name}`]);

  const primary = role("primary");
  const onPrimary = role("on-primary");
  const primaryFixed = role("primary-fixed");
  const onPrimaryFixed = role("on-primary-fixed");
  const primaryContainer = role("primary-container");
  const onPrimaryContainer = role("on-primary-container");
  const error = role("error");
  const onError = role("on-error");
  const errorContainer = role("error-container");
  const onErrorContainer = role("on-error-container");
  const secondaryContainer = role("secondary-container");
  const onSecondaryContainer = role("on-secondary-container");
  const tertiaryContainer = role("tertiary-container");
  const onTertiaryContainer = role("on-tertiary-container");
  const background = role("background");
  const onBackground = role("on-background");
  const surface = role("surface");
  const onSurface = role("on-surface");
  const surfaceVariant = role("surface-variant");
  const surfaceContainer = role("surface-container");
  const surfaceContainerHigh = role("surface-container-high");
  const surfaceContainerHighest = role("surface-container-highest");
  const onSurfaceVariant = role("on-surface-variant");
  const outline = role("outline");
  const outlineVariant = role("outline-variant");

  const disabledPrimary = over(alpha(primary, 0.38), surface);
  const disabledOnPrimary = over(alpha(onPrimary, 0.38), disabledPrimary);
  const disabledPrimaryButton = over(alpha(primary, 0.38), surface);
  const disabledOnPrimaryButton = over(
    alpha(onPrimary, 0.6),
    disabledPrimaryButton,
  );
  const disabledPrimarySlider = over(alpha(primary, 0.38), surface);
  const disabledSecondary = over(alpha(outlineVariant, 0.5), surface);
  const disabledOnSecondary = over(alpha(onSurface, 0.38), disabledSecondary);
  const disabledSecondaryVariant = over(
    alpha(surfaceContainerHigh, 0.6),
    surface,
  );
  const disabledOnSecondaryVariant = over(
    alpha(onSurface, 0.38),
    disabledSecondaryVariant,
  );

  return {
    "--m-color-primary": hex(primary),
    "--m-color-on-primary": hex(onPrimary),
    "--m-color-primary-variant": hex(primaryFixed),
    "--m-color-on-primary-variant": hex(onPrimaryFixed),
    "--m-color-error": hex(error),
    "--m-color-on-error": hex(onError),
    "--m-color-error-container": hex(errorContainer),
    "--m-color-on-error-container": hex(onErrorContainer),
    "--m-color-disabled-primary": hex(disabledPrimary),
    "--m-color-disabled-on-primary": hex(disabledOnPrimary),
    "--m-color-disabled-primary-button": hex(disabledPrimaryButton),
    "--m-color-disabled-on-primary-button": hex(disabledOnPrimaryButton),
    "--m-color-disabled-primary-slider": hex(disabledPrimarySlider),
    "--m-color-primary-container": hex(primaryContainer),
    "--m-color-on-primary-container": hex(onPrimaryContainer),
    "--m-color-secondary": hex(outlineVariant),
    "--m-color-on-secondary": hex(outline),
    "--m-color-secondary-variant": hex(surfaceContainerHigh),
    "--m-color-on-secondary-variant": hex(onSurface),
    "--m-color-disabled-secondary": hex(disabledSecondary),
    "--m-color-disabled-on-secondary": hex(disabledOnSecondary),
    "--m-color-disabled-secondary-variant": hex(disabledSecondaryVariant),
    "--m-color-disabled-on-secondary-variant": hex(disabledOnSecondaryVariant),
    "--m-color-secondary-container": hex(secondaryContainer),
    "--m-color-on-secondary-container": hex(onSecondaryContainer),
    "--m-color-secondary-container-variant": hex(surfaceContainerHighest),
    "--m-color-on-secondary-container-variant": hex(onSurfaceVariant),
    "--m-color-tertiary-container": hex(tertiaryContainer),
    "--m-color-on-tertiary-container": hex(onTertiaryContainer),
    "--m-color-tertiary-container-variant": hex(onTertiaryContainer),
    "--m-color-background": hex(background),
    "--m-color-on-background": hex(onBackground),
    "--m-color-on-background-variant": hex(primary),
    "--m-color-surface": hex(surface),
    "--m-color-on-surface": hex(onSurface),
    "--m-color-surface-variant": hex(surfaceVariant),
    "--m-color-on-surface-secondary": hex(over(alpha(onSurface, 0.8), surface)),
    "--m-color-on-surface-variant-summary": hex(onSurfaceVariant),
    "--m-color-on-surface-variant-actions": hex(onSurfaceVariant),
    "--m-color-disabled-on-surface": hex(onSurface),
    "--m-color-surface-container": hex(surfaceContainer),
    "--m-color-on-surface-container": hex(onSurface),
    "--m-color-on-surface-container-variant": hex(onSurfaceVariant),
    "--m-color-surface-container-high": hex(surfaceContainerHigh),
    "--m-color-on-surface-container-high": hex(
      over(alpha(onSurface, 0.8), surfaceContainerHigh),
    ),
    "--m-color-surface-container-highest": hex(surfaceContainerHighest),
    "--m-color-on-surface-container-highest": hex(onSurface),
    "--m-color-outline": hex(outline),
    "--m-color-divider-line": hex(outlineVariant),
    "--m-color-window-dimming": dark ? "#00000099" : "#0000004d",
    "--m-color-slider-key-point": hex(primary),
    "--m-color-slider-key-point-foreground": hex(surfaceContainerHigh),
    "--m-color-slider-background": hex(over(alpha(primary, 0.2), surface)),
  };
}

function parseHex(value: string): Rgb {
  const hex = value.replace("#", "");
  return {
    red: parseInt(hex.slice(0, 2), 16),
    green: parseInt(hex.slice(2, 4), 16),
    blue: parseInt(hex.slice(4, 6), 16),
  };
}

function alpha(color: Rgb, value: number): Rgba {
  return { ...color, alpha: value };
}

function over(foreground: Rgba, background: Rgb): Rgb {
  return {
    red: Math.round(
      foreground.red * foreground.alpha +
        background.red * (1 - foreground.alpha),
    ),
    green: Math.round(
      foreground.green * foreground.alpha +
        background.green * (1 - foreground.alpha),
    ),
    blue: Math.round(
      foreground.blue * foreground.alpha +
        background.blue * (1 - foreground.alpha),
    ),
  };
}

function hex(color: Rgb): string {
  return `#${channel(color.red)}${channel(color.green)}${channel(color.blue)}`;
}

function channel(value: number): string {
  return value.toString(16).padStart(2, "0");
}

function detectMonet(): boolean {
  try {
    const saved = localStorage.getItem(MIUIX_MONET_STORAGE_KEY);
    return saved === "true" || saved === "1";
  } catch {}
  return false;
}
