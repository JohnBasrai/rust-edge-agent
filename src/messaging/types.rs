//! Message type definitions for edge agent and device communication.

use serde::{Deserialize, Serialize};

/// Device type classification.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum DeviceType {
    // ---
    #[serde(rename = "temp")]
    Temperature,
    Humidity,
    Valve,
    Propulsion,
}

/// Telemetry message published by devices.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelemetryMessage {
    // ---
    pub device_id: String,
    pub device_type: DeviceType,
    pub timestamp: u64,
    pub value: f64,
}

/// Command request sent to actuator devices.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandRequest {
    // ---
    pub target_value: f64,
}

/// Command response from actuator devices.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandResponse {
    // ---
    pub status: String,
    pub message: Option<String>,
}
