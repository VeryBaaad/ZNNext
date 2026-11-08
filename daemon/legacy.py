import os
import re
import sys
import tarfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT))
import build as znn

TARGET_DIR = ROOT / "daemon" / "targets"
LIBC_VERSIONS = ("0.2.185", "0.2.191")
NDK_MIN_SDK = znn.ANDROID_MIN_SDK


def legacy_linker(ndk, abi, workdir):
    workdir.mkdir(parents=True, exist_ok=True)
    if abi == "armeabi":
        return _armv5_linker(ndk, workdir)
    return _mips_linker(ndk, abi, workdir)


def _compile_getrandom(cc, ndk, abi, destination):
    source = destination.parent / "getrandom.c"
    source.write_text(
        """#include <errno.h>
#include <fcntl.h>
#include <stddef.h>
#include <sys/syscall.h>
#include <unistd.h>

ssize_t getrandom(void *buffer, size_t length, unsigned int flags) {
    long result = syscall(SYS_getrandom, buffer, length, flags);
    if (result >= 0) return (ssize_t) result;
    if (errno != ENOSYS) return -1;
    int fd = open("/dev/urandom", O_RDONLY | O_CLOEXEC);
    if (fd < 0) return -1;
    ssize_t total = 0;
    while ((size_t) total < length) {
        ssize_t count = read(fd, (char *) buffer + total, length - (size_t) total);
        if (count <= 0) { close(fd); return total > 0 ? total : -1; }
        total += count;
    }
    close(fd);
    return total;
}
""",
        encoding="utf-8",
    )
    object = destination.parent / f"getrandom-{abi}.o"
    header = {
        "armeabi": "arm-linux-androideabi",
        "mips": "mipsel-linux-android",
        "mips64": "mips64el-linux-android",
    }[abi]
    flags = [f"--sysroot={ndk / 'sysroot'}", f"-I{ndk / 'sysroot' / 'usr' / 'include' / header}"]
    if abi == "armeabi":
        flags += ["-march=armv5te", "-msoft-float"]
    znn.run([cc, *flags, "-fPIC", "-c", source, "-o", object])
    ar = ndk / "toolchains" / "llvm" / "prebuilt" / znn.host_tag() / "bin" / "llvm-ar"
    znn.run([ar, "rcs", destination, object])


def _symlink_or_copy(source, destination):
    destination.unlink(missing_ok=True)
    destination.symlink_to(source)


def _armv5_linker(ndk, workdir):
    prefix = "arm-linux-androideabi"
    gcc_root = ndk / "toolchains" / f"{prefix}-4.9" / "prebuilt" / znn.host_tag()
    gcc = gcc_root / "bin" / f"{prefix}-gcc"
    clang = ndk / "toolchains" / "llvm" / "prebuilt" / znn.host_tag() / "bin" / "clang"
    ld = gcc_root / "bin" / f"{prefix}-ld"
    libs = workdir / "libs"
    libs.mkdir(parents=True, exist_ok=True)
    if not (libs / "libgetrandom-armeabi.a").is_file():
        _compile_getrandom(str(gcc), ndk, "armeabi", libs / "libgetrandom-armeabi.a")
    linker = workdir / "armv5te-linux-androideabi-clang"
    linker.write_text(
        "#!/bin/sh\n"
        f"exec {clang} --target=armv5te-linux-androideabi{NDK_MIN_SDK} "
        f"--sysroot={ndk / 'platforms' / f'android-{NDK_MIN_SDK}' / 'arch-arm'} "
        f"--gcc-toolchain={gcc_root} -fuse-ld={ld} "
        f"-L{ndk / 'sources' / 'cxx-stl' / 'llvm-libc++' / 'libs' / 'armeabi'} "
        f"-L{libs} -L{gcc_root / 'lib' / 'gcc' / prefix / '4.9.x'} "
        f"\"$@\" -lgetrandom-armeabi\n",
        encoding="utf-8",
    )
    linker.chmod(0o755)
    return linker


def _mips_linker(ndk, abi, workdir):
    prefix = "mipsel-linux-android" if abi == "mips" else "mips64el-linux-android"
    arch = "mips" if abi == "mips" else "mips64"
    gcc_root = ndk / "toolchains" / f"{prefix}-4.9" / "prebuilt" / znn.host_tag()
    gcc = gcc_root / "bin" / f"{prefix}-gcc"
    clang = ndk / "toolchains" / "llvm" / "prebuilt" / znn.host_tag() / "bin" / "clang"
    ld = gcc_root / "bin" / f"{prefix}-ld"
    libs = workdir / "libs"
    libs.mkdir(parents=True, exist_ok=True)
    _symlink_or_copy(
        gcc_root / "lib" / "gcc" / prefix / "4.9.x" / "libgcc.a",
        libs / "libunwind.a",
    )
    if not (libs / f"libgetrandom-{abi}.a").is_file():
        _compile_getrandom(str(gcc), ndk, abi, libs / f"libgetrandom-{abi}.a")
    linker = workdir / f"{prefix}-clang"
    linker.write_text(
        "#!/bin/sh\n"
        f"exec {clang} --target={prefix}{NDK_MIN_SDK} "
        f"--sysroot={ndk / 'platforms' / f'android-{NDK_MIN_SDK}' / f'arch-{arch}'} "
        f"-fuse-ld={ld} "
        f"-L{libs} -L{gcc_root / 'lib' / 'gcc' / prefix / '4.9.x'} "
        f"-L{ndk / 'platforms' / f'android-{NDK_MIN_SDK}' / f'arch-{arch}' / 'usr' / 'lib'} "
        f"\"$@\" -lgetrandom-{abi}\n",
        encoding="utf-8",
    )
    linker.chmod(0o755)
    return linker


def _libc_versions():
    sysroot = znn.run_capture(["rustc", "+nightly", "--print", "sysroot"]).strip()
    locks = [
        Path(sysroot) / "lib/rustlib/src/rust/library/Cargo.lock",
        ROOT / "daemon/Cargo.lock",
    ]
    versions = set()
    for lock in locks:
        if not lock.is_file():
            continue
        versions.update(
            re.findall(r'name = "libc"\nversion = "([^"]+)"', lock.read_text())
        )
    return sorted(versions) or list(LIBC_VERSIONS)


def prepare_cargo_home(workdir):
    cargo_home = workdir / "cargo-home"
    registry = cargo_home / "registry"
    real_home = Path(os.environ.get("CARGO_HOME", Path.home() / ".cargo"))
    registry.mkdir(parents=True, exist_ok=True)
    for name in ("index", "cache"):
        target = registry / name
        if not target.exists():
            target.symlink_to(real_home / "registry" / name)
    source_root = registry / "src"
    source_root.mkdir(parents=True, exist_ok=True)
    for index in (real_home / "registry" / "src").glob("*"):
        destination = source_root / index.name
        destination.mkdir(exist_ok=True)
    for version in _libc_versions():
        crate = _extract_libc(version, source_root, real_home, workdir)
        if crate is not None:
            patch_libc(crate)
    return cargo_home


def _libc_archive(version, real_home, workdir):
    for cache in (real_home / "registry" / "cache").glob(f"*/libc-{version}.crate"):
        return cache
    manifest = workdir / f"fetch-libc-{version}" / "Cargo.toml"
    manifest.parent.mkdir(parents=True, exist_ok=True)
    manifest.write_text(
        "[package]\n"
        f'name = "fetch-libc-{version.replace(".", "-")}"\n'
        'version = "0.0.0"\n'
        'edition = "2021"\n\n'
        "[dependencies]\n"
        f'libc = "={version}"\n',
        encoding="utf-8",
    )
    environment = os.environ.copy()
    environment["CARGO_HOME"] = str(real_home)
    znn.run(
        ["cargo", "+nightly", "fetch", "--manifest-path", manifest],
        env=environment,
    )
    for cache in (real_home / "registry" / "cache").glob(f"*/libc-{version}.crate"):
        return cache
    raise znn.BuildError(f"cannot fetch libc {version} for the MIPS build shim")


def _extract_libc(version, source_root, real_home, workdir):
    name = f"libc-{version}"
    for index in source_root.iterdir():
        candidate = index / name
        if candidate.is_dir():
            (candidate / ".cargo-ok").write_text('{"v":1}', encoding="utf-8")
            return candidate
    archive = _libc_archive(version, real_home, workdir)
    for index in source_root.iterdir():
        with tarfile.open(archive, "r:gz") as package:
            package.extractall(index)
        crate = index / name
        if crate.is_dir():
            (crate / ".cargo-ok").write_text('{"v":1}', encoding="utf-8")
            return crate
    return None


def patch_libc(crate):
    if (crate / "src/unix/linux_like/android/b32/mips.rs").is_file():
        return
    macros = (crate / "src/macros.rs").read_text(encoding="utf-8")
    modern = "either `safe` or `unsafe` must be specified" in macros
    for bits, arch, linux in (
        ("b32", "mips", "src/unix/linux_like/linux/gnu/b32/mips/mod.rs"),
        ("b64", "mips64", "src/unix/linux_like/linux/gnu/b64/mips64/mod.rs"),
    ):
        constants = [
            line
            for line in (crate / linux).read_text(encoding="utf-8").splitlines()
            if line.startswith("pub const SYS_")
        ]
        header = (
            "// MIPS Android support added by the ZNNext legacy build shim.\n"
            "use crate::prelude::*;\n"
        )
        if arch == "mips64":
            header += "use crate::off64_t;\n"
        header += (
            "\npub type wchar_t = i32;\n"
            "pub const O_DIRECTORY: c_int = 0x10000;\n"
            "pub const O_NOFOLLOW: c_int = 0x20000;\n"
            "pub const O_DIRECT: c_int = 0x8000;\n"
            "pub const O_LARGEFILE: c_int = 0x2000;\n\n"
        )
        version = tuple(int(part) for part in crate.name.removeprefix("libc-").split("."))
        needs_alias = arch == "mips64" and version < (0, 2, 191)
        if needs_alias:
            header += (
                "pub type __u64 = c_ulong;\n"
                "pub type __s64 = c_long;\n\n"
            )
        if arch == "mips64":
            header += (
                "s! {\n"
                "    pub struct stat {\n"
                "        pub st_dev: c_ulong,\n"
                "        st_pad1: Padding<[c_long; 3]>,\n"
                "        pub st_ino: crate::ino_t,\n"
                "        pub st_mode: crate::mode_t,\n"
                "        pub st_nlink: crate::nlink_t,\n"
                "        pub st_uid: crate::uid_t,\n"
                "        pub st_gid: crate::gid_t,\n"
                "        pub st_rdev: c_ulong,\n"
                "        st_pad2: Padding<[c_long; 1]>,\n"
                "        pub st_size: off64_t,\n"
                "        st_pad3: Padding<c_long>,\n"
                "        pub st_atime: crate::time_t,\n"
                "        pub st_atime_nsec: c_long,\n"
                "        pub st_mtime: crate::time_t,\n"
                "        pub st_mtime_nsec: c_long,\n"
                "        pub st_ctime: crate::time_t,\n"
                "        pub st_ctime_nsec: c_long,\n"
                "        pub st_blksize: crate::blksize_t,\n"
                "        st_pad4: Padding<c_long>,\n"
                "        pub st_blocks: crate::blkcnt_t,\n"
                "        st_pad5: Padding<[c_long; 7]>,\n"
                "    }\n"
                "    pub struct stat64 {\n"
                "        pub st_dev: c_ulong,\n"
                "        st_pad1: Padding<[c_long; 3]>,\n"
                "        pub st_ino: crate::ino64_t,\n"
                "        pub st_mode: crate::mode_t,\n"
                "        pub st_nlink: crate::nlink_t,\n"
                "        pub st_uid: crate::uid_t,\n"
                "        pub st_gid: crate::gid_t,\n"
                "        pub st_rdev: c_ulong,\n"
                "        st_pad2: Padding<[c_long; 2]>,\n"
                "        pub st_size: off64_t,\n"
                "        pub st_atime: crate::time_t,\n"
                "        pub st_atime_nsec: c_long,\n"
                "        pub st_mtime: crate::time_t,\n"
                "        pub st_mtime_nsec: c_long,\n"
                "        pub st_ctime: crate::time_t,\n"
                "        pub st_ctime_nsec: c_long,\n"
                "        pub st_blksize: crate::blksize_t,\n"
                "        st_pad3: Padding<c_long>,\n"
                "        pub st_blocks: crate::blkcnt_t,\n"
                "        st_pad5: Padding<[c_long; 7]>,\n"
                "    }\n"
                "}\n\n"
            )
        function = "pub unsafe fn" if modern else "pub fn"
        body = (
            "f! {\n"
            "    // Android before 5.0 does not expose accept4; call the syscall directly.\n"
            f"    {function} accept4(\n"
            "        fd: c_int,\n"
            "        addr: *mut crate::sockaddr,\n"
            "        len: *mut crate::socklen_t,\n"
            "        flg: c_int,\n"
            "    ) -> c_int {\n"
            "        crate::syscall(SYS_accept4, fd, addr, len, flg) as c_int\n"
            "    }\n"
            "}\n"
        )
        (crate / f"src/unix/linux_like/android/{bits}/{arch}.rs").write_text(
            header + "\n".join(constants) + "\n\n" + body, encoding="utf-8"
        )
        module = crate / f"src/unix/linux_like/android/{bits}/mod.rs"
        text = module.read_text(encoding="utf-8")
        marker = "    } else {\n        // Unknown target_arch\n"
        addition = (
            f'    }} else if #[cfg(target_arch = "{arch}")] {{\n'
            f'        mod {arch};\n'
            f'        pub use self::{arch}::*;\n'
            f'    }} else {{\n'
            f'        // Unknown target_arch\n'
        )
        if marker in text and f'"{arch}"' not in text:
            module.write_text(text.replace(marker, addition, 1), encoding="utf-8")


def patch_std_source():
    output = znn.run_capture(["rustc", "+nightly", "--print", "sysroot"])
    raw = Path(output.strip()) / "lib/rustlib/src/rust/library/std/src/os/android/raw.rs"
    text = raw.read_text(encoding="utf-8")
    if 'target_arch = "mips"' in text:
        return
    text = text.replace(
        '#[cfg(any(target_arch = "arm", target_arch = "x86"))]',
        '#[cfg(any(target_arch = "arm", target_arch = "mips", target_arch = "x86"))]',
    )
    text = text.replace(
        '#[cfg(any(target_arch = "aarch64", target_arch = "riscv64"))]',
        '#[cfg(any(target_arch = "aarch64", target_arch = "mips64", target_arch = "riscv64"))]',
    )
    try:
        raw.write_text(text, encoding="utf-8")
    except OSError as error:
        raise znn.BuildError(
            f"cannot patch {raw} for MIPS: {error}; the rust-src component must be writable"
        ) from error
