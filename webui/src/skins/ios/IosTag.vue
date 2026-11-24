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

import { kChip } from "konsta/vue";
import { computed } from "vue";

const props = withDefaults(
  defineProps<{ tone?: "ok" | "warn" | "info" | "muted" }>(),
  { tone: "muted" },
);

const TONES = {
  ok: {
    fillBgIos: "bg-green-500/15",
    fillTextIos: "text-green-600 dark:text-green-400",
  },
  warn: {
    fillBgIos: "bg-orange-500/15",
    fillTextIos: "text-orange-600 dark:text-orange-400",
  },
  info: {
    fillBgIos: "bg-blue-500/15",
    fillTextIos: "text-blue-600 dark:text-blue-400",
  },
  muted: {
    fillBgIos: "bg-black/5 dark:bg-white/10",
    fillTextIos: "text-black/55 dark:text-white/55",
  },
} as const;

const colors = computed(() => TONES[props.tone]);
</script>

<template>
  <k-chip class="ios-chip" :colors="colors">
    <slot />
  </k-chip>
</template>
