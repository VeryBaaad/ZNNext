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
import RefreshSection from "../../components/RefreshSection.vue";
import { t } from "../../i18n";
import { themeScheme } from "../../theme";
import IosAlert from "./IosAlert.vue";
import IosConfigCard from "./IosConfigCard.vue";
import IosEmptyCard from "./IosEmptyCard.vue";
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

const failureRecordCount = computed(() =>
  (props.modules ?? []).reduce(
    (total, module) => total + (module.failed ?? []).length,
    0,
  ),
);
</script>

<template>
  <k-app class="znn-ios" theme="ios" :dark="isDark" :class="{ dark: isDark }">
    <k-page>
      <IosNavbar :refreshing="props.loading" />

      <div v-if="props.ksuAvailable" class="ios-content">
        <IosAlert v-if="failureRecordCount > 0" class="ios-banner">
          <IosIcon name="warning" :size="18" />
          <span>{{
            t("modules.failedBanner", { n: failureRecordCount })
          }}</span>
        </IosAlert>

        <RefreshSection section="system">
          <IosInfoCard
            v-if="props.system"
            :system="props.system"
            :status="props.status"
          />
        </RefreshSection>

        <RefreshSection section="status">
          <IosStatusCard
            v-if="props.status"
            :status="props.status"
            :root="props.system?.root ?? null"
          />
        </RefreshSection>

        <RefreshSection section="config">
          <IosConfigCard v-if="props.config" :config="props.config" />
        </RefreshSection>

        <RefreshSection section="modules">
          <IosModulesCard v-if="props.modules" :modules="props.modules" />
        </RefreshSection>
      </div>

      <IosEmptyCard v-else />

      <IosFooter />
    </k-page>
  </k-app>
</template>
