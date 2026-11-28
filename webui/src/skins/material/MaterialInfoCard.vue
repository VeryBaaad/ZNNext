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

import { computed } from "vue";

import type { DaemonStatus } from "../../api/daemon";
import {
  cleanVersion,
  formatPageSize,
  type SystemInfo,
} from "../../api/system";
import { activeModeLabel, modeReasonText } from "../../config-model";
import { t } from "../../i18n";

interface RootEntry {
  name: string;
  version: string | null;
}

interface InfoRow {
  label: string;
  value: string;
  note?: string;
}

const props = defineProps<{
  system: SystemInfo;
  status: DaemonStatus | null;
}>();

const rootImplText = computed(() => {
  const entries: RootEntry[] = [
    { name: "KernelSU", version: cleanVersion(props.system.root.kernelSU) },
    { name: "Magisk", version: cleanVersion(props.system.root.magisk) },
    { name: "APatch", version: cleanVersion(props.system.root.apatch) },
  ].filter((entry) => entry.version !== null);
  if (entries.length === 0) return t("info.rootImpl.none");
  return entries.map((entry) => `${entry.name} (${entry.version})`).join(", ");
});

const abiText = computed(() => {
  const primary = (props.system.abi || "").trim();
  const rest = (props.system.abilist || "")
    .split(",")
    .map((value) => value.trim())
    .filter((value) => value.length > 0 && value !== primary);
  const parts = [primary ? `${primary} (primary)` : t("info.unknown")];
  parts.push(...rest);
  return parts.join(", ");
});

const rows = computed<InfoRow[]>(() => {
  const sdk =
    props.system.sdk > 0 ? String(props.system.sdk) : t("info.unknown");
  return [
    { label: t("info.rootImpl"), value: rootImplText.value },
    { label: "Kernel", value: props.system.kernel || t("info.unknown") },
    { label: "Android SDK", value: sdk },
    { label: "ABI", value: abiText.value },
    {
      label: t("info.pageSize"),
      value: formatPageSize(props.system.pageSize) ?? t("info.unknown"),
    },
    {
      label: t("info.mode"),
      value: activeModeLabel(props.status) ?? t("info.unknown"),
      note: modeReasonText(props.status),
    },
  ];
});
</script>

<template>
  <m3e-card class="info-card" variant="filled">
    <div slot="content" class="card-body">
      <div v-for="row in rows" :key="row.label" class="info-row">
        <span class="info-label">{{ row.label }}</span>
        <span class="info-value">{{ row.value }}</span>
        <span v-if="row.note" class="mode-reason">{{ row.note }}</span>
      </div>
    </div>
  </m3e-card>
</template>

<style scoped>
.mode-reason {
  font-size: var(--md-sys-typescale-body-small-font-size);
  line-height: var(--md-sys-typescale-body-small-line-height);
  color: var(--md-sys-color-on-surface-variant);
  overflow-wrap: anywhere;
}
</style>
