# rust-edge-agent

A Rust-based embedded Linux edge agent designed to run on ARM64 (AArch64) systems, with a focus on cross-compilation, messaging, and build discipline.

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
