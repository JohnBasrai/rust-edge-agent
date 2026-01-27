#!/usr/bin/env bash
set -euo pipefail

rustup target add aarch64-unknown-linux-gnu

cargo build --release --target aarch64-unknown-linux-gnu
