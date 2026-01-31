//! Agent entrypoint and public interface.
//!
//! The agent is the primary runtime unit of the system. It owns lifecycle
//! transitions and coordinates messaging and runtime components.

mod registry;
mod run;

pub use registry::{DeviceRegistry, DeviceState};
pub use run::run;
