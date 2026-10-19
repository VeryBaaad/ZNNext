<script setup lang="ts">
/*
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

import { GITHUB_URL } from "../../app-info";
import { t } from "../../i18n";
import ZnIcon from "./ZnIcon.vue";
</script>

<template>
  <footer class="zn-footer">
    <div class="zn-link-row">
      <a
        class="zn-link"
        :href="GITHUB_URL"
        target="_blank"
        rel="noopener noreferrer"
      >
        <ZnIcon class="github-logo" name="github" :size="18" />
        {{ t("footer.github") }}
      </a>
    </div>
    <span class="zn-credit">{{ t("zn.credit") }}</span>
  </footer>
</template>
