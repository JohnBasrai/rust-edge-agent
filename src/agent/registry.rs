//! Device registry and state tracking.

use crate::messaging::{DeviceType, TelemetryMessage};
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Device state tracked by the edge agent.
#[derive(Debug, Clone)]
pub struct DeviceState {
    // ---
    #[allow(dead_code)]
    pub device_id: String,
    #[allow(dead_code)]
    pub device_type: DeviceType,
    pub last_seen: Instant,
    pub last_value: Option<f64>,
}

/// Registry of known devices and their states.
///
/// # Behavior
///
/// - Devices are added on first telemetry message
/// - Last-seen timestamp updated on each message
/// - Devices considered offline after configured timeout
pub struct DeviceRegistry {
    // ---
    devices: HashMap<String, DeviceState>,
    #[allow(dead_code)]
    timeout: Duration,
}

impl DeviceRegistry {
    // ---
    /// Create a new device registry with the specified timeout.
    pub fn new(timeout: Duration) -> Self {
        // ---
        Self {
            devices: HashMap::new(),
            timeout,
        }
    }

    /// Update registry with telemetry message.
    ///
    /// Adds device if not present, updates last-seen and value.
    pub fn update(&mut self, msg: &TelemetryMessage) {
        // ---
        self.devices
            .entry(msg.device_id.clone())
            .and_modify(|state| {
                state.last_seen = Instant::now();
                state.last_value = Some(msg.value);
            })
            .or_insert_with(|| DeviceState {
                device_id: msg.device_id.clone(),
                device_type: msg.device_type,
                last_seen: Instant::now(),
                last_value: Some(msg.value),
            });
    }

    /// Check if a device is currently online.
    #[allow(dead_code)]
    pub fn is_online(&self, device_id: &str) -> bool {
        // ---
        self.devices
            .get(device_id)
            .map(|state| state.last_seen.elapsed() < self.timeout)
            .unwrap_or(false)
    }

    /// Get current device count.
    pub fn device_count(&self) -> usize {
        // ---
        self.devices.len()
    }
}
