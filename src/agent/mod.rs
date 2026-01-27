//! Agent entrypoint and public interface.
//!
//! The agent is the primary runtime unit of the system. It owns lifecycle
//! transitions and coordinates messaging and runtime components.

mod run;

pub use run::run;
