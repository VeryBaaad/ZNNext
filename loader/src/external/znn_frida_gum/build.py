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


def configure(source, build, deps, host, ndk):
    environment = os.environ.copy()
    environment["ANDROID_NDK_ROOT"] = str(ndk)
    environment["FRIDA_DEPS"] = str(deps)
    environment["PKG_CONFIG_PATH"] = str(deps / f"sdk-{host}" / "lib" / "pkgconfig")

    bootstrap = (
        "import sys; sys.path.insert(0, sys.argv[1]); "
        "from releng.meson_configure import main; main()"
    )
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

    configure(source, build, deps, args.host, ndk)
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
