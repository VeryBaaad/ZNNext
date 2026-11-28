<script setup lang="ts">
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
import { ref } from "vue";

import type { FailedProcess, ZnnModule } from "../api/modules";
import { t } from "../i18n";
import { formatFailureDetails } from "../util/failure-metadata";

const props = defineProps<{ module: ZnnModule; failure: FailedProcess }>();
const feedback = ref("actions.copy");
const { start, stop } = useTimeoutFn(
  () => (feedback.value = "actions.copy"),
  2500,
  {
    immediate: false,
  },
);
let copying = false;

async function writeClipboard(
  text: string,
  button: HTMLButtonElement,
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
  button.after(input);
  try {
    input.focus({ preventScroll: true });
    input.select();
    return document.execCommand("copy");
  } finally {
    input.remove();
    if (focused instanceof HTMLElement) focused.focus({ preventScroll: true });
  }
}

async function copy(event: MouseEvent): Promise<void> {
  if (copying) return;
  copying = true;
  stop();
  const button = event.currentTarget as HTMLButtonElement;
  try {
    const text = formatFailureDetails(props.module, props.failure);
    const copied = await writeClipboard(text, button);
    feedback.value = copied ? "actions.copied" : "actions.copyFailed";
  } catch {
    feedback.value = "actions.copyFailed";
  } finally {
    copying = false;
    start();
  }
}
</script>

<template>
  <button
    type="button"
    class="copy-failure"
    :aria-label="t('modules.copyFailure')"
    @click.stop="copy"
  >
    <span aria-live="polite">{{ t(feedback) }}</span>
  </button>
</template>

<style scoped>
.copy-failure {
  min-height: 32px;
  min-width: 44px;
  padding: 0 4px;
  border: 0;
  background: transparent;
  color: inherit;
  font: inherit;
  font-size: 0.8125rem;
  font-weight: 600;
  cursor: pointer;
}
</style>
