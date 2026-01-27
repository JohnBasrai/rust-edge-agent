#!/usr/bin/env bash
set -euo pipefail

OUT="$(qemu-aarch64 -L /usr/aarch64-linux-gnu \
  target/aarch64-unknown-linux-gnu/release/rust-edge-agent)"

echo "$OUT"

echo "$OUT" | grep -q "Hello world from aarch64!"
