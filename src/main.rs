mod agent;
mod runtime;

pub use runtime::LifecycleState;

fn main() {
    // ---
    agent::run();
}
