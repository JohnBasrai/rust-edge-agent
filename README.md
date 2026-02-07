# rust-edge-agent

[![CI](https://github.com/JohnBasrai/rust-edge-agent/actions/workflows/ci.yml/badge.svg)](https://github.com/JohnBasrai/rust-edge-agent/actions/workflows/ci.yml)

This repository explores design patterns for an edge gateway responsible for coordinating heterogeneous devices via a brokered control plane. The focus is on message-based command routing, device lifecycle tracking, and telemetry aggregation, with an emphasis on correctness, observability, and deployability across architectures.

Current state: Working dual-protocol edge agent with NATS backend and MQTT device communication. Demonstrates sensor/actuator/hybrid device patterns, dynamic device registration via RPC, command routing, and telemetry aggregation. Protocol bridge validated under AArch64 cross-compilation and QEMU emulation.

## Scope and Intent

This project is intentionally scoped to explore edge gateway coordination patterns rather than end-device protocols.

The gateway coordinates heterogeneous devices through a brokered control plane, handling command routing, telemetry aggregation, and device lifecycle tracking. The implementation emphasizes correctness and explicit state management over throughput or protocol coverage.

Cross-compilation to `AArch64` and execution under `QEMU` are used to validate that the system behaves correctly as a long-running service across architectures.

## What this is

- A **Linux-based edge agent** intended for embedded or gateway-class systems
- Built in **Rust**, targeting **`AArch64` (`ARM64`)** via cross-compilation
- Designed to run on **embedded Linux**, not bare metal
- **Dual-protocol architecture:**
  - Uses **NATS** for edge ↔ backend messaging (cloud communication)
  - Uses **MQTT + mom-rpc** for agent ↔ device messaging (local edge communication)
- Built with **Cargo as the primary build system**
- Validated using **`QEMU` user-mode emulation** to execute `AArch64` binaries on an x86_64 host
- Includes consideration of:
  - toolchain correctness
  - failure modes and reconnect behavior
  - messaging semantics (pub/sub and RPC)
  - protocol bridging (NATS ↔ MQTT)
  - build reproducibility

## What this is not

- Not bare-metal firmware
- Not a BSP, bootloader, or kernel project
- Not Android or AOSP implementation code
- Not a device driver or HAL
- Not a microcontroller demo (ESP32, Arduino, etc.)
- Not intended to showcase UI, cloud dashboards, or mobile clients

This repository intentionally avoids expanding into those areas in order to keep the scope constrained.

## What This Demonstrates

This project is intentionally scoped to demonstrate edge gateway patterns that commonly appear in real deployments:

- Coordinating heterogeneous devices (sensor, actuator, hybrid) behind a
  single control plane
- Separating device concerns from transport concerns using a message broker
- Handling intermittent devices via timeouts and retry semantics
- Designing long-running edge services that can be cross-compiled and
  validated under emulation (`AArch64` + `QEMU`)

## Architecture Overview

This project demonstrates a **dual-protocol edge gateway** architecture:

- **Backend ↔ Agent:** NATS messaging (cloud/REST API communication)
- **Agent ↔ Devices:** MQTT + mom-rpc (local edge device communication)

The edge agent acts as a protocol bridge, translating between cloud services (NATS) and edge devices (MQTT).

### Message Flows

**Telemetry (Device → Backend):**
```
Sensor/Hybrid Device → [MQTT RPC] → Agent → [NATS] → Backend
```

**Commands (Backend → Device):**
```
Backend → [NATS] → Agent → [MQTT RPC] → Actuator/Hybrid Device
```

**Device Registration:**
```
Device → [MQTT RPC] → Agent (register-device method)
```

### Device Modes

- **Sensor:** Produces telemetry, no command execution
- **Actuator:** Executes commands, no telemetry
- **Hybrid:** Both telemetry and command execution

### Key Design Decisions

1. **Transport-agnostic RPC:** Uses `mom-rpc` for RPC semantics over MQTT, avoiding hand-coded request/response correlation
2. **Polling-based telemetry:** Agent polls sensors every 5 seconds (configurable via `POLL_INTERVAL`)
3. **Dynamic device registration:** Devices register at runtime via RPC, no static configuration
4. **Dual-protocol bridge:** Separates concerns - NATS for reliable cloud messaging, MQTT for local device communication

See [docs/architecture.md](docs/architecture.md) for complete details including message formats, RPC methods, and sequence diagrams.

## Important Files

* [docs/architecture.md](docs/architecture.md)
  High-level diagram + text:

  * agent lifecycle
  * messaging flow
  * reconnect behavior

---

## Quick Start

### Prerequisites

**Required:**
- Docker (for NATS and MQTT brokers)
- Rust (stable, see `rust-toolchain.toml`)
- For cross-compilation: `gcc-aarch64-linux-gnu` and `qemu-user`

**Optional (for manual testing):**
- `natscli` for NATS command-line operations

**Install on Ubuntu/Debian:**
```bash
sudo apt-get update
sudo apt-get install -y \
  docker.io \
  qemu-user \
  gcc-aarch64-linux-gnu \
  libc6-arm64-cross

# Optional: NATS CLI
sudo apt-get install -y natscli
```

**Verify Docker is running:**
```bash
sudo systemctl start docker
sudo systemctl enable docker
```

**macOS users** may install equivalents via Homebrew, but CI and official support assume a Debian-based Linux environment.

**Ports required:**
- `4222` - NATS server (backend communication)
- `1883` - MQTT broker (device communication)

### Running the Demo

**1. Start infrastructure services**
```bash
./scripts/service-start.sh
```
This starts NATS (port 4222) and MQTT broker (port 1883) in Docker containers.

**2. Build the project**
```bash
./scripts/ci-build-native.sh
```

**3. Clean up any stale processes from previous runs**
```bash
pkill -9 rust-edge-agent device_sim 2>/dev/null || true
```

**4. Run the demo** (starts edge agent + 3 device simulators)
```bash
./scripts/demo.sh
```

The demo starts:
- Edge agent (NATS ↔ MQTT bridge) listening for device registrations
- 3 device simulators (actuator, hybrid, sensor modes)
- Devices register via MQTT RPC
- Agent polls sensors every 5 seconds and forwards telemetry to NATS backend

**Expected output:**
```
agent:: Edge agent running
agent:: Device registered: device-001 (mode: Actuator, type: Valve)
agent:: Device registered: device-002 (mode: Hybrid, type: Propulsion)
agent:: Device registered: device-003 (mode: Sensor, type: Temperature)
agent:: Telemetry: device-003 (Temperature) = 20.94
agent:: Telemetry: device-002 (Propulsion) = 19.14
```

**Monitor telemetry in a separate terminal:**
```bash
nats sub 'backend.telemetry'
```

**Send command to actuator/hybrid device:**
```bash
# Command to actuator
nats req 'backend.command.device-001' '{"target_value": 75.0}'

# Command to hybrid device  
nats req 'backend.command.device-002' '{"target_value": 50.0}'
```

**5. Stop the demo**

Press `Ctrl+C` in the terminal running demo.sh, then manually clean up processes:
```bash
pkill -9 rust-edge-agent device_sim
```

**Note:** The demo cleanup trap has known issues with signal handling. Manual process cleanup is required after stopping the demo. This will be addressed in a future update.

**6. Stop infrastructure services**
```bash
./scripts/service-stop.sh
```

### Troubleshooting

**Problem: Demo hangs or devices can't register**

**Cause:** Stale processes from previous runs using the same MQTT client IDs.

**Solution:**
```bash
# Kill all stale processes
pkill -9 rust-edge-agent device_sim

# Verify they're gone
ps aux | grep -E 'rust-edge-agent|device_sim' | grep -v grep

# Run demo again
./scripts/demo.sh
```

**Problem: MQTT connection errors**

**Cause:** MQTT broker (mosquitto) not running.

**Solution:**
```bash
# Check if mosquitto is running
docker ps | grep mosquitto

# If not, start services
./scripts/service-start.sh
```

---

## Cross-compilation smoke test

Before introducing agent logic, this repository verifies that an `AArch64` (`ARM64`) binary can be built on an x86_64 host and executed using `QEMU` user-mode emulation.

This establishes toolchain correctness before any higher-level functionality is introduced.

### Host prerequisites (Ubuntu/Debian)

```
sudo apt update
sudo apt install gcc-aarch64-linux-gnu qemu-user  # Install qemu-user
rustup target add aarch64-unknown-linux-gnu       # Add the Rust target
cargo build --release \
    --target aarch64-unknown-linux-gnu # Build for ARM64
```

This produces the following binary:

```
target/aarch64-unknown-linux-gnu/release/rust-edge-agent
```

### Execute under `QEMU`

```
qemu-aarch64 -L /usr/aarch64-linux-gnu \
    target/aarch64-unknown-linux-gnu/release/rust-edge-agent
```

**Expected behavior:**

The binary executes successfully under `QEMU` and begins normal agent startup. The process is expected to continue running until terminated. This initial smoke test was introduced early to validate toolchain
correctness before agent logic was added.

This smoke test verifies:

* Rust cross-compilation to `AArch64`
* Correct linker and sysroot configuration
* Ability to execute `AArch64` binaries using `QEMU` user-mode emulation

Subsequent development assumes this baseline.

## Build and validation workflow

This repository uses small, explicit shell scripts to encode build and validation steps. These scripts are used both locally and in CI to avoid divergence between developer workflows and automated checks.

The scripts are intentionally minimal and do not hide Cargo or toolchain behavior behind wrappers.

### CI-aligned scripts

The following scripts live under `scripts/` and are invoked directly by GitHub Actions:

* `scripts/ci-lint.sh`
  - Runs `cargo fmt` and `cargo clippy`
  - Enforces formatting and basic correctness
  - No build artifacts are produced

* `scripts/ci-build-native.sh`
  - Builds the agent for the host architecture
  - Verifies that the code continues to build natively as the project evolves

* `scripts/ci-build-aarch64.sh`
  - Cross-compiles the agent for `aarch64-unknown-linux-gnu`
  - Produces an `ARM64` Linux ELF binary:

    `target/aarch64-unknown-linux-gnu/release/rust-edge-agent`

* `scripts/ci-qemu-smoke.sh`
  - Executes the `ARM64` binary using `QEMU` user-mode emulation
  - Validates runtime correctness against an explicit `ARM64` sysroot
  - Fails if the binary does not successfully execute

These scripts are designed to be runnable locally and are used directly by CI.
