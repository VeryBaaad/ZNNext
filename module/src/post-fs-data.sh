#!/system/bin/sh

MODDIR=${0%/*}

cd "$MODDIR" || exit 1

mkdir -p /data/adb/zygisknextsu

./bin/znnd "$MODDIR" &
./bin/znnd --ctl apply-zn wait >/dev/null 2>&1 &
