mod agent;
mod messaging;
mod runtime;

use runtime::LifecycleState;

#[tokio::main]
async fn main() {
    // ---
    agent::run();
}
