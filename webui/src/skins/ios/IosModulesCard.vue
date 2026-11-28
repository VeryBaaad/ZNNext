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
import CopyFailure from "../../components/CopyFailure.vue";
import { t } from "../../i18n";
import { formatFailureMetadata } from "../../util/failure-metadata";
import IosTag from "./IosTag.vue";

const props = defineProps<{ modules: ZnnModule[] }>();

const expanded = ref(new Set<string>());

const title = computed(() =>
  t("zn.modules.title", { n: props.modules.length }),
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

function tagsSlot(module: ZnnModule): "after" | "subtitle" {
  return module.disabled === true ? "subtitle" : "after";
}

function basename(value: string): string {
  const cut = value.lastIndexOf("/");
  return cut >= 0 ? value.slice(cut + 1) : value;
}
</script>

<template>
  <section class="ios-section">
    <h2 class="ios-section-title">{{ title }}</h2>

    <k-list inset strong class="ios-list">
      <k-list-item
        v-if="props.modules.length === 0"
        :title="t('zn.modules.empty')"
      />

      <template v-for="module in props.modules" :key="module.id">
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
          <template #[tagsSlot(module)]>
            <div
              class="ios-module-tags"
              :class="{ 'is-disabled': module.disabled === true }"
            >
              <IosTag v-if="module.disabled === true" tone="muted">
                {{ t("modules.disabled") }}
              </IosTag>
              <IosTag v-if="failures(module) > 0" tone="warn">
                {{ t("modules.failed", { n: failures(module) }) }}
              </IosTag>
              <IosTag
                v-if="module.disabled !== true || module.processes.length > 0"
                tone="info"
              >
                {{ t("modules.processes", { n: module.processes.length }) }}
              </IosTag>
            </div>
          </template>
        </k-list-item>

        <template v-if="expanded.has(module.id)">
          <k-list-item
            v-if="module.version"
            class="ios-child-item"
            title-font-size-ios="text-[15px]"
            :title="t('info.version')"
          >
            <template #after>
              <span class="ios-value" :title="module.version">{{
                module.version
              }}</span>
            </template>
          </k-list-item>
          <k-list-item
            v-for="process in module.processes"
            :key="`proc-${module.id}-${process.pid}`"
            class="ios-child-item"
            title-font-size-ios="text-[15px]"
            :title="basename(process.name)"
            :after="`pid=${process.pid}`"
          />
          <k-list-item
            v-if="
              module.disabled !== true &&
              module.processes.length === 0 &&
              failures(module) === 0
            "
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
            :subtitle="formatFailureMetadata(failure)"
          >
            <template #after>
              <span class="ios-failure" :title="failure.reason">
                {{ failure.reason }}
              </span>
            </template>
            <template #inner>
              <CopyFailure :module="module" :failure="failure" />
            </template>
          </k-list-item>
        </template>
      </template>
    </k-list>
  </section>
</template>

<style scoped>
.ios-module-tags {
  display: contents;
}

.ios-module-tags.is-disabled {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 4px;
  margin-block-start: 2px;
}
</style>
