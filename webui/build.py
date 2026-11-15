#!/usr/bin/env python3
import argparse
import os
import re
import shutil
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT))
import build as znn

WEBUI_DIR = ROOT / "webui"
WEBROOT = ROOT / "module" / "webroot"


ICON_IMPORT = re.compile(r'@m3e/icons/[a-z0-9]+/([a-z0-9_]+)')
ICON_LITERAL = re.compile(r'<m3e-icon[^>]*\bname="([a-z0-9_]+)"')
ICON_BINDING = re.compile(r'<m3e-icon[^>]*:name="([^"]+)"')
STRING_LITERAL = re.compile(r'["\']([a-z][a-z0-9_]*)["\']')


def registered_icons():
    source = (WEBUI_DIR / "src" / "skins" / "material" / "elements.ts").read_text(
        encoding="utf-8"
    )
    return set(ICON_IMPORT.findall(source))


def bound_icon_names(expression, directory):
    names = set(STRING_LITERAL.findall(expression))
    for identifier in re.findall(r"([A-Za-z_$][A-Za-z0-9_$]*)\s*\[", expression):
        for path in directory.rglob("*"):
            if not path.is_file() or path.suffix not in (".ts", ".vue"):
                continue
            text = path.read_text(encoding="utf-8")
            match = re.search(
                rf"\b{re.escape(identifier)}\b\s*(?::[^=]+)?=\s*{{(.*?)}}",
                text,
                re.S,
            )
            if match is not None:
                names.update(STRING_LITERAL.findall(match.group(1)))
    return names


def verify_icon_coverage():
    """Fail when the bundled icon font is dropped but an icon is not an SVG."""
    directory = WEBUI_DIR / "src" / "skins" / "material"
    registered = registered_icons()
    missing = set()
    for path in sorted(directory.rglob("*.vue")):
        text = path.read_text(encoding="utf-8")
        used = set(ICON_LITERAL.findall(text))
        for expression in ICON_BINDING.findall(text):
            used.update(bound_icon_names(expression, directory))
        absent = used - registered
        if absent:
            missing.update(absent)
            print(
                f"webui: {path.name} uses unregistered icons "
                f"{', '.join(sorted(absent))}",
                file=sys.stderr,
            )
    if missing:
        raise znn.BuildError(
            "the icon font cannot be dropped, register these icons in "
            "webui/src/skins/material/elements.ts or pass --keep-icon-font"
        )


def npm_executable():
    executable = shutil.which("npm")
    if not executable:
        raise znn.BuildError("npm was not found on PATH")
    return executable


def install_needed():
    node_modules = WEBUI_DIR / "node_modules"
    marker = node_modules / ".package-lock.json"
    if not node_modules.is_dir() or not marker.is_file():
        return True
    lock = WEBUI_DIR / "package-lock.json"
    return lock.is_file() and lock.stat().st_mtime > marker.stat().st_mtime


def install(force):
    if not force and not install_needed():
        print("webui: npm dependencies are up to date", flush=True)
        return
    npm = npm_executable()
    command = [npm, "ci" if (WEBUI_DIR / "package-lock.json").is_file() else "install"]
    command += ["--no-audit", "--no-fund"]
    znn.run(command, cwd=WEBUI_DIR, prefix="webui")


def parse_arguments(argv):
    parser = argparse.ArgumentParser(prog="webui/build.py")
    parser.add_argument("--force-install", action="store_true")
    parser.add_argument("--skip-install", action="store_true")
    return parser.parse_args(argv)


def main(argv=None):
    options = parse_arguments(argv)
    if not options.skip_install:
        install(options.force_install)
    npm = npm_executable()
    environment = os.environ.copy()
    environment["ZNN_MODULE_ID"] = znn.MODULE_ID
    environment["ZNN_MODULE_NAME"] = znn.MODULE_NAME
    environment["ZNN_VER_NAME"] = znn.VER_NAME
    environment["ZNN_COMMIT_HASH"] = znn.commit_hash()
    settings = znn.optimize.from_environment()
    if settings.webui_drop_icon_font:
        verify_icon_coverage()
    environment["ZNN_WEBUI_DROP_ICON_FONT"] = (
        "1" if settings.webui_drop_icon_font else "0"
    )
    environment["ZNN_WEBUI_MINIFY"] = "1" if settings.webui_minify else "0"
    znn.run([npm, "run", "build"], cwd=WEBUI_DIR, env=environment, prefix="webui")
    if not (WEBROOT / "index.html").is_file():
        raise znn.BuildError(f"webui produced no index.html at {WEBROOT}")
    print(f"webui: built {WEBROOT.relative_to(ROOT)}", flush=True)
    return 0


if __name__ == "__main__":
    try:
        sys.exit(main())
    except znn.BuildError as error:
        print(f"error: {error}", file=sys.stderr)
        sys.exit(1)
