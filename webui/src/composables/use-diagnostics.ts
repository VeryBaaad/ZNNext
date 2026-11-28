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

import { computed, reactive, ref, watch } from "vue";

import { readDiagnosticLogs, saveDiagnosticReport } from "../api/diagnostics";
import { isKsuAvailable } from "../api/ksu";
import { readLoadHistory } from "../api/load-history";
import { useAppActions } from "./app-actions";
import { locale, t } from "../i18n";
import { useCopyText } from "./use-copy-text";

export function useDiagnostics() {
  const expanded = ref(false);
  const historyOpen = ref(false);
  const { diagnosticSummary } = useAppActions();
  const summary = computed(diagnosticSummary);
  const logs = ref<string | null>(null);
  const loadingLogs = ref(false);
  const logsFailed = ref(false);
  const exporting = ref(false);
  const exportFailed = ref(false);
  const savedPath = ref("");
  const history = ref<Awaited<ReturnType<typeof readLoadHistory>>>([]);
  const historyState = ref<"idle" | "loading" | "ready" | "failed">("idle");
  const elapsedFormat = computed(
    () =>
      new Intl.NumberFormat(locale.value === "lzh" ? "zh-Hant" : locale.value, {
        style: "unit",
        unit: "millisecond",
        maximumFractionDigits: 3,
      }),
  );

  const text = computed(() =>
    logs.value ? `${summary.value}\n\n${logs.value}` : summary.value,
  );

  const logsAction = computed(() => {
    if (loadingLogs.value) return t("diagnostics.loadingLogs");
    if (logsFailed.value) return t("diagnostics.retryLogs");
    return t(
      logs.value === null ? "diagnostics.loadLogs" : "diagnostics.refreshLogs",
    );
  });

  const exportFeedback = computed(() => {
    if (exportFailed.value) return t("diagnostics.exportFailed");
    return savedPath.value
      ? t("diagnostics.saved", { path: savedPath.value })
      : "";
  });

  const historyFeedback = computed(() => {
    if (historyState.value === "loading")
      return t("diagnostics.historyLoading");
    if (historyState.value === "failed") return t("diagnostics.historyFailed");
    if (historyState.value === "ready" && !history.value.length) {
      return t("diagnostics.historyEmpty");
    }
    return "";
  });

  async function loadLogs(): Promise<void> {
    if (loadingLogs.value) return;
    loadingLogs.value = true;
    logsFailed.value = false;
    logs.value = null;
    try {
      logs.value = await readDiagnosticLogs();
    } catch {
      logsFailed.value = true;
    } finally {
      loadingLogs.value = false;
    }
  }

  async function exportReport(): Promise<void> {
    if (exporting.value) return;
    exporting.value = true;
    exportFailed.value = false;
    savedPath.value = "";
    try {
      savedPath.value = await saveDiagnosticReport(
        summary.value,
        logs.value ?? undefined,
      );
    } catch {
      exportFailed.value = true;
    } finally {
      exporting.value = false;
    }
  }

  async function loadHistory(): Promise<void> {
    if (historyState.value === "loading") return;
    historyState.value = "loading";
    try {
      history.value = await readLoadHistory();
      historyState.value = "ready";
    } catch {
      historyState.value = "failed";
    }
  }

  watch(historyOpen, (open) => {
    if (open && historyState.value === "idle") void loadHistory();
  });

  return reactive({
    available: isKsuAvailable(),
    expanded,
    historyOpen,
    text,
    ...useCopyText(() => text.value),
    logsAction,
    loadingLogs,
    logsFailed,
    loadLogs,
    exportReport,
    exporting,
    exportFeedback,
    history,
    historyState,
    historyFeedback,
    loadHistory,
    formatElapsed: (elapsedUs: number) =>
      elapsedFormat.value.format(elapsedUs / 1000),
  });
}
