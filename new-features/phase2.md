I want to create `mqtt-rpc`, a Rust crate that provides RPC semantics over MQTT pub/sub.

## Problem Statement

MQTT is a lightweight pub/sub protocol widely used in IoT, but it lacks built-in RPC semantics. Every developer using MQTT for request/response patterns must solve the same problems:

1. **Request/response correlation** - Matching responses to requests using correlation IDs
2. **Timeout handling** - Detecting when a request will never get a response
3. **Concurrent request handling** - Processing multiple in-flight requests without blocking
4. **Code duplication** - This logic is reimplemented in every MQTT client/server application

This crate extracts that common pattern into a reusable library.

## Design Goals

**Client-side:**
- Send request, get Future that resolves when response arrives
- Automatic correlation ID generation (compact: counter-based, not UUIDs)
- Timeout support
- Concurrent requests (multiple in-flight)

**Server-side:**
- Register async handlers for request topics
- Handlers execute concurrently (spawned, not blocking event loop)
- Automatic correlation ID handling
- Response publishing

**Implementation:**
- Built on `rumqttc` (popular async Rust MQTT client)
- Tokio-based async runtime
- Compact correlation IDs: `{device_id}:{counter}` format
- Internal `HashMap<CorrelationId, oneshot::Sender<Response>>` for pending requests

## Proposed API

### Client Side
```rust
use mqtt_rpc::RpcClient;

let client = RpcClient::new(mqtt_client, "client-01").await?;

// Send request, wait for response (with timeout)
let response: Response = client
    .request("devices/valve-01/command", request_payload)
    .timeout(Duration::from_secs(5))
    .await?;
```

### Server Side
```rust
use mqtt_rpc::RpcServer;

let server = RpcServer::new(mqtt_client, "device-01").await?;

// Register handler - runs concurrently for each request
server.handle("command", |req: Request| async move {
    // Long-running async operation
    actuator.open().await?;
    
    Ok(Response {
        status: "opened",
        position: 1.0,
    })
}).await?;

server.run().await?;
```

## Technical Requirements

1. **Correlation ID Strategy:**
   - Use `AtomicU64` counter for compact IDs
   - Format: `"{namespace}:{counter}"` (e.g., "valve-01:42")
   - Namespace prevents conflicts in multi-device scenarios

2. **Concurrent Handler Execution:**
   - Spawn handler tasks with `tokio::spawn`
   - Don't block MQTT event loop
   - Multiple requests can be processed simultaneously

3. **Request/Response Flow:**
```
   Client:
   1. Generate correlation_id
   2. Store oneshot::Sender in HashMap
   3. Publish to request topic with correlation_id in payload
   4. Await on oneshot::Receiver
   
   Server:
   1. Receive request with correlation_id
   2. Spawn handler task
   3. Handler completes → publish response with same correlation_id
   
   Client:
   1. Receive response, extract correlation_id
   2. Find oneshot::Sender in HashMap
   3. Send response through channel
   4. Remove from HashMap
```

4. **Timeout Handling:**
   - Use `tokio::time::timeout` wrapper
   - Clean up HashMap entries on timeout
   - Return clear error (not silent drop)

5. **Topic Convention:**
```
   Request topic:  {base_topic}/request
   Response topic: {base_topic}/response
```

## Success Criteria

- [ ] Client can send request and receive response
- [ ] Multiple concurrent requests work correctly
- [ ] Timeouts are enforced and HashMap is cleaned up
- [ ] Server handles multiple concurrent requests (spawns tasks)
- [ ] Correlation IDs are compact and collision-free
- [ ] Works with JSON payloads (serde support)
- [ ] Clean error types for timeout, connection loss, etc.
- [ ] Example showing both client and server usage
- [ ] Unit tests for correlation logic
- [ ] Integration test with real MQTT broker

## Context from Phase 1

In `rust-edge-agent` Phase 1, we used NATS everywhere to avoid this correlation problem. Now we're extracting the pattern as a reusable crate that could be used with actual MQTT devices.

Key learnings:
- Request/response correlation is non-trivial
- Concurrent handler execution is critical
- Compact IDs matter for bandwidth-constrained IoT
- This pattern is repetitive across MQTT applications

## Constraints

- Keep API simple and ergonomic
- Minimize allocations (IoT devices are resource-constrained)
- No unsafe code unless absolutely necessary
- Clear error messages for debugging
- Works with both MQTT v4 and v5 (rumqttc supports both)

## Deliverables

1. `mqtt-rpc` library crate
2. Example client and server programs
3. README explaining the problem and solution
4. API documentation with examples
5. Basic test suite

Ready to implement `mqtt-rpc`. Where should we start - API design, correlation logic, or project structure?
