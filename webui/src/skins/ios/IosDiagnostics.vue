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

import { kButton, kList, kListItem } from "konsta/vue";

import { useDiagnostics } from "../../composables/use-diagnostics";
import { t } from "../../i18n";
import IosTag from "./IosTag.vue";

const d = useDiagnostics();
</script>

<template>
  <section
    v-if="d.available"
    class="ios-diagnostics"
    :aria-label="t('diagnostics.title')"
  >
    <k-list inset strong class="ios-list">
      <k-list-item
        class="ios-module-item"
        :class="{ 'ios-expanded': d.expanded }"
        link
        link-component="button"
        :link-props="{ type: 'button', 'aria-expanded': d.expanded }"
        :title="t('diagnostics.title')"
        @click="d.expanded = !d.expanded"
      />
      <template v-if="d.expanded">
        <k-list-item inner-class="min-w-0">
          <template #inner>
            <pre
              class="diagnostic-text"
              tabindex="0"
              :aria-busy="d.loadingLogs"
              >{{ d.text }}</pre>
            <div class="diagnostic-actions">
              <k-button
                inline
                clear
                type="button"
                :disabled="d.loadingLogs"
                @click="d.loadLogs"
              >
                {{ d.logsAction }}
              </k-button>
              <k-button
                inline
                clear
                type="button"
                :aria-label="t('diagnostics.copy')"
                @click="d.copy"
              >
                {{ d.copyLabel }}
              </k-button>
              <k-button
                inline
                clear
                type="button"
                :disabled="d.exporting"
                @click="d.exportReport"
              >
                {{
                  t(
                    d.exporting
                      ? "diagnostics.exporting"
                      : "diagnostics.export",
                  )
                }}
              </k-button>
            </div>
            <p v-if="d.logsFailed" class="diagnostic-feedback" role="status">
              {{ t("diagnostics.logsFailed") }}
            </p>
            <p
              v-if="d.exportFeedback"
              class="diagnostic-feedback"
              role="status"
            >
              {{ d.exportFeedback }}
            </p>
          </template>
        </k-list-item>
        <k-list-item
          class="ios-module-item"
          :class="{ 'ios-expanded': d.historyOpen }"
          link
          link-component="button"
          :link-props="{ type: 'button', 'aria-expanded': d.historyOpen }"
          :title="t('diagnostics.historyTitle')"
          @click="d.historyOpen = !d.historyOpen"
        />
        <template v-if="d.historyOpen">
          <k-list-item>
            <template #inner>
              <div class="diagnostic-actions">
                <k-button
                  inline
                  clear
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
                </k-button>
              </div>
              <p
                v-if="d.historyFeedback"
                class="diagnostic-feedback"
                role="status"
              >
                {{ d.historyFeedback }}
              </p>
            </template>
          </k-list-item>
          <template v-if="d.historyState === 'ready'">
            <k-list-item
              v-for="(entry, index) in d.history"
              :key="index"
              inner-class="min-w-0"
            >
              <template #title>
                <code class="diagnostic-filename">{{ entry.library }}</code>
              </template>
              <template #after>
                <time :datetime="entry.time">{{ entry.time }}</time>
              </template>
              <template #subtitle>
                <div class="ios-load-result">
                  <IosTag :tone="entry.loaded ? 'ok' : 'warn'">
                    {{
                      t(
                        entry.loaded
                          ? "diagnostics.historyLoaded"
                          : "diagnostics.historyLoadFailed",
                      )
                    }}
                  </IosTag>
                  <span>{{ d.formatElapsed(entry.elapsedUs) }}</span>
                </div>
              </template>
            </k-list-item>
          </template>
        </template>
      </template>
    </k-list>
  </section>
</template>

<style scoped>
.ios-diagnostics {
  align-self: stretch;
}

.ios-diagnostics .diagnostic-text {
  padding: 12px;
  border-radius: 10px;
  background: color-mix(in srgb, currentColor 5%, transparent);
}

.ios-diagnostics .k-button {
  min-height: 44px;
  max-width: 100%;
  white-space: normal;
}

.ios-load-result {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 6px;
  margin-top: 4px;
}
</style>
