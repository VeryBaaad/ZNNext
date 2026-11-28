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
  kActions,
  kActionsButton,
  kActionsGroup,
  kActionsLabel,
  kLink,
  kNavbar,
} from "konsta/vue";
import { computed, ref } from "vue";

import { MODULE_NAME } from "../../app-info";
import { useAppActions } from "../../composables/app-actions";
import {
  LOCALE_LABELS,
  LOCALES,
  locale,
  setLocale,
  t,
  type Locale,
} from "../../i18n";
import {
  THEME_MODES,
  setThemeMode,
  themeMode,
  type ThemeMode,
} from "../../theme";
import IosIcon from "./IosIcon.vue";
import type { IconName } from "./icons";

const props = defineProps<{ refreshing: boolean }>();

const actions = useAppActions();

type Picker = "theme" | "locale" | null;

const picker = ref<Picker>(null);

const themeIcon = computed<IconName>(() => {
  if (themeMode.value === "light") return "sun";
  if (themeMode.value === "dark") return "moon";
  return "auto";
});

const actionOpened = computed(() => picker.value !== null);

const THEME_ICONS: Record<ThemeMode, IconName> = {
  auto: "auto",
  light: "sun",
  dark: "moon",
};

const THEME_LABEL_KEYS: Record<ThemeMode, string> = {
  auto: "theme.auto",
  light: "theme.light",
  dark: "theme.dark",
};

function closePicker(): void {
  picker.value = null;
}

function openPicker(next: Exclude<Picker, null>): void {
  picker.value = picker.value === next ? null : next;
}

function pickTheme(mode: ThemeMode): void {
  closePicker();
  setThemeMode(mode);
}

function pickLocale(next: Locale): void {
  closePicker();
  setLocale(next);
}
</script>

<template>
  <k-navbar large :title="MODULE_NAME">
    <template #right>
      <div class="flex items-center gap-1">
        <k-link
          component="button"
          role="button"
          icon-only
          :link-props="{
            type: 'button',
            'aria-label': t('actions.refresh'),
            disabled: props.refreshing,
          }"
          @click="actions.refresh()"
        >
          <IosIcon
            name="refresh"
            :size="22"
            :class="{ 'ios-spin': props.refreshing }"
          />
        </k-link>

        <k-link
          component="button"
          icon-only
          :link-props="{
            type: 'button',
            'aria-label': t('topbar.themeLabel'),
            'aria-haspopup': 'dialog',
            'aria-expanded': picker === 'theme',
          }"
          @click="openPicker('theme')"
        >
          <IosIcon :name="themeIcon" :size="22" />
        </k-link>

        <k-link
          component="button"
          icon-only
          :link-props="{
            type: 'button',
            'aria-label': t('topbar.langLabel'),
            'aria-haspopup': 'dialog',
            'aria-expanded': picker === 'locale',
          }"
          @click="openPicker('locale')"
        >
          <IosIcon name="globe" :size="22" />
        </k-link>
      </div>
    </template>
  </k-navbar>

  <k-actions :opened="actionOpened" @backdropclick="closePicker">
    <k-actions-group v-if="picker === 'theme'">
      <k-actions-label>{{ t("topbar.themeLabel") }}</k-actions-label>
      <k-actions-button
        v-for="mode in THEME_MODES"
        :key="mode"
        @click="pickTheme(mode)"
      >
        <span class="flex w-full items-center justify-between gap-4">
          <span class="flex items-center gap-2">
            <IosIcon :name="THEME_ICONS[mode]" :size="20" />
            {{ t(THEME_LABEL_KEYS[mode]) }}
          </span>
          <IosIcon
            v-if="mode === themeMode"
            class="text-primary"
            name="check"
            :size="20"
          />
        </span>
      </k-actions-button>
    </k-actions-group>

    <k-actions-group v-else-if="picker === 'locale'">
      <k-actions-label>{{ t("topbar.langLabel") }}</k-actions-label>
      <k-actions-button
        v-for="item in LOCALES"
        :key="item"
        @click="pickLocale(item)"
      >
        <span class="flex w-full items-center justify-between gap-4">
          <span>{{ LOCALE_LABELS[item] }}</span>
          <IosIcon
            v-if="item === locale"
            class="text-primary"
            name="check"
            :size="20"
          />
        </span>
      </k-actions-button>
    </k-actions-group>

    <k-actions-group>
      <k-actions-button bold @click="closePicker">
        {{ t("color.done") }}
      </k-actions-button>
    </k-actions-group>
  </k-actions>
</template>
