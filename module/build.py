#!/usr/bin/env python3
import argparse
import hashlib
import os
import re
import shutil
import string
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT))
import build as znn

MODULE_DIR = ROOT / "module"
SOURCE_DIR = MODULE_DIR / "src"
STAGE_ROOT = MODULE_DIR / "build" / "outputs" / "module"
PRIVATE_KEY = MODULE_DIR / "private_key"
SIGNER = MODULE_DIR / "sign" / "Cargo.toml"

TEMPLATED_FILES = {"module.prop", "customize.sh", "late-load.sh", "post-fs-data.sh", "uninstall.sh"}
SKIPPED_FILES = {*TEMPLATED_FILES, "mazoku"}

MACHIKADO_LAYOUT = {
    "machikado.arm64": ("arm64-v8a", "armeabi-v7a", "arm64-v8a"),
    "machikado.x64": ("x86_64", "x86", "x86_64"),
    "machikado.arm": ("armeabi-v7a", "armeabi-v7a", "armeabi-v7a"),
    "machikado.armeabi": ("armeabi", "armeabi", "armeabi"),
    "machikado.x86": ("x86", "x86", "x86_64"),
    "machikado.riscv64": ("riscv64", "riscv64", "riscv64"),
    "machikado.mips": ("mips", "mips", "mips"),
    "machikado.mips64": ("mips64", "mips", "mips64"),
}


def write_text(path, text):
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_bytes(text.replace("\r\n", "\n").replace("\r", "\n").encode("utf-8"))


def sanitize_component(value):
    return re.sub(r"[^A-Z0-9]+", "_", value.upper()).strip("_")


def license_suffix(file_name):
    upper = file_name.upper()
    for prefix in ("LICENSE-", "LICENSE.", "COPYING-", "COPYING."):
        if upper.startswith(prefix):
            return sanitize_component(upper[len(prefix):])
    return ""


def prepare_licenses(stage):
    output = stage / "licenses"
    output.mkdir(parents=True, exist_ok=True)
    shutil.copyfile(ROOT / "LICENSE", output / "LICENSE-ZNNext")
    external = ROOT / "loader" / "src" / "external"
    licenses = []
    if external.is_dir():
        for component in external.iterdir():
            if not component.is_dir():
                continue
            for candidate in component.iterdir():
                if candidate.is_file() and re.match(
                    r"^(LICENSE|COPYING)([.-].*)?$", candidate.name
                ):
                    licenses.append(candidate)
    licenses.sort(key=lambda path: path.as_posix())

    used = {"LICENSE-ZNNext"}
    count = 1
    for license_file in licenses:
        component = sanitize_component(license_file.parent.name) or "EXTERNAL"
        suffix = license_suffix(license_file.name)
        base = f"LICENSE-{component}" + (f"-{suffix}" if suffix else "")
        target = base
        duplicate = 2
        while target in used:
            target = f"{base}-{duplicate}"
            duplicate += 1
        used.add(target)
        shutil.copyfile(license_file, output / target)
        count += 1
    print(f"licenses: packaged {count} files into licenses/", flush=True)


def copy_source_tree(stage):
    for path in sorted(SOURCE_DIR.rglob("*")):
        if not path.is_file():
            continue
        relative = path.relative_to(SOURCE_DIR).as_posix()
        if relative in SKIPPED_FILES:
            continue
        data = path.read_bytes()
        data = data.replace(b"\r\n", b"\n").replace(b"\r", b"\n")
        destination = stage / relative
        destination.parent.mkdir(parents=True, exist_ok=True)
        destination.write_bytes(data)
        shutil.copymode(path, destination)


def expand_module_prop(stage, variant):
    contents = (SOURCE_DIR / "module.prop").read_text(encoding="utf-8")
    values = {
        "moduleId": znn.MODULE_ID,
        "moduleName": znn.MODULE_NAME,
        "versionName": (
            f"{znn.VER_NAME} ({znn.version_code()}-{znn.commit_hash()}-{variant})"
        ),
        "versionCode": str(znn.version_code()),
        "updateUrl": znn.UPDATE_URL,
    }
    write_text(stage / "module.prop", string.Template(contents).substitute(values))


def expand_scripts(stage, variant):
    tokens = {
        "DEBUG": "true" if variant == "debug" else "false",
        "MIN_SDK": str(znn.ANDROID_MIN_SDK),
        "MIN_KSU_VERSION": str(znn.MIN_KSU_VERSION),
        "MIN_KSUD_VERSION": str(znn.MIN_KSUD_VERSION),
        "MIN_MAGISK_VERSION": str(znn.MIN_MAGISK_VERSION),
        "MIN_APATCH_VERSION": str(znn.MIN_APATCH_VERSION),
        "KPM_ID": znn.KPM_ID,
    }
    for name in ("customize.sh", "late-load.sh", "post-fs-data.sh", "uninstall.sh"):
        contents = (SOURCE_DIR / name).read_text(encoding="utf-8")
        expanded = re.sub(
            r"@([A-Z0-9_]+)@",
            lambda match: tokens.get(match.group(1), match.group(0)),
            contents,
        )
        write_text(stage / name, expanded)


def copy_mazoku(stage):
    shutil.copy2(SOURCE_DIR / "mazoku", stage / "mazoku")


def copy_readme(stage):
    shutil.copy2(ROOT / "README.md", stage / "README.md")


def copy_webroot(stage):
    webroot = MODULE_DIR / "webroot"
    if not webroot.is_dir():
        print("webui: module/webroot is missing; the zip has no WebUI", flush=True)
        return
    for path in webroot.rglob("*"):
        if not path.is_file():
            continue
        relative = path.relative_to(webroot)
        destination = stage / "webroot" / relative
        destination.parent.mkdir(parents=True, exist_ok=True)
        shutil.copy2(path, destination)


def copy_artifacts(stage, variant):
    for abi in znn.ANDROID_ABIS:
        loader = znn.loader_output(variant) / abi / "libzn_loader.so"
        if loader.is_file():
            destination = stage / "lib" / abi / "libzn_loader.so"
            destination.parent.mkdir(parents=True, exist_ok=True)
            shutil.copy2(loader, destination)
        else:
            print(f"loader: missing {loader.relative_to(ROOT)}", file=sys.stderr)

        daemon = znn.daemon_output(variant) / abi / "znnd"
        if daemon.is_file():
            destination = stage / "bin" / abi / "znnd"
            destination.parent.mkdir(parents=True, exist_ok=True)
            shutil.copy2(daemon, destination)
        else:
            print(f"daemon: missing {daemon.relative_to(ROOT)}", file=sys.stderr)


def copy_kpm(stage, variant, no_kpm):
    artifact = znn.kpm_output(variant) / "znn.kpm"
    if artifact.is_file():
        shutil.copy2(artifact, stage / "znn.kpm")
        print("kpm: packaging KernelPatch backend: znn.kpm", flush=True)
        return
    if no_kpm:
        print("kpm: disabled, the zip will not contain znn.kpm", file=sys.stderr)
        return
    raise znn.BuildError(
        "znn.kpm was not built; build with a complete NDK or pass --no-kpm"
    )


def sign_stage(stage):
    outputs = set(MACHIKADO_LAYOUT)
    if not PRIVATE_KEY.is_file():
        print("no private_key found, this build will not be signed", flush=True)
        for name in outputs:
            (stage / name).write_bytes(b"")
        return

    lines = []
    for output, (abi, abi32, abi64) in MACHIKADO_LAYOUT.items():
        entries = [
            ("module.prop.orig", "module.prop"),
            ("post-fs-data.sh", "post-fs-data.sh"),
            ("late-load.sh", "late-load.sh"),
            ("emulated-soft-reboot.sh", "emulated-soft-reboot.sh"),
            ("sepolicy.rule", "sepolicy.rule"),
            ("uninstall.sh", "uninstall.sh"),
            ("mazoku", "mazoku"),
            ("bin/znnd", f"bin/{abi}/znnd"),
            ("lib/libzn_loader.so", f"lib/{abi32}/libzn_loader.so"),
            ("lib64/libzn_loader.so", f"lib/{abi64}/libzn_loader.so"),
        ]
        for target, source in entries:
            lines.append(f"{output}\t{target}\t{source}")
        webroot = stage / "webroot"
        if webroot.is_dir():
            for path in sorted(webroot.rglob("*")):
                if path.is_file():
                    relative = path.relative_to(stage).as_posix()
                    lines.append(f"{output}\t{relative}\t{relative}")

    jobs_file = MODULE_DIR / "build" / "sign-jobs.txt"
    jobs_file.parent.mkdir(parents=True, exist_ok=True)
    jobs_file.write_text("\n".join(lines) + "\n", encoding="utf-8")

    environment = os.environ.copy()
    environment["CARGO_TARGET_DIR"] = str(MODULE_DIR / "build" / "sign-target")
    znn.run(
        [
            "cargo",
            "run",
            "--quiet",
            "--release",
            "--locked",
            "--manifest-path",
            str(SIGNER),
            "--",
            str(stage),
            str(PRIVATE_KEY),
            str(jobs_file),
        ],
        cwd=ROOT,
        env=environment,
        prefix="sign",
    )
    for name in outputs:
        path = stage / name
        if not path.is_file() or path.stat().st_size != 96:
            raise znn.BuildError(f"signer produced no valid {name}")


def write_checksums(stage):
    files = sorted(path for path in stage.rglob("*") if path.is_file())
    for path in files:
        digest = hashlib.sha256()
        with path.open("rb") as stream:
            for block in iter(lambda: stream.read(1024 * 1024), b""):
                digest.update(block)
        path.with_name(path.name + ".sha256").write_text(
            digest.hexdigest(), encoding="ascii"
        )
    print(f"checksums: wrote {len(files)} sha256 files", flush=True)


def build_zip(stage, variant, sevenzip_path):
    archive = znn.zip_path(variant)
    archive.parent.mkdir(parents=True, exist_ok=True)
    if archive.exists():
        archive.unlink()

    files = sorted(
        path.relative_to(stage).as_posix() for path in stage.rglob("*") if path.is_file()
    )
    deflated = [
        name
        for name in files
        if (stage / name).stat().st_size < znn.XZ_MIN_ENTRY_SIZE
    ]
    xz = [
        name
        for name in files
        if (stage / name).stat().st_size >= znn.XZ_MIN_ENTRY_SIZE
    ]

    for method, members in (("Deflate", deflated), ("XZ", xz)):
        if not members:
            continue
        znn.run(
            [
                sevenzip_path,
                "a",
                "-tzip",
                "-mx=9",
                f"-mm={method}",
                "-bso0",
                "-bsp0",
                str(archive),
                *members,
            ],
            cwd=stage,
            prefix="zip",
        )

    if not archive.is_file():
        raise znn.BuildError(f"7-Zip produced no archive at {archive}")
    print(
        f"{archive.name}: {archive.stat().st_size} bytes "
        f"({len(xz)} xz + {len(deflated)} deflate entries)",
        flush=True,
    )
    return archive


def package(variant, no_kpm, sevenzip=None):
    stage = STAGE_ROOT / variant
    shutil.rmtree(stage, ignore_errors=True)
    stage.mkdir(parents=True)

    copy_source_tree(stage)
    copy_mazoku(stage)
    expand_module_prop(stage, variant)
    expand_scripts(stage, variant)
    copy_readme(stage)
    copy_webroot(stage)
    prepare_licenses(stage)
    copy_artifacts(stage, variant)
    copy_kpm(stage, variant, no_kpm)
    sign_stage(stage)
    write_checksums(stage)
    return build_zip(stage, variant, znn.seven_zip(sevenzip))


def push(variant):
    archive = znn.zip_path(variant)
    if not archive.is_file():
        raise znn.BuildError(f"{archive.relative_to(ROOT)} does not exist; build it first")
    znn.run(["adb", "push", str(archive), "/data/local/tmp"], prefix="adb")
    return archive


def install_ksu(variant):
    archive = push(variant)
    remote = f"/data/local/tmp/{archive.name}"
    znn.run(
        [
            "adb",
            "shell",
            f"echo '/data/adb/ksud module install {remote}' > /data/local/tmp/install.sh",
        ],
        prefix="adb",
    )
    znn.run(
        ["adb", "shell", "chmod", "755", "/data/local/tmp/install.sh"],
        prefix="adb",
    )
    znn.run(
        ["adb", "shell", "su", "-c", "/data/local/tmp/install.sh"],
        prefix="adb",
    )


def install_magisk(variant):
    archive = push(variant)
    remote = f"/data/local/tmp/{archive.name}"
    znn.run(
        ["adb", "shell", "su", "-M", "-c", f"magisk --install-module {remote}"],
        prefix="adb",
    )


def reboot():
    znn.run(["adb", "reboot"], prefix="adb")


def parse_arguments(argv):
    parser = argparse.ArgumentParser(prog="module/build.py")
    subparsers = parser.add_subparsers(dest="command", required=True)

    package_parser = subparsers.add_parser("package")
    package_parser.add_argument("variant", choices=znn.VARIANTS)
    package_parser.add_argument("--no-kpm", action="store_true")
    package_parser.add_argument("--sevenzip", default=None)

    for name in (
        "push",
        "install-ksu",
        "install-magisk",
        "install-ksu-reboot",
        "install-magisk-reboot",
    ):
        command_parser = subparsers.add_parser(name)
        command_parser.add_argument("variant", choices=znn.VARIANTS)
    return parser.parse_args(argv)


def main(argv=None):
    options = parse_arguments(argv)
    if options.command == "package":
        package(options.variant, options.no_kpm, options.sevenzip)
    elif options.command == "push":
        push(options.variant)
    elif options.command == "install-ksu":
        install_ksu(options.variant)
    elif options.command == "install-magisk":
        install_magisk(options.variant)
    elif options.command == "install-ksu-reboot":
        install_ksu(options.variant)
        reboot()
    elif options.command == "install-magisk-reboot":
        install_magisk(options.variant)
        reboot()
    return 0


if __name__ == "__main__":
    try:
        sys.exit(main())
    except znn.BuildError as error:
        print(f"error: {error}", file=sys.stderr)
        sys.exit(1)
