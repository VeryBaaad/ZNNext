#!/usr/bin/env python3
import argparse
import os
import shutil
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT))
import build as znn

WEBUI_DIR = ROOT / "webui"
WEBROOT = ROOT / "module" / "webroot"


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
