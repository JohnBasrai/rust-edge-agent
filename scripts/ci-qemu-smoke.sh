#!/usr/bin/env bash
set -euo pipefail

: ${DEBUG:=}

SYSROOT="/usr/aarch64-linux-gnu/lib"

if [ -n "${DEBUG}" ] ; then
    ls -la "$SYSROOT/ld-linux-aarch64.so.1" \
       "$SYSROOT/libc.so.6" \
       "$SYSROOT/libstdc++.so.6" \
       "target/aarch64-unknown-linux-gnu/release/rust-edge-agent"
fi

BIN="$(find . -path '*/target/aarch64-unknown-linux-gnu/release/rust-edge-agent' -print -quit)"

if [ -n "${DEBUG}" ] ; then
    echo "$0: BIN=$BIN"
    echo "$0: === Inspecting ARM64 binary ==="
    file "$BIN"

    readelf -h "$BIN" | grep Machine
    readelf -d "$BIN"
    readelf -l "$BIN" | grep INTERP
    export QEMU_LD_DEBUG=libs
    echo "$0: === Running ARM64 binary under QEMU ==="
fi
chmod +x "$BIN"
set +e
OUT="$(timeout 5s qemu-aarch64 -L /usr/aarch64-linux-gnu "$BIN")"

status=$?

echo "$OUT"

if [ "$status" != 124 ]; then
    echo "$0: test failed"
    exit 1
fi
echo "$0: test passed"
exit 0
