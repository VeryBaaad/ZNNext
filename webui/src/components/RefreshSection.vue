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

import { useAppActions } from "../composables/app-actions";
import type { DataSection } from "../composables/use-app-data";
import { formatDateTime, t } from "../i18n";

const props = defineProps<{ section: DataSection }>();
const { sections, retrySection } = useAppActions();
const state = computed(() => sections[props.section]);
const sectionLabel = computed(() => t(`refresh.section.${props.section}`));

const message = computed(() => {
  const { lastSuccess } = state.value;
  const section = sectionLabel.value;
  if (lastSuccess === null) return t("refresh.unavailable", { section });
  return t("refresh.stale", { section, time: formatDateTime(lastSuccess) });
});
</script>

<template>
  <div
    v-if="state.failed || state.lastSuccess !== null"
    class="refresh-section"
  >
    <slot />
    <div v-if="state.failed" class="refresh-feedback" :data-section="section">
      <span role="status">{{ message }}</span>
      <button
        type="button"
        :disabled="state.loading"
        :aria-label="t('refresh.retrySection', { section: sectionLabel })"
        @click="retrySection(section)"
      >
        {{ t("refresh.retry") }}
      </button>
    </div>
  </div>
</template>

<style scoped>
.refresh-section {
  display: flex;
  flex-direction: column;
  min-width: 0;
  gap: 4px;
}

.refresh-feedback {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  margin-inline: 8px;
  font-size: 0.8125rem;
  line-height: 1.4;
  border-inline-start: 2px solid currentColor;
  padding-inline-start: 10px;
}

.refresh-feedback button {
  flex: none;
  min-width: 44px;
  min-height: 44px;
  padding: 0 8px;
  border: 0;
  background: transparent;
  color: inherit;
  font: inherit;
  font-weight: 600;
  text-decoration: underline;
  cursor: pointer;
}

.refresh-feedback button:disabled {
  cursor: wait;
  opacity: 0.6;
}
</style>
