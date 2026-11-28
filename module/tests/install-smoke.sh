#!/bin/sh

set -e

if [ "$#" -ne 2 ]; then
  echo "usage: $0 INITIAL_MODULE.zip UPGRADE_MODULE.zip" >&2
  exit 2
fi

exec bwrap --unshare-user --unshare-pid --die-with-parent --new-session --cap-drop ALL \
  --ro-bind /usr /usr --symlink usr/bin /bin --symlink usr/lib /lib --symlink usr/lib64 /lib64 \
  --dev /dev --tmpfs /tmp --tmpfs /data \
  --ro-bind "$(realpath "$1")" /initial.zip --ro-bind "$(realpath "$2")" /upgrade.zip \
  --clearenv --setenv PATH /usr/bin:/bin --chdir /tmp /bin/sh -e <<'SMOKE'
ui_print() { printf '%s\n' "$*"; }
abort() { ui_print "$*" >&2; exit 1; }
grep_prop() { sed -n "s/^$1=//p" "$2"; }
getprop() { [ "$1" = ro.product.cpu.abilist ] && echo arm64-v8a,armeabi-v7a; }
unzip() { bsdunzip "$@"; }
sha256sum() {
  if [ "$1" = -c ] && [ "$2" = -s ]; then
    shift 2
    command sha256sum --check --status "$@"
  else
    command sha256sum "$@"
  fi
}
set_perm_recursive() {
  find "$1" -type d -exec chmod "$4" {} +
  find "$1" -type f -exec chmod "$5" {} +
}

active=/data/adb/modules/zygisknextsu
staged=/data/adb/modules_update/zygisknextsu
config=/data/adb/zygisknextsu/config
mkdir -p /data/adb/modules /data/adb/modules_update

install_zip() (
  ZIPFILE=$1
  MODPATH=$staged
  TMPDIR=$(mktemp -d)
  BOOTMODE=true
  MAGISK_VER_CODE=99999
  API=35
  ARCH=arm64
  IS64BIT=true
  mkdir "$MODPATH"
  unzip -o "$ZIPFILE" customize.sh module.prop -d "$TMPDIR"
  set +e
  . "$TMPDIR/customize.sh"
)

check_member() {
  unzip -p "$1" "$2" >/tmp/expected-member
  cmp /tmp/expected-member "$3"
}

check_install() {
  check_member "$1" module.prop "$2/module.prop"
  cmp "$2/module.prop" "$2/module.prop.orig"
  check_member "$1" bin/arm64-v8a/znnd "$2/bin/znnd"
  check_member "$1" lib/arm64-v8a/libzn_loader.so "$2/lib64/libzn_loader.so"
  check_member "$1" lib/armeabi-v7a/libzn_loader.so "$2/lib/libzn_loader.so"
  check_member "$1" webroot/index.html "$2/webroot/index.html"
  test -x "$2/bin/znnd"
}

install_zip /initial.zip
mv "$staged" "$active"
check_install /initial.zip "$active"

mkdir -p "$(dirname "$config")"
printf 'mode=proc\nhot_update=true\n' >/tmp/expected-config
cp /tmp/expected-config "$config"

cp /upgrade.zip /tmp/damaged.zip
printf 'damaged\n' >bugreport.sh
7z a -tzip -bso0 /tmp/damaged.zip bugreport.sh
if install_zip /tmp/damaged.zip >/tmp/rejected.log 2>&1; then
  abort "installer accepted an archive with a damaged bugreport.sh"
fi
grep -F 'Failed to verify bugreport.sh' /tmp/rejected.log
cmp /tmp/expected-config "$config"
check_install /initial.zip "$active"
rm -rf "$staged"

install_zip /upgrade.zip
cmp /tmp/expected-config "$config"
check_install /initial.zip "$active"
mv "$active" /data/adb/previous-module
mv "$staged" "$active"
check_install /upgrade.zip "$active"
check_member /upgrade.zip bugreport.sh "$active/bugreport.sh"
cmp /tmp/expected-config "$config"
echo "ARM64 install, checksum rejection and upgrade passed"
SMOKE
