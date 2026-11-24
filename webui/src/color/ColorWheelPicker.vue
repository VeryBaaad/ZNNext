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

import { computed, ref, watch } from "vue";

import { t } from "../i18n";
import { normalizeSeed } from "./material-color";

interface Hsv {
  h: number;
  s: number;
  v: number;
}

const props = defineProps<{ modelValue: string }>();

const emit = defineEmits<{ "update:modelValue": [value: string] }>();

const hsv = ref<Hsv>(hexToHsv(props.modelValue));

const wheel = ref<HTMLElement | null>(null);

const svArea = ref<HTMLElement | null>(null);

let draggingWheel = false;

let draggingSv = false;

const hue = computed(() => hsv.value.h);

const thumbStyle = computed(() => ({
  left: `clamp(9px, ${hsv.value.s * 100}%, calc(100% - 9px))`,
  top: `clamp(9px, ${(1 - hsv.value.v) * 100}%, calc(100% - 9px))`,
}));

watch(
  () => props.modelValue,
  (value) => {
    const normalized = normalizeSeed(value);
    if (!normalized || hsvToHex(hsv.value) === normalized) return;
    hsv.value = hexToHsv(normalized);
  },
);

function commit(next: Hsv): void {
  hsv.value = {
    h: ((next.h % 360) + 360) % 360,
    s: clamp01(next.s),
    v: clamp01(next.v),
  };
  emit("update:modelValue", hsvToHex(hsv.value));
}

function onWheelPointerDown(event: PointerEvent): void {
  const element = wheel.value;
  if (!element) return;
  const rect = element.getBoundingClientRect();
  const distance = Math.hypot(
    event.clientX - (rect.left + rect.width / 2),
    event.clientY - (rect.top + rect.height / 2),
  );
  if (distance < Math.min(rect.width, rect.height) * 0.26) return;
  element.setPointerCapture(event.pointerId);
  draggingWheel = true;
  updateHue(event);
  event.preventDefault();
}

function onWheelPointerMove(event: PointerEvent): void {
  if (draggingWheel) updateHue(event);
}

function onWheelPointerUp(event: PointerEvent): void {
  if (!draggingWheel) return;
  draggingWheel = false;
  if (wheel.value?.hasPointerCapture(event.pointerId)) {
    wheel.value.releasePointerCapture(event.pointerId);
  }
}

function onWheelKeydown(event: KeyboardEvent): void {
  const step = event.shiftKey ? 10 : 1;
  let next: number;
  if (event.key === "ArrowLeft" || event.key === "ArrowDown") {
    next = hsv.value.h - step;
  } else if (event.key === "ArrowRight" || event.key === "ArrowUp") {
    next = hsv.value.h + step;
  } else {
    return;
  }
  event.preventDefault();
  commit({ ...hsv.value, h: next });
}

function updateHue(event: PointerEvent): void {
  const rect = wheel.value?.getBoundingClientRect();
  if (!rect) return;
  const dx = event.clientX - (rect.left + rect.width / 2);
  const dy = event.clientY - (rect.top + rect.height / 2);
  const angle = (Math.atan2(dx, -dy) * 180) / Math.PI;
  commit({ ...hsv.value, h: angle });
}

function onSvPointerDown(event: PointerEvent): void {
  const element = svArea.value;
  if (!element) return;
  element.setPointerCapture(event.pointerId);
  draggingSv = true;
  updateSv(event);
  event.preventDefault();
}

function onSvPointerMove(event: PointerEvent): void {
  if (draggingSv) updateSv(event);
}

function onSvPointerUp(event: PointerEvent): void {
  if (!draggingSv) return;
  draggingSv = false;
  if (svArea.value?.hasPointerCapture(event.pointerId)) {
    svArea.value.releasePointerCapture(event.pointerId);
  }
}

function onSvKeydown(event: KeyboardEvent): void {
  const step = event.shiftKey ? 0.1 : 0.02;
  const next = { ...hsv.value };
  if (event.key === "ArrowLeft") next.s -= step;
  else if (event.key === "ArrowRight") next.s += step;
  else if (event.key === "ArrowUp") next.v += step;
  else if (event.key === "ArrowDown") next.v -= step;
  else return;
  event.preventDefault();
  commit(next);
}

function updateSv(event: PointerEvent): void {
  const rect = svArea.value?.getBoundingClientRect();
  if (!rect) return;
  commit({
    ...hsv.value,
    s: (event.clientX - rect.left) / rect.width,
    v: 1 - (event.clientY - rect.top) / rect.height,
  });
}

function clamp01(value: number): number {
  return Math.min(1, Math.max(0, value));
}

function hexToHsv(value: string): Hsv {
  const hex = normalizeSeed(value) ?? "#6750a4";
  const red = parseInt(hex.slice(1, 3), 16) / 255;
  const green = parseInt(hex.slice(3, 5), 16) / 255;
  const blue = parseInt(hex.slice(5, 7), 16) / 255;
  const max = Math.max(red, green, blue);
  const min = Math.min(red, green, blue);
  const delta = max - min;
  let hue = 0;
  if (delta !== 0) {
    if (max === red) hue = ((green - blue) / delta) % 6;
    else if (max === green) hue = (blue - red) / delta + 2;
    else hue = (red - green) / delta + 4;
    hue *= 60;
    if (hue < 0) hue += 360;
  }
  return { h: hue, s: max === 0 ? 0 : delta / max, v: max };
}

function hsvToHex(value: Hsv): string {
  const chroma = value.v * value.s;
  const sector = (value.h / 60) % 2;
  const second = chroma * (1 - Math.abs(sector - 1));
  const offset = value.v - chroma;
  let red = 0;
  let green = 0;
  let blue = 0;
  if (value.h < 60) {
    red = chroma;
    green = second;
  } else if (value.h < 120) {
    red = second;
    green = chroma;
  } else if (value.h < 180) {
    green = chroma;
    blue = second;
  } else if (value.h < 240) {
    green = second;
    blue = chroma;
  } else if (value.h < 300) {
    red = second;
    blue = chroma;
  } else {
    red = chroma;
    blue = second;
  }
  return `#${channel(red + offset)}${channel(green + offset)}${channel(blue + offset)}`;
}

function channel(value: number): string {
  return Math.round(value * 255)
    .toString(16)
    .padStart(2, "0");
}
</script>

<template>
  <div class="color-picker">
    <div
      ref="wheel"
      class="hue-wheel"
      role="slider"
      tabindex="0"
      :aria-label="t('color.hue')"
      aria-valuemin="0"
      aria-valuemax="360"
      :aria-valuenow="Math.round(hue)"
      :style="{ '--znn-hue': hue }"
      @pointerdown="onWheelPointerDown"
      @pointermove="onWheelPointerMove"
      @pointerup="onWheelPointerUp"
      @pointercancel="onWheelPointerUp"
      @keydown="onWheelKeydown"
    >
      <span
        class="hue-thumb"
        :style="{ transform: `rotate(${hue}deg)` }"
      ></span>
    </div>

    <div
      ref="svArea"
      class="sv-area"
      role="group"
      tabindex="0"
      :aria-label="t('color.tone')"
      :style="{ '--znn-hue': hue }"
      @pointerdown="onSvPointerDown"
      @pointermove="onSvPointerMove"
      @pointerup="onSvPointerUp"
      @pointercancel="onSvPointerUp"
      @keydown="onSvKeydown"
    >
      <span class="sv-thumb" :style="thumbStyle"></span>
    </div>
  </div>
</template>

<style scoped>
.color-picker {
  display: grid;
  grid-template-columns: clamp(100px, 36%, 148px) minmax(0, 1fr);
  align-items: stretch;
  gap: var(--md-sys-measurement-space150, 12px);
}

.hue-wheel {
  position: relative;
  width: 100%;
  aspect-ratio: 1;
  overflow: hidden;
  border-radius: 50%;
  background: conic-gradient(
    from 0deg,
    #ff0000,
    #ffff00,
    #00ff00,
    #00ffff,
    #0000ff,
    #ff00ff,
    #ff0000
  );
  cursor: crosshair;
  touch-action: none;
}

.hue-wheel::after {
  content: "";
  position: absolute;
  inset: 26%;
  border-radius: 50%;
  background: var(--znn-picker-hole, transparent);
  box-shadow: 0 0 0 1px color-mix(in srgb, currentColor 22%, transparent);
  pointer-events: none;
}

.hue-wheel:focus-visible {
  outline: 2px solid currentColor;
  outline-offset: 3px;
}

.hue-thumb {
  position: absolute;
  inset: 0;
  border-radius: 50%;
  pointer-events: none;
}

.hue-thumb::before {
  content: "";
  position: absolute;
  top: 5.5%;
  left: 50%;
  width: 18px;
  height: 18px;
  margin-inline-start: -9px;
  border: 2px solid #fff;
  border-radius: 50%;
  background: hsl(var(--znn-hue) 100% 50%);
  box-shadow:
    0 0 0 1px rgba(0, 0, 0, 0.45),
    0 1px 3px rgba(0, 0, 0, 0.35);
}

.sv-area {
  position: relative;
  border-radius: 16px;
  background-color: hsl(var(--znn-hue) 100% 50%);
  background-image:
    linear-gradient(to top, #000, transparent),
    linear-gradient(to right, #fff, transparent);
  box-shadow: inset 0 0 0 1px color-mix(in srgb, currentColor 20%, transparent);
  cursor: crosshair;
  touch-action: none;
}

.sv-area:focus-visible {
  outline: 2px solid currentColor;
  outline-offset: 2px;
}

.sv-thumb {
  position: absolute;
  width: 18px;
  height: 18px;
  transform: translate(-50%, -50%);
  border: 2px solid #fff;
  border-radius: 50%;
  box-shadow:
    0 0 0 1px rgba(0, 0, 0, 0.45),
    0 1px 3px rgba(0, 0, 0, 0.35);
  pointer-events: none;
}
</style>
