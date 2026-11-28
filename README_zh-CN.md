<div align="center">

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

[English](README.md) [文言（華夏）](README_lzh.md) [Русский](README_ru.md)

</div>

> [!IMPORTANT]
> ZNNext is not a **Zygisk** implementation
> 
> ZNNext 不是一个 **Zygisk** 实现

---

### 介绍

ZNNext 是一个开源的 Magisk 模块用于实现 Zygisk Next API，旨在提供好用，超越原作的实现。

该模块可以向由 `init` fork 出的进程注入代码。

> [!NOTE]
> Zygisk Next API在 Zygisk Next 闭源后的 v1.2.0-preview1 版本被内置，在那时，没有任何一个开源的项目来加载这些 “ZN 模块”。
> 
> 因此，我创建了这个为开源而生的项目。

---

### 兼容性

ZNNext 支持 Android 5 及以上

支持的处理器架构：arm64-v8a，armeabi-v7a，armeabi，x86_64，x86，riscv64，mips64，mips

> [!TIP]
> 一些功能在某些不支持的 Linux 内核上不可用，但不影响日常使用。

---

### 编译

`armeabi`、`x86`、`riscv64`、`mips64` 和 `mips` 需自行编译。用 `--abis` 指定目标架构。

```sh
python3 build.py zipRelease --abis riscv64
```

调试版使用 `zipDebug`。产物位于 `module/build/outputs/release/`，加上 `--split-debug-symbols` 可同时生成 `.symbols.zip`。

---

### 安装

1. 安装 Magisk 26402+ 或者 KernelSU 11425+ 或者 APatch 10700+
2. [下载](#下载)并在你的 root 管理器中安装 ZNNext
3. 重启
4. 打开模块的WebUI，开启你的配置之旅 :)

---

### 下载

| Channel | Source |
| --- | --- |
| 稳定版 | [GitHub Releases](https://github.com/VeryBaaad/ZNNext/releases) |
| CI | [GitHub Actions](https://github.com/VeryBaaad/ZNNext/actions/workflows/ci.yml?query=branch%3Amaster) |

NOTE: 调试版本仅在 GitHub Actions 中提供。

---

### 帮助或贡献

** 我们只接受 **最新的调试版本构建****

** 在贡献之前，请先确认你为贡献语言的 **母语者****

- GitHub issues: [Issues](https://github.com/VeryBaaad/ZNNext/issues/)
- Pull Requests: [Pull Requests](https://github.com/VeryBaaad/ZNNext/pulls/)
- GitHub discussions: [Discussions](https://github.com/VeryBaaad/ZNNext/discussions)

---

### 版本命名

v[Major].[Minor].[Patch][.Revision]

- Major / Minor / Patch: 和语义化版本保持一致。
- Revision: 进行了影响内部行为，而不影响外部行为的向下兼容变更（非必需）

---

### 开发资源

- [ZygiskNextModuleSample](https://github.com/5ec1cff/ZygiskNextModuleSample): 模块示例
- [hyos_runtime.md](https://github.com/Dr-TSNG/ZygiskNext/blob/main/docs/hyos_runtime.md): HyperOS Runtime示例

---

### 鸣谢

项目的制成离不开这些开源项目的付出：

- [Magisk](https://github.com/topjohnwu/Magisk/): 使一切成为可能
- [ZygiskNext](https://github.com/Dr-TSNG/ZygiskNext): 原始 API 参考
- [KernelPatch](https://github.com/bmax121/KernelPatch): Kernel Patch 模块
- [libcxx](https://github.com/LSPosed/prefab-libcxx): libc++ 静态链接器

<details>
<summary>内联钩子</summary>

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
<summary>PLT 钩子</summary>

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
<summary>用于解压</summary>

- [LZMA SDK](https://www.7-zip.org/sdk.html): 7-Zip
- [xz-embedded](https://github.com/tukaani-project/xz-embedded): XZ Embedded
- [libarchive](https://github.com/libarchive/libarchive): Multi-format archive and compression library

</details>

---

### 许可证

ZNNext 采用了以下许可证：

- [**GNU General Public License v3 (GPL-3)**](LICENSE).
- [**Developers And Contributors Are Lesbians Public License v2.0 (DCLPLv2)**](LICENSE-DCLPL).

第三方协议已在模块压缩包中的 `licenses/` 被存放。
