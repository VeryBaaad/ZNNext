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

import { M3eSnackbar } from "@m3e/web/snackbar";
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

interface ConfigOption {
  id: string;
  label: string;
}

interface ConfigRow {
  kind: string;
  label: string;
  value: string;
  options: ConfigOption[];
}

const props = defineProps<{ config: HookConfig }>();

const actions = useAppActions();

const root = ref<HTMLElement | null>(null);

const busy = ref(false);

const rows = computed<ConfigRow[]>(() => {
  const make = (row: HookRow, entry: HookEngineEntry): ConfigRow => ({
    kind: row.kind,
    label: t(row.labelKey),
    value: entry.value,
    options: entry.options.map((id) => ({
      id,
      label: hookOptionLabel(row.kind, id),
    })),
  });
  const uiRow: ConfigRow = {
    kind: "ui",
    label: t(UI_ROW_LABEL_KEY),
    value: uiMode.value,
    options: UI_MODES.map((id) => ({ id, label: uiModeLabel(id) })),
  };
  return configRows(props.config, make, uiRow);
});

function optionLabel(row: ConfigRow, id: string): string {
  return row.options.find((option) => option.id === id)?.label ?? id;
}

function openMenu(kind: string, event: MouseEvent): void {
  const button = event.currentTarget as HTMLElement | null;
  if (!button) return;
  const menu = root.value?.querySelector<
    HTMLElement & { toggle(target: HTMLElement): Promise<void> }
  >(`#znn-config-menu-${kind}`);
  if (!menu) return;
  const rect = button.getBoundingClientRect();
  const viewport = document.documentElement.clientWidth;
  menu.style.setProperty(
    "--znn-menu-inline-end",
    `${Math.max(8, Math.round(viewport - rect.right))}px`,
  );
  void menu.toggle(button);
}

async function select(row: ConfigRow, value: string): Promise<void> {
  if (value === row.value) return;
  if (row.kind === "ui") {
    if (isUiMode(value)) setUiMode(value);
    return;
  }
  if (busy.value) return;
  busy.value = true;
  try {
    await actions.applyHookConfig(row.kind as HookKind, value);
    M3eSnackbar.open(t("config.saved"));
  } catch (e) {
    console.warn("[znn] config-set:", e);
    M3eSnackbar.open(t("config.saveFailed"));
  } finally {
    busy.value = false;
  }
}

function onSwitch(row: ConfigRow, event: Event): void {
  const target = event.target as HTMLElement & { checked?: boolean };
  void select(row, target.checked ? "true" : "false");
}
</script>

<template>
  <m3e-card ref="root" class="config-card" variant="filled">
    <div slot="content" class="card-body">
      <div v-for="row in rows" :key="row.kind" class="config-row">
        <span class="config-label">{{ row.label }}</span>
        <m3e-switch
          v-if="row.kind === 'disable-zn'"
          class="config-switch"
          :checked="row.value === 'true'"
          :disabled="busy"
          :aria-label="row.label"
          @change="onSwitch(row, $event)"
        />
        <template v-else>
          <m3e-button
            class="config-value"
            size="extra-small"
            variant="tonal"
            @click="openMenu(row.kind, $event)"
          >
            <span>{{ optionLabel(row, row.value) }}</span>
            <m3e-icon slot="trailing-icon" name="arrow_drop_down"></m3e-icon>
          </m3e-button>
          <m3e-menu :id="`znn-config-menu-${row.kind}`" position-x="before">
            <m3e-menu-item-radio
              v-for="option in row.options"
              :key="option.id"
              :checked="option.id === row.value"
              @click="select(row, option.id)"
            >
              {{ option.label }}
            </m3e-menu-item-radio>
          </m3e-menu>
        </template>
      </div>
    </div>
  </m3e-card>
</template>
