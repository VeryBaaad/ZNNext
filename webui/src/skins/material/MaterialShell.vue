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

import "./elements";

import { watch } from "vue";

import type { HookConfig } from "../../api/config";
import type { DaemonStatus } from "../../api/daemon";
import type { ZnnModule } from "../../api/modules";
import type { SystemInfo } from "../../api/system";
import RefreshSection from "../../components/RefreshSection.vue";
import { themeScheme } from "../../theme";
import MaterialConfigCard from "./MaterialConfigCard.vue";
import MaterialEmptyCard from "./MaterialEmptyCard.vue";
import MaterialFooter from "./MaterialFooter.vue";
import MaterialInfoCard from "./MaterialInfoCard.vue";
import MaterialModulesCard from "./MaterialModulesCard.vue";
import MaterialStatusCard from "./MaterialStatusCard.vue";
import MaterialTopBar from "./MaterialTopBar.vue";
import {
  applyMaterialTheme,
  materialSeed,
  materialSpec,
} from "../../color/material-color";

const props = defineProps<{
  ksuAvailable: boolean;
  loading: boolean;
  status: DaemonStatus | null;
  system: SystemInfo | null;
  modules: ZnnModule[] | null;
  config: HookConfig | null;
}>();

watch(
  [themeScheme, materialSeed, materialSpec],
  ([scheme]) => {
    applyMaterialTheme(
      materialSeed.value,
      materialSpec.value,
      scheme === "dark",
    );
    document
      .getElementById("znn-theme")
      ?.setAttribute("color", materialSeed.value);
  },
  { immediate: true },
);
</script>

<template>
  <MaterialTopBar
    :ksu-available="props.ksuAvailable"
    :refreshing="props.loading"
  />

  <main class="znn-content">
    <template v-if="props.ksuAvailable">
      <RefreshSection section="status">
        <MaterialStatusCard v-if="props.status" :status="props.status" />
      </RefreshSection>
      <RefreshSection section="system">
        <MaterialInfoCard
          v-if="props.system"
          :system="props.system"
          :status="props.status"
        />
      </RefreshSection>
      <RefreshSection section="config">
        <MaterialConfigCard v-if="props.config" :config="props.config" />
      </RefreshSection>
      <RefreshSection section="modules">
        <MaterialModulesCard v-if="props.modules" :modules="props.modules" />
      </RefreshSection>
    </template>
    <MaterialEmptyCard v-else />
  </main>

  <MaterialFooter />
</template>
