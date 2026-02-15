# rust-edge-agent

[![CI](https://github.com/JohnBasrai/rust-edge-agent/actions/workflows/ci.yml/badge.svg)](https://github.com/JohnBasrai/rust-edge-agent/actions/workflows/ci.yml)

This repository explores design patterns for an edge gateway responsible for coordinating heterogeneous devices via a brokered control plane. The focus is on message-based command routing, device lifecycle tracking, and telemetry aggregation, with an emphasis on correctness, observability, and deployability across architectures.

Current state: Working dual-protocol edge agent with NATS backend and MQTT device communication. Demonstrates sensor/actuator/hybrid device patterns, dynamic device registration via RPC, command routing, and telemetry aggregation. Protocol bridge validated under AArch64 cross-compilation and QEMU emulation.

---

## 1.0 Scope and Intent

This project is intentionally scoped to explore edge gateway coordination patterns rather than end-device protocols.

The gateway coordinates heterogeneous devices through a brokered control plane, handling command routing, telemetry aggregation, and device lifecycle tracking. The implementation emphasizes correctness and explicit state management over throughput or protocol coverage.

Cross-compilation to `AArch64` and execution under `QEMU` are used to validate that the system behaves correctly as a long-running service across architectures.

---

## 2.0 What this is

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

---

## 3.0 What this is not

- Not bare-metal firmware
- Not a BSP, bootloader, or kernel project
- Not Android or AOSP implementation code
- Not a device driver or HAL
- Not a microcontroller demo (ESP32, Arduino, etc.)
- Not intended to showcase UI, cloud dashboards, or mobile clients

This repository intentionally avoids expanding into those areas in order to keep the scope constrained.

---

## 4.0 What This Demonstrates

This project is intentionally scoped to demonstrate edge gateway patterns that commonly appear in real deployments:

- Coordinating heterogeneous devices (sensor, actuator, hybrid) behind a
  single control plane
- Separating device concerns from transport concerns using a message broker
- Handling intermittent devices via timeouts and retry semantics
- Designing long-running edge services that can be cross-compiled and
  validated under emulation (`AArch64` + `QEMU`)

---

## 5.0 Architecture Overview

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

---

## 6.0 Important Files

* [docs/architecture.md](docs/architecture.md)
  High-level diagram + text:

  * agent lifecycle
  * messaging flow
  * reconnect behavior

---

## 7.0 Quick Start

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
./scripts/stop-services.sh  # stop them if already running (harmless if not running)
./scripts/start-services.sh
```

This launches NATS on port 4222 and an MQTT broker on port 1883 inside Docker containers.

**2. Build the project**

```bash
./scripts/ci-build-native.sh
```

**3. Run the demo** (starts edge agent + 3 device simulators)

```bash
./scripts/demo.sh
```

Let it run for about 30 seconds, then press Control-C to stop the demo.

**4. Stop the demo**

```bash
Control-C # Stops demo
```

**5. Stop infrastructure services**

```bash
./scripts/stop-services.sh
```
The demo starts the edge agent (acting as a NATS ↔ MQTT bridge), listening for device registrations. Three device simulators will launch (actuator, hybrid, sensor), register via RPC, and the agent will poll sensors every 5 seconds, forwarding telemetry to the NATS backend.

Expected output includes device registrations and telemetry updates, and you can monitor or send commands via NATS as shown.

<details>
<summary><b>Expected test output</b></summary>

```
 $ ./scripts/stop-services.sh 
Stopping NATS broker...
Stopping MQTT broker...
 $ ./scripts/start-services.sh 
Starting NATS broker on localhost:4222
de1472c8c7e4f1088f204a1831ce2d034d587bad7ed527d2f087750aa371783b
Starting mosquitto broker on localhost:MQTT_PORT
f92d3ff7f5385c889fca944f0333335f2eecc97a8bfaa81c428d2a7312ed0cce
 $ ./scripts/ci-build-native.sh 
    Finished `release` profile [optimized] target(s) in 0.09s
 $ ./scripts/demo.sh 
./scripts/demo.sh: NUM_DEVICES     : 3
./scripts/demo.sh: DEVICE_INTERVAL : 5
./scripts/demo.sh: MODE            : interactive
./scripts/demo.sh: Checking for stale processes...
./scripts/demo.sh: Started agent (PID: 726495)
agent:: Starting edge agent...
agent:: NATS URL: nats://localhost:4222
agent:: MQTT Broker: mqtt://localhost:1883
agent:: Device timeout: 30s
agent:: Poll interval: 5s
Connected to NATS at nats://localhost:4222
agent:: Connecting to MQTT broker...
2026-02-15T03:26:35.304702Z  INFO async_nats: event: connected
Connected to MQTT broker at mqtt://localhost:1883 (client_id: agent-transport)
agent:: MQTT transport ready
agent:: Creating RPC server...
agent:: Creating RPC client...
2026-02-15T03:26:35.305268Z  INFO mom_rpc::transport::rumqttc::transport: rumqttc: connected to broker
2026-02-15T03:26:35.305372Z  INFO mom_rpc::transport::rumqttc::transport: rumqttc: successfully subscribed to topic responses/agent-client
agent:: RPC client ready
agent:: RPC server listening for device registrations
agent:: Edge agent running
2026-02-15T03:26:35.305489Z  INFO mom_rpc::transport::rumqttc::transport: rumqttc: successfully subscribed to topic requests/agent
./scripts/demo.sh: Starting device: 1 mode:actuator   TYPE:valve       
./scripts/demo.sh: Starting device: 2 mode:hybrid     TYPE:propulsion  
./scripts/demo.sh: Starting device: 3 mode:sensor     TYPE:temp        
Edge agent and 3 devices running.

Monitor telemetry:
  nats sub 'backend.telemetry'

Send command to actuator:
  nats req 'backend.command.device-002' '{"target_value": 75.0}'

Press Ctrl+C to stop.
agent:: Device registered: device-001 (mode: Actuator, type: Valve)
agent:: Device registered: device-002 (mode: Hybrid, type: Propulsion)
agent:: Device registered: device-003 (mode: Sensor, type: Temperature)
agent:: Telemetry: device-002 (Propulsion) = 20.26
agent:: Telemetry: device-003 (Temperature) = 20.21
agent:: Telemetry: device-002 (Propulsion) = 21.05
agent:: Telemetry: device-003 (Temperature) = 21.20
agent:: Telemetry: device-002 (Propulsion) = 21.75
agent:: Telemetry: device-003 (Temperature) = 21.65
agent:: Telemetry: device-002 (Propulsion) = 21.99
agent:: Telemetry: device-003 (Temperature) = 21.75
agent:: Telemetry: device-002 (Propulsion) = 21.85
agent:: Telemetry: device-003 (Temperature) = 21.17
agent:: Telemetry: device-002 (Propulsion) = 21.51
agent:: Telemetry: device-003 (Temperature) = 21.82
^C
./scripts/demo.sh: Cleaning up...
./scripts/demo.sh: Stopping agent (PID: 726495)...
[1]   Terminated              ./target/release/rust-edge-agent
./scripts/demo.sh: Stopping device (PID: 726525)...
[2]   Terminated              ./target/release/device_sim --id "device-$(printf "%03d" $i)" --mode $MODE_ARG --type $TYPE_ARG --interval $DEVICE_INTERVAL >&/dev/null
./scripts/demo.sh: Stopping device (PID: 726526)...
[3]-  Terminated              ./target/release/device_sim --id "device-$(printf "%03d" $i)" --mode $MODE_ARG --type $TYPE_ARG --interval $DEVICE_INTERVAL >&/dev/null
./scripts/demo.sh: Stopping device (PID: 726528)...
[4]+  Terminated              ./target/release/device_sim --id "device-$(printf "%03d" $i)" --mode $MODE_ARG --type $TYPE_ARG --interval $DEVICE_INTERVAL >&/dev/null
./scripts/demo.sh: Cleanup complete
 $ ./scripts/stop-services.sh 
Stopping NATS broker...
nats_svc
nats_svc
Stopping MQTT broker...
mqtt_svc
mqtt_svc
 $ 
```

</details>

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
./scripts/start-services.sh
```

---

## 8.0 Cross-compilation smoke test

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

---

## 9.0 Build and validation workflow

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
  - **Note:** When running locally, start services first with `./scripts/start-services.sh`

These scripts are designed to be runnable locally and are used directly by CI.

**Expected output of smoke test**

```
 $ ./scripts/start-services.sh 
Starting NATS broker on localhost:4222
b269c86b88431ddee1cda182fea52e6198d19bd4d812ede692c2f9df02cc4a6b
Starting mosquitto broker on localhost:MQTT_PORT
7c89fea656922989e19bc969507bd32873501b78f78ba1e805636096b0f43229
 $ ./scripts/ci-qemu-smoke.sh 
agent:: Starting edge agent...
agent:: NATS URL: nats://localhost:4222
agent:: MQTT Broker: mqtt://localhost:1883
agent:: Device timeout: 30s
agent:: Poll interval: 5s
Connected to NATS at nats://localhost:4222
agent:: Connecting to MQTT broker...
Connected to MQTT broker at mqtt://localhost:1883 (client_id: agent-transport)
agent:: MQTT transport ready
agent:: Creating RPC server...
agent:: Creating RPC client...
agent:: RPC client ready
agent:: RPC server listening for device registrations
agent:: Edge agent running
2026-02-15T03:35:00.364392Z  INFO async_nats: event: connected
2026-02-15T03:35:00.384464Z  INFO mom_rpc::transport::rumqttc::transport: rumqttc: connected to broker
2026-02-15T03:35:00.387569Z  INFO mom_rpc::transport::rumqttc::transport: rumqttc: successfully subscribed to topic responses/agent-client
2026-02-15T03:35:00.393665Z  INFO mom_rpc::transport::rumqttc::transport: rumqttc: successfully subscribed to topic requests/agent
./scripts/ci-qemu-smoke.sh: test passed
 $ 
```
