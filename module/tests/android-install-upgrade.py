# SPDX-License-Identifier: GPL-3.0-or-later AND LicenseRef-DCLPL-2.0
#
# This file is part of ZNNext.
#
# ZNNext is free software: you can redistribute it and/or modify
# it under the terms of the GNU General Public License as published by
# the Free Software Foundation, either version 3 of the License, or
# (at your option) any later version.
#
# ZNNext is distributed in the hope that it will be useful,
# but WITHOUT ANY WARRANTY; without even the implied warranty of
# MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
# GNU General Public License for more details.
#
# You should have received a copy of the GNU General Public License
# along with ZNNext. If not, see <https://www.gnu.org/licenses/>.
#
# Copyright (C) 2026 VeryBaaad <verybaaad@outlook.com>

import argparse
import os
import shutil
import subprocess
import tempfile
import uuid
import zipfile
from contextlib import ExitStack
from pathlib import Path


def read_member(seven_zip, archive, name):
    return subprocess.check_output([seven_zip, "e", "-so", str(archive), name])


def version_code(seven_zip, archive):
    lines = read_member(seven_zip, archive, "module.prop").decode().splitlines()
    props = dict(line.split("=", 1) for line in lines if "=" in line)
    if props.get("id") != "zygisknextsu":
        raise ValueError(f"{archive} is not a ZNNext module ZIP")
    return int(props["versionCode"])


def test_install(serial, apk, initial, upgrade):
    seven_zip = shutil.which("7z") or shutil.which("7zz")
    if not seven_zip:
        raise RuntimeError("7-Zip is required")
    initial = initial.resolve(strict=True)
    upgrade = upgrade.resolve(strict=True)
    if version_code(seven_zip, initial) >= version_code(seven_zip, upgrade):
        raise ValueError("The upgrade archive must have a higher versionCode")

    adb_command = [os.environ.get("ADB", "adb"), "-s", serial]

    def shell(*args):
        return subprocess.check_output(
            [*adb_command, "shell", *args], text=True, timeout=30
        ).strip()

    if not any(shell("getprop", prop) == "1" for prop in ("ro.boot.qemu", "ro.kernel.qemu")):
        raise RuntimeError("A disposable Android emulator is required")
    abi = shell("getprop", "ro.product.cpu.abi")
    scratch = f"/data/local/tmp/znnext-install-{uuid.uuid4().hex}"
    with tempfile.TemporaryDirectory() as workspace, ExitStack() as cleanup:
        temp = Path(workspace)
        busybox = temp / "busybox"
        utilities = temp / "util_functions.sh"
        with zipfile.ZipFile(apk) as magisk:
            busybox.write_bytes(magisk.read(f"lib/{abi}/libbusybox.so"))
            utilities.write_bytes(magisk.read("assets/util_functions.sh"))

        damaged = temp / "damaged.zip"
        shutil.copyfile(upgrade, damaged)
        script = "post-fs-data.sh"
        (temp / script).write_bytes(read_member(seven_zip, upgrade, script) + b"\n")
        subprocess.run(
            [seven_zip, "a", "-bso0", "-tzip", str(damaged), script],
            cwd=temp,
            check=True,
        )

        subprocess.run([*adb_command, "root"], check=True, timeout=30)
        subprocess.run([*adb_command, "wait-for-device"], check=True, timeout=120)
        if shell("id", "-u") != "0":
            raise RuntimeError("The emulator must support adb root")
        mount_point = subprocess.run(
            [*adb_command, "shell", "test", "-d", "/data/adb"], timeout=30
        )
        if mount_point.returncode:
            shell("mkdir", "/data/adb")
            cleanup.callback(shell, "rmdir", "/data/adb")

        shell("mkdir", "-m", "700", scratch)
        cleanup.callback(shell, "rm", "-rf", scratch)
        files = {
            "busybox": busybox,
            "util_functions.sh": utilities,
            "initial.zip": initial,
            "upgrade.zip": upgrade,
            "damaged.zip": damaged,
            "guest.sh": Path(__file__).with_name("android-install-guest.sh"),
        }
        for name, source in files.items():
            subprocess.run(
                [*adb_command, "push", source, f"{scratch}/{name}"],
                check=True,
                timeout=120,
            )
        remote_busybox = f"{scratch}/busybox"
        shell("chmod", "700", remote_busybox)
        subprocess.run(
            [*adb_command, "shell", "/system/bin/unshare", "-m",
             remote_busybox, "sh", f"{scratch}/guest.sh", scratch],
            check=True,
            timeout=240,
        )


if __name__ == "__main__":
    parser = argparse.ArgumentParser(
        description="Test a ZNNext installation and upgrade on a disposable Android emulator."
    )
    parser.add_argument("serial")
    parser.add_argument("magisk_apk", type=Path)
    parser.add_argument("initial_zip", type=Path)
    parser.add_argument("upgrade_zip", type=Path)
    args = parser.parse_args()
    test_install(args.serial, args.magisk_apk, args.initial_zip, args.upgrade_zip)
