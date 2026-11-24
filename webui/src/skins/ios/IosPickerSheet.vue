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

import { kList, kListItem, kSheet } from "konsta/vue";

import { t } from "../../i18n";
import IosIcon from "./IosIcon.vue";

const props = defineProps<{
  opened: boolean;
  title: string;
  options: string[];
  value: string;
  label: (id: string) => string;
}>();

const emit = defineEmits<{ close: []; select: [id: string] }>();

function pick(id: string): void {
  emit("select", id);
}
</script>

<template>
  <k-sheet :opened="props.opened" @backdropclick="emit('close')">
    <div class="ios-picker">
      <div class="ios-picker-header">
        <span class="ios-picker-title">{{ props.title }}</span>
        <button class="ios-picker-done" type="button" @click="emit('close')">
          {{ t("color.done") }}
        </button>
      </div>

      <k-list nested strong class="ios-picker-list">
        <k-list-item
          v-for="id in props.options"
          :key="id"
          link
          :chevron="false"
          link-component="button"
          :link-props="{
            type: 'button',
            'aria-selected': id === props.value,
          }"
          :title="props.label(id)"
          @click="pick(id)"
        >
          <template #after>
            <IosIcon
              v-if="id === props.value"
              class="text-primary"
              name="check"
              :size="20"
            />
          </template>
        </k-list-item>
      </k-list>
    </div>
  </k-sheet>
</template>
