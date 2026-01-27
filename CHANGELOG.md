## [0.1.0] – 2026-01-27

### Added
- Initial `rust-edge-agent` implementation targeting embedded Linux / ARM64.
- Cross-compilation support for `aarch64-unknown-linux-gnu`.
- QEMU-based smoke test validating ARM64 binaries on x86_64 CI runners.
- Deterministic Rust toolchain via `rust-toolchain.toml`.

### CI
- Parallelized GitHub Actions workflow:
  - Lint (fmt + clippy)
  - Native build
  - AArch64 cross build
  - QEMU smoke test
- Explicit artifact handoff between build and smoke-test jobs.
- Hardened shell scripts using `set -uo pipefail`.
- Output-based QEMU smoke test (behavioral validation, not syscall noise).

### Tooling
- Removed redundant Rust setup steps in CI in favor of repo-pinned toolchain.
- Added explicit permission handling for cross-built artifacts in CI.

### Notes
- This release establishes the baseline for future embedded and edge-focused features.
