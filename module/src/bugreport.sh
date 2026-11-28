#!/system/bin/sh
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

fail() {
  printf '%s\n' "$1" >&2
  exit 1
}

[ "$#" -ge 1 ] && [ "$#" -le 2 ] || fail "usage: bugreport.sh <summary> [logs]"

busybox=
if ! command -v tar >/dev/null 2>&1 || ! command -v mktemp >/dev/null 2>&1; then
  for candidate in /data/adb/magisk/busybox /data/adb/ksu/bin/busybox /data/adb/ap/bin/busybox; do
    if [ -x "$candidate" ]; then
      busybox=$candidate
      break
    fi
  done
  [ -n "$busybox" ] || fail "tar or mktemp is unavailable"
fi

run_tool() {
  if [ -n "$busybox" ]; then
    "$busybox" "$@"
  else
    "$@"
  fi
}

download=/sdcard/Download
umask 022
run_tool mkdir -p "$download" || fail "cannot open the Download directory"
umask 077
workdir=$(run_tool mktemp -d /data/local/tmp/znnext-bugreport.XXXXXX) || fail "cannot create temporary report directory"
trap 'run_tool rm -rf "$workdir"' 0
trap 'exit 1' HUP INT TERM

printf '%s' "$1" >"$workdir/basic.txt" || fail "cannot write report summary"
if [ "$#" -eq 2 ]; then
  run_tool mkdir "$workdir/log" || fail "cannot create report log directory"
  printf '%s' "$2" >"$workdir/log/znnd.log" || fail "cannot write report logs"
  set -- basic.txt log/znnd.log
else
  set -- basic.txt
fi

timestamp=$(run_tool date -u +%Y%m%d-%H%M%S) || fail "cannot read report time"
archive="$download/ZNNext-bugreport-$timestamp-${workdir##*.}.tar.gz"
(
  set -C
  umask 022
  (
    trap 'run_tool rm -f "$archive"' 0
    trap 'exit 1' HUP INT TERM
    run_tool tar -czf - -C "$workdir" "$@" >&3 || exit 1
    trap - 0
  ) 3>"$archive"
) || fail "cannot write bug report archive"
printf '%s\n' "$archive"
