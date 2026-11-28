#!/bin/sh
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

set -eu

fail() {
  printf '%s\n' "$1" >&2
  exit 1
}

if [ "$#" -ne 3 ]; then
  printf 'Usage: %s SERIAL EXPECTED_PAGE_SIZE ZNND_BINARY\n' "$0" >&2
  exit 2
fi

serial=$1
expected_page_size=$2
binary=$3
adb_command=${ADB:-adb}

[ -n "$serial" ] || fail 'An ADB device serial is required.'
case $expected_page_size in
  4096 | 16384) ;;
  *) fail 'The expected page size must be 4096 or 16384 bytes.' ;;
esac
[ -f "$binary" ] && [ -r "$binary" ] || fail "Cannot read znnd binary: $binary"
case $binary in
  /*) ;;
  *) binary=$PWD/$binary ;;
esac

adb_device() {
  "$adb_command" -s "$serial" "$@"
}

carriage_return=$(printf '\r')
page_size=$(adb_device shell getconf PAGESIZE) || fail "Could not read the Android page size on $serial."
page_size=${page_size%"$carriage_return"}
[ "$page_size" = "$expected_page_size" ] || fail "Expected $expected_page_size bytes on $serial, got '$page_size'."

remote_dir=
cleanup() {
  status=$?
  trap - EXIT
  if [ -n "$remote_dir" ] && ! adb_device shell "rm -f '$remote_dir/znnd' && rmdir '$remote_dir'"; then
    printf 'Could not remove the test directory on %s: %s\n' "$serial" "$remote_dir" >&2
    [ "$status" -ne 0 ] || status=1
  fi
  exit "$status"
}
trap cleanup EXIT
trap 'exit 129' HUP
trap 'exit 130' INT
trap 'exit 143' TERM

remote_prefix=/data/local/tmp/znnext-page-size.
temp_dir=$(adb_device shell mktemp -d "${remote_prefix}XXXXXXXX") || fail "Could not create a test directory on $serial."
temp_dir=${temp_dir%"$carriage_return"}
case $temp_dir in
  "$remote_prefix"*) ;;
  *) fail "Invalid test directory from $serial." ;;
esac
suffix=${temp_dir#"$remote_prefix"}
case $suffix in
  '' | *[!abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789]*) fail "Invalid test directory from $serial." ;;
esac
[ "${#suffix}" -eq 8 ] || fail "Invalid test directory from $serial."
remote_dir=$temp_dir
remote_binary=$remote_dir/znnd

adb_device push "$binary" "$remote_binary" >/dev/null || fail "Could not push znnd to $serial."
adb_device shell chmod 700 "$remote_binary" || fail "Could not make znnd executable on $serial."
version=$(adb_device shell "$remote_binary" --version) || fail "znnd --version failed on $serial."
version=${version%"$carriage_return"}
case $version in
  *'
'*) fail "Invalid znnd version on $serial: $version" ;;
esac
printf '%s\n' "$version" | LC_ALL=C grep -Eq '^znnd v?[0-9]+([.][0-9]+){2,3}([-+][0-9A-Za-z][0-9A-Za-z.+-]*)?$' ||
  fail "Invalid znnd version on $serial: $version"

printf '%s: %s-byte page size, %s\n' "$serial" "$page_size" "$version"
