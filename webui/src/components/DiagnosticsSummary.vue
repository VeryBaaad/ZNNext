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

import { computed, ref } from "vue";

import { readDiagnosticLogs, saveDiagnosticReport } from "../api/diagnostics";
import { isKsuAvailable } from "../api/ksu";
import { useAppActions } from "../composables/app-actions";
import { t } from "../i18n";
import CopyButton from "./CopyButton.vue";

const available = isKsuAvailable();
const { diagnosticSummary } = useAppActions();
const summary = computed(diagnosticSummary);
const logs = ref<string | null>(null);
const loadingLogs = ref(false);
const logsFailed = ref(false);
const exporting = ref(false);
const exportFailed = ref(false);
const savedPath = ref("");

const text = computed(() =>
  logs.value ? `${summary.value}\n\n${logs.value}` : summary.value,
);

const logsAction = computed(() => {
  if (loadingLogs.value) return t("diagnostics.loadingLogs");
  if (logsFailed.value) return t("diagnostics.retryLogs");
  return t(
    logs.value === null ? "diagnostics.loadLogs" : "diagnostics.refreshLogs",
  );
});

const exportFeedback = computed(() => {
  if (exportFailed.value) return t("diagnostics.exportFailed");
  return savedPath.value
    ? t("diagnostics.saved", { path: savedPath.value })
    : "";
});

async function loadLogs(): Promise<void> {
  if (loadingLogs.value) return;
  loadingLogs.value = true;
  logsFailed.value = false;
  logs.value = null;
  try {
    logs.value = await readDiagnosticLogs();
  } catch {
    logsFailed.value = true;
  } finally {
    loadingLogs.value = false;
  }
}

async function exportReport(): Promise<void> {
  if (exporting.value) return;
  exporting.value = true;
  exportFailed.value = false;
  savedPath.value = "";
  try {
    savedPath.value = await saveDiagnosticReport(
      summary.value,
      logs.value ?? undefined,
    );
  } catch {
    exportFailed.value = true;
  } finally {
    exporting.value = false;
  }
}
</script>

<template>
  <details v-if="available" class="diagnostics-summary">
    <summary>{{ t("diagnostics.title") }}</summary>
    <div class="diagnostics-actions">
      <button
        class="diagnostics-button"
        type="button"
        :disabled="loadingLogs"
        @click="loadLogs"
      >
        {{ logsAction }}
      </button>
    </div>
    <p class="diagnostics-feedback" role="status">
      {{ logsFailed ? t("diagnostics.logsFailed") : "" }}
    </p>
    <pre tabindex="0" :aria-busy="loadingLogs">{{ text }}</pre>
    <div class="diagnostics-actions">
      <CopyButton :label="t('diagnostics.copy')" :text="text" />
      <button
        class="diagnostics-button"
        type="button"
        :disabled="exporting"
        @click="exportReport"
      >
        {{ t(exporting ? "diagnostics.exporting" : "diagnostics.export") }}
      </button>
    </div>
    <p class="diagnostics-feedback" role="status">{{ exportFeedback }}</p>
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
  flex-wrap: wrap;
  justify-content: flex-end;
  gap: 0 12px;
  margin-block-start: 4px;
}

.diagnostics-actions button {
  min-height: 44px;
}

.diagnostics-button {
  min-width: 44px;
  padding: 0 4px;
  border: 0;
  background: transparent;
  color: inherit;
  font: inherit;
  font-size: 0.8125rem;
  font-weight: 600;
  cursor: pointer;
}

.diagnostics-button:disabled {
  opacity: 0.6;
  cursor: wait;
}

.diagnostics-feedback {
  margin: 0;
  font-size: 0.8125rem;
  overflow-wrap: anywhere;
}

.diagnostics-feedback:not(:empty) {
  margin-block: 4px 8px;
}
</style>
