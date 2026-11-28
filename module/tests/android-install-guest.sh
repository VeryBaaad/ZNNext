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

set -e
set -o standalone
work=$1
mount --make-rprivate /
mkdir "$work/adb"
mount -o bind "$work/adb" /data/adb

BOOTMODE=true
. "$work/util_functions.sh"
active=/data/adb/modules/zygisknextsu
staged=/data/adb/modules_update/zygisknextsu
config=/data/adb/zygisknextsu/config
primary=$(getprop ro.product.cpu.abi)
case $primary in
  x86 | x86_64)
    lib32=x86
    lib64=x86_64
    ;;
  arm64-v8a)
    lib32=armeabi-v7a
    lib64=$primary
    ;;
  mips64)
    lib32=mips
    lib64=$primary
    ;;
  *)
    lib32=$primary
    lib64=$primary
    ;;
esac

install_zip() (
  ZIPFILE=$1
  TMPDIR=$work/unpack
  set +e
  install_module
)

check_member() {
  unzip -p "$1" "$2" >"$work/expected"
  cmp "$work/expected" "$3"
}

check_payload() {
  check_member "$1" module.prop "$2/module.prop.orig"
  check_member "$1" post-fs-data.sh "$2/post-fs-data.sh"
  check_member "$1" "bin/$primary/znnd" "$2/bin/znnd"
  check_member "$1" "lib/$lib32/libzn_loader.so" "$2/lib/libzn_loader.so"
  check_member "$1" "lib/$lib64/libzn_loader.so" "$2/lib64/libzn_loader.so"
  check_member "$1" webroot/index.html "$2/webroot/index.html"
  test "$(stat -c '%u:%g:%a' "$2/bin/znnd")" = 0:0:755
  test "$(stat -c '%u:%g:%a' "$2/lib/libzn_loader.so")" = 0:0:644
  test "$(stat -c '%u:%g:%a' "$2/lib64/libzn_loader.so")" = 0:0:644
}

install_valid() {
  if ! install_zip "$1" >"$work/install.log" 2>&1; then
    cat "$work/install.log" >&2
    exit 1
  fi
  test -f "$active/update"
  check_member "$1" module.prop "$staged/module.prop"
  check_payload "$1" "$staged"
}

initial=$work/initial.zip
upgrade=$work/upgrade.zip
install_valid "$initial"
mv "$active" /data/adb/initial-marker
mv "$staged" "$active"
mkdir -p "${config%/*}"
printf 'mode=proc\nhot_update=true\n' >"$work/expected-config"
cp "$work/expected-config" "$config"

if install_zip "$work/damaged.zip" >"$work/rejected.log" 2>&1; then
  echo "Installer accepted a damaged upgrade" >&2
  exit 1
fi
if ! grep -F 'Failed to verify post-fs-data.sh' "$work/rejected.log"; then
  cat "$work/rejected.log" >&2
  exit 1
fi
test ! -e "$staged"
check_member "$initial" module.prop "$active/module.prop"
check_payload "$initial" "$active"
cmp "$work/expected-config" "$config"

install_valid "$upgrade"
check_payload "$initial" "$active"
cmp "$work/expected-config" "$config"
mv "$active" /data/adb/previous-module
mv "$staged" "$active"
check_member "$upgrade" module.prop "$active/module.prop"
check_payload "$upgrade" "$active"
cmp "$work/expected-config" "$config"
echo "$primary: installation, checksum rejection, and upgrade passed"
