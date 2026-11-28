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

import { computed, defineAsyncComponent, onMounted, type Component } from "vue";

import { provideAppActions } from "./composables/app-actions";
import { useAppData } from "./composables/use-app-data";
import { modeSaveMessage } from "./config-model";
import { applyLocale, t } from "./i18n";
import { initTheme } from "./theme";
import { initUiMode, uiMode, type UiMode } from "./ui-mode";
import { formatDiagnosticSummary } from "./util/diagnostic-summary";

const MaterialShell = defineAsyncComponent(
  () => import("./skins/material/MaterialShell.vue"),
);
const MiuixShell = defineAsyncComponent(
  () => import("./skins/miuix/MiuixShell.vue"),
);
const ZnShell = defineAsyncComponent(() => import("./skins/zn/ZnShell.vue"));
const IosShell = defineAsyncComponent(() => import("./skins/ios/IosShell.vue"));

const data = useAppData();
const {
  ksuAvailable,
  loading,
  status,
  system,
  modules,
  config,
  sections,
  load,
  retrySection,
  applyHookConfig,
} = data;

provideAppActions({
  refresh: () => {
    void load();
  },
  sections,
  retrySection,
  diagnosticSummary: () => formatDiagnosticSummary(data),
  async applyHookConfig(kind, value) {
    await applyHookConfig(kind, value);
    if (kind !== "mode") return t("config.saved");
    return modeSaveMessage(value, sections.status.failed ? null : status.value);
  },
});

const SHELLS: Record<UiMode, Component> = {
  material: MaterialShell,
  miuix: MiuixShell,
  zn: ZnShell,
  ios: IosShell,
};

const shell = computed(() => SHELLS[uiMode.value]);

initTheme();
initUiMode();
applyLocale();

onMounted(() => {
  void load();
});
</script>

<template>
  <component
    :is="shell"
    :key="uiMode"
    :ksu-available="ksuAvailable"
    :loading="loading"
    :status="status"
    :system="system"
    :modules="modules"
    :config="config"
  />
</template>
