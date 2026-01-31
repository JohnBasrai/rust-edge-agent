#!/usr/bin/env bash
set -euo pipefail

cargo fmt --check
cargo clippy --release --all-targets --all-features -- -D warnings \
  -D warnings \
  -D clippy::unwrap-used \
  -D clippy::expect_used \
  -D clippy::indexing_slicing \
  -D clippy::panic $*
