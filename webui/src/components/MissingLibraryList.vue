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

import { t } from "../i18n";

const props = defineProps<{ libraries: string[] }>();
</script>

<template>
  <div class="missing-library-list">
    <div>{{ t("modules.missingLibraries") }}</div>
    <ul>
      <li v-for="library in props.libraries" :key="library">
        <code>{{ library }}</code>
      </li>
    </ul>
  </div>
</template>

<style scoped>
.missing-library-list {
  min-width: 0;
  max-width: 100%;
}

.missing-library-list ul {
  margin: 4px 0 0;
  padding-inline-start: 1.25em;
  list-style-type: disc;
}

.missing-library-list li {
  font-size: 0.8125rem;
  overflow-wrap: anywhere;
  white-space: normal;
}
</style>
