# rust-edge-agent

[![CI](https://github.com/JohnBasrai/rust-edge-agent/actions/workflows/ci.yml/badge.svg)](https://github.com/JohnBasrai/rust-edge-agent/actions/workflows/ci.yml)

A Rust-based embedded Linux edge agent targeting ARM64 (AArch64), focused on cross-compilation, messaging primitives, and reproducible build workflows.

This project explores how to build and reason about **edge systems as constrained distributed systems**, rather than firmware or device drivers.

## What this is

- A **Linux-based edge agent** intended for embedded or gateway-class systems
- Built in **Rust**, targeting **AArch64 (ARM64)** via cross-compilation
- Designed to run on **embedded Linux**, not bare metal
- Uses **NATS** for edge ↔ backend messaging
- Built with **Cargo as the primary build system**, with **Bazel as an orchestration layer**
- Validated using **QEMU user-mode emulation** to execute AArch64 binaries on an x86_64 host
- Focused on:
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

This repository intentionally avoids expanding into those areas in order to keep the scope focused and deep.

## Important Files

* `docs/architecture.md`
  High-level diagram + text:

  * agent lifecycle
  * messaging flow
  * reconnect behavior

* `docs/aosp-context.md`
  Short, non-code explanation:

  * where this would live in an Android-based system
  * why it’s out of scope here

* `build/Dockerfile.cross`
  Encapsulates:

  * aarch64 toolchain
  * sysroot
  * reproducible builds

---

* `messaging/`
* `runtime/`
* `build/`
* `docs/`
* `bazel/`

---

## Cross-compilation smoke test (Phase 2.0)

Before introducing agent logic, this repository verifies that an AArch64 (ARM64) binary can be built on an x86_64 host and executed using QEMU user-mode emulation.

This establishes toolchain correctness before any higher-level functionality is introduced.

### Host prerequisites (Ubuntu/Debian)

```
sudo apt update
sudo apt install gcc-aarch64-linux-gnu qemu-user  # Install quem-user
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
Hello, world from AArch64!
```

This smoke test verifies:

* Rust cross-compilation to `AArch64`
* Correct linker and sysroot configuration
* Ability to execute `AArch64` binaries using QEMU user-mode emulation

All subsequent development builds on this foundation.
