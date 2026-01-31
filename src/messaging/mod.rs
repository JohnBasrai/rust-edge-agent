//! Messaging boundary for the edge agent.
//!
//! Encapsulates all NATS-related functionality and prevents
//! messaging details from leaking into higher-level components.

mod nats;
mod subjects;
mod types;

// Public API exports
pub use nats::connect_with_retry;
pub use subjects::{
    // ---
    all_backend_commands,
    all_device_telemetry,
    backend_telemetry,
    device_command,
    device_telemetry,
};
pub use types::{CommandRequest, CommandResponse, DeviceType, TelemetryMessage};
