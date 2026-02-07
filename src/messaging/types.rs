//! Message type definitions for edge agent and device communication.

use serde::{Deserialize, Serialize};

/// Device type classification.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum DeviceType {
    #[serde(rename = "temp")]
    Temperature,
    Humidity,
    Valve,
    Propulsion,
}

/// Device operational mode.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum DeviceMode {
    Sensor,
    Actuator,
    Hybrid,
}

/// Telemetry message published by devices.
///
/// Used for both MQTT RPC responses and NATS backend forwarding.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelemetryMessage {
    pub device_id: String,
    pub device_type: DeviceType,
    pub timestamp: u64,
    pub value: f64,
}

/// Command request sent to actuator devices.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandRequest {
    pub target_value: f64,
}

/// Command response from actuator devices.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandResponse {
    pub status: String,
    pub message: Option<String>,
}

/// Device registration request sent during device startup.
///
/// Devices call the agent's `register-device` RPC method to announce
/// their presence and capabilities.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterRequest {
    pub device_id: String,
    pub device_type: DeviceType,
    pub mode: DeviceMode,
}

/// Device registration response from agent.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterResponse {
    pub accepted: bool,
    pub message: Option<String>,
}

/// Actuator state query response.
///
/// Returned by the `read-state` RPC method on actuator devices.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActuatorState {
    pub current_value: f64,
    pub timestamp: u64,
}
