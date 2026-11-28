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

const d = useDiagnostics();
</script>

<template>
  <m3e-expansion-panel
    v-if="d.available"
    class="material-diagnostics"
    :open.prop="d.expanded"
    @opening.self="d.expanded = true"
    @closing.self="d.expanded = false"
  >
    <span slot="header">{{ t("diagnostics.title") }}</span>
    <div class="diagnostic-actions">
      <m3e-button
        size="small"
        variant="text"
        :disabled="d.loadingLogs"
        @click="d.loadLogs"
      >
        {{ d.logsAction }}
      </m3e-button>
    </div>
    <p class="diagnostic-feedback" role="status">
      {{ d.logsFailed ? t("diagnostics.logsFailed") : "" }}
    </p>
    <pre class="diagnostic-text" tabindex="0" :aria-busy="d.loadingLogs">{{
      d.text
    }}</pre>
    <div class="diagnostic-actions">
      <m3e-button
        size="small"
        variant="tonal"
        :aria-label="t('diagnostics.copy')"
        @click="d.copy"
      >
        <span aria-live="polite">{{ d.copyLabel }}</span>
      </m3e-button>
      <m3e-button
        size="small"
        variant="tonal"
        :disabled="d.exporting"
        @click="d.exportReport"
      >
        {{ t(d.exporting ? "diagnostics.exporting" : "diagnostics.export") }}
      </m3e-button>
    </div>
    <p class="diagnostic-feedback" role="status">{{ d.exportFeedback }}</p>

    <m3e-expansion-panel
      class="material-diagnostics-history"
      :open.prop="d.historyOpen"
      @opening.self="d.historyOpen = true"
      @closing.self="d.historyOpen = false"
    >
      <span slot="header">{{ t("diagnostics.historyTitle") }}</span>
      <div class="diagnostic-actions">
        <m3e-button
          size="small"
          variant="text"
          :disabled="d.historyState === 'loading'"
          @click="d.loadHistory"
        >
          {{
            t(d.historyState === "failed" ? "refresh.retry" : "actions.refresh")
          }}
        </m3e-button>
      </div>
      <p class="diagnostic-feedback" role="status">{{ d.historyFeedback }}</p>
      <m3e-list
        v-if="d.historyState === 'ready' && d.history.length"
        class="material-diagnostics-history-list"
      >
        <m3e-list-item v-for="(entry, index) in d.history" :key="index">
          <code class="diagnostic-filename">{{ entry.library }}</code>
          <span class="history-metadata" slot="supporting-text">
            <time :datetime="entry.time">{{ entry.time }}</time>
            <span :class="{ 'history-error': !entry.loaded }">
              {{
                t(
                  entry.loaded
                    ? "diagnostics.historyLoaded"
                    : "diagnostics.historyLoadFailed",
                )
              }}
            </span>
            <span>{{ d.formatElapsed(entry.elapsedUs) }}</span>
          </span>
        </m3e-list-item>
      </m3e-list>
    </m3e-expansion-panel>
  </m3e-expansion-panel>
</template>

<style scoped>
.material-diagnostics {
  box-sizing: border-box;
  width: 100%;
  min-width: 0;
  --m3e-expansion-panel-container-color: var(--md-sys-color-surface-container);
  --m3e-expansion-panel-shape: var(--md-sys-shape-corner-extra-large);
  --m3e-expansion-panel-content-padding: 0 12px 12px;
  --m3e-expansion-header-expanded-height: 48px;
  --m3e-expansion-header-padding-left: 16px;
  --m3e-expansion-header-padding-right: 16px;
  --m3e-button-container-height: 44px;
}

.diagnostic-actions {
  justify-content: flex-end;
}

.diagnostic-text {
  padding: 12px;
  border-radius: var(--md-sys-shape-corner-medium);
  background: var(--md-sys-color-surface-container-high);
}

.material-diagnostics-history {
  margin-block-start: var(--md-sys-measurement-space100);
  --m3e-expansion-panel-container-color: var(
    --md-sys-color-surface-container-high
  );
  --m3e-expansion-panel-shape: var(--md-sys-shape-corner-large);
  --m3e-expansion-panel-content-padding: 0 8px 8px;
  --m3e-expansion-header-padding-left: 12px;
  --m3e-expansion-header-padding-right: 12px;
  --m3e-expansion-header-font-size: var(
    --md-sys-typescale-body-medium-font-size
  );
}

.material-diagnostics-history-list {
  --m3e-list-item-two-line-height: 48px;
  --m3e-list-item-leading-space: 8px;
  --m3e-list-item-trailing-space: 8px;
  --m3e-list-item-font-size: var(--md-sys-typescale-body-medium-font-size);
  --m3e-list-item-supporting-text-font-size: var(
    --md-sys-typescale-body-small-font-size
  );
}

.history-metadata {
  display: flex;
  flex-wrap: wrap;
  gap: 0 8px;
}

.history-error {
  color: var(--md-sys-color-error);
}
</style>
