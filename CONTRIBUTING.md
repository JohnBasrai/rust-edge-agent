# Contributing to rust-edge-agent

Thanks for considering contributing!

## Quick Start

**New to the project?** See README.md for cross-compilation setup and QEMU validation.

## Local Development

### Starting Services

Before running the agent locally:

```bash
# Start NATS broker
./scripts/start-services.sh

# When done
./scripts/stop-services.sh
```

### Running the Demo

```bash
# Start 3 devices (default)
./scripts/demo.sh

# Start 10 devices
NUM_DEVICES=10 ./scripts/demo.sh

# Use custom telemetry interval
DEVICE_INTERVAL=2 ./scripts/demo.sh
```

See `scripts/demo.sh` for monitoring and testing examples.

**Before submitting a pull request:**

- Run local CI scripts (includes fmt, clippy, and builds):
  ```bash
  ./scripts/ci-lint.sh
  ./scripts/ci-build-native.sh
  ./scripts/ci-build-aarch64.sh
  cargo test --release
  ```
- If your change affects behavior, please update `CHANGELOG.md` under the [Unreleased] section
- Keep commits focused and descriptive

We follow [Keep a Changelog](https://keepachangelog.com/en/1.0.0/) and [Semantic Versioning](https://semver.org/).

## Code Formatting

This project uses `rustfmt` for consistent code formatting. All code should be formatted before committing.

### Visual Separators

Since `rustfmt` removes blank lines at the start of impl blocks, function bodies, and module blocks, we use comment separators `// ---` for visual clarity:

```rust
// Module blocks
mod messaging {
    // ---
    use super::*;

    pub fn start_control_handler() {
        // ---
        // function body
    }
}

// Struct definitions
pub struct DeviceState {
    // ---
    device_id: String,
    last_seen: Instant,
}

// Impl blocks
impl ControlHandler for NatsControlHandler {
    // ---
    async fn handle_command(&self, cmd: Command) -> Result<Response> {
        // ---
        // implementation
    }
}

// Regular functions
pub async fn route_device_command(device_id: &str, cmd: Command) {
    // ---
    let client = get_device_client(device_id);
    // ...
}

// Struct literals (construction) - NO separator
let config = MqttOptions {
    client_id,
    broker_addr,
    port,
};

// Test modules
#[cfg(test)]
mod tests {
    // ---
    use super::*;

    #[tokio::test]
    async fn test_command_routing() {
        // ---
        // test body
    }
}
```

**Style Guidelines:**
1) Use `// ---` for visual separation in at a minimum **module blocks**, **impl blocks**, **struct definitions**, and **function bodies**
2) Place separators after the opening brace and before the first meaningful line
3) Between meaningful steps of logic processing (e.g., separating message parsing, routing, and response handling)
4) For modules: place separator after `mod name {` and before imports/content
5) For impl blocks: place separator after `impl ... {` and before the first method
6) For struct definitions: place separator after `struct Name {` and before field declarations
7) For functions: place separator after function signature and before the main logic
8) Do NOT use separators inside struct literals (during construction)
9) Keep separators consistent across the codebase

**Note:** This project uses rustfmt's default configuration. The `// ---` separator pattern is a formatting convention to work around rustfmt's blank line removal in stable Rust.

## Documentation and Doc Comments

This project follows a **production-grade documentation standard** for Rust code, with special attention to embedded systems and messaging patterns.

### Required Doc Comments

Use Rust doc comments (`///`) for:

- Public structs and enums (especially messaging types like `ControlCommand`, `TelemetryMessage`)
- Public functions (especially handlers and control plane methods)
- Public modules that define architectural boundaries
- Critical system behavior (device lifecycle, message routing, failure handling)
- Macros that encode non-obvious behavior or policy decisions

Doc comments should describe **intent, guarantees, and failure semantics** —
not restate what the code obviously does.

### Messaging/Edge-Specific Documentation

For messaging and control plane code, doc comments should explicitly describe:

- **Failure modes** - What happens when devices disconnect, messages timeout, etc.?
- **Message flow** - Which part of the control/telemetry flow is this?
- **Delivery semantics** - At-most-once, at-least-once, exactly-once?
- **Concurrency** - Can multiple messages be in-flight? How are they handled?

Example:
```rust
/// Routes a control command to the specified device.
///
/// This implements the edge agent's command routing logic, translating
/// backend requests into device-specific commands.
///
/// # Behavior
///
/// - Uses NATS request/reply for synchronous command execution
/// - Waits up to 5 seconds for device acknowledgment
/// - Returns error if device is offline or command times out
///
/// # Errors
///
/// Returns an error if:
/// - The device ID is unknown or offline
/// - The command times out (5s default)
/// - The device returns an error response
pub async fn route_command(device_id: &str, cmd: Command) -> Result<Response> {
    // ---
    // implementation
}
```

### Optional (Encouraged) Doc Comments

Doc comments or short block comments are encouraged for:

- Internal functions with concurrency or timing implications
- Device state management logic
- Message serialization and validation
- Configuration parsing and validation
- Startup and initialization logic

### Not Required

Doc comments are not required for:

- Trivial helpers
- Simple getters or pass-through functions
- Test code (assert messages should be sufficient)
- Obvious glue code

### General Guidance

- Prefer documenting *why* over *how*
- Be explicit about failure behavior and recovery
- Keep comments accurate and up to date
- Avoid over-documenting trivial code
- For messaging patterns, describe delivery semantics clearly

Well-written doc comments are considered part of the code's correctness, especially for distributed systems and edge infrastructure.

## Architecture Guidelines

This project uses the [Explicit Module Boundary Pattern (EMBP)](https://github.com/JohnBasrai/architecture-patterns/blob/main/rust/embp.md) for module organization. Please review the EMBP documentation before making structural changes.

### Key EMBP Principles

- Each module's public API is defined in its `mod.rs` gateway file
- Sibling modules import from each other using `super::`
- External modules import through `crate::module::`
- Never bypass module gateways with deep imports

### Edge Agent Module Structure

```
src/
├── agent/          # Agent lifecycle and coordination
├── messaging/      # NATS/MQTT messaging abstraction
├── runtime/        # Device registry, state management
└── bin/
    └── device_sim.rs  # Device simulator for testing
```

## Test Coverage

This project uses a **layered testing approach** optimized for embedded systems:

### Current Test Strategy

**QEMU Smoke Tests (Primary):**
- `scripts/ci-qemu-smoke.sh` - Validates ARM64 binary execution
- Ensures cross-compilation correctness
- Tests basic runtime behavior

**Integration Tests (Planned):**
- Multi-device scenarios with NATS broker
- Command routing and telemetry aggregation
- Failure recovery and reconnect logic

**Unit Tests:**
- Core logic (device state, message parsing)
- Lifecycle transitions
- Error handling

### When to Add Tests

**Add integration tests when:**
- Adding new messaging patterns
- Changing device lifecycle behavior
- Implementing failure recovery logic

**Add unit tests when:**
- Complex business logic needs isolated testing
- Edge cases are difficult to trigger via integration tests
- Testing device state transitions

### Test Organization

```
scripts/
  ci-lint.sh              # Formatting and clippy
  ci-build-native.sh      # Native x86_64 build
  ci-build-aarch64.sh     # ARM64 cross-compilation
  ci-qemu-smoke.sh        # QEMU validation
```

**Running tests:**
```bash
# Local build validation
./scripts/ci-lint.sh
./scripts/ci-build-native.sh
./scripts/ci-build-aarch64.sh

# QEMU smoke test (requires qemu-user and cross-compilation tools)
./scripts/ci-qemu-smoke.sh

# All workspace tests (use --release to match build configuration)
cargo test --release
```

**Note:** This is a portfolio/demo project showcasing embedded Linux patterns and cross-compilation. Production code would include more comprehensive integration tests and hardware-in-the-loop validation.

## Testing Edge Agent Behavior

When testing edge agent and messaging flows:

- Test both connected and disconnected device scenarios
- Verify message routing and aggregation
- Test timeout and retry behavior
- Include tests for edge cases (duplicate messages, out-of-order delivery)
- Validate device lifecycle transitions

## Cross-Compilation Notes

This project targets `aarch64-unknown-linux-gnu`. When adding dependencies:

- Verify they support cross-compilation (check for C dependencies)
- Test on both native and ARM64 targets
- Document any platform-specific behavior
- Update CI scripts if new system dependencies are needed
