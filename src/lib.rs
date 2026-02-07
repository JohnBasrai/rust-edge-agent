//! Rust edge agent library.
//!
//! Provides components for building edge gateway systems that bridge
//! backend services (via NATS) with local devices (via MQTT+mom-rpc).

mod agent;
pub mod messaging;
mod runtime;

// Agent runtime
pub use agent::{run, DeviceRegistry, DeviceState};

// Runtime lifecycle management
pub use runtime::{Lifecycle, LifecycleState};

// Re-export commonly used messaging types for convenience
pub use messaging::{
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
