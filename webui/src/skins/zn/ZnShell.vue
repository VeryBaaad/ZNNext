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

import "../../styles/zn.css";

import { computed } from "vue";

import type { HookConfig } from "../../api/config";
import type { DaemonStatus } from "../../api/daemon";
import type { ZnnModule } from "../../api/modules";
import type { SystemInfo } from "../../api/system";
import RefreshSection from "../../components/RefreshSection.vue";
import { t } from "../../i18n";
import ZnAlert from "./ZnAlert.vue";
import ZnConfigCard from "./ZnConfigCard.vue";
import ZnEmptyCard from "./ZnEmptyCard.vue";
import ZnFooter from "./ZnFooter.vue";
import ZnInfoCard from "./ZnInfoCard.vue";
import ZnStatusCard from "./ZnStatusCard.vue";
import ZnTopBar from "./ZnTopBar.vue";

const props = defineProps<{
  ksuAvailable: boolean;
  loading: boolean;
  status: DaemonStatus | null;
  system: SystemInfo | null;
  modules: ZnnModule[] | null;
  config: HookConfig | null;
}>();

const failureRecordCount = computed(() =>
  (props.modules ?? []).reduce(
    (total, module) => total + (module.failed ?? []).length,
    0,
  ),
);
</script>

<template>
  <div class="znn-zn">
    <ZnTopBar />

    <main class="zn-container">
      <ZnAlert v-if="props.ksuAvailable && failureRecordCount > 0">
        {{ t("modules.failedBanner", { n: failureRecordCount }) }}
      </ZnAlert>

      <template v-if="props.ksuAvailable">
        <RefreshSection section="system">
          <ZnInfoCard
            v-if="props.system"
            :system="props.system"
            :status="props.status"
          />
        </RefreshSection>

        <ZnStatusCard
          :status="props.status"
          :root="props.system?.root ?? null"
          :modules="props.modules"
          :refreshing="props.loading"
        />

        <RefreshSection section="config">
          <ZnConfigCard v-if="props.config" :config="props.config" />
        </RefreshSection>
      </template>

      <ZnEmptyCard v-else />
    </main>

    <ZnFooter />
  </div>
</template>
