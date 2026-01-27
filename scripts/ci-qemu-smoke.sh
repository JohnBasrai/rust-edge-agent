#!/usr/bin/env bash
set -euo pipefail

SYSROOT="/usr/aarch64-linux-gnu/lib"

ls -la "$SYSROOT/ld-linux-aarch64.so.1" \
       "$SYSROOT/libc.so.6" \
       "$SYSROOT/libstdc++.so.6" \
       "target/aarch64-unknown-linux-gnu/release/rust-edge-agent"

BIN="$(find . -path '*/target/aarch64-unknown-linux-gnu/release/rust-edge-agent' -print -quit)"

test -n "$BIN" || { echo "binary not found"; exit 1; }
echo "$0: BIN=$BIN"

echo "$0: === Inspecting ARM64 binary ==="
file "$BIN"

readelf -h "$BIN" | grep Machine
readelf -d "$BIN"
readelf -l "$BIN" | grep INTERP
chmod +x "$BIN"

echo "$0: === Running ARM64 binary under QEMU ==="

OUT="$(QEMU_LD_DEBUG=libs qemu-aarch64 -L /usr/aarch64-linux-gnu \
       target/aarch64-unknown-linux-gnu/release/rust-edge-agent)"

echo "$OUT"

echo "$OUT" | grep -q "Hello world from aarch64!"
