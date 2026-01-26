#[derive(Debug, Copy, Clone)]
pub enum LifecycleState {
    // ---
    Init,
    Running,
    Shutdown,
}
