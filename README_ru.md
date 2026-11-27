<div align="center">

## !!! Этот документ был переведен с использованием ИИ и может содержать ошибки !!!

# ZNNext

**A from-scratch, standalone implementation of the Zygisk Next API.**

[![Build][build-badge]][build-url]
[![Version][version-badge]][version-url]
[![GitHub Downloads][download-badge]][download-url]
[![License][license-badge]][license-url]
[![License-DCLPL][license-dclpl-badge]][license-dclpl-url]

[build-badge]: https://img.shields.io/github/actions/workflow/status/VeryBaaad/ZNNext/ci.yml?branch=master&event=push
[build-url]: https://github.com/VeryBaaad/ZNNext/actions/workflows/ci.yml?query=branch%3Amaster+event%3Apush
[version-badge]: https://img.shields.io/github/v/release/VeryBaaad/ZNNext
[version-url]: https://github.com/VeryBaaad/ZNNext/releases/latest
[download-badge]: https://img.shields.io/github/downloads/VeryBaaad/ZNNext/total
[download-url]: https://github.com/VeryBaaad/ZNNext/releases/
[license-badge]: https://img.shields.io/github/license/VeryBaaad/ZNNext
[license-url]: LICENSE
[license-dclpl-badge]: https://img.shields.io/badge/license-DCLPL--2.0-orange
[license-dclpl-url]: LICENSE-DCLPL

[English](README.md) [中文（简体）](README_zh-CN.md) [文言（華夏）](README_lzh.md)

</div>

> [!IMPORTANT]
> ZNNext is not a **Zygisk** implementation
> 
> ZNNext 不是一个 **Zygisk** 实现

---

### Введение

ZNNext — это модуль Magisk с открытым исходным кодом, предоставляющий API Zygisk Next. Его цель — не только реализовать функционал оригинального Zygisk Next, но и превзойти его.

Данный модуль позволяет внедрять код в любой процесс, порожденный (созданный через `fork`) процессом `init`.

> [!NOTE]
> API Zygisk Next был представлен в версии v1.2.0-preview1 — выпуске Zygisk Next с закрытым исходным кодом. На тот момент ни один из существующих проектов с открытым исходным кодом не поддерживал загрузку подобных «ZN-модулей».
> 
> Именно поэтому я создал данный проект — специально для сообщества разработчиков открытого программного обеспечения.

---

### Совместимость

ZNNext поддерживает Android 5 и выше.

Поддерживаемые архитектуры (ABI): arm64-v8a, armeabi-v7a, armeabi, x86_64, x86, riscv64, mips64, mips

> [!TIP]
> Некоторые функции могут не работать на определенных ядрах Linux; это не влияет на повседневное использование.
>
> Для архитектур armeabi, x86, riscv64, mips64 и mips требуется самостоятельная сборка:
> ```
> build.py zip<Release|Debug> --abis <armeabi,x86,riscv64,mips64,mips>
> ```

---

### Установка

1. Установите Magisk 26402+ (или новее), KernelSU 11425+ (или новее) либо APatch 10700+ (или новее).
2. [Скачайте](#download) и установите ZNNext через используемый вами менеджер root-прав.
3. Перезагрузите устройство.
4. Откройте веб-интерфейс (WebUI) модуля и приступайте к настройке под свои предпочтения :)

---

### Скачать

| Канал | Источник |
| --- | --- |
| Стабильная версия (Stable) | [GitHub Releases](https://github.com/VeryBaaad/ZNNext/releases) |
| CI (сборки из исходного кода) | [GitHub Actions](https://github.com/VeryBaaad/ZNNext/actions/workflows/ci.yml?query=branch%3Amaster) |

ПРИМЕЧАНИЕ: Отладочная (debug) версия доступна только в GitHub Actions.

---

### Получение помощи и участие в разработке

**Принимаются только сообщения об ошибках, обнаруженных в **ПОСЛЕДНЕЙ ОТЛАДОЧНОЙ СБОРКЕ (DEBUG BUILD)**.**

**Если вы хотите помочь с переводом, убедитесь, что язык, на который вы переводите, является для вас **РОДНЫМ**.**

- Проблемы (Issues) на GitHub: [Issues](https://github.com/VeryBaaad/ZNNext/issues/)
- Запросы на включение кода (Pull Requests): [Pull Requests](https://github.com/VeryBaaad/ZNNext/pulls/)
- Обсуждения на GitHub: [Discussions](https://github.com/VeryBaaad/ZNNext/discussions)

---

### Именование версий

v[Major].[Minor].[Patch][.Revision]

- Major / Minor / Patch: так же, как в semver.
- Revision: при внесении обратно совместимого изменения, которое затрагивает только внутреннее поведение, не меняя при этом внешнее поведение. (не является обязательным требованием)

---

### Ресурсы для разработчиков

- [ZygiskNextModuleSample](https://github.com/5ec1cff/ZygiskNextModuleSample): пример модуля
- [hyos_runtime.md](https://github.com/Dr-TSNG/ZygiskNext/blob/main/docs/hyos_runtime.md): для HyperOS Runtime

---

### Авторы

Этот проект стал возможен благодаря следующему вкладу в виде открытого исходного кода:

- [Magisk](https://github.com/topjohnwu/Magisk/): makes all these possible
- [ZygiskNext](https://github.com/Dr-TSNG/ZygiskNext): original API reference
- [KernelPatch](https://github.com/bmax121/KernelPatch): for kernel patch module
- [libcxx](https://github.com/LSPosed/prefab-libcxx): libc++ statically linker

<details>
<summary>используется для inline-хукинга</summary>

- [Dobby](https://github.com/LSPosed/Dobby): a lightweight, multi-platform, multi-architecture hook framework
- [rv64hook](https://github.com/eirv/riscv64-inline-hook): Lightweight inline hook library for RISC-V 64 / 适用于 RISC-V 64 的轻量级 Inline Hook 库
- [ShadowHook](https://github.com/bytedance/android-inline-hook): 通用的Android inline hook库，支持thumb，arm32，arm64
- [And64InlineHook](https://github.com/Rprop/And64InlineHook): Lightweight ARMv8-A(ARM64, AArch64, Little-Endian) Inline Hook Library for Android C/C++
- [frida-gum](https://github.com/frida/frida-gum): Cross-platform instrumentation and introspection library written in C
- [HookZz](https://github.com/jmpews/HookZz/tree/2b48240203c95eb4164eded8f2b31f4bc846528a): a hook framework for arm/arm64/ios/android
- [UrHook](https://github.com/SMlc666/urhook): 一个安卓/Linux Arm64的多功能hook库
- [Whale](https://github.com/asLody/whale): Hook Framework for Android/IOS/Linux/MacOS
- [BaaadHook](https://github.com/VeryBaaad/BaaadHook): inline/plt hook
- [YukiZygisk](https://github.com/Rouyashiki/YukiZygisk): A new kernel-level Zygisk implementation designed to explore better, cleaner, and more flexible ways of injecting.
- [SandHook](https://github.com/asLody/SandHook): Android ART Hook/Native Inline Hook/Single Instruction Hook - support 4.4 - 11.0 32/64 bit - Xposed API Compat
- [srx_inline_hook](https://github.com/Kindness-Kismet/srx_inline_hook): Rust inline hook

</details>

<details>
<summary>используется для перехвата через PLT</summary>

- [LSPlt](https://github.com/LSPosed/LSPlt): Simple PLT hook for Android
- [ByteHook](https://github.com/bytedance/bhook): 通用的Android PLT hook库，支持armeabi-v7a，arm64-v8a，x86，x86_64
- [xHook](https://github.com/iqiyi/xHook): A PLT hook library for Android native ELF
- [PLTI](https://github.com/PerformanC/PLTI): PerformanC's modern PLT hooking library
- [PLTHook](https://github.com/metacall/plthook): Hook function calls by replacing PLT (Procedure Linkage Table) entries.
- [UrHook](https://github.com/SMlc666/urhook): 一个安卓/Linux Arm64的多功能hook库
- [BaaadHook](https://github.com/VeryBaaad/BaaadHook): inline/plt hook
- [YukiPLT](https://github.com/Rouyashiki/YukiPLT): A standalone Android PLT/GOT hooking library
- [Profilo](https://github.com/facebookarchive/profilo): A library for performance traces from production.
- [srx_hook](https://github.com/Kindness-Kismet/srx_hook): Rust PLT Hook 库

</details>

<details>
<summary>декомпрессоры</summary>

- [LZMA SDK](https://www.7-zip.org/sdk.html): 7-Zip
- [xz-embedded](https://github.com/tukaani-project/xz-embedded): XZ Embedded
- [libarchive](https://github.com/libarchive/libarchive): Multi-format archive and compression library

</details>

---

### Лицензия

Программное обеспечение ZNNext распространяется на условиях следующих лицензий:

- [**GNU General Public License v3 (GPL-3)**](LICENSE).
- [**Developers And Contributors Are Lesbians Public License v2.0 (DCLPLv2)**](LICENSE-DCLPL).

Тексты сторонних лицензий включены в состав ZIP-архива модуля (в папку `licenses/`) в неизменном виде.
