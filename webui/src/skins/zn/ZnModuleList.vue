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

import { ref } from "vue";

import { moduleStatusKey, type ZnnModule } from "../../api/modules";
import CopyFailure from "../../components/CopyFailure.vue";
import { t } from "../../i18n";
import { formatFailureMetadata } from "../../util/failure-metadata";
import ZnTag from "./ZnTag.vue";

const props = defineProps<{ modules: ZnnModule[] }>();

const expanded = ref(new Set<string>());

function toggle(id: string): void {
  const next = new Set(expanded.value);
  if (next.has(id)) next.delete(id);
  else next.add(id);
  expanded.value = next;
}

function failures(module: ZnnModule): number {
  return (module.failed ?? []).length;
}

const basename = (value: string): string => {
  const cut = value.lastIndexOf("/");
  return cut >= 0 ? value.slice(cut + 1) : value;
};
</script>

<template>
  <div v-if="props.modules.length === 0" class="zn-empty-line">
    {{ t("zn.modules.empty") }}
  </div>

  <div v-else class="zn-module-list">
    <div v-for="module in props.modules" :key="module.id" class="zn-module">
      <button
        class="zn-module-trigger"
        :class="{ 'has-module-status': moduleStatusKey(module) }"
        type="button"
        :aria-expanded="expanded.has(module.id)"
        @click="toggle(module.id)"
      >
        <span class="zn-module-name" :title="`${module.name} (${module.id})`">
          {{ module.name }}
        </span>
        <span class="zn-module-tags">
          <ZnTag
            v-if="moduleStatusKey(module)"
            class="zn-module-status"
            tone="info"
            small
          >
            {{ t(moduleStatusKey(module)!) }}
          </ZnTag>
          <ZnTag v-if="failures(module) > 0" tone="warn" small>
            {{ t("modules.failed", { n: failures(module) }) }}
          </ZnTag>
          <ZnTag
            v-if="!moduleStatusKey(module) || module.processes.length > 0"
            tone="info"
            small
          >
            {{ t("modules.processes", { n: module.processes.length }) }}
          </ZnTag>
        </span>
      </button>

      <div v-if="expanded.has(module.id)" class="zn-module-body">
        <div v-if="module.version" class="zn-proc-row">
          <span>{{ t("info.version") }}</span>
          <span class="zn-field-value">{{ module.version }}</span>
        </div>
        <div
          v-if="!moduleStatusKey(module) && module.processes.length === 0"
          class="zn-empty-line"
        >
          {{ t("modules.noProcesses") }}
        </div>
        <div
          v-for="process in module.processes"
          :key="process.pid"
          class="zn-proc-row"
        >
          <span class="zn-proc-name" :title="process.name">{{
            basename(process.name)
          }}</span>
          <span class="zn-proc-meta">pid={{ process.pid }}</span>
        </div>

        <template v-if="failures(module) > 0">
          <div class="zn-proc-section">{{ t("modules.failedTitle") }}</div>
          <div
            v-for="failure in module.failed ?? []"
            :key="`${failure.name}-${failure.reason}`"
            class="zn-proc-row zn-proc-failed"
          >
            <span class="zn-proc-name" :title="failure.name">{{
              basename(failure.name)
            }}</span>
            <div class="zn-proc-meta">
              <div :title="failure.reason">{{ failure.reason }}</div>
              <small
                v-if="
                  failure.pid !== undefined || failure.lastSeenMs !== undefined
                "
              >
                {{ formatFailureMetadata(failure) }}
              </small>
              <div>
                <CopyFailure :module="module" :failure="failure" />
              </div>
            </div>
          </div>
        </template>
      </div>
    </div>
  </div>
</template>

<style scoped>
.zn-module-trigger.has-module-status,
.zn-module-trigger.has-module-status .zn-module-tags {
  flex-wrap: wrap;
}

.zn-module-trigger.has-module-status .zn-module-tags {
  max-width: 100%;
}

.zn-module-status {
  max-width: 100%;
  min-height: 24px;
  height: auto;
  padding-block: 3px;
  line-height: 1.25;
  overflow-wrap: anywhere;
  white-space: normal;
}
</style>
