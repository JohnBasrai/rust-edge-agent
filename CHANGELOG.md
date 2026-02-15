## [Unreleased]


## [0.3.2] - 2026-02-15

### Fixed
- Add missing services startup to CI qemu-smoke job
- Fix demo.sh signal handling for clean process cleanup
- Fix MQTT_PORT interpolation in start-services.sh

### Changed
- Update mom-rpc to 0.7.3
- Replace env_logger with tracing-subscriber
- Disable ANSI codes in logs for better script output
- Redirect device_sim output to /dev/null in demo

### Documentation
- Remove manual cleanup steps from README
- Add smoke test output example
- Rename service scripts throughout documentation


## [0.3.1] - 2026-02-07

### Changed

- Enable `clippy::uninlined_format_args` lint and convert format strings to inline syntax
- Replace `std::process::exit()` with idiomatic `Result`/`bail!()` error handling
- Add package metadata (keywords, categories, description)


## [0.3.0] - 2026-02-06

### Changed
- **BREAKING**: Replaced NATS device communication with MQTT + mom-rpc
  - Devices now connect via MQTT instead of NATS
  - Uses mom-rpc v0.3.0 for RPC semantics over MQTT
  - Agent acts as NATS (backend) ↔ MQTT (devices) bridge
  - Requires MQTT broker (mosquitto) on port 1883

### Added
- Dependencies: `mom-rpc = "0.3.0"`, `env_logger = "0.11"`
- Environment variable: `MQTT_BROKER_URL` (default: `mqtt://localhost:1883`)
- Enhanced startup logging for debugging
- Pre-flight stale process check in demo script

### Infrastructure
- `service-start.sh` now starts both NATS and MQTT brokers
- Docker compose includes mosquitto container


### Migration from v0.2.x
- MQTT broker required (port 1883): `./scripts/service-start.sh`
- Set `MQTT_BROKER_URL` environment variable if not using default
- Old NATS-only device code incompatible - devices must use MQTT
- Backend/cloud API unchanged (still uses NATS)

### Known Issues
- demo.sh cleanup trap needs manual `pkill -9 rust-edge-agent device_sim` after Ctrl+C

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
