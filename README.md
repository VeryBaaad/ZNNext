<div align="center">

# ZNNext

**A from-scratch, standalone implementation of the Zygisk Next module.**

[![Build][build-badge]][build-url]
[![Version][version-badge]][version-url]
[![GitHub Downloads][download-badge]][download-url]

[build-badge]: https://img.shields.io/github/actions/workflow/status/VeryBaaad/ZNNext/ci.yml?branch=master&event=push&logo=github
[build-url]: https://github.com/VeryBaaad/ZNNext/actions/workflows/ci.yml?query=branch%3Amaster+event%3Apush
[version-badge]: https://img.shields.io/github/v/release/VeryBaaad/ZNNext?logo=github
[version-url]: https://github.com/VeryBaaad/ZNNext/releases/latest
[download-badge]: https://img.shields.io/github/downloads/VeryBaaad/ZNNext/total?logo=github
[download-url]: https://github.com/VeryBaaad/ZNNext/releases/


</div>

> [!IMPORTANT]
> ZNNext is not a **Zygisk** implementation
> 
> ZNNext 不是一个 **Zygisk** 实现

## Introduction

A Magisk module that attempts to implement the ZygiskNext API.

> A Zygisk Next module is a valid Magisk module. Starting with version v1.2.0—preview1 after the project went closed-source—ZygiskNext introduced support for "ZygiskNext modules." These provide more convenient interfaces for inline and PLT hooking and even support advanced injection into the HyperOS Runtime. Consequently, I created this open-source Magisk module to load those ZygiskNext modules.

## Supported Versions

- Android 5.0 or newer
- Linux 3.17 or newer
- armv8, armv7, armv5, mips, mips64, riscv64, x86, x64

> Kernel event tracking is optional and uses whatever the kernel provides: tracefs
> `sched_process_exec` events first, then the proc connector and eBPF. A `/proc`
> polling mode needs no kernel support at all, so any supported kernel works
> without kernel events.

## Install

1. Install Magisk 26402+ or KernelSU 11425+ or APatch 10700+
2. [Download](#download) and install ZNNext in manager
3. Reboot
4. Have fun :)

## Download

- For stable releases, please go to [Github Releases page](https://github.com/VeryBaaad/ZNNext/releases)
- For canary build, please check [Github Actions](https://github.com/VeryBaaad/ZNNext/actions/workflows/ci.yml?query=branch%3Amaster)

Note: debug builds are only available in Github Actions.

## Get Help

**Only bug reports from **THE LATEST DEBUG BUILD** will be accepted.**

- GitHub issues: [Issues](https://github.com/VeryBaaad/ZNNext/issues/)

## For Developers

Read the [ZygiskNextModuleSample](https://github.com/5ec1cff/ZygiskNextModuleSample)

### HyperOS Runtime

Look this: [hyos_runtime.md](https://github.com/LSPosed/ZygiskNext/blob/main/docs/hyos_runtime.md)

## Community Discussion

- Telegram: [@EdLSPesodITed](https://t.me/EdLSPesodITed)

Notice: These community groups don't accept any bug report, please use [Get help](#get-help) to report.

## Translation Contributing

Create a Pull Request.

## Credits 

This project is made possible by the following open-source contributions:

- [Magisk](https://github.com/topjohnwu/Magisk/): makes all these possible
- [ZygiskNext](https://github.com/LSPosed/ZygiskNext): zygisk next api
- [KernelPatch](https://github.com/bmax121/KernelPatch): for kernel patch module
- [libcxx](https://github.com/LSPosed/prefab-libcxx): libc++ statically linker.

<details>
<summary>used for inline hooking</summary>

- [Dobby](https://github.com/LSPosed/Dobby)
- [rv64hook](https://github.com/eirv/riscv64-inline-hook)
- [ShadowHook](https://github.com/bytedance/android-inline-hook)
- [And64InlineHook](https://github.com/Rprop/And64InlineHook)
- [frida-gum](https://github.com/frida/frida-gum)
- [HookZz](https://github.com/jmpews/HookZz/tree/2b48240203c95eb4164eded8f2b31f4bc846528a)
- [UrHook](https://github.com/SMlc666/urhook)
- [Whale](https://github.com/asLody/whale)
- [BaaadHook](https://github.com/VeryBaaad/BaaadHook)
- [YukiZygisk](https://github.com/Rouyashiki/YukiZygisk)

</details>

<details>
<summary>used for plt hooking</summary>

- [LSPlt](https://github.com/LSPosed/LSPlt)
- [ByteHook](https://github.com/bytedance/bhook)
- [xHook](https://github.com/iqiyi/xHook)
- [PLTI](https://github.com/PerformanC/PLTI)
- [PLTHook](https://github.com/metacall/plthook)
- [UrHook](https://github.com/SMlc666/urhook)
- [BaaadHook](https://github.com/VeryBaaad/BaaadHook)
- [YukiPLT](https://github.com/Rouyashiki/YukiPLT)
- [Profilo](https://github.com/facebookarchive/profilo)
- [srx_hook](https://github.com/Kindness-Kismet/srx_hook)

</details>

<details>
<summary>decompressors</summary>

- [LZMA SDK](https://www.7-zip.org/sdk.html)
- [xz-embedded](https://github.com/tukaani-project/xz-embedded)
- [libarchive](https://github.com/libarchive/libarchive)

</details>

## License

ZNNext is licensed under the **GNU General Public License v3 (GPL-3)** (http://www.gnu.org/copyleft/gpl.html).

Third-party license texts are bundled verbatim under `licenses/` in the module zip.
