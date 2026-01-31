# rust-edge-agent

[![CI](https://github.com/JohnBasrai/rust-edge-agent/actions/workflows/ci.yml/badge.svg)](https://github.com/JohnBasrai/rust-edge-agent/actions/workflows/ci.yml)

This project demonstrates edge gateway patterns through cross-compilation, NATS messaging, and device coordination. Phase 1 implements device simulators and command routing to establish gateway architecture foundations.

Current state: Working NATS-based edge agent with device simulators demonstrating sensor/actuator patterns, command routing, and telemetry aggregation.

## What this is

- A **Linux-based edge agent** intended for embedded or gateway-class systems
- Built in **Rust**, targeting **AArch64 (ARM64)** via cross-compilation
- Designed to run on **embedded Linux**, not bare metal
- Uses **NATS** for edge ↔ backend messaging
- Built with **Cargo as the primary build system**, with **Bazel as an orchestration layer**
- Validated using **QEMU user-mode emulation** to execute AArch64 binaries on an x86_64 host
- Includes consideration of:
  - toolchain correctness
  - failure modes and reconnect behavior
  - messaging semantics
  - build reproducibility

## What this is not

- Not bare-metal firmware
- Not a BSP, bootloader, or kernel project
- Not Android or AOSP implementation code
- Not a device driver or HAL
- Not a microcontroller demo (ESP32, Arduino, etc.)
- Not intended to showcase UI, cloud dashboards, or mobile clients

This repository intentionally avoids expanding into those areas in order to keep the scope constrained.

## Important Files

* `docs/architecture.md`
  High-level diagram + text:

  * agent lifecycle
  * messaging flow
  * reconnect behavior

---

## Quick Start

### Prerequisites
- Docker (for NATS broker)
- Rust toolchain (see `rust-toolchain.toml`)
- For cross-compilation: `gcc-aarch64-linux-gnu` and `qemu-user`

### Running the Demo

**1. Start NATS broker**
```bash
./scripts/service-start.sh
```

**2. Build the project**
```bash
cargo build --release
```

**3. Run the demo** (starts edge agent + 3 device simulators)
```bash
./scripts/demo.sh
```

The demo starts:
- Edge agent listening for device telemetry and backend commands
- 3 device simulators (sensor, actuator, hybrid modes)

**Monitor telemetry:**
```bash
nats sub 'backend.telemetry'
```

**Send command to actuator:**
```bash
nats req 'backend.command.device-002' '{"target_value": 75.0}'
```

**4. Cleanup**
```bash
./scripts/service-stop.sh
```

---

## Cross-compilation smoke test

Before introducing agent logic, this repository verifies that an AArch64 (ARM64) binary can be built on an x86_64 host and executed using QEMU user-mode emulation.

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

### Execute under QEMU

```
qemu-aarch64 -L /usr/aarch64-linux-gnu \
    target/aarch64-unknown-linux-gnu/release/rust-edge-agent
```

Expected output:

```
Hello world from aarch64!
```

This smoke test verifies:

* Rust cross-compilation to `AArch64`
* Correct linker and sysroot configuration
* Ability to execute `AArch64` binaries using QEMU user-mode emulation

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
  - Produces an ARM64 Linux ELF binary:

    `target/aarch64-unknown-linux-gnu/release/rust-edge-agent`

* `scripts/ci-qemu-smoke.sh`
  - Executes the ARM64 binary using QEMU user-mode emulation
  - Validates runtime correctness against an explicit ARM64 sysroot
  - Fails if the binary does not successfully execute

These scripts are designed to be runnable locally and are used directly by CI.
