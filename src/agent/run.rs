use crate::LifecycleState;

pub fn run() {
    let _state = LifecycleState::Init;

    println!("Hello world from {}!", std::env::consts::ARCH);
}
