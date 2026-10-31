# ZNNext

A from-scratch, standalone implementation of the **Zygisk Next module**.

> [!IMPORTANT]
> ZNNext is not a **Zygisk** implementation
> 
> ZNNext 不是一个 **Zygisk** 实现

## Introduction

A Magisk module that attempts to implement the ZygiskNext API.

> A Zygisk Next module is a valid Magisk module. Starting with version v1.2.0—preview1 after the project went closed-source—ZygiskNext introduced support for "ZygiskNext modules." These provide more convenient interfaces for inline and PLT hooking and even support advanced injection into the HyperOS Runtime. Consequently, I created this open-source Magisk module to load those ZygiskNext modules.

## Supported Versions

- Android 5.0 - 17 QPR3 Beta1
- Linux 3.17 or newer

or

- GrapheneOS
- Linux 3.17 or newer

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

- [Magisk](https://github.com/topjohnwu/Magisk/): makes all these possible
- [ZygiskNext](https://github.com/LSPosed/ZygiskNext): OG implementation
- [Dobby](https://github.com/LSPosed/Dobby), [rv64hook](https://github.com/eirv/riscv64-inline-hook), [ShadowHook](https://github.com/bytedance/android-inline-hook), [And64InlineHook](https://github.com/Rprop/And64InlineHook), [frida-gum](https://github.com/frida/frida-gum), [HookZz](https://github.com/jmpews/HookZz/tree/2b48240203c95eb4164eded8f2b31f4bc846528a), [UrHook](https://github.com/SMlc666/urhook), [Whale](https://github.com/asLody/whale) and [BaaadHook](https://github.com/VeryBaaad/BaaadHook): used for inline hooking
- [LSPlt](https://github.com/LSPosed/LSPlt), [ByteHook](https://github.com/bytedance/bhook), [xHook](https://github.com/iqiyi/xHook), [PLTI](https://github.com/PerformanC/PLTI), [PLTHook](https://github.com/metacall/plthook), [UrHook](https://github.com/SMlc666/urhook), [BaaadHook](https://github.com/VeryBaaad/BaaadHook) and [YukiPLT](https://github.com/Rouyashiki/YukiPLT): used for plt hooking
- [libcxx](https://github.com/LSPosed/prefab-libcxx): libc++/libc++abi statically linked into the loader
- [LZMA SDK](https://www.7-zip.org/sdk.html) and [xz-embedded](https://github.com/tukaani-project/xz-embedded): `.gnu_debugdata` decompressor
- [KernelPatch](https://github.com/bmax121/KernelPatch): for KernelPatch Module

## License

ZNNext is licensed under the **GNU General Public License v3 (GPL-3)** (http://www.gnu.org/copyleft/gpl.html).

Third-party license texts are bundled verbatim under `licenses/` in the module zip.
