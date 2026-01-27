/// High-level lifecycle states for the edge agent.
#[derive(Debug, Copy, Clone)]
pub enum LifecycleState {
    /// Initial startup state.
    Init,
    /// Agent is running and connected.
    #[allow(dead_code)]
    Running,
    /// Agent is shutting down.
    #[allow(dead_code)]
    Shutdown,
}

/// Tracks and manages lifecycle transitions.
#[allow(dead_code)]
pub struct Lifecycle {
    state: LifecycleState,
}

impl Lifecycle {
    /// Create a new lifecycle starting in `Init`.
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self {
            state: LifecycleState::Init,
        }
    }

    /// Transition the lifecycle into the running state.
    #[allow(dead_code)]
    pub fn transition_to_running(&mut self) {
        self.state = LifecycleState::Running;
    }

    /// Return the current lifecycle state.
    #[allow(dead_code)]
    pub fn state(&self) -> LifecycleState {
        self.state
    }
}
