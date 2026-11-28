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

import { isKsuAvailable } from "../api/ksu";
import { useAppActions } from "../composables/app-actions";
import { t } from "../i18n";
import CopyButton from "./CopyButton.vue";

const available = isKsuAvailable();
const { diagnosticSummary } = useAppActions();
const text = computed(diagnosticSummary);
</script>

<template>
  <details v-if="available" class="diagnostics-summary">
    <summary>{{ t("diagnostics.title") }}</summary>
    <pre tabindex="0">{{ text }}</pre>
    <div class="diagnostics-actions">
      <CopyButton :label="t('diagnostics.copy')" :text="text" />
    </div>
  </details>
</template>

<style scoped>
.diagnostics-summary {
  box-sizing: border-box;
  width: min(100%, var(--znn-column-width));
  min-width: 0;
  align-self: center;
  font-size: 0.875rem;
}

.diagnostics-summary summary {
  width: fit-content;
  max-width: 100%;
  min-height: 44px;
  margin-inline: auto;
  padding-block: 12px;
  line-height: 20px;
  cursor: pointer;
}

.diagnostics-summary pre {
  max-height: 20rem;
  margin: 0;
  padding: 12px;
  overflow: auto;
  overflow-wrap: anywhere;
  white-space: pre-wrap;
  border-radius: 8px;
  background: color-mix(in srgb, currentColor 6%, transparent);
  font-size: 0.8125rem;
  line-height: 1.5;
}

.diagnostics-actions {
  display: flex;
  justify-content: flex-end;
  margin-block-start: 4px;
}

.diagnostics-actions button {
  min-height: 44px;
}
</style>
