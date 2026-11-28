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
import { formatPageSize, type SystemInfo } from "../../api/system";
import { activeModeLabel, modeReasonText } from "../../config-model";
import { t } from "../../i18n";
import ZnCard from "./ZnCard.vue";

const props = defineProps<{
  system: SystemInfo;
  status: DaemonStatus | null;
}>();

interface InfoRow {
  label: string;
  value: string;
  note?: string;
}

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

const rows = computed<InfoRow[]>(() => [
  { label: "Kernel", value: props.system.kernel || t("info.unknown") },
  {
    label: "Android SDK",
    value: props.system.sdk > 0 ? String(props.system.sdk) : t("info.unknown"),
  },
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
]);
</script>

<template>
  <ZnCard :title="t('card.basic')">
    <template v-for="row in rows" :key="row.label">
      <div class="zn-divider" role="separator"></div>
      <div class="zn-field-label">{{ row.label }}</div>
      <div class="zn-field-value">{{ row.value }}</div>
      <div v-if="row.note" class="zn-field-value mode-reason">
        {{ row.note }}
      </div>
    </template>
  </ZnCard>
</template>

<style scoped>
.mode-reason {
  margin-block-start: 2px;
  color: var(--zn-text-muted);
  font-size: 0.85em;
}
</style>
