## [Unreleased]

## [0.2.0] – 2026-01-31

### Added
- **Device simulator** (`device_sim` binary) supporting three modes:
  - Sensor: Publish-only telemetry (temperature, humidity)
  - Actuator: Command handling with status publishing (valve, propulsion)
  - Hybrid: Both telemetry and command capabilities
- **NATS-based messaging architecture**:
  - Request/reply for actuator commands with timeout handling
  - Pub/sub for telemetry forwarding
  - Subject-based routing (`devices.*`, `backend.*`)
- **Device registry** with state tracking and configurable timeout detection
- **Exponential backoff connection retry** (1s → 2s → 4s → 8s → 16s → 30s cap)
  - Uses bit-shift implementation for efficient exponential calculation
  - Resilient reconnection for embedded systems without UI
- **Service management scripts**:
  - `service-start.sh` - Start NATS broker in Docker
  - `service-stop.sh` - Stop and remove NATS broker
  - `demo.sh` - Interactive demo with configurable device count
- **CONTRIBUTING.md** with comprehensive guidelines:
  - Code formatting conventions (`// ---` separators)
  - Documentation standards for messaging/edge systems
  - EMBP (Explicit Module Boundary Pattern) architecture reference
  - Testing strategy and coverage expectations

### Changed
- **Applied EMBP architecture pattern** throughout codebase:
  - Private module declarations with gateway exports
  - Sibling imports via `super::`, external via `crate::`
  - Messaging module serves as public API gateway
- **CLI argument parsing** via CLAP with environment variable fallbacks:
  - `--nats-url` / `NATS_URL` (default: `nats://localhost:4222`)
  - `--device-timeout` / `DEVICE_TIMEOUT` (default: 30s)
  - `--interval` / `DEVICE_INTERVAL` for device simulator
- **Async runtime** using Tokio for agent and device simulators

### Documentation
- **README.md**: Added Quick Start guide and demo instructions
- **docs/architecture.md**: Describes gateway vs leaf device patterns
- **CONTRIBUTING.md**: Production-grade documentation standards and EMBP patterns

### Architecture Decisions
- **NATS over MQTT**: Chose NATS for request/reply semantics and simpler implementation
  - Avoids MQTT correlation ID complexity (deferred to Phase 2 as `mqtt-rpc` library)
- **Forward raw telemetry**: Edge agent forwards telemetry without aggregation
  - Backend has compute/storage for aggregation; keeps edge agent simple
- **Gateway pattern**: Demonstrates coordination between devices and backend
  - Not a leaf device - maintains state, routes bidirectionally

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
