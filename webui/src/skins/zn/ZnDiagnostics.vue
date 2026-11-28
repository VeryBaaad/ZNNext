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

import { useDiagnostics } from "../../composables/use-diagnostics";
import { t } from "../../i18n";
import ZnCollapse from "./ZnCollapse.vue";
import ZnTag from "./ZnTag.vue";

const d = useDiagnostics();
</script>

<template>
  <section v-if="d.available" class="zn-card zn-diagnostics">
    <ZnCollapse :title="t('diagnostics.title')">
      <pre class="diagnostic-text" tabindex="0" :aria-busy="d.loadingLogs">{{
        d.text
      }}</pre>
      <div class="diagnostic-actions">
        <button
          class="zn-link"
          type="button"
          :disabled="d.loadingLogs"
          @click="d.loadLogs"
        >
          {{ d.logsAction }}
        </button>
        <button
          class="zn-link"
          type="button"
          :aria-label="t('diagnostics.copy')"
          @click="d.copy"
        >
          <span aria-live="polite">{{ d.copyLabel }}</span>
        </button>
        <button
          class="zn-link"
          type="button"
          :disabled="d.exporting"
          @click="d.exportReport"
        >
          {{ t(d.exporting ? "diagnostics.exporting" : "diagnostics.export") }}
        </button>
      </div>
      <p class="diagnostic-feedback" role="status">
        {{ d.logsFailed ? t("diagnostics.logsFailed") : "" }}
      </p>
      <p class="diagnostic-feedback" role="status">{{ d.exportFeedback }}</p>
      <ZnCollapse
        class="zn-history"
        :title="t('diagnostics.historyTitle')"
        :initially-open="d.historyOpen"
        @toggle="d.historyOpen = $event"
      >
        <div class="diagnostic-actions">
          <button
            class="zn-link"
            type="button"
            :disabled="d.historyState === 'loading'"
            @click="d.loadHistory"
          >
            {{
              t(
                d.historyState === "failed"
                  ? "refresh.retry"
                  : "actions.refresh",
              )
            }}
          </button>
        </div>
        <p class="diagnostic-feedback" role="status">{{ d.historyFeedback }}</p>
        <ol
          v-if="d.historyState === 'ready' && d.history.length"
          class="zn-history-list"
        >
          <li v-for="(entry, index) in d.history" :key="index">
            <time :datetime="entry.time">{{ entry.time }}</time>
            <div>
              <code class="diagnostic-filename">{{ entry.library }}</code>
              <div class="zn-history-result">
                <ZnTag :tone="entry.loaded ? 'ok' : 'warn'" small>{{
                  t(
                    entry.loaded
                      ? "diagnostics.historyLoaded"
                      : "diagnostics.historyLoadFailed",
                  )
                }}</ZnTag>
                <span>{{ d.formatElapsed(entry.elapsedUs) }}</span>
              </div>
            </div>
          </li>
        </ol>
      </ZnCollapse>
    </ZnCollapse>
  </section>
</template>

<style scoped>
.zn-diagnostics {
  padding: 0 16px;
  margin-block: 4px 12px;
}

.zn-diagnostics :deep(.zn-collapse-header) {
  min-height: 48px;
  padding: 10px 0;
  font-weight: 500;
}

.zn-diagnostics :deep(.zn-collapse-panel) {
  margin: 0;
  padding-bottom: 12px;
}

.zn-diagnostics .diagnostic-text {
  padding: 10px 12px;
  border: 1px solid var(--zn-hairline);
  border-radius: 6px;
  background: var(--zn-fill);
}

.zn-diagnostics .diagnostic-actions {
  margin-block: 8px;
  gap: 4px;
}

.zn-diagnostics .zn-link {
  min-height: 40px;
  height: auto;
  max-width: 100%;
  padding: 4px 10px;
  color: var(--zn-accent);
  font: inherit;
}

.zn-diagnostics .zn-link:disabled,
.zn-diagnostics .diagnostic-feedback,
.zn-history-list time,
.zn-history-result {
  color: var(--zn-text-muted);
}

.zn-history,
.zn-history-list li + li {
  border-top: 1px solid var(--zn-hairline);
}

.zn-history :deep(.zn-collapse-panel) {
  padding-bottom: 0;
}

.zn-history-list {
  margin: 0;
  padding: 0;
  list-style: none;
}

.zn-history-list li {
  display: grid;
  grid-template-columns: auto minmax(0, 1fr);
  gap: 12px;
  padding-block: 8px;
  font-variant-numeric: tabular-nums;
}

.zn-history-result {
  display: flex;
  align-items: center;
  flex-wrap: wrap;
  gap: 6px;
  margin-top: 2px;
  font-size: 0.8125rem;
}
</style>
