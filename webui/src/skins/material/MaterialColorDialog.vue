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

import { ref, watch } from "vue";

import { t } from "../../i18n";
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
</script>

<template>
  <Teleport to="body">
    <m3e-dialog
      class="color-dialog"
      :open="props.open"
      dismissible
      :close-label="t('color.close')"
      @closed="emit('close')"
    >
      <span slot="header">{{ t("color.title") }}</span>

      <div class="color-dialog-body">
        <ColorWheelPicker
          :model-value="materialSeed"
          @update:model-value="chooseSeed"
        />

        <div class="hex-row">
          <label class="seed-hex-field">
            <input
              class="seed-hex-input"
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
          <m3e-icon-button
            class="hex-reset"
            :aria-label="t('color.reset')"
            :title="t('color.reset')"
            @click="resetSeed"
          >
            <m3e-icon name="restart_alt"></m3e-icon>
          </m3e-icon-button>
        </div>

        <div class="seed-presets">
          <button
            v-for="preset in MATERIAL_SEED_PRESETS"
            :key="preset"
            class="seed-preset"
            :class="{ 'is-active': isPresetActive(preset) }"
            type="button"
            :aria-label="preset"
            :aria-pressed="isPresetActive(preset)"
            :title="preset"
            :style="{ '--znn-swatch': preset }"
            @click="chooseSeed(preset)"
          >
            <span class="seed-preset-dot"></span>
          </button>
        </div>

        <div class="color-section">
          <span class="color-section-label">{{ t("color.spec") }}</span>
          <m3e-segmented-button class="spec-switch">
            <m3e-button-segment
              v-for="spec in MATERIAL_SPECS"
              :key="spec"
              :checked="spec === materialSpec"
              @click="chooseSpec(spec)"
            >
              {{ specLabel(spec) }}
            </m3e-button-segment>
          </m3e-segmented-button>
        </div>
      </div>

      <div slot="actions" end>
        <m3e-button variant="text" @click="resetSeed">
          {{ t("color.reset") }}
        </m3e-button>
        <m3e-button variant="filled">
          <m3e-dialog-action return-value="done">
            {{ t("color.done") }}
          </m3e-dialog-action>
        </m3e-button>
      </div>
    </m3e-dialog>
  </Teleport>
</template>
