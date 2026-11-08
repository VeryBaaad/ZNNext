#!/usr/bin/env python3

import argparse
import os
import shutil
import subprocess
import sys
from pathlib import Path


def parseArgs():
    parser = argparse.ArgumentParser()
    parser.add_argument("--source", type=Path, required=True)
    parser.add_argument("--build", type=Path, required=True)
    parser.add_argument("--deps", type=Path, required=True)
    parser.add_argument("--host", required=True)
    parser.add_argument("--build-machine", required=True)
    parser.add_argument("--ndk", type=Path, required=True)
    parser.add_argument("--cflags", default="")
    parser.add_argument("--ninja", type=Path, required=True)
    parser.add_argument("--strip", type=Path, required=True)
    return parser.parse_args()


def run(argv, cwd=None, env=None):
    print("+ " + " ".join(str(item) for item in argv), flush=True)
    subprocess.run([str(item) for item in argv], cwd=cwd, env=env, check=True)


def lockDirectory(path):
    path.mkdir(parents=True, exist_ok=True)
    handle = open(path / ".znn-frida-gum.lock", "a+")
    try:
        import fcntl

        fcntl.flock(handle.fileno(), fcntl.LOCK_EX)
        unlock = lambda: fcntl.flock(handle.fileno(), fcntl.LOCK_UN)
    except ImportError:
        import msvcrt

        handle.seek(0, os.SEEK_END)
        if handle.tell() == 0:
            handle.write("0")
            handle.flush()
        handle.seek(0)
        msvcrt.locking(handle.fileno(), msvcrt.LK_LOCK, 1)
        unlock = lambda: msvcrt.locking(handle.fileno(), msvcrt.LK_UNLCK, 1)
    return handle, unlock


def syncDependencies(source, deps, host, buildMachine):
    depsScript = source / "releng" / "deps.py"
    run(
        [
            sys.executable,
            depsScript,
            "sync",
            "toolchain",
            buildMachine,
            deps / f"toolchain-{buildMachine}",
        ]
    )
    run(
        [
            sys.executable,
            depsScript,
            "sync",
            "sdk",
            buildMachine,
            deps / f"sdk-{buildMachine}",
        ]
    )
    run(
        [
            sys.executable,
            depsScript,
            "sync",
            "sdk",
            host,
            deps / f"sdk-{host}",
        ]
    )


def prepare_legacy_ndk(ndk, host, workdir):
    prefix = "arm-linux-androideabi" if host == "android-arm" else None
    if prefix is None:
        return ndk
    host_tag = "darwin-x86_64" if sys.platform == "darwin" else (
        "windows-x86_64" if sys.platform == "win32" else "linux-x86_64"
    )
    real_bin = ndk / "toolchains" / "llvm" / "prebuilt" / host_tag / "bin"
    gcc_bin = (
        ndk / "toolchains" / f"{prefix}-4.9" / "prebuilt" / host_tag / "bin"
    )
    shim = workdir / f"legacy-ndk-{host}"
    shim_bin = shim / "toolchains" / "llvm" / "prebuilt" / host_tag / "bin"
    shim_bin.mkdir(parents=True, exist_ok=True)
    (shim / "source.properties").unlink(missing_ok=True)
    (shim / "source.properties").symlink_to(ndk / "source.properties")
    sysroot = shim / "toolchains" / "llvm" / "prebuilt" / host_tag / "sysroot"
    sysroot.unlink(missing_ok=True)
    sysroot.symlink_to(ndk / "toolchains" / "llvm" / "prebuilt" / host_tag / "sysroot")
    tools = {
        "llvm-nm": gcc_bin / f"{prefix}-nm",
        "llvm-strip": gcc_bin / f"{prefix}-strip",
        "llvm-readelf": gcc_bin / f"{prefix}-readelf",
        "llvm-objcopy": gcc_bin / f"{prefix}-objcopy",
    }
    for name in ("clang", "clang++", "llvm-ar", "llvm-as", "llvm-dis", "llvm-link",
                 "llvm-profdata", "llvm-symbolizer"):
        source = real_bin / name
        if source.exists():
            destination = shim_bin / name
            destination.unlink(missing_ok=True)
            destination.symlink_to(source)
    for name, source in tools.items():
        destination = shim_bin / name
        destination.unlink(missing_ok=True)
        destination.write_text(
            f"#!/bin/sh\nexec {source} \"$@\"\n", encoding="utf-8"
        )
        destination.chmod(0o755)
    ranlib = shim_bin / "llvm-ranlib"
    ranlib.unlink(missing_ok=True)
    ranlib.write_text(
        f"#!/bin/sh\nexec {real_bin / 'llvm-ar'} s \"$@\"\n", encoding="utf-8"
    )
    ranlib.chmod(0o755)
    return shim


def ndk_major(ndk):
    properties = ndk / "source.properties"
    for line in properties.read_text(encoding="utf-8", errors="replace").splitlines():
        if line.startswith("Pkg.Revision"):
            return int(line.split("=", 1)[1].strip().split(".")[0])
    return 0


def configure(source, build, deps, host, ndk, cflags):
    environment = os.environ.copy()
    real_ndk = ndk
    legacy = ndk_major(ndk) < 17
    if legacy:
        ndk = prepare_legacy_ndk(ndk, host, build)
    environment["ANDROID_NDK_ROOT"] = str(ndk)
    extra_flags = cflags
    if legacy:
        header_triple = "arm-linux-androideabi" if host == "android-arm" else ""
        host_tag = "darwin-x86_64" if sys.platform == "darwin" else (
            "windows-x86_64" if sys.platform == "win32" else "linux-x86_64"
        )
        include = real_ndk / "sources" / "cxx-stl" / "llvm-libc++" / "include"
        abi_include = real_ndk / "sources" / "cxx-stl" / "llvm-libc++abi" / "include"
        gcc_root = (
            real_ndk / "toolchains" / f"{header_triple}-4.9" / "prebuilt" / host_tag
        )
        extra_flags = " ".join(
            flag for flag in (
                f"--sysroot={real_ndk / 'sysroot'}",
                f"--gcc-toolchain={gcc_root}",
                f"-B{gcc_root / 'bin'}",
                f"-isystem {real_ndk / 'sysroot' / 'usr' / 'include' / header_triple}",
                cflags,
                f"-isystem {include}",
                f"-isystem {abi_include}",
            ) if flag
        )
    if extra_flags:
        environment["CFLAGS"] = extra_flags
        environment["CXXFLAGS"] = extra_flags
        environment["LDFLAGS"] = extra_flags
    environment["FRIDA_DEPS"] = str(deps)
    environment["PKG_CONFIG_PATH"] = str(deps / f"sdk-{host}" / "lib" / "pkgconfig")

    bootstrap = "import sys; sys.path.insert(0, sys.argv[1]); "
    if legacy:
        bootstrap += (
            "from releng import env_android; "
            "env_android._needs_patched_fstream = lambda machine, api: False; "
        )
    bootstrap += "from releng.meson_configure import main; main()"
    argv = [
        sys.executable,
        "-c",
        bootstrap,
        source,
        f"--host={host}",
        "--enable-diet",
        "--disable-graft-tool",
        "--disable-tests",
        "--with-devkits=gum",
        "--without-prebuilds=sdk:host",
    ]
    run(argv, cwd=build, env=environment)


def main():
    args = parseArgs()

    source = args.source.resolve()
    build = args.build.resolve()
    deps = args.deps.resolve()
    ndk = args.ndk.resolve()
    library = build / "gum" / "devkit" / "libfrida-gum.a"

    if not (source / "releng" / "meson" / "meson.py").exists():
        raise SystemExit(
            "frida-gum's releng submodule is not initialized; run "
            "'git submodule update --init --recursive'"
        )
    if not (source / "releng" / "tomlkit").is_dir():
        raise SystemExit(
            "frida-gum's releng/tomlkit submodule is not initialized; run "
            "'git submodule update --init --recursive'"
        )
    if not (ndk / "source.properties").exists():
        raise SystemExit(f"invalid Android NDK: {ndk}")

    handle, unlock = lockDirectory(deps)
    try:
        syncDependencies(source, deps, args.host, args.build_machine)
    finally:
        unlock()
        handle.close()

    shutil.rmtree(build, ignore_errors=True)
    build.mkdir(parents=True, exist_ok=True)

    configure(source, build, deps, args.host, ndk, args.cflags)
    run([args.ninja], cwd=build)
    if not library.exists():
        raise SystemExit(f"frida-gum build did not produce {library}")

    try:
        run([args.strip, "--strip-debug", library])
    except subprocess.CalledProcessError:
        library.unlink(missing_ok=True)
        raise


if __name__ == "__main__":
    main()
