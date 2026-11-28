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

import type { Dictionary } from "./en";

export const lzh: Dictionary = {
  "topbar.themeLabel": "體制",
  "topbar.langLabel": "雅言",

  "theme.auto": "順應天時",
  "theme.light": "陽",
  "theme.dark": "夕",

  "actions.copy": "謄錄",
  "actions.copied": "已謄",
  "actions.copyFailed": "謄錄未成",
  "actions.refresh": "賡續",

  "refresh.unavailable": "未能載入{section}。",
  "refresh.stale": "未能賡續{section}，仍示 {time} 之舊錄。",
  "refresh.retry": "復試",
  "refresh.retrySection": "復試載入{section}",
  "refresh.section.status": "氣象",
  "refresh.section.system": "機要",
  "refresh.section.modules": "諸司名錄",
  "refresh.section.config": "規制",

  "status.active": "已啟",
  "status.inactive": "蟄伏",

  "card.basic": "根本",
  "card.status": "氣象",
  "card.settings": "規制",

  "zn.running": "履職中",
  "zn.stopped": "罷工",
  "zn.daemon": "宿衛",
  "zn.modules.title": "天樞諸司 ({n})",
  "zn.modules.empty": "無司",
  "zn.credit": "Designed by Mufanc, 5ec1cff & VeryBaaad",

  "info.version": "版次",
  "info.rootImpl": "判官",
  "info.rootImpl.none": "未見神明",
  "info.mode": "今法",
  "info.pageSize": "記憶體頁之大小",
  "info.unknown": "迷濛",

  "modules.disabled": "已停",
  "modules.noDeclarations": "無可辨之聲明",
  "modules.missingLibraries": "缺庫檔",
  "modules.waitingForTarget": "候所需之務",
  "modules.processes": "公務有 {n}",
  "modules.failed": "未成者 {n} 條",
  "modules.copyFailure": "謄錄失敗之詳",
  "modules.failedTitle": "載入未成之錄",
  "modules.failedBanner": "載入未成之錄凡 {n} 條，請閱模組列表。",
  "modules.noModules": "無司",
  "modules.noProcesses": "無事",

  "config.inlineHook": "中道截擊",
  "config.inlineHook32": "中道截擊（三十二）",
  "config.pltHook": "依表截擊",
  "config.pltHook32": "依表截擊（三十二）",
  "config.mode": "巡伺之法",
  "config.mode.auto": "任運",
  "config.mode.kernel": "察核心之變",
  "config.mode.ptrace": "御下之法",
  "config.mode.proc": "案牘之法",
  "config.mode.late": "遲滯之法",
  "config.modeSavedCurrent": "已識。znnd 所報今法為 {mode}。",
  "config.modeSavedPtraceRestart":
    "已識。znnd 所報今法為 {mode}。欲易為 ptrace，須重啟 znnd。",
  "config.modeSavedStopped": "已識。znnd 次啟時讀此規制。",
  "config.modeSavedUnconfirmed": "已識。今法未能核實。",
  "config.disableZn": "總攬生化源",
  "config.disableZn.true": "准",
  "config.disableZn.false": "駁",
  "config.hotUpdate": "權變新法",
  "config.hotUpdate.true": "准",
  "config.hotUpdate.false": "駁",
  "config.debugdata": "符節秘錄解封考校",
  "config.debugdata.lzma": "太乙神數祕法",
  "config.debugdata.xz": "奇門遁甲祕法",
  "config.debugdata.libarchive": "六壬神課",
  "config.uiStyle": "容貌",
  "config.saved": "已識",
  "config.saveFailed": "未載",
  "config.disable": "黜置",

  "color.title": "Monet 取色之器",
  "color.seed": "子色",
  "color.spec": "取色之準",
  "color.hue": "色相",
  "color.tone": "濃淡",
  "color.reset": "復其初",
  "color.done": "已",
  "color.close": "閉",

  "miuix.monetEnable": "啟用 Monet 之色",

  "ui.material": "質樸",
  "ui.miuix": "米格",
  "ui.zn": "擬古",
  "ui.ios": "蘋果之風",

  "empty.title": "無卷可展",

  "error.title": "莫能讀識",

  "diagnostics.title": "診斷撮要",
  "diagnostics.copy": "謄錄診斷撮要",
  "diagnostics.loadLogs": "載近誌",
  "diagnostics.refreshLogs": "更載近誌",
  "diagnostics.loadingLogs": "載誌中…",
  "diagnostics.retryLogs": "重試載誌",
  "diagnostics.logsFailed": "近誌未能載入。",
  "diagnostics.export": "匯出呈報",
  "diagnostics.exporting": "匯出中…",
  "diagnostics.exportFailed": "呈報未能匯出，請重試。",
  "diagnostics.saved": "呈報已存於 {path}",

  "footer.github": "章魚貓",
};
