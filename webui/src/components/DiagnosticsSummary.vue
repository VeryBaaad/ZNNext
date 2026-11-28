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
import { readLoadHistory } from "../api/load-history";
import { useAppActions } from "../composables/app-actions";
import { locale, t } from "../i18n";
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
const history = ref<Awaited<ReturnType<typeof readLoadHistory>>>([]);
const historyState = ref<"idle" | "loading" | "ready" | "failed">("idle");
const elapsedFormat = computed(
  () =>
    new Intl.NumberFormat(locale.value === "lzh" ? "zh-Hant" : locale.value, {
      style: "unit",
      unit: "millisecond",
      maximumFractionDigits: 3,
    }),
);

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

const historyFeedback = computed(() => {
  if (historyState.value === "loading") return t("diagnostics.historyLoading");
  if (historyState.value === "failed") return t("diagnostics.historyFailed");
  if (historyState.value === "ready" && !history.value.length) {
    return t("diagnostics.historyEmpty");
  }
  return "";
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

function onHistoryToggle(event: Event): void {
  const details = event.currentTarget as HTMLDetailsElement;
  if (details.open && historyState.value === "idle") void loadHistory();
}

async function loadHistory(): Promise<void> {
  if (historyState.value === "loading") return;
  historyState.value = "loading";
  try {
    history.value = await readLoadHistory();
    historyState.value = "ready";
  } catch {
    historyState.value = "failed";
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
    <details class="diagnostics-history" @toggle="onHistoryToggle">
      <summary>{{ t("diagnostics.historyTitle") }}</summary>
      <div class="diagnostics-actions">
        <button
          class="diagnostics-button"
          type="button"
          :disabled="historyState === 'loading'"
          @click="loadHistory"
        >
          {{
            t(historyState === "failed" ? "refresh.retry" : "actions.refresh")
          }}
        </button>
      </div>
      <p class="diagnostics-feedback" role="status">{{ historyFeedback }}</p>
      <ol
        v-if="historyState === 'ready' && history.length"
        class="diagnostics-history-list"
        role="list"
      >
        <li v-for="(entry, index) in history" :key="index">
          <time :datetime="entry.time">{{ entry.time }}</time>
          <code>{{ entry.library }}</code>
          <div class="diagnostics-history-result">
            <span>
              {{
                t(
                  entry.loaded
                    ? "diagnostics.historyLoaded"
                    : "diagnostics.historyLoadFailed",
                )
              }}
            </span>
            <span>{{ elapsedFormat.format(entry.elapsedUs / 1000) }}</span>
          </div>
        </li>
      </ol>
    </details>
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

.diagnostics-history {
  border-block-start: 1px solid
    color-mix(in srgb, currentColor 15%, transparent);
}

.diagnostics-history-list {
  margin: 0;
  padding: 4px 12px;
  border-radius: 8px;
  background: color-mix(in srgb, currentColor 6%, transparent);
  list-style: none;
  font-size: 0.8125rem;
  text-align: start;
}

.diagnostics-history-list li {
  display: grid;
  grid-template-columns: auto minmax(0, 1fr);
  gap: 2px 10px;
  padding-block: 8px;
  overflow-wrap: anywhere;
}

.diagnostics-history-list li + li {
  border-block-start: 1px solid
    color-mix(in srgb, currentColor 12%, transparent);
}

.diagnostics-history-result {
  display: flex;
  flex-wrap: wrap;
  grid-column: 2;
  gap: 0 8px;
}
</style>
