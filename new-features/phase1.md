I'm working on `rust-edge-agent`, an embedded Linux edge gateway for ARM64 systems. The project demonstrates cross-compilation and QEMU validation.

**Current state:** Basic "hello world" with NATS dependencies, cross-compiles to aarch64, runs under QEMU.

**Next milestone (Phase 1):** Add real functionality using NATS messaging to demonstrate edge gateway patterns.

**Architecture Decision:**
- Use NATS for ALL messaging (devices ↔ edge agent ↔ backend)
- Rationale: Focus on edge agent coordination logic, avoid reimplementing MQTT request/response correlation
- Phase 2 will extract this correlation as a reusable `mqtt-rpc` crate

**What to build:**

1. **Device Simulator** (`src/bin/device_sim.rs`):
   - Single binary with CLI args: `--id <device-id> --mode <sensor|actuator|hybrid> --type <temp|humidity|valve|propulsion>`
   - Modes:
     - `sensor`: Publishes telemetry periodically (pub only)
     - `actuator`: Handles commands, publishes status/acks (sub + pub)
     - `hybrid`: Both telemetry and command handling
   - Uses NATS request/reply for actuator commands (proper RPC semantics)
   - Simple JSON payloads
   - Can run multiple instances with different IDs

2. **Edge Agent** (enhance `src/main.rs` and `src/agent/`):
   - Connects to NATS
   - Subscribes to device telemetry (aggregates, forwards to backend)
   - Routes control commands from backend to devices
   - Uses NATS request/reply for commands to actuators
   - Maintains basic device registry/state

**Success criteria:**
- Start 3-4 device simulators (mix of sensors/actuators)
- Edge agent aggregates telemetry, routes commands
- Can send backend commands via `nats` CLI and see device responses
- Demonstrates gateway-class coordination patterns

**Constraints:**
- Keep it simple - this is foundation for Phase 2 (mqtt-rpc library)
- Focus on architecture, not feature breadth
- Maintain cross-compilation and QEMU validation
- Document the MQTT correlation problem this avoids

Also let's use `CONTRIBUTING.md` in this repo and we work under `WORKFLOW-v1.2.md` workflow.

Ready to implement Phase 1. Where should we start?
