<div align="center">

# ZNNext

**自草創而獨立，別為一體之 Zygisk Next API 實現也。**

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
[license-url]: ../LICENSE
[license-dclpl-badge]: https://img.shields.io/badge/license-DCLPL--2.0-orange
[license-dclpl-url]: ../LICENSE-DCLPL

[English](README.md) [中文（简体）](README_zh-CN.md)

</div>

> [!IMPORTANT]
> ZNNext is not a **Zygisk** implementation
> 
> ZNNext 不是一个 **Zygisk** 实现

---

### 緣起

ZNNext 者，開源之 Magisk 模塊也，所以行 Zygisk Next API，既期躬行，亦欲軼乎原作。

此模塊能將代碼注入凡由 `init` 分衍之進程。

> [!NOTE]
> Zygisk Next API 始見於閉源之 v1.2.0-preview1 版；當是時，開源項目未有能載此「ZN 模塊」者。
> 
> 故余創此項目，專為開源而設。

---

### 相容性

ZNNext 施於 Android 5 以上。

所容之架：arm64-v8a、armeabi-v7a、armeabi、x86_64、x86、riscv64、mips64、mips

> [!TIP]
> 數端或不行於若干之核，然無礙日用。
>
> armeabi、x86、riscv64、mips64、mips 者，須自撰：
> ```
> build.py zip<Release|Debug> --abis <armeabi,x86,riscv64,mips64,mips>
> ```

---

### 安置

1. 安置 Magisk 26402+，或 KernelSU 11425+，或 APatch 10700+
2. [下載](#下載)並於所根管理器（判官）中安置 ZNNext
3. 復啟
4. 啟模塊之 WebUI，始爾自定規制之遊 :)

---

### 下

| 徑 | 源 |
| --- | --- |
| 穩版 | [GitHub Releases](https://github.com/VeryBaaad/ZNNext/releases) |
| CI | [GitHub Actions](https://github.com/VeryBaaad/ZNNext/actions/workflows/ci.yml?query=branch%3Amaster) |

註：調版唯見於 GitHub Actions。

---

### 求助與共襄

**所納錯之報，唯出於最新調建者。**

**助譯者，須為所譯語之母語者。**

- GitHub issues：[Issues](https://github.com/VeryBaaad/ZNNext/issues/)
- Pull Requests：[Pull Requests](https://github.com/VeryBaaad/ZNNext/pulls/)
- GitHub discussions：[Discussions](https://github.com/VeryBaaad/ZNNext/discussions)

---

### 版本命名

v[Major].[Minor].[Patch][.Revision]

- Major / Minor / Patch：與語義化版本同。
- Revision：為向後兼容之變，僅易內部行為，不改外部表現者。（非必）

---

### 開發資源

- [ZygiskNextModuleSample](https://github.com/5ec1cff/ZygiskNextModuleSample)：模示
- [hyos_runtime.md](https://github.com/Dr-TSNG/ZygiskNext/blob/main/docs/hyos_runtime.md)：HyperOS Runtime 之用

---

### 鳴謝

此項目之成，實賴諸開源項目之力：

- [Magisk](https://github.com/topjohnwu/Magisk/)：使諸事皆成
- [ZygiskNext](https://github.com/Dr-TSNG/ZygiskNext)：原樞機之參照
- [KernelPatch](https://github.com/bmax121/KernelPatch)：補核模塊所賴
- [libcxx](https://github.com/LSPosed/prefab-libcxx)：libc++ 靜態鏈接器

<details>
<summary>中道截擊之用</summary>

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

</details>

<details>
<summary>依表截擊之用</summary>

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
<summary>解封諸法</summary>

- [LZMA SDK](https://www.7-zip.org/sdk.html): 7-Zip
- [xz-embedded](https://github.com/tukaani-project/xz-embedded): XZ Embedded
- [libarchive](https://github.com/libarchive/libarchive): Multi-format archive and compression library

</details>

---

### 許

ZNNext 所採之許可如左：

- [**GNU 眾所通用之許第三版（GPL-3）**](../LICENSE).
- [**創制者與襄贊皆為女同者公共許第二版（DCLPLv2）**](../LICENSE-DCLPL).

第三方許可之文，已原附於模包之 `licenses/` 下。
