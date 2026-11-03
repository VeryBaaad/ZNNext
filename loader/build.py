#!/usr/bin/env python3
import argparse
import hashlib
import os
import re
import shutil
import sys
import urllib.request
import zipfile
from concurrent.futures import ThreadPoolExecutor, as_completed
from functools import lru_cache
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT))
import build as znn

DEPENDENCY_DIR = ROOT / "loader" / "build" / "deps"


def sha256_file(path):
    digest = hashlib.sha256()
    with path.open("rb") as stream:
        for block in iter(lambda: stream.read(1024 * 1024), b""):
            digest.update(block)
    return digest.hexdigest()


def download(url, destination):
    temporary = destination.with_name(destination.name + ".part")
    with urllib.request.urlopen(url) as response, temporary.open("wb") as stream:
        shutil.copyfileobj(response, stream)
    temporary.replace(destination)


def ensure_libcxx(version, override, verbose):
    base = DEPENDENCY_DIR / f"libcxx-{version}"
    marker = base / ".complete"
    if (
        marker.is_file()
        and (base / "prefab" / "modules" / "cxx" / "include" / "type_traits").is_file()
    ):
        return base

    explicit = override or os.environ.get("ZNN_LIBCXX_AAR")
    archive = DEPENDENCY_DIR / f"libcxx-{version}.aar"
    if explicit:
        archive = Path(explicit).expanduser().resolve()
        if not archive.is_file():
            raise znn.BuildError(f"libcxx AAR not found: {archive}")
    else:
        archive.parent.mkdir(parents=True, exist_ok=True)
        valid = archive.is_file() and sha256_file(archive) == znn.LIBCXX_SHA256
        if not valid:
            if archive.exists():
                archive.unlink()
            if verbose:
                print(f"loader: downloading {znn.LIBCXX_URL}", flush=True)
            try:
                download(znn.LIBCXX_URL, archive)
            except OSError as error:
                raise znn.BuildError(
                    f"cannot download {znn.LIBCXX_URL}: {error}; "
                    "set ZNN_LIBCXX_AAR to a local AAR"
                ) from error
        digest = sha256_file(archive)
        if digest != znn.LIBCXX_SHA256:
            raise znn.BuildError(
                f"libcxx AAR checksum mismatch: {digest} != {znn.LIBCXX_SHA256}"
            )

    shutil.rmtree(base, ignore_errors=True)
    base.mkdir(parents=True)
    try:
        with zipfile.ZipFile(archive) as package:
            package.extractall(base)
    except (OSError, zipfile.BadZipFile) as error:
        raise znn.BuildError(f"cannot extract {archive}: {error}") from error

    include = base / "prefab" / "modules" / "cxx" / "include" / "type_traits"
    if not include.is_file():
        raise znn.BuildError(f"{archive} does not contain a prefab cxx package")
    marker.write_text(version, encoding="utf-8")
    return base


def write_cxx_config(prefab_root, abi, destination):
    module = prefab_root / "prefab" / "modules" / "cxx"
    library = module / "libs" / f"android.{abi}" / "libcxx.a"
    include = module / "include"
    if not library.is_file():
        raise znn.BuildError(f"libcxx has no {abi} library: {library}")
    destination.mkdir(parents=True, exist_ok=True)
    content = (
        "if(NOT TARGET cxx::cxx)\n"
        "    add_library(cxx::cxx STATIC IMPORTED)\n"
        "    set_target_properties(cxx::cxx PROPERTIES\n"
        f'        IMPORTED_LOCATION "{library.as_posix()}"\n'
        f'        INTERFACE_INCLUDE_DIRECTORIES "{include.as_posix()}")\n'
        "endif()\n"
    )
    (destination / "cxxConfig.cmake").write_text(content, encoding="utf-8")


def cmake_version(executable):
    try:
        output = znn.run_capture([executable, "--version"])
    except znn.BuildError:
        return None
    match = re.search(r"version (\d+)\.(\d+)\.(\d+)", output)
    if not match:
        return None
    return tuple(int(part) for part in match.groups())


@lru_cache(maxsize=1)
def cmake_executable():
    override = os.environ.get("CMAKE")
    if override:
        version = cmake_version(override)
        if version is None:
            raise znn.BuildError(f"cannot run CMake: {override}")
        if version < (3, 22, 1):
            raise znn.BuildError(
                f"CMake 3.22.1 or newer is required, found {fmt_version(version)}"
            )
        return override, version

    sdk_roots = []
    for variable in ("ANDROID_HOME", "ANDROID_SDK_ROOT"):
        value = os.environ.get(variable)
        if value:
            sdk_roots.append(Path(value))
    configured = znn.local_sdk()
    if configured is not None:
        sdk_roots.append(configured)

    sdk_candidates = []
    for sdk_root in dict.fromkeys(sdk_roots):
        root = sdk_root / "cmake"
        if root.is_dir():
            sdk_candidates.extend(root.glob("*/bin/cmake"))

    scored = []
    for candidate in sdk_candidates:
        version = cmake_version(candidate)
        if version is not None and version >= (3, 22, 1):
            scored.append((version, str(candidate)))
    if scored:
        scored.sort(reverse=True)
        return scored[0][1], scored[0][0]

    executable = shutil.which("cmake")
    if not executable:
        raise znn.BuildError(
            "cmake was not found in the Android SDK or on PATH"
        )
    version = cmake_version(executable)
    if version is None:
        raise znn.BuildError(f"cannot determine the version of {executable}")
    if version < (3, 22, 1):
        raise znn.BuildError(
            f"CMake 3.22.1 or newer is required, found {fmt_version(version)}"
        )
    return executable, version


def fmt_version(version):
    return ".".join(str(part) for part in version)


def build_abi(abi, variant, jobs, ndk, prefab_root, ccache, verbose):
    build_dir = ROOT / "loader" / "build" / "cxx" / variant / abi
    output_dir = znn.loader_output(variant) / abi
    cxx_dir = ROOT / "loader" / "build" / "prefab" / variant / abi
    output_dir.mkdir(parents=True, exist_ok=True)
    write_cxx_config(prefab_root, abi, cxx_dir)

    cmake, cmake_version_value = cmake_executable()
    build_type, optimization = znn.variant_cmake_optimization(variant)
    cflags = ["-std=c18", *znn.DEFAULT_CFLAGS, *znn.variant_cflags(variant)]
    cxxflags = ["-std=c++20", *znn.DEFAULT_CFLAGS, *znn.variant_cflags(variant)]
    linker_flags = znn.variant_linker_flags(variant)

    arguments = [
        "-S",
        str(ROOT / "loader" / "src"),
        "-B",
        str(build_dir),
        "-G",
        "Ninja",
        f"-DCMAKE_TOOLCHAIN_FILE={ndk / 'build/cmake/android.toolchain.cmake'}",
        f"-DANDROID_ABI={abi}",
        f"-DANDROID_PLATFORM=android-{znn.ANDROID_MIN_SDK}",
        "-DANDROID_STL=none",
        f"-DANDROID_NDK={ndk}",
        f"-DCMAKE_BUILD_TYPE={build_type}",
        "-DLSPLT_STANDALONE=ON",
        f"-Dcxx_DIR={cxx_dir}",
        f"-DCMAKE_C_FLAGS={' '.join(cflags)}",
        f"-DCMAKE_CXX_FLAGS={' '.join(cxxflags)}",
        f"-DCMAKE_C_FLAGS_{build_type.upper()}={optimization}",
        f"-DCMAKE_CXX_FLAGS_{build_type.upper()}={optimization}",
        f"-DCMAKE_SHARED_LINKER_FLAGS={linker_flags}",
        f"-DCMAKE_EXE_LINKER_FLAGS={linker_flags}",
        f"-DCMAKE_LIBRARY_OUTPUT_DIRECTORY={output_dir}",
        f"-DCMAKE_RUNTIME_OUTPUT_DIRECTORY={output_dir}",
        "-DCMAKE_EXPORT_COMPILE_COMMANDS=ON",
    ]
    if cmake_version_value >= (4, 0, 0):
        arguments.append("-DCMAKE_POLICY_VERSION_MINIMUM=3.5")
    if ccache:
        arguments.append(f"-DNDK_CCACHE={ccache}")

    if verbose:
        znn.run([cmake, *arguments], prefix=f"loader {abi} configure")
    else:
        znn.run_capture([cmake, *arguments])

    znn.run(
        [
            cmake,
            "--build",
            str(build_dir),
            "--target",
            "zn_loader",
            "--parallel",
            str(jobs),
        ],
        prefix=f"loader {abi}",
    )

    artifact = output_dir / "libzn_loader.so"
    if not artifact.is_file():
        raise znn.BuildError(f"loader produced no library at {artifact}")
    print(
        f"loader: built {abi} ({artifact.stat().st_size} bytes)",
        flush=True,
    )


def parse_arguments(argv):
    parser = argparse.ArgumentParser(prog="loader/build.py")
    parser.add_argument("variant", choices=znn.VARIANTS)
    parser.add_argument("--jobs", type=int, default=znn.cpu_count())
    parser.add_argument("--abi", action="append", choices=znn.ANDROID_ABIS)
    parser.add_argument("--ndk", default=None)
    parser.add_argument("--ccache", default=None)
    parser.add_argument("--libcxx-aar", default=None)
    parser.add_argument("--verbose", action="store_true")
    return parser.parse_args(argv)


def main(argv=None):
    options = parse_arguments(argv)
    jobs = max(1, options.jobs)
    abis = options.abi or list(znn.ANDROID_ABIS)
    ndk = znn.ndk_dir(options.ndk)
    ccache = znn.ccache_path(options.ccache)
    prefab_root = ensure_libcxx(znn.LIBCXX_VERSION, options.libcxx_aar, options.verbose)
    per_abi = max(1, jobs // len(abis))

    print(
        f"loader: {options.variant}, {len(abis)} ABI(s), {per_abi} job(s) each, "
        f"NDK {ndk.name}",
        flush=True,
    )
    if ccache:
        print(f"loader: ccache {ccache}", flush=True)
    cmake, cmake_version_value = cmake_executable()
    print(
        f"loader: cmake {cmake} ({fmt_version(cmake_version_value)})",
        flush=True,
    )

    errors = []
    with ThreadPoolExecutor(max_workers=len(abis)) as pool:
        futures = {
            pool.submit(
                build_abi,
                abi,
                options.variant,
                per_abi,
                ndk,
                prefab_root,
                ccache,
                options.verbose,
            ): abi
            for abi in abis
        }
        for future in as_completed(futures):
            abi = futures[future]
            try:
                future.result()
            except Exception as error:
                errors.append((abi, error))
                print(f"[loader {abi}] failed: {error}", file=sys.stderr, flush=True)
    if errors:
        abi, error = errors[0]
        raise znn.BuildError(f"loader {abi} failed: {error}")
    return 0


if __name__ == "__main__":
    try:
        sys.exit(main())
    except znn.BuildError as error:
        print(f"error: {error}", file=sys.stderr)
        sys.exit(1)
