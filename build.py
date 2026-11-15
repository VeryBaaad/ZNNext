#!/usr/bin/env python3
import argparse
import dataclasses
import os
import re
import shutil
import subprocess
import sys
import tempfile
from concurrent.futures import ThreadPoolExecutor, as_completed
from functools import lru_cache
from pathlib import Path

import optimize

ROOT = Path(__file__).resolve().parent
LOADER_DIR = ROOT / "loader"
DAEMON_DIR = ROOT / "daemon"
KPM_DIR = ROOT / "kpm"
MODULE_DIR = ROOT / "module"
WEBUI_DIR = ROOT / "webui"

BUILD_SCRIPT = ROOT / "build.py"
KPM_MANIFEST = KPM_DIR / "version.properties"
DAEMON_MANIFEST = DAEMON_DIR / "Cargo.toml"
DAEMON_LOCK = DAEMON_DIR / "Cargo.lock"
VERSION_PATTERN = re.compile(r"^[vV]?(\d+)\.(\d+)\.(\d+)(?:\.(\d+))?$")

MODULE_ID = "zygisknextsu"
MODULE_NAME = "ZNNext"
VER_NAME = "v2.18.0"
UPDATE_URL = "https://raw.githubusercontent.com/VeryBaaad/ZNNext/refs/heads/update/update.json"
KPM_ID = "znnext"
KPM_VERSION = VER_NAME.removeprefix("v")

MIN_KSU_VERSION = 10940
MIN_KSUD_VERSION = 11425
MIN_MAGISK_VERSION = 26402
MIN_APATCH_VERSION = 10700

ANDROID_MIN_SDK = 21
ANDROID_COMPILE_SDK = 37
ANDROID_COMPILE_SDK_MINOR = 2
ANDROID_BUILD_TOOLS_VERSION = "37.0.0"
ANDROID_NDK_VERSION = "30.0.16248370"
LEGACY_NDK_VERSION = "16.1.4479499"

LIBCXX_VERSION = ANDROID_NDK_VERSION
LIBCXX_SHA256 = "8bb6839964cdd5b814255c2674a9bb4a7ba5a6b2e15284c307694a571e8d5f7c"
LIBCXX_URL = (
    "https://repo1.maven.org/maven2/org/lsposed/libcxx/libcxx/"
    f"{LIBCXX_VERSION}/libcxx-{LIBCXX_VERSION}.aar"
)

ANDROID_ABIS = (
    "arm64-v8a",
    "armeabi-v7a",
    "armeabi",
    "x86",
    "x86_64",
    "riscv64",
    "mips",
    "mips64",
)
LEGACY_ANDROID_ABIS = ("armeabi", "mips", "mips64")
ANDROID_TRIPLES = {
    "arm64-v8a": "aarch64-linux-android",
    "armeabi-v7a": "arm-linux-androideabi",
    "armeabi": "armv5te-linux-androideabi",
    "x86": "i686-linux-android",
    "x86_64": "x86_64-linux-android",
    "riscv64": "riscv64-linux-android",
    "mips": "mipsel-linux-android",
    "mips64": "mips64el-linux-android",
}
RUST_TARGETS = {
    "arm64-v8a": "aarch64-linux-android",
    "armeabi-v7a": "armv7-linux-androideabi",
    "armeabi": "arm-linux-androideabi",
    "x86": "i686-linux-android",
    "x86_64": "x86_64-linux-android",
    "riscv64": "riscv64-linux-android",
    "mips": "mipsel-linux-android",
    "mips64": "mips64el-linux-android",
}
VARIANTS = ("debug", "release")

DEFAULT_CFLAGS = [
    "-Wall",
    "-Wextra",
    "-fno-rtti",
    "-fno-exceptions",
    "-fno-stack-protector",
    "-fomit-frame-pointer",
    "-Wno-builtin-macro-redefined",
    "-D__FILE__=__FILE_NAME__",
]
OPTIMIZED_CFLAGS = [
    "-flto",
    "-Wno-unused",
    "-Wno-unused-parameter",
    "-fvisibility=hidden",
    "-fvisibility-inlines-hidden",
    "-fno-unwind-tables",
    "-fno-asynchronous-unwind-tables",
]
DEBUG_CFLAGS = ["-Os", *OPTIMIZED_CFLAGS]
RELEASE_CFLAGS = ["-Oz", *OPTIMIZED_CFLAGS]
COMMON_LINKER_FLAGS = "-Wl,--exclude-libs,ALL -Wl,--gc-sections -Wl,--icf=safe"
DEBUG_LINKER_FLAGS = f"-Os {COMMON_LINKER_FLAGS}"
RELEASE_LINKER_FLAGS = f"-Oz {COMMON_LINKER_FLAGS} -Wl,--strip-all"

XZ_MIN_ENTRY_SIZE = 1024


class BuildError(RuntimeError):
    pass


@lru_cache(maxsize=None)
def git(*args):
    try:
        result = subprocess.run(
            ["git", *args],
            cwd=ROOT,
            stdout=subprocess.PIPE,
            stderr=subprocess.DEVNULL,
            text=True,
            check=False,
        )
    except OSError:
        return ""
    if result.returncode != 0:
        return ""
    return result.stdout.strip()


@lru_cache(maxsize=1)
def version_code():
    value = git("rev-list", "HEAD", "--count")
    return int(value) if value.isdigit() else 1


@lru_cache(maxsize=1)
def commit_hash():
    return git("rev-parse", "--verify", "--short", "HEAD") or "unknown"


def reported_version(variant):
    return f"{VER_NAME}-{version_code()}-{commit_hash()}-{variant}"


def normalize_version(value):
    match = VERSION_PATTERN.fullmatch(value.strip())
    if match is None:
        raise BuildError(
            f"invalid version: {value} "
            "(expected v[major].[minor].[patch][.revision], e.g. v0.0.0.0)"
        )
    major, minor, patch, revision = match.groups()
    version = f"v{major}.{minor}.{patch}"
    return version if revision is None else f"{version}.{revision}"


def cargo_version(version_name=None):
    name = VER_NAME if version_name is None else version_name
    parts = name.removeprefix("v").split(".")
    if len(parts) not in (3, 4) or not all(part.isdigit() for part in parts):
        raise BuildError(
            f"{name} does not follow v[major].[minor].[patch][.revision]"
        )
    if len(parts) == 4:
        return ".".join(parts[:3]) + f"+{parts[3]}"
    return ".".join(parts)


def render_version_field(path, pattern, replacement):
    contents = path.read_text(encoding="utf-8")
    updated, count = re.subn(pattern, lambda _: replacement, contents, count=1)
    if count != 1:
        raise BuildError(
            f"could not find the version field in {path.relative_to(ROOT)}"
        )
    return contents, updated


def sync_version(value):
    global VER_NAME, KPM_VERSION
    version = normalize_version(value)
    kpm = version.removeprefix("v")
    cargo = cargo_version(version)

    updates = [
        (BUILD_SCRIPT, r'(?m)^VER_NAME\s*=\s*"[^"]*"\s*$', f'VER_NAME = "{version}"'),
        (KPM_MANIFEST, r"(?m)^\s*version\s*=.*$", f"version={kpm}"),
        (DAEMON_MANIFEST, r'(?m)^version\s*=\s*"[^"]*"\s*$', f'version = "{cargo}"'),
    ]
    if DAEMON_LOCK.is_file():
        updates.append(
            (
                DAEMON_LOCK,
                r'(?m)^\[\[package\]\]\nname = "daemon"\nversion = "[^"]*"',
                f'[[package]]\nname = "daemon"\nversion = "{cargo}"',
            )
        )

    rendered = []
    for path, pattern, replacement in updates:
        contents, updated = render_version_field(path, pattern, replacement)
        rendered.append((path, contents, updated))

    for path, contents, updated in rendered:
        if updated != contents:
            path.write_text(updated, encoding="utf-8")
            print(f"synced {path.relative_to(ROOT)}", flush=True)

    VER_NAME = version
    KPM_VERSION = kpm
    print(f"version: {version} (kpm {kpm}, cargo {cargo})", flush=True)


def kpm_manifest_version():
    for line in KPM_MANIFEST.read_text(encoding="utf-8").splitlines():
        match = re.match(r"^\s*version\s*=\s*(.*)$", line)
        if match:
            return match.group(1).strip()
    return ""


def daemon_manifest_version():
    match = re.search(
        r'(?m)^version\s*=\s*"([^"]*)"',
        DAEMON_MANIFEST.read_text(encoding="utf-8"),
    )
    return match.group(1) if match else ""


def daemon_lock_version():
    if not DAEMON_LOCK.is_file():
        return None
    match = re.search(
        r'(?m)^\[\[package\]\]\nname = "daemon"\nversion = "([^"]*)"',
        DAEMON_LOCK.read_text(encoding="utf-8"),
    )
    return match.group(1) if match else ""


def verify_versions(check_kpm):
    expected = cargo_version()
    problems = []
    if check_kpm and kpm_manifest_version() != KPM_VERSION:
        problems.append("kpm/version.properties")
    if daemon_manifest_version() != expected:
        problems.append("daemon/Cargo.toml")
    lock_version = daemon_lock_version()
    if lock_version is not None and lock_version != expected:
        problems.append("daemon/Cargo.lock")
    if problems:
        raise BuildError(
            f"version files are out of sync with {VER_NAME}: "
            f"{', '.join(problems)}; run ./build.py syncVersion {VER_NAME}"
        )


def variant_cflags(variant, legacy=False):
    flags = DEBUG_CFLAGS if variant == "debug" else RELEASE_CFLAGS
    if legacy:
        flags = [flag for flag in flags if flag != "-flto"]
    return flags


def variant_linker_flags(variant, legacy=False):
    if legacy:
        optimization = "-Os" if variant == "debug" else "-Oz"
        flags = (
            f"{optimization} -Wl,--exclude-libs,ALL -Wl,--gc-sections"
        )
        if variant != "debug":
            flags += " -Wl,--strip-all"
        return flags
    return DEBUG_LINKER_FLAGS if variant == "debug" else RELEASE_LINKER_FLAGS


def variant_cmake_optimization(variant):
    if variant == "debug":
        return "Debug", "-Os -g"
    return "RelWithDebInfo", "-Oz -g -DNDEBUG"


def loader_output(variant):
    return LOADER_DIR / "build" / variant


def daemon_output(variant):
    return DAEMON_DIR / "build" / "rust" / variant


def kpm_output(variant):
    return KPM_DIR / "build" / "kpm" / variant


def zip_name(variant):
    return (
        f"{MODULE_NAME}-{VER_NAME}-{version_code()}-{commit_hash()}-{variant}.zip"
        .replace(" ", "-")
    )


def zip_path(variant):
    return MODULE_DIR / "build" / "outputs" / "release" / zip_name(variant)


def cpu_count():
    try:
        return len(os.sched_getaffinity(0))
    except AttributeError:
        return os.cpu_count() or 1


def host_tag():
    if sys.platform.startswith("linux"):
        return "linux-x86_64"
    if sys.platform == "darwin":
        return "darwin-x86_64"
    return "windows-x86_64"


def local_sdk():
    properties = ROOT / "local.properties"
    if not properties.is_file():
        return None
    for line in properties.read_text(encoding="utf-8", errors="replace").splitlines():
        if line.startswith("sdk.dir="):
            value = line.split("=", 1)[1].strip()
            if value:
                return Path(value)
    return None


def android_sdk():
    for variable in ("ANDROID_HOME", "ANDROID_SDK_ROOT"):
        value = os.environ.get(variable)
        if not value:
            continue
        path = Path(value)
        if path.is_dir():
            return path
    configured = local_sdk()
    if configured is not None and configured.is_dir():
        return configured
    raise BuildError(
        "ANDROID_HOME or ANDROID_SDK_ROOT must point to the Android SDK"
    )


def ndk_revision(path):
    properties = path / "source.properties"
    if not properties.is_file():
        return None
    for line in properties.read_text(encoding="utf-8", errors="replace").splitlines():
        if line.startswith("Pkg.Revision"):
            return line.split("=", 1)[1].strip()
    return None


def ndk_usable(path, version=ANDROID_NDK_VERSION):
    binaries = path / "toolchains" / "llvm" / "prebuilt" / host_tag() / "bin"
    suffix = ".exe" if sys.platform == "win32" else ""
    if not (binaries / f"clang{suffix}").is_file():
        return False
    if version == LEGACY_NDK_VERSION:
        return True
    return (binaries / f"ld.lld{suffix}").is_file()


@lru_cache(maxsize=None)
def ndk_dir(override=None, version=ANDROID_NDK_VERSION):
    if override:
        path = Path(override).expanduser()
        if not path.is_dir():
            raise BuildError(f"Android NDK directory not found: {path}")
        if not ndk_usable(path, version):
            raise BuildError(
                f"Android NDK is incomplete (clang/ld.lld missing): {path}"
            )
        found = ndk_revision(path)
        if found != version:
            raise BuildError(
                f"Android NDK {version} is required, found {found or 'unknown'} at {path}"
            )
        return path.resolve()

    candidates = []
    for variable in ("ANDROID_NDK_HOME", "ANDROID_NDK_ROOT", "ANDROID_NDK"):
        value = os.environ.get(variable)
        if value:
            candidates.append(Path(value))
    configured = local_sdk()
    if configured is not None:
        candidates.append(configured / "ndk" / version)
    try:
        candidates.append(android_sdk() / "ndk" / version)
    except BuildError:
        pass

    existing = [path for path in candidates if path.is_dir()]
    usable = [path for path in existing if ndk_usable(path, version)]
    if not usable:
        raise BuildError(
            "no usable Android NDK found; set ANDROID_NDK_HOME to "
            f"NDK {version} or install it under <sdk>/ndk/"
            + (
                " (checked: "
                + ", ".join(str(path) for path in existing)
                + ")"
                if existing
                else ""
            )
        )
    for path in usable:
        if ndk_revision(path) == version:
            return path.resolve()
    raise BuildError(
        f"Android NDK {version} is required, found "
        + ", ".join(f"{path} ({ndk_revision(path) or 'unknown'})" for path in usable)
    )


def ccache_path(explicit=None):
    if explicit is not None:
        value = explicit.strip()
        if value.lower() in ("", "none", "off", "false", "no"):
            return None
        path = Path(value).expanduser()
        if path.is_file():
            return str(path.resolve())
        found = shutil.which(value)
        if found:
            return found
        raise BuildError(f"ccache not found: {value}")
    value = os.environ.get("CCACHE")
    if value and Path(value).is_file():
        return value
    return shutil.which("ccache")


def seven_zip_probe(candidate):
    try:
        with tempfile.TemporaryDirectory(prefix="znn-7z-") as directory:
            root = Path(directory)
            (root / "probe.txt").write_bytes(b"z" * 4096)
            added = subprocess.run(
                [candidate, "a", "-tzip", "-mm=XZ", "probe.zip", "probe.txt"],
                cwd=root,
                stdout=subprocess.DEVNULL,
                stderr=subprocess.DEVNULL,
                check=False,
            )
            if added.returncode != 0:
                return False
            listing = subprocess.run(
                [candidate, "l", "-slt", "probe.zip"],
                cwd=root,
                stdout=subprocess.PIPE,
                stderr=subprocess.DEVNULL,
                text=True,
                check=False,
            )
            return listing.returncode == 0 and "Method = xz" in listing.stdout
    except OSError:
        return False


@lru_cache(maxsize=None)
def seven_zip(explicit=None):
    candidates = []
    if explicit:
        candidates.append(explicit)
    value = os.environ.get("SEVENZIP")
    if value:
        candidates.append(value)
    candidates.extend(["7zz", "7z", "7za"])
    checked = []
    for candidate in candidates:
        if candidate in checked:
            continue
        checked.append(candidate)
        if seven_zip_probe(candidate):
            return candidate
    raise BuildError(
        "no 7-Zip with XZ support found; pass --sevenzip or set SEVENZIP "
        f"(tried: {', '.join(checked)})"
    )


def run(cmd, cwd=None, env=None, prefix=None):
    command = [str(part) for part in cmd]
    try:
        process = subprocess.Popen(
            command,
            cwd=str(cwd) if cwd else None,
            env=env,
            stdout=subprocess.PIPE,
            stderr=subprocess.STDOUT,
            text=True,
            errors="replace",
            bufsize=1,
        )
    except FileNotFoundError as error:
        raise BuildError(f"executable not found: {command[0]}") from error
    lines = []
    assert process.stdout is not None
    for line in process.stdout:
        line = line.rstrip("\n")
        lines.append(line)
        if prefix:
            print(f"[{prefix}] {line}", flush=True)
        else:
            print(line, flush=True)
    code = process.wait()
    if code != 0:
        tail = "\n".join(lines[-80:])
        raise BuildError(f"command failed ({code}): {' '.join(command)}\n{tail}")
    return code


def run_capture(cmd, cwd=None, env=None):
    command = [str(part) for part in cmd]
    try:
        result = subprocess.run(
            command,
            cwd=str(cwd) if cwd else None,
            env=env,
            stdout=subprocess.PIPE,
            stderr=subprocess.STDOUT,
            text=True,
            errors="replace",
            check=False,
        )
    except FileNotFoundError as error:
        raise BuildError(f"executable not found: {command[0]}") from error
    if result.returncode != 0:
        tail = "\n".join(result.stdout.splitlines()[-80:])
        raise BuildError(f"command failed ({result.returncode}): {' '.join(command)}\n{tail}")
    return result.stdout


def script(module, *args):
    return [sys.executable, str(ROOT / module / "build.py"), *[str(arg) for arg in args]]


def run_parallel(tasks):
    if not tasks:
        return
    errors = []
    with ThreadPoolExecutor(max_workers=len(tasks)) as pool:
        futures = {pool.submit(task): name for name, task in tasks}
        for future in as_completed(futures):
            name = futures[future]
            try:
                future.result()
            except Exception as error:
                errors.append((name, error))
                print(f"[{name}] failed: {error}", file=sys.stderr, flush=True)
    if errors:
        name, error = errors[0]
        raise BuildError(f"{name} failed: {error}")


def selected_abis(options):
    if not options.abis:
        return list(ANDROID_ABIS)
    selected = [abi.strip() for abi in options.abis.split(",") if abi.strip()]
    unknown = [abi for abi in selected if abi not in ANDROID_ABIS]
    if unknown:
        raise BuildError(
            f"unknown ABI(s): {', '.join(unknown)}; "
            f"choose from {', '.join(ANDROID_ABIS)}"
        )
    return [abi for abi in ANDROID_ABIS if abi in selected]


def build_artifacts(variant, options, state):
    verify_versions(not options.no_kpm)
    env = state["env"]
    jobs = options.jobs
    loader_jobs = max(1, jobs * 55 // 100)
    daemon_jobs = max(1, jobs - loader_jobs)
    abis = selected_abis(options)

    loader_cmd = script("loader", variant, "--jobs", loader_jobs)
    for abi in abis:
        loader_cmd += ["--abi", abi]
    if options.ccache is not None:
        loader_cmd += ["--ccache", options.ccache]
    if options.ndk:
        loader_cmd += ["--ndk", options.ndk]
    if options.ndk_legacy:
        loader_cmd += ["--ndk-legacy", options.ndk_legacy]
    if options.libcxx_aar:
        loader_cmd += ["--libcxx-aar", options.libcxx_aar]

    daemon_cmd = script("daemon", variant, "--jobs", daemon_jobs)
    for abi in abis:
        daemon_cmd += ["--abi", abi]
    if options.ndk:
        daemon_cmd += ["--ndk", options.ndk]
    if options.ndk_legacy:
        daemon_cmd += ["--ndk-legacy", options.ndk_legacy]

    kpm_cmd = script("kpm", variant)
    if options.no_kpm:
        kpm_cmd.append("--no-kpm")
    if options.kpm_btf:
        kpm_cmd.append("--btf")
    if options.ndk:
        kpm_cmd += ["--ndk", options.ndk]

    tasks = [
        ("loader", lambda: run(loader_cmd, env=env)),
        ("daemon", lambda: run(daemon_cmd, env=env)),
        ("kpm", lambda: run(kpm_cmd, env=env)),
    ]
    build_webui = not options.skip_webui and not state["webui"]
    if build_webui:
        tasks.append(
            ("webui", lambda: run(script("webui"), env=env))
        )
    run_parallel(tasks)
    if build_webui:
        state["webui"] = True


def package_variant(variant, options, state):
    command = script("module", "package", variant)
    if options.no_kpm:
        command.append("--no-kpm")
    if options.sevenzip:
        command += ["--sevenzip", options.sevenzip]
    run(command, env=state["env"], prefix="module")


def adb_command(action, variant, options, state):
    run(script("module", action, variant), env=state["env"])


def clean():
    targets = [
        ROOT / "build",
        LOADER_DIR / "build",
        LOADER_DIR / ".cxx",
        DAEMON_DIR / "build",
        DAEMON_DIR / "target",
        KPM_DIR / "build",
        MODULE_DIR / "build",
        MODULE_DIR / "webroot",
        MODULE_DIR / "sign" / "target",
        WEBUI_DIR / ".npm-cache",
    ]
    for target in targets:
        if target.exists():
            shutil.rmtree(target)
            print(f"removed {target.relative_to(ROOT)}")


TASK_MAP = {
    "ziprelease": ("zip", "release"),
    "zipdebug": ("zip", "debug"),
    "assemblerelease": ("assemble", "release"),
    "assembledebug": ("assemble", "debug"),
    "pushrelease": ("push", "release"),
    "pushdebug": ("push", "debug"),
    "installksurelease": ("install-ksu", "release"),
    "installksudebug": ("install-ksu", "debug"),
    "installmagiskrelease": ("install-magisk", "release"),
    "installmagiskdebug": ("install-magisk", "debug"),
    "installksuandrebootrelease": ("install-ksu-reboot", "release"),
    "installksuandrebootdebug": ("install-ksu-reboot", "debug"),
    "installmagiskandrebootrelease": ("install-magisk-reboot", "release"),
    "installmagiskandrebootdebug": ("install-magisk-reboot", "debug"),
    "clean": ("clean", None),
    "delete": ("clean", None),
    "webui": ("webui", None),
    "buildwebui": ("webui", None),
    "syncversion": ("sync-version", None),
}


def parse_task(name):
    key = name.lower()
    if key not in TASK_MAP:
        raise BuildError(
            f"unknown task: {name} (try zipRelease, zipDebug, assembleRelease, "
            "clean, syncVersion, pushRelease, installKsuRelease, installMagiskRelease)"
        )
    return TASK_MAP[key]


def parse_tasks(names):
    parsed = []
    pending = list(names)
    while pending:
        name = pending.pop(0)
        kind, variant = parse_task(name)
        value = None
        if kind == "sync-version":
            if not pending:
                raise BuildError(
                    "syncVersion requires a version, "
                    "e.g. ./build.py syncVersion v0.0.0.0"
                )
            value = pending.pop(0)
        parsed.append((name, kind, variant, value))
    return parsed


OPTIMIZATION_OVERRIDES = (
    "codegen_level",
    "lto",
    "icf",
    "strip",
    "unwind",
    "eh_frame_hdr",
    "rosegment",
    "hash_style",
    "build_id",
    "merge_constants",
    "ident",
    "section_gc",
    "lto_codegen",
    "linker_opt",
    "rust_opt",
    "rust_lto",
    "rust_codegen_units",
    "rust_panic",
    "rust_panic_immediate_abort",
    "rust_location_detail",
    "rust_build_std",
    "rust_debuginfo",
    "debug_symbols",
    "xz_level",
    "xz_threshold",
    "webui_minify",
    "webui_drop_icon_font",
    "extra_cflags",
    "extra_ldflags",
    "extra_rustflags",
)


def add_optimization_arguments(parser):
    group = parser.add_argument_group(
        "optimization",
        "size/perf profiles plus fine-grained overrides; "
        "every override wins over the profile default",
    )
    group.add_argument(
        "--opt",
        choices=optimize.PROFILES,
        default=optimize.DEFAULT_PROFILE,
        help=(
            "size: smallest output, performance second (default); "
            "perf: fastest output, size second; "
            "max: smaller than size, drops crash lookup tables; "
            "none: pre-optimization behavior"
        ),
    )
    group.add_argument("--opt-level", dest="codegen_level", choices=["z", "s", "1", "2", "3"])
    group.add_argument("--lto", choices=["off", "thin", "full"])
    group.add_argument("--icf", choices=["none", "safe", "all"])
    group.add_argument("--strip", choices=["none", "debug", "all"])
    group.add_argument("--unwind", choices=["full", "compact"])
    group.add_argument("--eh-frame-hdr", dest="eh_frame_hdr", action="store_true", default=None)
    group.add_argument("--no-eh-frame-hdr", dest="eh_frame_hdr", action="store_false", default=None)
    group.add_argument("--rosegment", dest="rosegment", action="store_true", default=None)
    group.add_argument("--no-rosegment", dest="rosegment", action="store_false", default=None)
    group.add_argument("--hash-style", choices=["gnu", "both"])
    group.add_argument("--build-id", dest="build_id", action="store_true", default=None)
    group.add_argument("--no-build-id", dest="build_id", action="store_false", default=None)
    group.add_argument("--merge-constants", dest="merge_constants", action="store_true", default=None)
    group.add_argument(
        "--no-merge-constants", dest="merge_constants", action="store_false", default=None
    )
    group.add_argument("--ident", dest="ident", action="store_true", default=None)
    group.add_argument("--no-ident", dest="ident", action="store_false", default=None)
    group.add_argument("--gc-sections", dest="section_gc", action="store_true", default=None)
    group.add_argument("--no-gc-sections", dest="section_gc", action="store_false", default=None)
    group.add_argument("--lto-codegen", type=int, choices=range(0, 4))
    group.add_argument("--linker-opt", type=int, choices=range(0, 3))
    group.add_argument("--rust-opt", choices=["z", "s", "1", "2", "3"])
    group.add_argument("--rust-lto", choices=["off", "thin", "fat"])
    group.add_argument("--rust-codegen-units", type=int)
    group.add_argument("--rust-panic", choices=["abort", "unwind"])
    group.add_argument(
        "--rust-panic-immediate-abort",
        dest="rust_panic_immediate_abort",
        action="store_true",
        default=None,
    )
    group.add_argument(
        "--no-rust-panic-immediate-abort",
        dest="rust_panic_immediate_abort",
        action="store_false",
        default=None,
    )
    group.add_argument("--rust-location-detail", choices=["none", "full"])
    group.add_argument("--rust-build-std", dest="rust_build_std", action="store_true", default=None)
    group.add_argument(
        "--no-rust-build-std", dest="rust_build_std", action="store_false", default=None
    )
    group.add_argument("--rust-debuginfo", choices=["none", "line-tables"])
    group.add_argument("--debug-symbols", action="store_true", default=None)
    group.add_argument("--xz-level", type=int, choices=range(0, 10))
    group.add_argument("--xz-threshold", type=int)
    group.add_argument("--webui-minify", dest="webui_minify", action="store_true", default=None)
    group.add_argument(
        "--no-webui-minify", dest="webui_minify", action="store_false", default=None
    )
    group.add_argument(
        "--drop-icon-font", dest="webui_drop_icon_font", action="store_true", default=None
    )
    group.add_argument(
        "--keep-icon-font", dest="webui_drop_icon_font", action="store_false", default=None
    )
    group.add_argument(
        "--cflag",
        dest="extra_cflags",
        action="append",
        default=None,
        help="extra compiler flag for the loader; repeatable",
    )
    group.add_argument(
        "--ldflag",
        dest="extra_ldflags",
        action="append",
        default=None,
        help="extra linker flag for the loader; repeatable",
    )
    group.add_argument(
        "--rust-flag",
        dest="extra_rustflags",
        action="append",
        default=None,
        help="extra rustc flag for the daemon; repeatable",
    )
    group.add_argument("--no-debug-symbols", dest="debug_symbols", action="store_false", default=None)


def optimization_settings(options):
    settings = optimize.profile_settings(options.opt)
    changes = {}
    for name in OPTIMIZATION_OVERRIDES:
        value = getattr(options, name)
        if value is not None:
            changes[name] = tuple(value) if isinstance(value, list) else value
    return dataclasses.replace(settings, **changes)


def parse_arguments(argv):
    parser = argparse.ArgumentParser(
        prog="build.py",
        description="Build ZNNext without Gradle.",
    )
    parser.add_argument(
        "tasks",
        nargs="+",
        help="Gradle-compatible task names; syncVersion takes a version argument",
    )
    parser.add_argument("--jobs", type=int, default=cpu_count())
    parser.add_argument("--ccache", default=None)
    parser.add_argument("--sevenzip", default=None)
    parser.add_argument("--ndk", default=None)
    parser.add_argument("--ndk-legacy", default=None)
    parser.add_argument("--libcxx-aar", default=None)
    parser.add_argument("--no-kpm", action="store_true")
    parser.add_argument("--kpm-btf", action="store_true")
    parser.add_argument("--skip-webui", action="store_true")
    parser.add_argument("--no-build", action="store_true")
    parser.add_argument(
        "--abis",
        default=None,
        help=(
            "comma-separated ABIs to build and package "
            f"(default: all of {', '.join(ANDROID_ABIS)})"
        ),
    )
    add_optimization_arguments(parser)
    return parser.parse_args(argv)


def report_sizes(variants, abis):
    lines = []
    for variant in variants:
        archive = zip_path(variant)
        if not archive.is_file():
            continue
        lines.append(
            f"{archive.name}: {archive.stat().st_size} bytes"
        )
        for abi in abis:
            loader = loader_output(variant) / abi / "libzn_loader.so"
            daemon = daemon_output(variant) / abi / "znnd"
            parts = []
            if loader.is_file():
                parts.append(f"loader {loader.stat().st_size}")
            if daemon.is_file():
                parts.append(f"daemon {daemon.stat().st_size}")
            if parts:
                lines.append(f"  {abi}: {', '.join(parts)}")
    if lines:
        print("size report:", flush=True)
        for line in lines:
            print(f"  {line}", flush=True)


def main(argv=None):
    options = parse_arguments(argv)
    if options.jobs < 1:
        raise BuildError("--jobs must be at least 1")

    settings = optimization_settings(options)
    abis = selected_abis(options)
    if len(abis) != len(ANDROID_ABIS):
        print(f"abis: {', '.join(abis)}", flush=True)
    parsed = parse_tasks(options.tasks)
    needs_ndk = any(
        kind not in ("clean", "webui", "sync-version") for _, kind, _, _ in parsed
    )
    env = os.environ.copy()
    env[optimize.ENVIRONMENT_KEY] = settings.to_environment()
    env["ZNN_ABIS"] = ",".join(abis)
    if needs_ndk:
        ndk = ndk_dir(options.ndk)
        legacy_ndk = ndk_dir(options.ndk_legacy, LEGACY_NDK_VERSION)
        env["ANDROID_NDK_HOME"] = str(ndk)
        env["ANDROID_NDK_ROOT"] = str(ndk)
        env["ANDROID_NDK_LEGACY_HOME"] = str(legacy_ndk)
        if not env.get("ANDROID_HOME") and not env.get("ANDROID_SDK_ROOT"):
            env["ANDROID_HOME"] = str(android_sdk())
    if any(kind not in ("clean", "sync-version") for _, kind, _, _ in parsed):
        print(f"optimization: {optimize.describe(settings)}", flush=True)

    state = {"webui": False, "built": set(), "packaged": set(), "env": env}
    for _, kind, variant, value in parsed:
        if kind == "clean":
            clean()
            continue
        if kind == "sync-version":
            sync_version(value)
            continue
        if kind == "webui":
            run(script("webui"), env=env)
            state["webui"] = True
            continue

        if kind in ("zip", "assemble"):
            if variant not in state["built"]:
                if not options.no_build:
                    build_artifacts(variant, options, state)
                state["built"].add(variant)
            if kind == "zip" and variant not in state["packaged"]:
                package_variant(variant, options, state)
                state["packaged"].add(variant)
            continue

        if kind in (
            "push",
            "install-ksu",
            "install-magisk",
            "install-ksu-reboot",
            "install-magisk-reboot",
        ):
            if variant not in state["built"] and not options.no_build:
                build_artifacts(variant, options, state)
                state["built"].add(variant)
            if variant not in state["packaged"]:
                if options.no_build:
                    if not zip_path(variant).is_file():
                        raise BuildError(
                            f"{zip_path(variant).relative_to(ROOT)} does not exist"
                        )
                else:
                    package_variant(variant, options, state)
                state["packaged"].add(variant)
            adb_command(kind, variant, options, state)
            continue

    report_sizes(
        [variant for variant in VARIANTS if variant in state["packaged"]],
        abis,
    )
    return 0


if __name__ == "__main__":
    try:
        sys.exit(main())
    except BuildError as error:
        print(f"error: {error}", file=sys.stderr)
        sys.exit(1)
