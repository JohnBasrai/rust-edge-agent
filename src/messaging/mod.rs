//! Messaging boundary for the edge agent.
//!
//! This module encapsulates all messaging functionality and provides a clean
//! gateway API following the Explicit Module Boundary Pattern (EMBP).
//!
//! # Architecture
//!
//! The agent uses dual messaging protocols:
//!
//! - **NATS**: Backend ↔ Agent communication (unchanged)
//!
//! NATS subjects are kept for backward compatibility but are marked as legacy
//! since device communication now uses MQTT RPC methods.

mod nats;
mod subjects;
mod types;

// NATS connection helpers (for Backend ↔ Agent)
pub use nats::connect_with_retry as connect_nats_with_retry;

// NATS subjects (legacy - used only for Backend ↔ Agent communication)
pub use subjects::{
    //
    all_backend_commands,
    all_device_telemetry,
    backend_telemetry,
    device_command,
    device_telemetry,
};

// Shared message types (used on both NATS and MQTT)
pub use types::{
    //
    ActuatorState,
    CommandRequest,
    CommandResponse,
    DeviceMode,
    DeviceType,
    RegisterRequest,
    RegisterResponse,
    TelemetryMessage,
};
