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

import { reactive, ref, type Ref } from "vue";

import {
  getHookConfig,
  setHookConfig,
  type HookConfig,
  type HookKind,
} from "../api/config";
import { getDaemonStatus, type DaemonStatus } from "../api/daemon";
import { isKsuAvailable } from "../api/ksu";
import { getModules, type ZnnModule } from "../api/modules";
import { getSystemInfo, type SystemInfo } from "../api/system";

const DATA_SECTIONS = ["status", "system", "modules", "config"] as const;

export type DataSection = (typeof DATA_SECTIONS)[number];

interface SectionData {
  status: DaemonStatus;
  system: SystemInfo;
  modules: ZnnModule[];
  config: HookConfig;
}

interface SectionState {
  loading: boolean;
  failed: boolean;
  lastSuccess: number | null;
}

const readers: {
  [Section in DataSection]: () => Promise<SectionData[Section]>;
} = {
  status: getDaemonStatus,
  system: getSystemInfo,
  modules: getModules,
  config: getHookConfig,
};

export interface AppData {
  ksuAvailable: boolean;
  status: Ref<DaemonStatus | null>;
  system: Ref<SystemInfo | null>;
  modules: Ref<ZnnModule[] | null>;
  config: Ref<HookConfig | null>;
  loading: Ref<boolean>;
  sections: Record<DataSection, SectionState>;
  load: () => Promise<void>;
  retrySection: (section: DataSection) => Promise<void>;
  applyHookConfig: (kind: HookKind, value: string) => Promise<void>;
}

export function useAppData(): AppData {
  const ksuAvailable = isKsuAvailable();
  const status = ref<DaemonStatus | null>(null);
  const system = ref<SystemInfo | null>(null);
  const modules = ref<ZnnModule[] | null>(null);
  const config = ref<HookConfig | null>(null);
  const data: { [Section in DataSection]: Ref<SectionData[Section] | null> } = {
    status,
    system,
    modules,
    config,
  };
  const initialState: SectionState = {
    loading: ksuAvailable,
    failed: false,
    lastSuccess: null,
  };
  const sections = reactive({
    status: { ...initialState },
    system: { ...initialState },
    modules: { ...initialState },
    config: { ...initialState },
  });
  const loading = ref(ksuAvailable);
  const requests = new Map<DataSection, Promise<void>>();
  let refreshing = false;
  let configWrite = Promise.resolve();

  async function readSection<Section extends DataSection>(
    section: Section,
  ): Promise<void> {
    const state = sections[section];
    for (;;) {
      const write = configWrite;
      if (section === "config") {
        await write;
        if (write !== configWrite) continue;
      }

      try {
        const value = await readers[section]();
        if (section === "config" && write !== configWrite) continue;
        data[section].value = value;
        state.lastSuccess = Date.now();
        state.failed = false;
      } catch (error) {
        if (section === "config" && write !== configWrite) continue;
        console.warn(`[znn] ${section}:`, error);
        state.failed = true;
      }
      return;
    }
  }

  function retrySection(section: DataSection): Promise<void> {
    if (!ksuAvailable) return Promise.resolve();
    const pending = requests.get(section);
    if (pending) return pending;

    const request = Promise.resolve()
      .then(() => readSection(section))
      .finally(() => {
        requests.delete(section);
        sections[section].loading = false;
      });
    requests.set(section, request);
    sections[section].loading = true;
    return request;
  }

  async function load(): Promise<void> {
    if (!ksuAvailable || refreshing) return;
    refreshing = true;
    loading.value = true;
    try {
      await Promise.all(DATA_SECTIONS.map(retrySection));
    } finally {
      loading.value = false;
      refreshing = false;
    }
  }

  function applyHookConfig(kind: HookKind, value: string): Promise<void> {
    const update = configWrite.then(async () => {
      config.value = await setHookConfig(kind, value);
      sections.config.lastSuccess = Date.now();
      sections.config.failed = false;
    });
    configWrite = update.catch(() => {});
    return update;
  }

  return {
    ksuAvailable,
    status,
    system,
    modules,
    config,
    loading,
    sections,
    load,
    retrySection,
    applyHookConfig,
  };
}
