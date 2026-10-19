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

const props = defineProps<{
  modelValue: boolean;
  disabled?: boolean;
  label?: string;
}>();

const emit = defineEmits<{ "update:modelValue": [value: boolean] }>();

function toggle(): void {
  if (props.disabled) return;
  emit("update:modelValue", !props.modelValue);
}
</script>

<template>
  <button
    class="zn-switch"
    :class="{ 'is-on': props.modelValue, 'is-disabled': props.disabled }"
    type="button"
    role="switch"
    :aria-checked="props.modelValue"
    :aria-label="props.label"
    :disabled="props.disabled"
    @click="toggle"
  >
    <span class="zn-switch-rail">
      <span class="zn-switch-thumb"></span>
    </span>
  </button>
</template>
