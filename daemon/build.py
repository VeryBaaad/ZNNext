#!/usr/bin/env python3
import argparse
import os
import re
import shutil
import sys
from concurrent.futures import ThreadPoolExecutor, as_completed
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT))
import build as znn

DAEMON_DIR = ROOT / "daemon"


def sync_cargo_version():
    manifest = DAEMON_DIR / "Cargo.toml"
    contents = manifest.read_text(encoding="utf-8")
    updated = re.sub(
        r'(?m)^version\s*=\s*"[^"]*"',
        f'version = "{znn.KPM_VERSION}"',
        contents,
        count=1,
    )
    if updated != contents:
        manifest.write_text(updated, encoding="utf-8")


def riscv64_linker(ndk):
    binaries = ndk / "toolchains" / "llvm" / "prebuilt" / znn.host_tag() / "bin"
    candidates = sorted(
        (
            path
            for path in binaries.glob("riscv64-linux-android*-clang")
            if path.is_file()
            and path.name.removeprefix("riscv64-linux-android")
            .removesuffix("-clang")
            .isdigit()
        ),
        key=lambda path: path.name,
    )
    if not candidates:
        raise znn.BuildError(
            f"the NDK has no riscv64 clang in {binaries}; NDK r27 or newer is required"
        )
    return candidates[-1]


def build_abi(abi, variant, jobs, ndk):
    target = znn.RUST_TARGETS[abi]
    profile = "debug" if variant == "debug" else "release"
    target_dir = DAEMON_DIR / "target" / abi
    target_dir.mkdir(parents=True, exist_ok=True)

    environment = os.environ.copy()
    environment["ZNN_VERSION"] = znn.reported_version(variant)
    environment["KPM_VERSION"] = znn.KPM_VERSION
    environment["CARGO_TARGET_DIR"] = str(target_dir)
    environment["CARGO_BUILD_JOBS"] = str(jobs)
    environment["ANDROID_NDK_HOME"] = str(ndk)
    environment["ANDROID_NDK_ROOT"] = str(ndk)

    if abi == "riscv64":
        linker = riscv64_linker(ndk)
        environment["CARGO_TARGET_RISCV64_LINUX_ANDROID_LINKER"] = str(linker)
        environment["CC_riscv64_linux_android"] = str(linker)
        environment["AR_riscv64_linux_android"] = str(linker.parent / "llvm-ar")
        command = [
            "cargo",
            "+nightly",
            "build",
            "-Z",
            "build-std=std,panic_abort",
            "--target",
            target,
        ]
    else:
        command = [
            "cargo",
            "ndk",
            "-t",
            abi,
            "-P",
            str(znn.ANDROID_MIN_SDK),
            "build",
        ]
    if variant == "release":
        command.append("--release")

    znn.run(command, cwd=DAEMON_DIR, env=environment, prefix=f"daemon {abi}")

    produced = target_dir / target / profile / "znnd"
    if not produced.is_file():
        raise znn.BuildError(f"cargo produced no binary at {produced}")
    destination = znn.daemon_output(variant) / abi / "znnd"
    destination.parent.mkdir(parents=True, exist_ok=True)
    shutil.copy2(produced, destination)
    print(
        f"daemon: built {abi} ({destination.stat().st_size} bytes)",
        flush=True,
    )


def parse_arguments(argv):
    parser = argparse.ArgumentParser(prog="daemon/build.py")
    parser.add_argument("variant", choices=znn.VARIANTS)
    parser.add_argument("--jobs", type=int, default=znn.cpu_count())
    parser.add_argument("--abi", action="append", choices=znn.ANDROID_ABIS)
    parser.add_argument("--ndk", default=None)
    return parser.parse_args(argv)


def main(argv=None):
    options = parse_arguments(argv)
    jobs = max(1, options.jobs)
    abis = options.abi or list(znn.ANDROID_ABIS)
    ndk = znn.ndk_dir(options.ndk)
    per_abi = max(1, jobs // len(abis))

    sync_cargo_version()
    print(
        f"daemon: {options.variant}, {len(abis)} ABI(s), {per_abi} job(s) each, "
        f"NDK {ndk.name}",
        flush=True,
    )

    errors = []
    with ThreadPoolExecutor(max_workers=len(abis)) as pool:
        futures = {
            pool.submit(build_abi, abi, options.variant, per_abi, ndk): abi
            for abi in abis
        }
        for future in as_completed(futures):
            abi = futures[future]
            try:
                future.result()
            except Exception as error:
                errors.append((abi, error))
                print(f"[daemon {abi}] failed: {error}", file=sys.stderr, flush=True)
    if errors:
        abi, error = errors[0]
        raise znn.BuildError(f"daemon {abi} failed: {error}")
    return 0


if __name__ == "__main__":
    try:
        sys.exit(main())
    except znn.BuildError as error:
        print(f"error: {error}", file=sys.stderr)
        sys.exit(1)
