//! NATS subject routing and construction.

/// Construct device telemetry subject.
#[allow(dead_code)]
pub fn device_telemetry(device_id: &str) -> String {
    // ---
    format!("devices.{}.telemetry", device_id)
}

/// Construct device status subject.
#[allow(dead_code)]
pub fn device_status(device_id: &str) -> String {
    // ---
    format!("devices.{}.status", device_id)
}

/// Construct device command subject (for device to subscribe).
#[allow(dead_code)]
pub fn device_command(device_id: &str) -> String {
    // ---
    format!("devices.{}.command", device_id)
}

/// Construct backend command subject (for agent to subscribe).
#[allow(dead_code)]
pub fn backend_command(device_id: &str) -> String {
    // ---
    format!("backend.command.{}", device_id)
}

/// Backend telemetry aggregation subject.
#[allow(dead_code)]
pub fn backend_telemetry() -> &'static str {
    // ---
    "backend.telemetry"
}

/// Wildcard pattern for all device telemetry.
#[allow(dead_code)]
pub fn all_device_telemetry() -> &'static str {
    // ---
    "devices.*.telemetry"
}

/// Wildcard pattern for all backend commands.
#[allow(dead_code)]
pub fn all_backend_commands() -> &'static str {
    // ---
    "backend.command.*"
}
