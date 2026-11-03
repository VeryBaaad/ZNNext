#!/usr/bin/env python3
import argparse
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT))
import build as znn

KPM_DIR = ROOT / "kpm"
MANIFEST = KPM_DIR / "version.properties"

COMPILER_FLAGS = [
    "-O2",
    "-ffreestanding",
    "-nostdlib",
    "-fno-stack-protector",
    "-fno-pic",
    "-fno-builtin",
    "-fno-asynchronous-unwind-tables",
    "-fno-unwind-tables",
    "-fvisibility=hidden",
    "-mno-outline-atomics",
]
INCLUDE_DIRECTORIES = [
    "external/kpatch/include",
    "external/kpatch/patch/include",
    "external/kpatch/linux/include",
    "external/kpatch/linux/arch/arm64/include",
]


def sync_version():
    contents = MANIFEST.read_text(encoding="utf-8")
    updated = re.sub(
        r"(?m)^version\s*=.*$",
        f"version={znn.KPM_VERSION}",
        contents,
        count=1,
    )
    if updated != contents:
        MANIFEST.write_text(updated, encoding="utf-8")


def manifest_version():
    for line in MANIFEST.read_text(encoding="utf-8").splitlines():
        if line.strip().startswith("version"):
            return line.split("=", 1)[1].strip()
    return ""


def toolchain(ndk):
    binaries = ndk / "toolchains" / "llvm" / "prebuilt" / znn.host_tag() / "bin"
    if not binaries.is_dir():
        raise znn.BuildError(f"NDK toolchain directory not found: {binaries}")
    candidates = sorted(
        (
            path
            for path in binaries.glob("aarch64-linux-android*-clang")
            if path.is_file()
            and path.name.removeprefix("aarch64-linux-android")
            .removesuffix("-clang")
            .isdigit()
        ),
        key=lambda path: path.name,
    )
    if not candidates:
        raise znn.BuildError(f"no aarch64-linux-android<api>-clang in {binaries}")
    clang = next(
        (
            path
            for path in candidates
            if path.name == "aarch64-linux-android26-clang"
        ),
        candidates[0],
    )
    linker = binaries / "ld.lld"
    if not linker.is_file():
        linker = binaries / "aarch64-linux-android-ld"
    if not linker.is_file():
        raise znn.BuildError(f"no aarch64 linker in {binaries}")
    return clang, linker


def compile_kpm(ndk, output_dir, btf):
    clang, linker = toolchain(ndk)
    output_dir.mkdir(parents=True, exist_ok=True)
    source = KPM_DIR / "src" / "znn.c"
    object_file = output_dir / "znn.o"
    artifact = output_dir / "znn.kpm"
    version = manifest_version()
    if not version:
        raise znn.BuildError(f"{MANIFEST} has no version= entry")

    missing = [entry for entry in INCLUDE_DIRECTORIES if not (KPM_DIR / entry).is_dir()]
    if missing:
        raise znn.BuildError(
            "KPM headers are missing; initialize submodules: "
            + ", ".join(missing)
        )

    command = [
        clang,
        *COMPILER_FLAGS,
        f'-DZNN_KPM_VERSION="{version}"',
    ]
    if btf:
        command.append("-DZNN_WITH_BTF=1")
    for entry in INCLUDE_DIRECTORIES:
        command.append(f"-I{(KPM_DIR / entry).resolve()}")
    command += ["-c", source, "-o", object_file]
    znn.run(command, cwd=KPM_DIR, prefix="kpm")

    if not object_file.is_file():
        raise znn.BuildError(f"KPM compile produced no object at {object_file}")

    znn.run(
        [linker, "-r", object_file, "-o", artifact],
        cwd=KPM_DIR,
        prefix="kpm",
    )
    if not artifact.is_file():
        raise znn.BuildError(f"KPM link produced no artifact at {artifact}")
    print(
        f"kpm: {artifact} ({artifact.stat().st_size} bytes, {clang.name}, "
        f"NDK {ndk.name})",
        flush=True,
    )


def parse_arguments(argv):
    parser = argparse.ArgumentParser(prog="kpm/build.py")
    parser.add_argument("variant", choices=znn.VARIANTS)
    parser.add_argument("--ndk", default=None)
    parser.add_argument("--no-kpm", action="store_true")
    parser.add_argument("--btf", action="store_true")
    return parser.parse_args(argv)


def main(argv=None):
    options = parse_arguments(argv)
    sync_version()
    if options.no_kpm:
        print(f"kpm ({options.variant}): disabled", flush=True)
        return 0
    ndk = znn.ndk_dir(options.ndk)
    compile_kpm(ndk, znn.kpm_output(options.variant), options.btf)
    return 0


if __name__ == "__main__":
    try:
        sys.exit(main())
    except znn.BuildError as error:
        print(f"error: {error}", file=sys.stderr)
        sys.exit(1)
