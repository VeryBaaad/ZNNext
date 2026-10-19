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

import "./styles/base.css";

import { createApp } from "vue";

import { enableEdgeToEdge, fullScreen } from "kernelsu";

import App from "./App.vue";
import { MODULE_NAME, VER_NAME } from "./app-info";
import { isKsuAvailable } from "./api/ksu";

document.title = `${MODULE_NAME} ${VER_NAME}`;

if (isKsuAvailable()) {
  try {
    enableEdgeToEdge(true);
    fullScreen(true);
  } catch {}
}

createApp(App).mount("#app");
