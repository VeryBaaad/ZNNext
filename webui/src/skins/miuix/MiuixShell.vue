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

import "miuix-vue/style.css";
import "../../styles/miuix.css";

import {
  MiuixScrollArea,
  MiuixSnackbarHost,
  setThemeMode as setMiuixThemeMode,
} from "miuix-vue";
import { onBeforeUnmount, watch } from "vue";

import type { HookConfig } from "../../api/config";
import type { DaemonStatus } from "../../api/daemon";
import type { ZnnModule } from "../../api/modules";
import type { SystemInfo } from "../../api/system";
import RefreshSection from "../../components/RefreshSection.vue";
import { materialSeed, materialSpec } from "../../color/material-color";
import { themeScheme } from "../../theme";
import MiuixAppBar from "./MiuixAppBar.vue";
import MiuixConfigCard from "./MiuixConfigCard.vue";
import MiuixEmptyCard from "./MiuixEmptyCard.vue";
import MiuixFooter from "./MiuixFooter.vue";
import MiuixInfoCard from "./MiuixInfoCard.vue";
import MiuixLargeTitle from "./MiuixLargeTitle.vue";
import MiuixModulesCard from "./MiuixModulesCard.vue";
import MiuixStatusCard from "./MiuixStatusCard.vue";
import { applyMiuixMonet, clearMiuixMonet, miuixMonetEnabled } from "./monet";

const props = defineProps<{
  ksuAvailable: boolean;
  loading: boolean;
  status: DaemonStatus | null;
  system: SystemInfo | null;
  modules: ZnnModule[] | null;
  config: HookConfig | null;
}>();

/**
 * miuix-vue owns the `m-theme-dark` class and re-asserts its own default the
 * moment its module is evaluated, so the resolved scheme has to be pushed into
 * its controller as well as onto the document.
 */
watch(themeScheme, (scheme) => setMiuixThemeMode(scheme), { immediate: true });

watch(
  [themeScheme, materialSeed, materialSpec, miuixMonetEnabled],
  ([scheme, , , enabled]) => {
    if (enabled) {
      applyMiuixMonet(
        materialSeed.value,
        materialSpec.value,
        scheme === "dark",
      );
    } else {
      clearMiuixMonet();
    }
  },
  { immediate: true },
);

onBeforeUnmount(clearMiuixMonet);
</script>

<template>
  <div class="znn-miuix">
    <MiuixAppBar
      :ksu-available="props.ksuAvailable"
      :refreshing="props.loading"
    />

    <MiuixScrollArea class="znn-miuix-scroll">
      <MiuixLargeTitle />

      <main class="znn-content">
        <template v-if="props.ksuAvailable">
          <RefreshSection section="status">
            <MiuixStatusCard v-if="props.status" :status="props.status" />
          </RefreshSection>
          <RefreshSection section="system">
            <MiuixInfoCard
              v-if="props.system"
              :system="props.system"
              :status="props.status"
            />
          </RefreshSection>
          <RefreshSection section="config">
            <MiuixConfigCard v-if="props.config" :config="props.config" />
          </RefreshSection>
          <RefreshSection section="modules">
            <MiuixModulesCard v-if="props.modules" :modules="props.modules" />
          </RefreshSection>
        </template>
        <MiuixEmptyCard v-else />
      </main>

      <MiuixFooter />
    </MiuixScrollArea>

    <MiuixSnackbarHost />
  </div>
</template>
