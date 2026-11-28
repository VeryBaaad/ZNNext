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

export const ru: Dictionary = {
  "topbar.themeLabel": "Тема оформления",
  "topbar.langLabel": "Язык",

  "theme.auto": "По системе",
  "theme.light": "Светлая",
  "theme.dark": "Тёмная",

  "actions.copy": "Копировать",
  "actions.copied": "Скопировано",
  "actions.copyFailed": "Сбой копирования",
  "actions.refresh": "Обновить",

  "refresh.unavailable": "Не удалось загрузить {section}.",
  "refresh.stale": "Не удалось обновить {section}. Показаны данные на {time}.",
  "refresh.retry": "Повторить",
  "refresh.retrySection": "Повторно загрузить {section}",
  "refresh.section.status": "статус",
  "refresh.section.system": "сведения о системе",
  "refresh.section.modules": "список модулей",
  "refresh.section.config": "настройки",

  "status.active": "Активно",
  "status.inactive": "Не активно",

  "card.basic": "Основная информация",
  "card.status": "Статус",
  "card.settings": "Настройки",

  "zn.running": "Работает",
  "zn.stopped": "Не работает",
  "zn.daemon": "Демон",
  "zn.modules.title": "Модули ZN ({n})",
  "zn.modules.empty": "Нет модулей ZN",
  "zn.credit": "Designed by Mufanc, 5ec1cff & VeryBaaad",

  "info.version": "Версия",
  "info.rootImpl": "Реализация root",
  "info.rootImpl.none": "Не обнаружено",
  "info.mode": "Текущий режим",
  "info.modeReason.kernelUnavailable": "События ядра недоступны.",
  "info.modeReason.ptraceUnavailable": "ptrace недоступен.",
  "info.modeReason.otherLoaderActive": "Работает другой загрузчик.",
  "info.modeReason.switchDeferred": "Смена режима отложена.",
  "info.modeReason.restartRequired":
    "Для смены режима нужно перезапустить znnd.",
  "info.pageSize": "Размер страницы памяти",
  "info.unknown": "Неизвестно",

  "modules.disabled": "Отключён",
  "modules.noDeclarations": "Нет распознанных объявлений",
  "modules.missingLibraries": "Файлы библиотек отсутствуют",
  "modules.waitingForTarget": "Ожидание целевого процесса",
  "modules.processes": "{n} процесс(ов)",
  "modules.failed": "Записи о сбоях ({n})",
  "modules.copyFailure": "Скопировать сведения об ошибке",
  "modules.failedTitle": "Записи о сбоях загрузки",
  "modules.failedBanner":
    "В списке модулей есть записи о сбоях загрузки ({n}).",
  "modules.noModules": "Нет модулей",
  "modules.noProcesses": "Нет процессов",

  "config.inlineHook": "Inline Hook",
  "config.inlineHook32": "Inline Hook (32-бит)",
  "config.pltHook": "PLT Hook",
  "config.pltHook32": "PLT Hook (32-бит)",
  "config.mode": "Режим отслеживания",
  "config.mode.auto": "Авто",
  "config.mode.kernel": "kernel events",
  "config.mode.ptrace": "ptrace mode",
  "config.mode.proc": "proc mode",
  "config.mode.late": "late mode",
  "config.modeSavedCurrent":
    "Сохранено. Текущий режим по данным znnd — {mode}.",
  "config.modeSavedStopped":
    "Сохранено. znnd прочитает эту настройку при следующем запуске.",
  "config.modeSavedUnconfirmed":
    "Сохранено. Не удалось подтвердить текущий режим отслеживания.",
  "config.disableZn": "Перехватить ZygiskNext",
  "config.disableZn.true": "Вкл",
  "config.disableZn.false": "Выкл",
  "config.hotUpdate": "Горячее обновление",
  "config.hotUpdate.true": "Вкл",
  "config.hotUpdate.false": "Выкл",
  "config.debugdata": "Распаковщик gnu_debugdata",
  "config.debugdata.lzma": "LZMA SDK",
  "config.debugdata.xz": "XZ Embedded",
  "config.debugdata.libarchive": "libarchive",
  "config.uiStyle": "Стиль интерфейса",
  "config.saved": "Сохранено",
  "config.saveFailed": "Не удалось сохранить конфигурацию",
  "config.disable": "Отключить",

  "color.title": "Инструмент Monet",
  "color.seed": "Цвет семени",
  "color.spec": "Цветовая схема",
  "color.hue": "Оттенок",
  "color.tone": "Насыщенность и яркость",
  "color.reset": "Сброс",
  "color.done": "Готово",
  "color.close": "Закрыть",

  "miuix.monetEnable": "Включить цвета Monet",

  "ui.material": "Material",
  "ui.miuix": "Miuix",
  "ui.zn": "ZN-like",
  "ui.ios": "iOS Design",

  "empty.title": "Нет карточек для отображения",

  "error.title": "Не удалось загрузить данные",

  "diagnostics.title": "Сводка диагностики",
  "diagnostics.copy": "Скопировать сводку диагностики",
  "diagnostics.loadLogs": "Загрузить недавние логи",
  "diagnostics.refreshLogs": "Обновить логи",
  "diagnostics.loadingLogs": "Загрузка логов…",
  "diagnostics.retryLogs": "Повторить загрузку",
  "diagnostics.logsFailed": "Не удалось загрузить недавние логи.",
  "diagnostics.export": "Экспортировать отчёт",
  "diagnostics.exporting": "Экспорт…",
  "diagnostics.exportFailed":
    "Не удалось экспортировать отчёт. Попробуйте ещё раз.",
  "diagnostics.saved": "Отчёт сохранён в {path}",
  "diagnostics.historyTitle": "Недавние загрузки библиотек",
  "diagnostics.historyLoading": "Чтение истории…",
  "diagnostics.historyFailed":
    "Не удалось прочитать историю загрузок библиотек.",
  "diagnostics.historyEmpty": "Недавних загрузок библиотек нет.",
  "diagnostics.historyLoaded": "Загружена",
  "diagnostics.historyLoadFailed": "Ошибка загрузки",

  "footer.github": "Github",
};
