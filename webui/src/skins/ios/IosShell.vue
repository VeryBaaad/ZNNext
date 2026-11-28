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

import "../../styles/ios.css";

import { kApp, kPage } from "konsta/vue";
import { computed } from "vue";

import type { HookConfig } from "../../api/config";
import type { DaemonStatus } from "../../api/daemon";
import type { ZnnModule } from "../../api/modules";
import type { SystemInfo } from "../../api/system";
import { t } from "../../i18n";
import { themeScheme } from "../../theme";
import IosAlert from "./IosAlert.vue";
import IosConfigCard from "./IosConfigCard.vue";
import IosEmptyCard from "./IosEmptyCard.vue";
import IosErrorCard from "./IosErrorCard.vue";
import IosFooter from "./IosFooter.vue";
import IosIcon from "./IosIcon.vue";
import IosInfoCard from "./IosInfoCard.vue";
import IosModulesCard from "./IosModulesCard.vue";
import IosNavbar from "./IosNavbar.vue";
import IosStatusCard from "./IosStatusCard.vue";

const props = defineProps<{
  ksuAvailable: boolean;
  loading: boolean;
  status: DaemonStatus | null;
  system: SystemInfo | null;
  modules: ZnnModule[] | null;
  config: HookConfig | null;
}>();

const isDark = computed(() => themeScheme.value === "dark");

const failed = computed(
  () =>
    props.ksuAvailable &&
    !props.loading &&
    !props.status &&
    !props.system &&
    !props.modules &&
    !props.config,
);

const showDashboard = computed(() => props.ksuAvailable && !failed.value);

function unavailable(present: boolean): boolean {
  return props.ksuAvailable && !props.loading && !present;
}

const failureRecordCount = computed(() =>
  (props.modules ?? []).reduce(
    (total, module) => total + (module.failed ?? []).length,
    0,
  ),
);

const EMPTY_SYSTEM: SystemInfo = {
  kernel: "",
  sdk: 0,
  abi: "",
  abilist: "",
  root: { magisk: null, kernelSU: null, apatch: null },
};

const EMPTY_CONFIG: HookConfig = {
  inlineHook: { value: "", options: [] },
  pltHook: { value: "", options: [] },
  mode: { value: "", options: [] },
  disableZn: { value: "", options: [] },
  debugdata: { value: "", options: [] },
  hotUpdate: { value: "", options: [] },
};
</script>

<template>
  <k-app class="znn-ios" theme="ios" :dark="isDark" :class="{ dark: isDark }">
    <k-page>
      <IosNavbar :refreshing="props.loading" />

      <template v-if="showDashboard">
        <div class="ios-content">
          <IosAlert v-if="failureRecordCount > 0" class="ios-banner">
            <IosIcon name="warning" :size="18" />
            <span>{{
              t("modules.failedBanner", { n: failureRecordCount })
            }}</span>
          </IosAlert>

          <IosInfoCard
            v-if="props.system || unavailable(!!props.system)"
            :system="props.system ?? EMPTY_SYSTEM"
            :status="props.status"
            :error="unavailable(!!props.system)"
          />

          <IosStatusCard
            :status="props.status"
            :root="props.system?.root ?? null"
            :error="unavailable(!!props.status)"
          />

          <IosConfigCard
            v-if="props.config || unavailable(!!props.config)"
            :config="props.config ?? EMPTY_CONFIG"
            :error="unavailable(!!props.config)"
          />

          <IosModulesCard
            v-if="props.modules || unavailable(!!props.modules)"
            :modules="props.modules"
            :error="unavailable(!!props.modules)"
          />
        </div>
      </template>

      <template v-else>
        <IosErrorCard v-if="props.ksuAvailable" />
        <IosEmptyCard v-else />
      </template>

      <IosFooter />
    </k-page>
  </k-app>
</template>
