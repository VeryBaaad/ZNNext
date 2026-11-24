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

import { kList, kListItem } from "konsta/vue";
import { computed, ref } from "vue";

import type { ZnnModule } from "../../api/modules";
import { t } from "../../i18n";
import IosIcon from "./IosIcon.vue";
import IosTag from "./IosTag.vue";

const props = defineProps<{
  modules: ZnnModule[] | null;
  error?: boolean;
}>();

const expanded = ref(new Set<string>());

const modules = computed<ZnnModule[]>(() => props.modules ?? []);

const title = computed(() =>
  t("zn.modules.title", { n: modules.value.length }),
);

function toggle(id: string): void {
  const next = new Set(expanded.value);
  if (next.has(id)) next.delete(id);
  else next.add(id);
  expanded.value = next;
}

function failures(module: ZnnModule): number {
  return (module.failed ?? []).length;
}

function basename(value: string): string {
  const cut = value.lastIndexOf("/");
  return cut >= 0 ? value.slice(cut + 1) : value;
}
</script>

<template>
  <section class="ios-section">
    <h2 class="ios-section-title">{{ title }}</h2>

    <k-list v-if="!props.error" inset strong class="ios-list">
      <k-list-item v-if="modules.length === 0" :title="t('zn.modules.empty')" />

      <template v-for="module in modules" :key="module.id">
        <k-list-item
          class="ios-module-item"
          :class="{ 'ios-expanded': expanded.has(module.id) }"
          link
          link-component="button"
          :link-props="{
            type: 'button',
            'aria-expanded': expanded.has(module.id),
          }"
          @click="toggle(module.id)"
        >
          <template #title>
            <span class="ios-module-name">{{ module.name }}</span>
          </template>
          <template #after>
            <IosTag v-if="failures(module) > 0" tone="warn">
              {{ t("modules.failed", { n: failures(module) }) }}
            </IosTag>
            <IosTag tone="info">
              {{ t("modules.processes", { n: module.processes.length }) }}
            </IosTag>
          </template>
        </k-list-item>

        <template v-if="expanded.has(module.id)">
          <k-list-item
            v-for="process in module.processes"
            :key="`proc-${module.id}-${process.pid}`"
            class="ios-child-item"
            title-font-size-ios="text-[15px]"
            :title="basename(process.name)"
            :after="`pid=${process.pid}`"
          />
          <k-list-item
            v-if="module.processes.length === 0 && failures(module) === 0"
            class="ios-child-item"
            title-font-size-ios="text-[15px]"
            :title="t('modules.noProcesses')"
          />
          <k-list-item
            v-for="failure in module.failed ?? []"
            :key="`fail-${module.id}-${failure.name}-${failure.reason}`"
            class="ios-child-item"
            title-font-size-ios="text-[15px]"
            :title="basename(failure.name)"
          >
            <template #after>
              <span class="ios-failure" :title="failure.reason">
                {{ failure.reason }}
              </span>
            </template>
          </k-list-item>
        </template>
      </template>
    </k-list>

    <k-list v-else inset strong class="ios-list">
      <k-list-item :title="t('error.title')">
        <template #after>
          <IosIcon class="text-red-500" name="warning" :size="20" />
        </template>
      </k-list-item>
    </k-list>
  </section>
</template>
