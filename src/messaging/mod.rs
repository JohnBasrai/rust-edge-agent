//! Messaging boundary for the edge agent.
//!
//! Encapsulates all NATS-related functionality and prevents
//! messaging details from leaking into higher-level components.

mod nats;

//pub use nats::{start_control, start_heartbeat};
