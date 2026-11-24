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

import { kList, kListItem, kToggle } from "konsta/vue";
import { computed, ref } from "vue";

import type { HookConfig, HookEngineEntry, HookKind } from "../../api/config";
import { useAppActions } from "../../composables/app-actions";
import {
  configRows,
  UI_ROW_LABEL_KEY,
  hookOptionLabel,
  uiModeLabel,
} from "../../config-model";
import type { HookRow } from "../../config-model";
import { t } from "../../i18n";
import { UI_MODES, isUiMode, setUiMode, uiMode } from "../../ui-mode";
import IosAlert from "./IosAlert.vue";
import IosIcon from "./IosIcon.vue";
import IosPickerSheet from "./IosPickerSheet.vue";

interface ConfigRow {
  kind: string;
  label: string;
  value: string;
  options: string[];
  toggle?: boolean;
}

const props = defineProps<{ config: HookConfig; error?: boolean }>();

const actions = useAppActions();

const busy = ref(false);
const failure = ref("");
const toggleEpoch = ref(0);
const pickerRow = ref<ConfigRow | null>(null);

const pickerOpened = computed(() => pickerRow.value !== null);

const rows = computed<ConfigRow[]>(() => {
  const make = (row: HookRow, entry: HookEngineEntry): ConfigRow => ({
    kind: row.kind,
    label: t(row.labelKey),
    value: entry.value,
    options: [...entry.options],
    toggle: row.toggle,
  });
  const uiRow: ConfigRow = {
    kind: "ui",
    label: t(UI_ROW_LABEL_KEY),
    value: uiMode.value,
    options: [...UI_MODES],
  };
  return configRows(props.config, make, uiRow);
});

function optionLabel(row: ConfigRow, id: string): string {
  if (row.kind === "ui") return uiModeLabel(id);
  return hookOptionLabel(row.kind as HookKind, id);
}

function pickerLabel(id: string): string {
  const row = pickerRow.value;
  return row ? optionLabel(row, id) : id;
}

function closePicker(): void {
  pickerRow.value = null;
}

function openPicker(row: ConfigRow): void {
  if (busy.value || props.error) return;
  failure.value = "";
  pickerRow.value = row;
}

function pickOption(id: string): void {
  const row = pickerRow.value;
  closePicker();
  if (row) void select(row, id);
}

async function select(row: ConfigRow, value: string): Promise<void> {
  failure.value = "";
  if (value === row.value) return;
  if (row.kind === "ui") {
    if (isUiMode(value)) setUiMode(value);
    return;
  }
  if (busy.value) {
    toggleEpoch.value += 1;
    return;
  }
  busy.value = true;
  try {
    await actions.applyHookConfig(row.kind as HookKind, value);
  } catch (e) {
    console.warn("[znn] config-set:", e);
    failure.value = t("config.saveFailed");
    toggleEpoch.value += 1;
  } finally {
    busy.value = false;
  }
}

function onToggle(row: ConfigRow, event: Event): void {
  const target = event.target as HTMLInputElement | null;
  const enabled = target?.checked ?? false;
  void select(row, enabled ? "true" : "false");
}
</script>

<template>
  <section class="ios-section">
    <h2 class="ios-section-title">{{ t("card.settings") }}</h2>

    <k-list v-if="!props.error" inset strong class="ios-list">
      <template v-for="row in rows" :key="row.kind">
        <k-list-item v-if="row.toggle" :title="row.label">
          <template #after>
            <k-toggle
              :key="toggleEpoch"
              :checked="row.value === 'true'"
              :disabled="busy"
              @change="onToggle(row, $event)"
            >
              <span class="sr-only">{{ row.label }}</span>
            </k-toggle>
          </template>
        </k-list-item>

        <k-list-item
          v-else
          link
          link-component="button"
          :link-props="{
            type: 'button',
            'aria-haspopup': 'dialog',
            'aria-expanded': pickerRow?.kind === row.kind,
            disabled: busy,
          }"
          :title="row.label"
          @click="openPicker(row)"
        >
          <template #after>
            <span class="ios-value">{{ optionLabel(row, row.value) }}</span>
          </template>
        </k-list-item>
      </template>
    </k-list>

    <k-list v-else inset strong class="ios-list">
      <k-list-item :title="t('error.title')">
        <template #after>
          <IosIcon class="text-red-500" name="warning" :size="20" />
        </template>
      </k-list-item>
    </k-list>

    <IosAlert v-if="failure" class="ios-alert-inline">
      <IosIcon name="warning" :size="18" />
      <span>{{ failure }}</span>
    </IosAlert>
  </section>

  <IosPickerSheet
    :opened="pickerOpened"
    :title="pickerRow?.label ?? ''"
    :options="pickerRow?.options ?? []"
    :value="pickerRow?.value ?? ''"
    :label="pickerLabel"
    @close="closePicker"
    @select="pickOption"
  />
</template>
