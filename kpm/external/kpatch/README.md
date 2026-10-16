# Vendored KernelPatch headers

These headers are copied verbatim from KernelPatch so that `:kpm` compiles
offline. Nothing here is modified; do not edit these files locally.

- Upstream: <https://github.com/bmax121/KernelPatch>
- Pinned commit: `ef52c62b339bd2da51880922249bf4edef71b1cf` (`ef52c62`, 2026-10-14, KernelPatch 0.13.9)
- Upstream license: GPL-2.0-or-later (repository `LICENSE` is GPL-2.0). GPL-2.0-or-later
  is compatible with this project's GPL-3.0-or-later, so the combination is redistributable
  under GPLv3.
- Provenance check: `git -C <KernelPatch> log -1 --format=%H` and compare the list below
  against `git show <commit>:<path>`.

## Layout

The directory layout mirrors the KernelPatch `kernel/` include roots, with `kernel/`
stripped, so the vendored paths stay greppable against upstream:

| Vendored path | Upstream path | Compiler flag |
| --- | --- | --- |
| `include/` | `kernel/include/` | `-I.../include` |
| `patch/include/` | `kernel/patch/include/` | `-I.../patch/include` |
| `linux/include/` | `kernel/linux/include/` | `-I.../linux/include` |
| `linux/arch/arm64/include/` | `kernel/linux/arch/arm64/include/` | `-I.../linux/arch/arm64/include` |

Upstream's `kernel/linux/tools/arch/arm64/include` root is not needed by
`znn.c` and is therefore not vendored. The build passes the four roots above
with `-I` ahead of the NDK sysroot, and this exact file set was verified to
compile with `-nostdinc` and no NDK sysroot headers at all, so the closure below
is complete.

## Vendored files

Files whose first lines carry the upstream `SPDX-License-Identifier: GPL-2.0-or-later`
notice (preserved verbatim):

- `include/hook.h`
- `include/kpmodule.h`
- `include/log.h`
- `include/pgtable.h`
- `patch/include/kbtf.h`
- `patch/include/kputils.h`
- `patch/include/ksyms.h`
- `patch/include/syscall.h`
- `patch/include/uapi/scdefs.h`

Files that upstream ships without a per-file SPDX notice. They are still part of
KernelPatch and covered by its repository-level GPL-2.0 license; a few of them
(`stdint.h`, `stdbool.h`, `linux/printk.h`, `linux/kallsyms.h`,
`linux/kern_levels.h`, `asm/*`, `uapi/asm/*`, `uapi/asm-generic/*`) are
KernelPatch's own stubs or derived from Linux UAPI:

- `include/compiler.h`
- `include/kallsyms.h`
- `include/ktypes.h`
- `include/stdbool.h`
- `include/stdint.h`
- `linux/arch/arm64/include/asm/current.h`
- `linux/arch/arm64/include/asm/ptrace.h`
- `linux/arch/arm64/include/uapi/asm/ptrace.h`
- `linux/include/linux/kallsyms.h`
- `linux/include/linux/kern_levels.h`
- `linux/include/linux/printk.h`
- `linux/include/uapi/asm-generic/errno-base.h`
- `linux/include/uapi/asm-generic/errno.h`
- `linux/include/uapi/asm-generic/unistd.h`

## Updating

Re-run the compiler dependency closure for `kpm/src/znn.c` against the desired
KernelPatch commit and copy only the files it lists, keeping this layout. Never fetch
from the network at build time; extend this directory instead.
