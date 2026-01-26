# Architecture Overview

This document describes the architecture of `rust-edge-agent`, a Rust-based embedded Linux edge agent designed for ARM64 (AArch64) systems.

The goal of this project is to explore **edge gateway systems design** with realistic constraints: cross-compilation, messaging semantics, failure modes, and build discipline—without expanding into firmware, Android internals, or hardware-specific concerns.

---

## System Role

`rust-edge-agent` represents a **gateway-class embedded component**, not a leaf device.

It is intended to run on Linux-based embedded systems (e.g. vehicle gateways, telematics units, or edge compute nodes) that sit between constrained devices and backend services.

Key characteristics:

- Runs on embedded Linux
- Maintains local state
- Communicates bidirectionally with backend systems
- Aggregates and forwards telemetry
- Accepts and applies control commands
- Operates in unreliable network conditions

This differs from **leaf devices** (e.g. sensors, cameras, doorbells), which typically publish telemetry only and often use MQTT-style pub/sub without request/response semantics.

---

## High-Level Architecture

```

         ┌──────────────────┐
         │ Backend Systems  │
         │                  │
         │  ─ Control APIs  │
         │  ─ Telemetry     │
         └────────↑─────────┘
                  │
                  │ NATS
                  │
         ┌────────↓─────────┐
         │ Rust Edge Agent  │
         │                  │
         │  ─ Control Plane │
         │  ─ Telemetry     │
         │  ─ Runtime Mgmt  │
         └──────────────────┘

```

The agent is a single long-running process with a clearly defined lifecycle and explicit separation between control and telemetry concerns.

---

## Messaging Model

### Transport

- **NATS (Core)**
- TCP-based, lightweight, low-latency
- Suitable for edge ↔ backend communication
- Supports both pub/sub and request/reply semantics

JetStream is **not required** for this project but is discussed as an optional extension.

---

### Control Plane (Request / Reply)

**Purpose**
- Configuration updates
- Lifecycle commands (start/stop/reload)
- Health checks
- Version queries

**Model**
- NATS request/reply
- One request → one response
- Explicit timeouts
- Clear failure signaling

**Example Subject**
```

edge.<device_id>.control

````

**Example Payload (JSON)**

```json
{
  "command": "reload_config",
  "config_version": 42
}
````

**Response**

```json
{
  "status": "ok",
  "applied_version": 42
}
```

**Rationale**

* Control messages require causality
* Request/reply avoids ambiguity inherent in pub/sub
* Callers can distinguish between:

  * agent offline
  * command rejected
  * command applied successfully

Pub/sub is intentionally avoided for control traffic.

---

### Telemetry Plane (Publish-only)

**Purpose**

* Heartbeats
* State snapshots
* Metrics
* Lightweight status signals

**Model**

* Fire-and-forget publish
* Best-effort delivery
* Loss-tolerant

**Example Subjects**

```
edge.<device_id>.heartbeat
edge.<device_id>.telemetry
edge.<device_id>.status
```

**Characteristics**

* Small payloads
* Periodic
* Idempotent where possible
* No reliance on ordering guarantees

**Rationale**

* Telemetry loss is acceptable
* Simpler failure semantics
* Avoids backpressure coupling with control flow

---

## Message Encoding

### JSON (Deliberate Choice)

All messages use **JSON** encoding.

This is an intentional design decision.

**Reasons**

* Human-readable and debuggable
* Can be inspected live using `nats sub`
* No schema registry or code generation required
* Minimal operational overhead
* Suitable for small control and telemetry payloads

Binary IDL-based formats (e.g. Protobuf, Thrift) are intentionally avoided to keep scope and operational complexity low. The transport already provides structure and routing; message payloads prioritize clarity and debuggability.

---

## Ordering and Delivery Semantics

* NATS Core does **not guarantee global ordering**
* Per-publisher ordering is generally preserved
* Ordering is not guaranteed across reconnects or multiple publishers

The agent architecture does **not rely on message ordering**.

Design assumptions:

* Commands are idempotent or versioned
* Telemetry is eventually consistent
* Control commands fail fast when the agent is unavailable

---

## Failure and Reconnect Behavior

### Startup

* Agent attempts to connect to NATS
* Retries with backoff
* Does not crash on initial failure

### NATS Disconnect

* Agent continues running
* Telemetry may be dropped or bounded-buffered
* Control requests fail upstream via timeout

### Reconnect

* Subscriptions are re-established
* Control handlers become active immediately
* No message replay unless JetStream is introduced

These behaviors mirror real-world embedded gateway expectations.

---

## Execution Environment

### Target Platform

* **AArch64 (ARM64)**
* Embedded Linux
* Target triple: `aarch64-unknown-linux-gnu`

### Host Platform

* x86_64 Linux development host

### Cross-Compilation

* Rust cross-compiles on x86_64 host
* Uses GNU toolchain and sysroot
* Native dependencies resolved at build time

### Execution Validation

* AArch64 binaries executed via **QEMU user-mode emulation**
* Allows ARM64 verification without physical hardware
* Common practice in CI and embedded Linux development

---

## AOSP Context (Non-Implementation)

This project does **not** implement Android or AOSP components.

A separate document (`docs/aosp-context.md`) describes how an agent like this would conceptually integrate into an Android-based system, without including Android-specific code or build artifacts.

---

## Out of Scope

The following are intentionally excluded:

* Bare-metal firmware
* Bootloaders or BSPs
* Kernel drivers
* Android services or HALs
* Device-specific hardware integration
* UI or mobile applications

These exclusions are deliberate to keep the project focused and deep.

---

## Summary

`rust-edge-agent` models an embedded **gateway-class system**, emphasizing:

* ARM64 cross-compilation
* Messaging semantics
* Failure handling
* Operational clarity
* Build reproducibility

The architecture favors explicit behavior, debuggability, and realistic constraints over breadth or feature count.

```
---
