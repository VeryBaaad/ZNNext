<script setup lang="ts">
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
import ZnAlert from "./ZnAlert.vue";
import ZnCard from "./ZnCard.vue";
import ZnSelect from "./ZnSelect.vue";
import ZnSwitch from "./ZnSwitch.vue";

interface ConfigRow {
  kind: string;
  label: string;
  value: string;
  options: string[];
}

const props = defineProps<{ config: HookConfig; error?: boolean }>();

const actions = useAppActions();

const busy = ref(false);
const failure = ref("");

const rows = computed<ConfigRow[]>(() => {
  const make = (row: HookRow, entry: HookEngineEntry): ConfigRow => ({
    kind: row.kind,
    label: t(row.labelKey),
    value: entry.value,
    options: [...entry.options],
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

async function select(row: ConfigRow, value: string): Promise<void> {
  failure.value = "";
  if (value === row.value) return;
  if (row.kind === "ui") {
    if (isUiMode(value)) setUiMode(value);
    return;
  }
  if (busy.value) return;
  busy.value = true;
  try {
    await actions.applyHookConfig(row.kind as HookKind, value);
  } catch (e) {
    console.warn("[znn] config-set:", e);
    failure.value = t("config.saveFailed");
  } finally {
    busy.value = false;
  }
}
</script>

<template>
  <ZnCard :title="t('card.settings')" :error="props.error">
    <div v-if="!props.error" class="zn-rows">
      <div v-for="row in rows" :key="row.kind" class="zn-row">
        <span class="zn-field-label">{{ row.label }}</span>
        <ZnSwitch
          v-if="row.kind === 'disable-zn'"
          :model-value="row.value === 'true'"
          :disabled="busy"
          :label="row.label"
          @update:model-value="
            (value: boolean) => select(row, value ? 'true' : 'false')
          "
        />
        <ZnSelect
          v-else
          :model-value="row.value"
          :options="row.options"
          :label="(id: string) => optionLabel(row, id)"
          :disabled="busy"
          @select="(id: string) => select(row, id)"
        />
      </div>
    </div>
    <ZnAlert v-if="failure">{{ failure }}</ZnAlert>
  </ZnCard>
</template>
