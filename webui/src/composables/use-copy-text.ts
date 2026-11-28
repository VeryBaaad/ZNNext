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

import { useTimeoutFn } from "@vueuse/core";
import { computed, ref } from "vue";

import { t } from "../i18n";

async function writeClipboard(
  text: string,
  button: HTMLElement | null,
): Promise<boolean> {
  if (navigator.clipboard?.writeText) {
    try {
      await navigator.clipboard.writeText(text);
      return true;
    } catch {}
  }

  const input = document.createElement("textarea");
  input.value = text;
  input.readOnly = true;
  input.style.cssText =
    "position:fixed;top:0;left:0;opacity:0;pointer-events:none";
  const focused = document.activeElement;
  if (button?.isConnected) button.after(input);
  else document.body.append(input);
  try {
    input.focus({ preventScroll: true });
    input.select();
    return document.execCommand("copy");
  } finally {
    input.remove();
    if (focused instanceof HTMLElement) focused.focus({ preventScroll: true });
  }
}

export function useCopyText(text: () => string) {
  const feedback = ref("actions.copy");
  const { start, stop } = useTimeoutFn(
    () => (feedback.value = "actions.copy"),
    2500,
    {
      immediate: false,
    },
  );
  let copying = false;

  async function copy(event?: Event): Promise<void> {
    if (copying) return;
    copying = true;
    stop();
    const button =
      event?.currentTarget instanceof HTMLElement ? event.currentTarget : null;
    try {
      const copied = await writeClipboard(text(), button);
      feedback.value = copied ? "actions.copied" : "actions.copyFailed";
    } catch {
      feedback.value = "actions.copyFailed";
    } finally {
      copying = false;
      start();
    }
  }

  return { copy, copyLabel: computed(() => t(feedback.value)) };
}
