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
 * Copyright (C) 2026 VeryBaaad <verybaad@outlook.com>
 */

import { MiuixButton, MiuixDialog } from "miuix-vue";
import { ref, watch } from "vue";

import ColorWheelPicker from "../../color/ColorWheelPicker.vue";
import {
  DEFAULT_MATERIAL_SEED,
  MATERIAL_SEED_PRESETS,
  MATERIAL_SPECS,
  materialSeed,
  materialSpec,
  normalizeSeed,
  setMaterialSeed,
  setMaterialSpec,
  specLabel,
  type MaterialSpec,
} from "../../color/material-color";
import { t } from "../../i18n";

const props = defineProps<{ open: boolean }>();

const emit = defineEmits<{ close: [] }>();

const hexDraft = ref(formatHex(materialSeed.value));

watch(
  () => props.open,
  (open) => {
    if (open) hexDraft.value = formatHex(materialSeed.value);
  },
);

watch(materialSeed, (seed) => {
  if (normalizeSeed(hexDraft.value) !== seed) hexDraft.value = formatHex(seed);
});

function formatHex(seed: string): string {
  return (normalizeSeed(seed) ?? DEFAULT_MATERIAL_SEED).toUpperCase();
}

function chooseSeed(seed: string): void {
  setMaterialSeed(seed);
  hexDraft.value = formatHex(materialSeed.value);
}

function resetSeed(): void {
  chooseSeed(DEFAULT_MATERIAL_SEED);
}

function onHexInput(event: Event): void {
  const value = (event.target as HTMLInputElement).value;
  hexDraft.value = value;
  const normalized = normalizeSeed(value);
  if (normalized) setMaterialSeed(normalized);
}

function commitHex(): void {
  const normalized = normalizeSeed(hexDraft.value);
  if (normalized) {
    setMaterialSeed(normalized);
    hexDraft.value = formatHex(normalized);
    return;
  }
  hexDraft.value = formatHex(materialSeed.value);
}

function onHexKeydown(event: KeyboardEvent): void {
  if (event.key !== "Enter") return;
  event.preventDefault();
  commitHex();
  (event.target as HTMLInputElement).blur();
}

function chooseSpec(spec: MaterialSpec): void {
  setMaterialSpec(spec);
}

function isPresetActive(seed: string): boolean {
  return normalizeSeed(seed) === materialSeed.value;
}

function onDialogUpdate(value: boolean): void {
  if (!value) emit("close");
}
</script>

<template>
  <MiuixDialog
    :model-value="props.open"
    :title="t('color.title')"
    @update:model-value="onDialogUpdate"
  >
    <div class="miuix-color-body">
      <ColorWheelPicker
        :model-value="materialSeed"
        @update:model-value="chooseSeed"
      />

      <div class="miuix-color-hex">
        <label class="miuix-color-hex-field">
          <input
            class="miuix-color-hex-input"
            type="text"
            inputmode="text"
            spellcheck="false"
            maxlength="7"
            :aria-label="t('color.seed')"
            :value="hexDraft"
            @input="onHexInput"
            @change="commitHex"
            @blur="commitHex"
            @keydown="onHexKeydown"
          />
        </label>
      </div>

      <div class="miuix-color-presets">
        <button
          v-for="preset in MATERIAL_SEED_PRESETS"
          :key="preset"
          class="miuix-color-preset"
          :class="{ 'is-active': isPresetActive(preset) }"
          type="button"
          :aria-label="preset"
          :aria-pressed="isPresetActive(preset)"
          :title="preset"
          :style="{ '--znn-swatch': preset }"
          @click="chooseSeed(preset)"
        >
          <span class="miuix-color-preset-dot"></span>
        </button>
      </div>

      <div class="miuix-color-spec">
        <span class="miuix-color-spec-label">{{ t("color.spec") }}</span>
        <div class="miuix-color-spec-buttons">
          <MiuixButton
            v-for="spec in MATERIAL_SPECS"
            :key="spec"
            :type="spec === materialSpec ? 'primary' : 'default'"
            @click="chooseSpec(spec)"
          >
            {{ specLabel(spec) }}
          </MiuixButton>
        </div>
      </div>
    </div>

    <div class="miuix-color-actions">
      <MiuixButton @click="resetSeed">{{ t("color.reset") }}</MiuixButton>
      <MiuixButton type="primary" @click="emit('close')">
        {{ t("color.done") }}
      </MiuixButton>
    </div>
  </MiuixDialog>
</template>
