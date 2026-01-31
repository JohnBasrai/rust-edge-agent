//! Rust edge agent library.
mod agent;
mod messaging;
mod runtime;

pub use agent::{run, DeviceRegistry, DeviceState};
pub use messaging::{
    // ---
    all_backend_commands,
    all_device_telemetry,
    backend_telemetry,
    connect_with_retry,
    device_command,
    device_telemetry,
};
pub use messaging::{CommandRequest, CommandResponse, DeviceType, TelemetryMessage};
pub use runtime::{Lifecycle, LifecycleState};
