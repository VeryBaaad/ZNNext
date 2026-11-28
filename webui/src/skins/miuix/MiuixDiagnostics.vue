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

import {
  MiuixArrowPreference,
  MiuixBasicComponent,
  MiuixButton,
  MiuixCard,
} from "miuix-vue";
import { useId } from "vue";

import { useDiagnostics } from "../../composables/use-diagnostics";
import { t } from "../../i18n";

const d = useDiagnostics();
const summaryId = useId();
const historyId = useId();
</script>

<template>
  <MiuixCard v-if="d.available" class="miuix-diagnostics">
    <MiuixArrowPreference
      :title="t('diagnostics.title')"
      :aria-expanded="d.expanded"
      :aria-controls="summaryId"
      @click="d.expanded = !d.expanded"
    />
    <div v-show="d.expanded" :id="summaryId">
      <div class="miuix-diagnostics-body">
        <div class="diagnostic-actions">
          <MiuixButton :disabled="d.loadingLogs" @click="d.loadLogs">
            {{ d.logsAction }}
          </MiuixButton>
        </div>
        <p v-if="d.logsFailed" class="diagnostic-feedback" role="status">
          {{ t("diagnostics.logsFailed") }}
        </p>
        <pre class="diagnostic-text" tabindex="0" :aria-busy="d.loadingLogs">{{
          d.text
        }}</pre>
        <div class="diagnostic-actions">
          <MiuixButton :aria-label="t('diagnostics.copy')" @click="d.copy">
            {{ d.copyLabel }}
          </MiuixButton>
          <MiuixButton :disabled="d.exporting" @click="d.exportReport">
            {{
              t(d.exporting ? "diagnostics.exporting" : "diagnostics.export")
            }}
          </MiuixButton>
        </div>
        <p v-if="d.exportFeedback" class="diagnostic-feedback" role="status">
          {{ d.exportFeedback }}
        </p>
      </div>
      <MiuixArrowPreference
        :title="t('diagnostics.historyTitle')"
        :aria-expanded="d.historyOpen"
        :aria-controls="historyId"
        @click="d.historyOpen = !d.historyOpen"
      />
      <div v-show="d.historyOpen" :id="historyId">
        <div class="miuix-diagnostics-body">
          <div class="diagnostic-actions">
            <MiuixButton
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
            </MiuixButton>
          </div>
          <p v-if="d.historyFeedback" class="diagnostic-feedback" role="status">
            {{ d.historyFeedback }}
          </p>
        </div>
        <ul
          v-if="d.historyState === 'ready' && d.history.length"
          class="miuix-history"
        >
          <li v-for="(entry, index) in d.history" :key="index">
            <MiuixBasicComponent>
              <span class="diagnostic-filename">{{ entry.library }}</span>
              <span class="miuix-history-meta">
                <time :datetime="entry.time">{{ entry.time }}</time>
                ·
                {{
                  t(
                    entry.loaded
                      ? "diagnostics.historyLoaded"
                      : "diagnostics.historyLoadFailed",
                  )
                }}
                · {{ d.formatElapsed(entry.elapsedUs) }}
              </span>
            </MiuixBasicComponent>
          </li>
        </ul>
      </div>
    </div>
  </MiuixCard>
</template>

<style scoped>
.miuix-diagnostics {
  align-self: stretch;
  min-width: 0;
  margin-block-start: 12px;
}

.miuix-diagnostics [aria-expanded="true"] :deep(.m-arrow-preference__chevron) {
  transform: rotate(90deg);
}

.miuix-diagnostics-body {
  display: grid;
  gap: 12px;
  padding: 0 16px 14px;
}

.diagnostic-text {
  padding: 12px;
  border-radius: var(--m-radius-md);
  background: var(--m-color-secondary-container);
  color: var(--m-color-on-surface);
}

.diagnostic-actions :deep(.m-button) {
  max-width: 100%;
  white-space: normal;
  overflow-wrap: anywhere;
}

.miuix-history {
  list-style: none;
  margin: 0;
  padding: 0;
}

.miuix-history-meta {
  color: var(--m-color-on-surface-variant-summary);
  font-size: var(--m-text-body2-size);
}
</style>
