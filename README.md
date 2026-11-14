<div align="center">

# ZNNext

**A from-scratch, standalone implementation of the Zygisk Next API.**

[![Build][build-badge]][build-url]
[![Version][version-badge]][version-url]
[![GitHub Downloads][download-badge]][download-url]
[![License][license-badge]][license-url]

[build-badge]: https://img.shields.io/github/actions/workflow/status/VeryBaaad/ZNNext/ci.yml?branch=master&event=push
[build-url]: https://github.com/VeryBaaad/ZNNext/actions/workflows/ci.yml?query=branch%3Amaster+event%3Apush
[version-badge]: https://img.shields.io/github/v/release/VeryBaaad/ZNNext
[version-url]: https://github.com/VeryBaaad/ZNNext/releases/latest
[download-badge]: https://img.shields.io/github/downloads/VeryBaaad/ZNNext/total
[download-url]: https://github.com/VeryBaaad/ZNNext/releases/
[license-badge]: https://img.shields.io/github/license/VeryBaaad/ZNNext
[license-url]: LICENSE

</div>

> [!IMPORTANT]
> ZNNext is not a **Zygisk** implementation
> 
> ZNNext 不是一个 **Zygisk** 实现

---

### Introduction

ZNNext is an open-source Magisk module designed to provide the Zygisk Next API, striving to both implement and surpass the original Zygisk Next.

This module enables code injection into any process forked from `init`.

> [!NOTE]
> The Zygisk Next API was introduced in the closed-source v1.2.0-preview1 release of Zygisk Next; at that time, no existing open-source project was capable of loading these "ZN modules."
> 
> Thus, I created this project—built specifically for the open-source community.

---

### Compatibility

ZNNext supports Android 5 and above.

> [!TIP]
> Some features may not function on certain Linux kernels, this does not affect daily use.

---

### Install

1. Install Magisk 26402+ or KernelSU 11425+ or APatch 10700+
2. [Download](#download) and install ZNNext in your root solution manager
3. Reboot
4. Open the module's WebUI, begin your journey of custom configuration :)

---

### Download

| Channel | Source |
| --- | --- |
| Stable | [GitHub Releases](https://github.com/VeryBaaad/ZNNext/releases) |
| CI | [GitHub Actions](https://github.com/VeryBaaad/ZNNext/actions/workflows/ci.yml?query=branch%3Amaster) |

NOTE: The debug version is provided only in GitHub Actions.

---

### Get Help and Contributing

**Only bug reports from **THE LATEST DEBUG BUILD** will be accepted.**

**Please ensure that you are a **NATIVE SPEAKER** of the language in question for translation.**

- GitHub issues: [Issues](https://github.com/VeryBaaad/ZNNext/issues/)
- Pull Requests: [Pull Requests](https://github.com/VeryBaaad/ZNNext/pulls/)

---

### Community Discussion

- GitHub discussions: [Discussions](https://github.com/VeryBaaad/ZNNext/discussions)

---

### Developer Resources

- [ZygiskNextModuleSample](https://github.com/5ec1cff/ZygiskNextModuleSample): module sample
- [hyos_runtime.md](https://github.com/Dr-TSNG/ZygiskNext/blob/main/docs/hyos_runtime.md): for HyperOS Runtime

---

### Credits 

This project is made possible by the following open-source contributions:

- [Magisk](https://github.com/topjohnwu/Magisk/): makes all these possible
- [ZygiskNext](https://github.com/Dr-TSNG/ZygiskNext): original API reference
- [KernelPatch](https://github.com/bmax121/KernelPatch): for kernel patch module
- [libcxx](https://github.com/LSPosed/prefab-libcxx): libc++ statically linker

<details>
<summary>used for inline hooking</summary>

- [Dobby](https://github.com/LSPosed/Dobby): a lightweight, multi-platform, multi-architecture hook framework
- [rv64hook](https://github.com/eirv/riscv64-inline-hook): lightweight inline hook library for RISC-V 64
- [ShadowHook](https://github.com/bytedance/android-inline-hook): a general-purpose Android inline hook library, supporting Thumb, ARM32, and ARM64
- [And64InlineHook](https://github.com/Rprop/And64InlineHook): lightweight ARMv8-A(ARM64, AArch64, Little-Endian) Inline Hook Library for Android C/C++
- [frida-gum](https://github.com/frida/frida-gum): cross-platform instrumentation and introspection library written in C
- [HookZz](https://github.com/jmpews/HookZz/tree/2b48240203c95eb4164eded8f2b31f4bc846528a): a hook framework for arm/arm64/ios/android
- [UrHook](https://github.com/SMlc666/urhook): a versatile hooking library for Android/Linux ARM64
- [Whale](https://github.com/asLody/whale): hook Framework for Android/IOS/Linux/MacOS
- [BaaadHook](https://github.com/VeryBaaad/BaaadHook): inline/plt hook
- [YukiZygisk](https://github.com/Rouyashiki/YukiZygisk): a new kernel-level Zygisk implementation designed to explore better, cleaner, and more flexible ways of injecting

</details>

<details>
<summary>used for plt hooking</summary>

- [LSPlt](https://github.com/LSPosed/LSPlt): simple PLT hook for Android
- [ByteHook](https://github.com/bytedance/bhook): a universal Android PLT hook library, supporting armeabi-v7a, arm64-v8a, x86, and x86_64
- [xHook](https://github.com/iqiyi/xHook): a PLT hook library for Android native ELF
- [PLTI](https://github.com/PerformanC/PLTI): PerformanC's modern PLT hooking library
- [PLTHook](https://github.com/metacall/plthook): hook function calls by replacing PLT (Procedure Linkage Table) entries
- [UrHook](https://github.com/SMlc666/urhook): a versatile hooking library for Android/Linux ARM64
- [BaaadHook](https://github.com/VeryBaaad/BaaadHook): inline/plt hook
- [YukiPLT](https://github.com/Rouyashiki/YukiPLT): a standalone Android PLT/GOT hooking library
- [Profilo](https://github.com/facebookarchive/profilo): a library for performance traces from production
- [srx_hook](https://github.com/Kindness-Kismet/srx_hook): rust PLT Hook library

</details>

<details>
<summary>decompressors</summary>

- [LZMA SDK](https://www.7-zip.org/sdk.html): 7-Zip
- [xz-embedded](https://github.com/tukaani-project/xz-embedded): XZ Embedded
- [libarchive](https://github.com/libarchive/libarchive): multi-format archive and compression library

</details>

---

### License

ZNNext is licensed under the:

- [**GNU General Public License v3 (GPL-3)**](LICENSE).
- [**Developers And Contributors Are Lesbians Public License v2.0 (DCLPLv2)**](LICENSE-DCLPL).

Third-party license texts are bundled verbatim under `licenses/` in the module zip.
