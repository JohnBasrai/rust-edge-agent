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
         │  — Control APIs  │
         │  — Telemetry     │
         └────────↑─────────┘
                  │
                  │ NATS
                  │
         ┌────────↓─────────┐
         │ Rust Edge Agent  │
         │                  │
         │  — NATS Client   │  Backend communication
         │  — MQTT Client   │  Device polling/commands
         │  — MQTT Server   │  Device registration
         │  — Control Plane │
         │  — Telemetry     │
         │  — Runtime Mgmt  │
         └────────↓─────────┘
                  │
                  │ MQTT + mom-rpc
                  │
     ┌────────────┼────────────┬─────────────┐
     │            │            │             │
┌────▼────┐  ┌───▼────┐  ┌────▼──────┐  ┌──▼──────┐
│sensor-1 │  │sensor-2│  │actuator-1 │  │hybrid-1 │
│         │  │        │  │           │  │         │
│ RPC Svr │  │ RPC Svr│  │  RPC Svr  │  │ RPC Svr │
└─────────┘  └────────┘  └───────────┘  └─────────┘
```

The agent is a **dual-protocol bridge** with clearly defined communication boundaries:

- **Northbound (Backend ↔ Agent)**: NATS for cloud communication
- **Southbound (Agent ↔ Devices)**: MQTT + mom-rpc for local edge communication

---

## Messaging Model

The agent operates two independent messaging systems:

### Northbound: NATS (Backend Communication)

**Purpose:**
- Connect agent to cloud/backend services
- Receive control commands from backend
- Forward aggregated telemetry to backend

**Protocol:** NATS Core (TCP-based, lightweight)

**Subjects:**
```
backend.command.<device-id>    # Backend → Agent (request/reply)
backend.telemetry              # Agent → Backend (publish)
```

**Characteristics:**
- Request/reply semantics for commands
- Pub/sub for telemetry forwarding
- Resilient reconnection with backoff

### Southbound: MQTT + mom-rpc (Device Communication)

**Purpose:**
- Communicate with local edge devices
- Accept device registrations
- Poll sensor telemetry
- Send actuator commands

**Protocol:** MQTT with mom-rpc RPC layer

**RPC Methods:**

| Method | Service | Direction | Purpose |
|--------|---------|-----------|---------|
| `register-device` | `agent` | Device → Agent | Device announces presence |
| `read-telemetry` | `{mode}-{id}` | Agent → Device | Poll sensor readings |
| `execute-command` | `{mode}-{id}` | Agent → Device | Send actuator commands |
| `read-state` | `{mode}-{id}` | Agent → Device | Query actuator state |

**Service Naming Convention:**
- Sensors: `sensor-{id}` (e.g., `sensor-1`)
- Actuators: `actuator-{id}` (e.g., `actuator-2`)
- Hybrid: `hybrid-{id}` (e.g., `hybrid-3`)

---

## Device Discovery and Registration

### Dynamic Registration Flow

Devices register with the agent on startup using RPC:

```
┌──────────┐                          ┌───────────┐
│  Device  │                          │   Agent   │
│(sensor-1)│                          │           │
└────┬─────┘                          └─────┬─────┘
     │                                      │
     │  1. Create RPC Server                │
     │     (service: "sensor-1")            │
     │                                      │
     │  2. Create RPC Client                │
     │                                      │
     │  3. register-device(RegisterRequest) │
     ├─────────────────────────────────────>│
     │                                      │
     │                                      │  4. Add to registry
     │                                      │  5. Start polling task
     │                                      │
     │  RegisterResponse(accepted: true)    │
     │<─────────────────────────────────────┤
     │                                      │
     │  6. Listen for RPC calls             │  7. Poll periodically
     │     (read-telemetry, etc.)           │     (every 5 seconds)
     │                                      │
```

**Registration Request:**
```json
{
  "device_id": "1",
  "device_type": "temp",
  "mode": "sensor"
}
```

**Registration Response:**
```json
{
  "accepted": true,
  "message": "Registration successful"
}
```

Once registered, the agent:
1. Adds device to internal registry
2. Spawns telemetry polling task (for sensors/hybrids)
3. Routes backend commands to device (for actuators/hybrids)

---

## Message Flows

### Telemetry Flow (Sensor → Backend)

```
Sensor Device                 Agent                    Backend
     │                          │                          │
     │  (Every 5 seconds)       │                          │
     │                          │                          │
     │  RPC: read-telemetry     │                          │
     │<─────────────────────────┤                          │
     │                          │                          │
     │  TelemetryMessage        │                          │
     ├─────────────────────────>│                          │
     │                          │                          │
     │                          │  Update registry         │
     │                          │                          │
     │                          │  NATS: backend.telemetry │
     │                          ├─────────────────────────>│
     │                          │                          │
```

**Telemetry Message (Shared across NATS and MQTT):**
```json
{
  "device_id": "1",
  "device_type": "temp",
  "timestamp": 1738800000,
  "value": 23.5
}
```

### Command Flow (Backend → Actuator)

```
Backend                       Agent                    Actuator Device
   │                            │                            │
   │  NATS: backend.command.2   │                            │
   │  CommandRequest            │                            │
   ├───────────────────────────>│                            │
   │                            │                            │
   │                            │  Lookup device in registry │
   │                            │                            │
   │                            │  RPC: execute-command      │
   │                            ├───────────────────────────>│
   │                            │                            │
   │                            │                            │  Execute
   │                            │                            │
   │                            │  CommandResponse           │
   │                            │<───────────────────────────┤
   │                            │                            │
   │  CommandResponse (NATS)    │                            │
   │<───────────────────────────┤                            │
   │                            │                            │
```

**Command Request:**
```json
{
  "target_value": 75.5
}
```

**Command Response:**
```json
{
  "status": "ok",
  "message": "Set to 75.50"
}
```

---

## Device Operational Modes

Devices operate in one of three modes:

### Sensor Mode

**Capabilities:**
- Generates telemetry readings
- Responds to `read-telemetry` RPC calls

**Example Types:**
- Temperature sensor
- Humidity sensor

**RPC Methods:**
- `read-telemetry` → `TelemetryMessage`

### Actuator Mode

**Capabilities:**
- Executes commands
- Reports current state

**Example Types:**
- Valve controller
- Propulsion controller

**RPC Methods:**
- `execute-command(CommandRequest)` → `CommandResponse`
- `read-state()` → `ActuatorState`

### Hybrid Mode

**Capabilities:**
- Both sensor and actuator functionality
- Useful for devices that both sense and act

**Example Types:**
- Smart valve with temperature sensor
- Propulsion system with state monitoring

**RPC Methods:**
- All sensor methods
- All actuator methods

---

## Telemetry Strategy: Polling vs Pub/Sub

### Current Implementation: Polling

The agent polls sensor devices every 5 seconds:

**Advantages:**
- Simple implementation with mom-rpc
- No wildcard subscriptions needed
- Clear request/response semantics
- Timeout handling built-in

**Disadvantages:**
- Not efficient for high-frequency sensors
- Fixed polling interval for all devices
- Bandwidth overhead from regular polling

### Future: Subscription-Based Push

A future enhancement could use RPC-initiated subscriptions:

```rust
// Device subscribes for updates
agent.request("subscribe-telemetry", SubscribeRequest {
    device_id: "sensor-1",
    interval: Duration::from_secs(1),
    reply_to: "agent/telemetry/sensor-1",
}).await?;

// Device publishes to subscribed topic
loop {
    let telemetry = read_sensor();
    mqtt.publish("agent/telemetry/sensor-1", telemetry).await?;
    sleep(interval).await;
}
```

This combines RPC setup with pub/sub delivery for efficient telemetry streaming.

---

## Why NATS for Backend and MQTT for Devices?

### NATS (Northbound)

**Chosen for backend communication because:**
- Native request/reply semantics
- Lightweight and fast for cloud communication
- Simple connection model
- Well-suited for control plane traffic
- No broker configuration complexity

**Use case:**
- Backend services in cloud/datacenter
- Control commands from REST APIs
- Telemetry aggregation to analytics systems

### MQTT + mom-rpc (Southbound)

**Chosen for device communication because:**
- MQTT is ubiquitous in IoT/edge environments
- mom-rpc provides clean RPC semantics over MQTT
- Automatic correlation ID management
- Standard protocol for device integration
- Supports future mixed-vendor device ecosystems

**Use case:**
- Local edge devices (sensors, actuators)
- Embedded Linux devices
- Resource-constrained systems
- Multi-vendor device integration

### Why Not One Protocol?

**NATS for devices** would work but:
- Less common in edge/IoT deployments
- Devices often expect MQTT
- Limits future device ecosystem options

**MQTT for backend** would work but:
- Requires implementing correlation manually
- NATS is simpler for request/reply patterns
- Backend services typically have more resources

The dual-protocol approach optimizes each boundary for its specific use case.

---

## Message Encoding

### JSON (Deliberate Choice)

All messages use **JSON** encoding on both NATS and MQTT.

This is an intentional design decision.

**Reasons:**
- Human-readable and debuggable
- Can be inspected live using `nats sub` or `mosquitto_sub`
- No schema registry or code generation required
- Minimal operational overhead
- Suitable for small control and telemetry payloads

**Message Type Sharing:**

Types like `TelemetryMessage`, `CommandRequest`, and `CommandResponse` are used on both NATS and MQTT, ensuring consistent serialization across protocols.

Binary IDL-based formats (e.g. Protobuf, Thrift) are intentionally avoided to keep scope and operational complexity low.

---

## Ordering and Delivery Semantics

### NATS (Backend ↔ Agent)

- NATS Core does **not guarantee global ordering**
- Per-publisher ordering is generally preserved
- Ordering is not guaranteed across reconnects

### MQTT + mom-rpc (Agent ↔ Devices)

- MQTT QoS determines delivery guarantees
- mom-rpc uses request/reply pattern (no ordering concerns)
- Polling-based telemetry has no ordering requirements

The agent architecture does **not rely on message ordering** in either protocol.

Design assumptions:
- Commands are idempotent or versioned
- Telemetry represents point-in-time snapshots
- Control commands fail fast when unreachable

---

## Failure and Reconnect Behavior

### NATS Connection Failure

**Startup:**
- Agent attempts to connect to NATS
- Retries with exponential backoff
- Does not crash on initial failure

**Disconnect:**
- Agent continues running
- Backend commands unavailable until reconnect
- Device communication unaffected (MQTT independent)

**Reconnect:**
- Subscriptions re-established automatically
- Command handlers become active immediately
- No message replay (unless JetStream added in future)

### MQTT Connection Failure

**Startup:**
- Agent attempts to connect to MQTT broker
- Retries with exponential backoff
- Does not crash on initial failure

**Disconnect:**
- Agent continues running
- Device communication unavailable until reconnect
- Backend communication unaffected (NATS independent)

**Reconnect:**
- RPC server/client re-establish connections
- Device polling resumes automatically
- Devices may need to re-register (depending on session persistence)

### Device Offline Handling

**Registration timeout:**
- Devices that don't register are not added to registry
- Agent logs warning and continues

**Polling timeout:**
- mom-rpc automatically times out on unresponsive devices
- Agent logs error and continues polling other devices
- Device marked offline in registry

**Command timeout:**
- Backend receives error response if device unreachable
- Agent does not block on failed commands

These behaviors mirror real-world embedded gateway expectations.

---

## Transport Sharing

Each entity (agent or device) creates a single MQTT transport instance shared between RpcClient and RpcServer:

**Device:**
```rust
let transport = create_transport_with_retry(mqtt_broker, "sensor-1-transport").await;
let server = RpcServer::with_transport(transport.clone(), "sensor-1");
let client = RpcClient::with_transport(transport.clone(), "sensor-1-client").await?;
```

**Agent:**
```rust
let transport = create_transport_with_retry(mqtt_broker, "agent-transport").await;
let agent_server = RpcServer::with_transport(transport.clone(), "agent");
let agent_client = RpcClient::with_transport(transport.clone(), "agent-client").await?;
```

This approach:
- Uses a single MQTT connection per entity
- Reduces broker connection overhead
- Simplifies configuration and lifecycle management

---

## Execution Environment

### Target Platform

- **AArch64 (ARM64)**
- Embedded Linux
- Target triple: `aarch64-unknown-linux-gnu`

### Host Platform

- x86_64 Linux development host

### Cross-Compilation

- Rust cross-compiles on x86_64 host
- Uses GNU toolchain and sysroot
- Native dependencies resolved at build time

### Execution Validation

- AArch64 binaries executed via **QEMU user-mode emulation**
- Allows ARM64 verification without physical hardware
- Common practice in CI and embedded Linux development

---

## Configuration

### Edge Agent

**Environment variables only (no CLI arguments):**

```bash
NATS_URL=nats://backend.example.com:4222      # Backend communication
MQTT_BROKER_URL=mqtt://192.168.1.100:1883     # Device communication
DEVICE_TIMEOUT=30                              # Offline timeout (seconds)
POLL_INTERVAL=5                                # Telemetry polling interval (seconds)
```

### Device Simulator

**CLI arguments with environment variable fallback:**

```bash
./device_sim \
  --id 1 \
  --mode sensor \
  --type temp \
  --mqtt-broker mqtt://192.168.1.100:1883 \
  --interval 5
```

**Environment variables:**
```bash
MQTT_BROKER_URL=mqtt://192.168.1.100:1883
DEVICE_INTERVAL=5
```

**Priority:** CLI arguments > Environment variables > Compiled defaults

---

## Security Considerations

### NATS Security

- TLS/mTLS supported by NATS server
- Username/password authentication
- Token-based authentication
- Network-level security (VPN, firewall)

### MQTT Security

- TLS/mTLS supported by rumqttc transport
- Broker authentication (username/password, certificates)
- Network-level security (VPN, firewall)
- Application-level encryption (if needed)

Transport security is delegated to the broker and connection configuration, keeping RPC semantics decoupled from cryptographic policy.

---

## AOSP Context (Non-Implementation)

This project does **not** implement Android or AOSP components.

A separate document (`docs/aosp-context.md`) describes how an agent like this would conceptually integrate into an Android-based system, without including Android-specific code or build artifacts.

---

## Out of Scope

The following are intentionally excluded:

- Bare-metal firmware
- Bootloaders or BSPs
- Kernel drivers
- Android services or HALs
- Device-specific hardware integration
- UI or mobile applications
- JetStream or durable message replay
- Exactly-once delivery guarantees
- Distributed consensus
- Broker configuration management

These exclusions are deliberate to keep the project focused and deep.

---

## Future Enhancements

1. **Subscription-based telemetry** - RPC-initiated pub/sub for efficient streaming
2. **Full-duplex RPC** - Single transport for bidirectional calls (when mom-rpc supports)
3. **JetStream integration** - Durable message storage for NATS backend
4. **TLS/mTLS everywhere** - End-to-end encryption for production
5. **Batch telemetry** - Reduce NATS publish overhead by batching
6. **Device authentication** - Certificate-based device identity
7. **Command acknowledgment** - Explicit ACK/NACK for actuator commands
8. **Metrics and observability** - Prometheus metrics, OpenTelemetry traces

---

## Summary

`rust-edge-agent` models an embedded **gateway-class system** with dual-protocol architecture:

- **Northbound (NATS)**: Lightweight backend communication
- **Southbound (MQTT + mom-rpc)**: Standardized device integration
- **Bridge pattern**: Agent translates between protocols

The architecture emphasizes:
- ARM64 cross-compilation
- Messaging semantics
- Failure handling
- Operational clarity
- Build reproducibility
- Clean abstraction boundaries

The design favors explicit behavior, debuggability, and realistic constraints over breadth or feature count.
