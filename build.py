#!/usr/bin/env python3
import argparse
import os
import shutil
import subprocess
import sys
import tempfile
from concurrent.futures import ThreadPoolExecutor, as_completed
from functools import lru_cache
from pathlib import Path

ROOT = Path(__file__).resolve().parent
LOADER_DIR = ROOT / "loader"
DAEMON_DIR = ROOT / "daemon"
KPM_DIR = ROOT / "kpm"
MODULE_DIR = ROOT / "module"
WEBUI_DIR = ROOT / "webui"

MODULE_ID = "zygisknextsu"
MODULE_NAME = "ZNNext"
VER_NAME = "v2.16.0"
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


def build_artifacts(variant, options, state):
    env = state["env"]
    jobs = options.jobs
    loader_jobs = max(1, jobs * 55 // 100)
    daemon_jobs = max(1, jobs - loader_jobs)

    loader_cmd = script("loader", variant, "--jobs", loader_jobs)
    if options.ccache is not None:
        loader_cmd += ["--ccache", options.ccache]
    if options.ndk:
        loader_cmd += ["--ndk", options.ndk]
    if options.ndk_legacy:
        loader_cmd += ["--ndk-legacy", options.ndk_legacy]
    if options.libcxx_aar:
        loader_cmd += ["--libcxx-aar", options.libcxx_aar]

    daemon_cmd = script("daemon", variant, "--jobs", daemon_jobs)
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
}


def parse_task(name):
    key = name.lower()
    if key not in TASK_MAP:
        raise BuildError(
            f"unknown task: {name} (try zipRelease, zipDebug, assembleRelease, "
            "clean, pushRelease, installKsuRelease, installMagiskRelease)"
        )
    return TASK_MAP[key]


def parse_arguments(argv):
    parser = argparse.ArgumentParser(
        prog="build.py",
        description="Build ZNNext without Gradle.",
    )
    parser.add_argument("tasks", nargs="+", help="Gradle-compatible task names")
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
    return parser.parse_args(argv)


def main(argv=None):
    options = parse_arguments(argv)
    if options.jobs < 1:
        raise BuildError("--jobs must be at least 1")

    parsed = [(name, *parse_task(name)) for name in options.tasks]
    needs_ndk = any(kind not in ("clean", "webui") for _, kind, _ in parsed)
    env = os.environ.copy()
    if needs_ndk:
        ndk = ndk_dir(options.ndk)
        legacy_ndk = ndk_dir(options.ndk_legacy, LEGACY_NDK_VERSION)
        env["ANDROID_NDK_HOME"] = str(ndk)
        env["ANDROID_NDK_ROOT"] = str(ndk)
        env["ANDROID_NDK_LEGACY_HOME"] = str(legacy_ndk)
        if not env.get("ANDROID_HOME") and not env.get("ANDROID_SDK_ROOT"):
            env["ANDROID_HOME"] = str(android_sdk())

    state = {"webui": False, "built": set(), "packaged": set(), "env": env}
    for _, kind, variant in parsed:
        if kind == "clean":
            clean()
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

    return 0


if __name__ == "__main__":
    try:
        sys.exit(main())
    except BuildError as error:
        print(f"error: {error}", file=sys.stderr)
        sys.exit(1)
