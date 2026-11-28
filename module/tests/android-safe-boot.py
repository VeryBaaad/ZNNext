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
import json
import os
import subprocess
import time
import uuid
from contextlib import ExitStack
from pathlib import Path


def test_safe_boot(serial, binary):
    def adb(*args, check=True, timeout=30, input_text=None):
        return subprocess.run(
            [os.environ.get("ADB", "adb"), "-s", serial, *args],
            check=check,
            text=True,
            input=input_text,
            stdout=subprocess.PIPE,
            stderr=None if check else subprocess.PIPE,
            timeout=timeout,
        )

    def shell(*args):
        return adb("shell", *args).stdout.strip()

    def root():
        adb("root")
        adb("wait-for-device", timeout=120)
        if shell("id", "-u") != "0":
            raise RuntimeError("Use a disposable emulator that supports adb root")

    def reboot():
        previous = uuid.UUID(shell("cat", "/proc/sys/kernel/random/boot_id"))
        adb("reboot")
        deadline = time.monotonic() + 180
        while time.monotonic() < deadline:
            ready = adb("shell", "getprop", "sys.boot_completed", check=False)
            if ready.returncode == 0 and ready.stdout.strip() == "1":
                root()
                current = shell("cat", "/proc/sys/kernel/random/boot_id")
                if uuid.UUID(current) != previous:
                    return
            time.sleep(2)
        raise RuntimeError("Timed out waiting for a new Android boot")

    binary = binary.resolve(strict=True)
    if not binary.is_file():
        raise ValueError(f"Expected a znnd binary, got {binary}")
    if not any(shell("getprop", prop) == "1" for prop in ("ro.boot.qemu", "ro.kernel.qemu")):
        raise RuntimeError("This test reboots a disposable Android emulator twice")
    root()
    modules = "/data/adb/modules"
    state = "/data/adb/zygisknextsu"
    scratch = f"/data/local/tmp/znnext-safe-boot-{uuid.uuid4().hex}"
    remote_binary = f"{scratch}/znnd"
    first, second, disabled = "znnext_test_a", "znnext_test_b", "znnext_test_disabled"
    malformed = "znnext_test_malformed"
    valid_line = "name=znnext_test_unused fixture.so"
    invalid_lines = "\n".join((
        "name= fixture.so",
        "path=relative fixture.so",
        "name=dir/fixture fixture.so",
        "name=unused broken\0filename",
        "path=/fixture/unused",
    ))

    def expect(*action, active=False, armed=False, enabled=(), paused=()):
        actual = json.loads(shell(remote_binary, "--ctl", "safe-boot", *action))
        expected = {
            "active": active,
            "armed": armed,
            "enabled": sorted(enabled),
            "paused": sorted(paused),
        }
        if actual != expected:
            raise AssertionError(f"{action}: expected {expected}, got {actual}")

    with ExitStack() as cleanup:
        def mkdir(path, shared=False):
            if shared and adb("shell", "test", "-d", path, check=False).returncode == 0:
                return
            shell("mkdir", path)
            cleanup.callback(shell, "rmdir", path)

        mkdir("/data/adb", shared=True)
        mkdir(state)
        cleanup.callback(shell, "rm", "-f", f"{state}/safe-boot.json")
        mkdir(modules, shared=True)
        mkdir(scratch)
        cleanup.callback(shell, "rm", "-f", remote_binary)
        adb("push", binary, remote_binary)
        shell("chmod", "700", remote_binary)
        for name, contents in (
            (first, f"{invalid_lines}\n{valid_line}"),
            (second, valid_line),
            (disabled, valid_line),
            (malformed, invalid_lines),
        ):
            directory = f"{modules}/{name}"
            mkdir(directory)
            declaration = f"{directory}/zn_modules.txt"
            cleanup.callback(shell, "rm", "-f", declaration)
            adb("shell", "-T", f"cat > {declaration}", input_text=f"{contents}\n")
        marker = f"{modules}/{disabled}/disable"
        cleanup.callback(shell, "rm", "-f", marker)
        shell("touch", marker)

        expect("status")
        expect("arm", armed=True)
        expect("cancel")
        expect("status")
        expect("arm", armed=True)
        reboot()
        expect("status", active=True, paused=(first, second))
        for name in (disabled, malformed):
            result = adb(
                "shell", remote_binary, "--ctl", "safe-boot", "enable", name, check=False
            )
            if result.returncode != 1 or f"unavailable ZN module: {name}" not in result.stderr:
                raise AssertionError(f"Unexpected result enabling {name}: {result}")
        expect("status", active=True, paused=(first, second))
        expect("enable", first, active=True, enabled=(first,), paused=(second,))
        expect("status", active=True, enabled=(first,), paused=(second,))
        reboot()
        expect("status")
        shell("test", "!", "-e", f"{state}/safe-boot.json")
    print(f"{serial}: one-boot safe mode and per-module recovery passed")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(
        description="Test safe boot across two reboots of a clean disposable emulator."
    )
    parser.add_argument("serial")
    parser.add_argument("znnd", type=Path)
    args = parser.parse_args()
    test_safe_boot(args.serial, args.znnd)

