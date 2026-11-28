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

import { kList, kListItem } from "konsta/vue";
import { computed } from "vue";

import type { DaemonStatus } from "../../api/daemon";
import { cleanVersion, type RootImplementation } from "../../api/system";
import { t } from "../../i18n";
import IosTag from "./IosTag.vue";

const props = defineProps<{
  status: DaemonStatus;
  root: RootImplementation | null;
}>();

const rootLabel = computed(() => {
  const impl = props.root;
  if (!impl) return t("info.rootImpl.none");
  const entries = [
    { name: "KernelSU", version: cleanVersion(impl.kernelSU) },
    { name: "Magisk", version: cleanVersion(impl.magisk) },
    { name: "APatch", version: cleanVersion(impl.apatch) },
  ].filter((entry) => entry.version !== null);
  if (entries.length === 0) return t("info.rootImpl.none");
  return entries.map((entry) => `${entry.name} (${entry.version})`).join(", ");
});

const daemonLabel = computed(() => {
  if (!props.status.running) return t("zn.stopped");
  return props.status.pid
    ? `${t("zn.running")} (${props.status.pid})`
    : t("zn.running");
});
</script>

<template>
  <section class="ios-section">
    <h2 class="ios-section-title">{{ t("card.status") }}</h2>

    <k-list inset strong class="ios-list">
      <k-list-item v-if="props.root" :title="t('info.rootImpl')">
        <template #after>
          <IosTag tone="ok">{{ rootLabel }}</IosTag>
        </template>
      </k-list-item>

      <k-list-item :title="t('zn.daemon')">
        <template #after>
          <IosTag :tone="props.status.running ? 'ok' : 'warn'">
            {{ daemonLabel }}
          </IosTag>
        </template>
      </k-list-item>
    </k-list>
  </section>
</template>
