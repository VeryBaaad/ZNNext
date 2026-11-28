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

export const zhCN: Dictionary = {
  "topbar.themeLabel": "主题",
  "topbar.langLabel": "语言",

  "theme.auto": "跟随系统",
  "theme.light": "浅色",
  "theme.dark": "深色",

  "actions.copy": "复制",
  "actions.copied": "已复制",
  "actions.copyFailed": "复制失败",
  "actions.refresh": "刷新",

  "refresh.unavailable": "无法加载{section}。",
  "refresh.stale": "无法刷新{section}，当前显示 {time} 的数据。",
  "refresh.retry": "重试",
  "refresh.retrySection": "重新加载{section}",
  "refresh.section.status": "状态",
  "refresh.section.system": "设备信息",
  "refresh.section.modules": "模块列表",
  "refresh.section.config": "设置",

  "status.active": "已激活",
  "status.inactive": "未激活",

  "card.basic": "基本信息",
  "card.status": "状态",
  "card.settings": "设置",

  "zn.running": "运行中",
  "zn.stopped": "未运行",
  "zn.daemon": "守护进程",
  "zn.modules.title": "ZN 模块 ({n})",
  "zn.modules.empty": "未加载任何模块",
  "zn.credit": "Designed by Mufanc, 5ec1cff & VeryBaaad",

  "info.version": "版本",
  "info.rootImpl": "Root 实现",
  "info.rootImpl.none": "未检测到",
  "info.mode": "当前模式",
  "info.unknown": "未知",

  "modules.processes": "{n} 个进程",
  "modules.failed": "{n} 条失败记录",
  "modules.copyFailure": "复制失败记录详情",
  "modules.failedTitle": "加载失败记录",
  "modules.failedBanner": "有 {n} 条加载失败记录，请查看模块列表。",
  "modules.noModules": "无模块",
  "modules.noProcesses": "无进程",

  "config.inlineHook": "Inline Hook",
  "config.inlineHook32": "Inline Hook（32 位）",
  "config.pltHook": "PLT Hook",
  "config.pltHook32": "PLT Hook（32 位）",
  "config.mode": "跟踪模式",
  "config.mode.auto": "自动",
  "config.mode.kernel": "kernel events",
  "config.mode.ptrace": "ptrace mode",
  "config.mode.proc": "proc mode",
  "config.mode.late": "late mode",
  "config.modeSavedCurrent": "已保存。znnd 当前上报的跟踪模式为 {mode}。",
  "config.modeSavedPtraceRestart":
    "已保存。znnd 当前上报的跟踪模式为 {mode}。切换到 ptrace 需要重启 znnd。",
  "config.modeSavedStopped": "已保存。znnd 将在下次启动时读取此设置。",
  "config.modeSavedUnconfirmed": "已保存。无法确认当前跟踪模式。",
  "config.disableZn": "接管 ZygiskNext",
  "config.disableZn.true": "开启",
  "config.disableZn.false": "关闭",
  "config.hotUpdate": "热更新",
  "config.hotUpdate.true": "开启",
  "config.hotUpdate.false": "关闭",
  "config.debugdata": "gnu_debugdata 解压后端",
  "config.debugdata.lzma": "LZMA SDK",
  "config.debugdata.xz": "XZ Embedded",
  "config.debugdata.libarchive": "libarchive",
  "config.uiStyle": "界面样式",
  "config.saved": "已保存",
  "config.saveFailed": "保存失败",
  "config.disable": "禁用",

  "color.title": "Monet 取色工具",
  "color.seed": "种子色",
  "color.spec": "取色标准",
  "color.hue": "色相",
  "color.tone": "饱和度与明度",
  "color.reset": "恢复默认",
  "color.done": "完成",
  "color.close": "关闭",

  "miuix.monetEnable": "启用 Monet 颜色",

  "ui.material": "Material",
  "ui.miuix": "Miuix",
  "ui.zn": "类ZN风格",
  "ui.ios": "iOS 风格",

  "empty.title": "无卡片可绘制",

  "error.title": "无法读取数据",

  "diagnostics.title": "诊断摘要",
  "diagnostics.copy": "复制诊断摘要",

  "footer.github": "Github",
};
