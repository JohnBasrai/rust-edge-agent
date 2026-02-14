use super::registry::DeviceRegistry;
use crate::messaging::{
    //
    self,
    CommandRequest,
    CommandResponse,
    DeviceMode,
    RegisterRequest,
    RegisterResponse,
    TelemetryMessage,
};
use anyhow::{anyhow, Result};
use async_nats::Client;
use clap::Parser;
use futures::StreamExt;
use mom_rpc::{RpcClient, RpcServer};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;

/// Edge agent for ARM64 embedded Linux systems.
#[derive(Parser, Debug)]
#[command(version, about)]
struct Args {
    /// NATS server URL (for backend communication)
    #[arg(long, env = "NATS_URL", default_value = "nats://localhost:4222")]
    nats_url: String,

    /// MQTT broker URL (for device communication)
    #[arg(long, env = "MQTT_BROKER_URL", default_value = "mqtt://localhost:1883")]
    mqtt_broker: String,

    /// Device offline timeout in seconds
    #[arg(long, env = "DEVICE_TIMEOUT", default_value = "30")]
    device_timeout: u64,

    /// Telemetry polling interval in seconds
    #[arg(long, env = "POLL_INTERVAL", default_value = "5")]
    poll_interval: u64,
}

/// Registered device information.
#[derive(Debug, Clone)]
struct RegisteredDevice {
    #[allow(dead_code)]
    device_id: String,
    #[allow(dead_code)]
    device_type: crate::messaging::DeviceType,
    mode: DeviceMode,
    service_name: String,
}

/// Run the edge agent.
///
/// # Behavior
///
/// - Connects to NATS for backend communication
/// - Connects to MQTT for device communication
/// - Accepts device registrations via RPC
/// - Polls sensor devices for telemetry
/// - Routes backend commands to actuator devices
/// - Forwards telemetry to backend via NATS
pub async fn run() -> Result<()> {
    let args = Args::parse();

    eprintln!("agent:: Starting edge agent...");
    eprintln!("agent:: NATS URL: {}", args.nats_url);
    eprintln!("agent:: MQTT Broker: {}", args.mqtt_broker);
    eprintln!("agent:: Device timeout: {}s", args.device_timeout);
    eprintln!("agent:: Poll interval: {}s", args.poll_interval);

    // Connect to NATS (for backend communication)
    let nats_client = messaging::connect_nats_with_retry(&args.nats_url).await;

    // Connect to MQTT (for device communication)
    eprintln!("agent:: Connecting to MQTT broker...");
    let mqtt_transport =
        messaging::create_transport_with_retry(&args.mqtt_broker, "agent-transport").await;
    eprintln!("agent:: MQTT transport ready");

    // Create RPC server for device registrations
    eprintln!("agent:: Creating RPC server...");
    let agent_server = RpcServer::with_transport(mqtt_transport.clone(), "agent");

    // Create RPC client for polling devices
    eprintln!("agent:: Creating RPC client...");
    let agent_client = RpcClient::with_transport(mqtt_transport.clone(), "agent-client").await?;
    eprintln!("agent:: RPC client ready");

    let timeout = Duration::from_secs(args.device_timeout);
    let registry = Arc::new(Mutex::new(DeviceRegistry::new(timeout)));
    let devices = Arc::new(Mutex::new(HashMap::<String, RegisteredDevice>::new()));

    // Register device registration handler
    let devices_clone = devices.clone();
    agent_server.register("register-device", move |req: RegisterRequest| {
        let devices = devices_clone.clone();
        async move {
            let service_name = match req.mode {
                DeviceMode::Sensor => format!("sensor-{}", req.device_id),
                DeviceMode::Actuator => format!("actuator-{}", req.device_id),
                DeviceMode::Hybrid => format!("hybrid-{}", req.device_id),
            };

            let device = RegisteredDevice {
                device_id: req.device_id.clone(),
                device_type: req.device_type,
                mode: req.mode,
                service_name,
            };

            devices.lock().await.insert(req.device_id.clone(), device);

            eprintln!(
                "agent:: Device registered: {} (mode: {:?}, type: {:?})",
                req.device_id, req.mode, req.device_type
            );

            Ok(RegisterResponse {
                accepted: true,
                message: Some("Registration successful".to_string()),
            })
        }
    });

    // Spawn RPC server
    let _server_handle = agent_server.spawn();
    eprintln!("agent:: RPC server listening for device registrations");

    // Subscribe to backend commands via NATS
    let mut command_sub = match nats_client
        .subscribe(messaging::all_backend_commands())
        .await
    {
        Ok(c) => c,
        Err(e) => {
            return Err(anyhow!(
                "agent::run: failed to subscribe to backend commands: {e}"
            ));
        }
    };

    eprintln!("agent:: Edge agent running");

    // Spawn telemetry polling task
    let devices_poll = devices.clone();
    let nats_poll = nats_client.clone();
    let mqtt_poll = agent_client.clone();
    let registry_poll = registry.clone();
    let poll_interval = args.poll_interval;
    tokio::spawn(async move {
        poll_device_telemetry(
            devices_poll,
            nats_poll,
            mqtt_poll,
            registry_poll,
            poll_interval,
        )
        .await;
    });

    // Main event loop: handle backend commands
    loop {
        tokio::select! {
            Some(msg) = command_sub.next() => {
                if let Err(e) = handle_backend_command(
                    &nats_client, &agent_client, &devices, msg).await {
                    eprintln!("agent:: Command error: {e}");
                }
            }
        }
    }
}

/// Poll sensor devices for telemetry and forward to backend.
async fn poll_device_telemetry(
    devices: Arc<Mutex<HashMap<String, RegisteredDevice>>>,
    nats_client: Client,
    mqtt_client: RpcClient,
    registry: Arc<Mutex<DeviceRegistry>>,
    interval_secs: u64,
) {
    let mut interval = tokio::time::interval(Duration::from_secs(interval_secs));

    loop {
        interval.tick().await;

        let devices_snapshot = devices.lock().await.clone();

        for (_, device) in devices_snapshot.iter() {
            // Only poll sensors and hybrids
            if !matches!(device.mode, DeviceMode::Sensor | DeviceMode::Hybrid) {
                continue;
            }

            // Poll device for telemetry
            let result: Result<TelemetryMessage, _> = mqtt_client
                .request_to(&device.service_name, "read-telemetry", ())
                .await;

            match result {
                Ok(telemetry) => {
                    // Update registry
                    registry.lock().await.update(&telemetry);

                    eprintln!(
                        "agent:: Telemetry: {} ({:?}) = {:.2}",
                        telemetry.device_id, telemetry.device_type, telemetry.value
                    );

                    // Forward to backend via NATS
                    if let Ok(payload) = serde_json::to_vec(&telemetry) {
                        let _ = nats_client
                            .publish(messaging::backend_telemetry(), payload.into())
                            .await;
                    }
                }
                Err(e) => {
                    eprintln!("agent:: Failed to poll {}: {}", device.service_name, e);
                }
            }
        }
    }
}

/// Handle backend command and route to appropriate device.
async fn handle_backend_command(
    nats_client: &Client,
    mqtt_client: &RpcClient,
    devices: &Arc<Mutex<HashMap<String, RegisteredDevice>>>,
    msg: async_nats::Message,
) -> Result<()> {
    // Extract device ID from NATS subject
    let device_id = match msg.subject.strip_prefix("backend.command.") {
        Some(dev) => dev,
        None => {
            return Err(anyhow!(
                "agent::command: Invalid command subject: {}",
                msg.subject
            ));
        }
    };

    eprintln!("agent:: Backend command for device: {device_id}");

    // Look up device
    let device = {
        let devices = devices.lock().await;
        devices.get(device_id).cloned()
    };

    let device = match device {
        Some(d) => d,
        None => {
            eprintln!("agent:: Device not registered: {device_id}");
            if let Some(reply) = msg.reply {
                let error_response = CommandResponse {
                    status: "error".to_string(),
                    message: Some(format!("Device not registered: {device_id}")),
                };
                let payload = serde_json::to_vec(&error_response)?;
                let _ = nats_client.publish(reply, payload.into()).await;
            }
            return Ok(());
        }
    };

    // Parse command request
    let command: CommandRequest = serde_json::from_slice(&msg.payload)?;

    // Route to device via MQTT RPC
    let result: Result<CommandResponse, _> = mqtt_client
        .request_to(&device.service_name, "execute-command", command)
        .await;

    match result {
        Ok(response) => {
            eprintln!("agent:: Command success: {response:?}");
            if let Some(reply) = msg.reply {
                let payload = serde_json::to_vec(&response)?;
                let _ = nats_client.publish(reply, payload.into()).await;
            }
        }
        Err(e) => {
            eprintln!("agent:: Command failed: {e}");
            if let Some(reply) = msg.reply {
                let error_response = CommandResponse {
                    status: "error".to_string(),
                    message: Some(format!("Device unreachable: {e}")),
                };
                let payload = serde_json::to_vec(&error_response)?;
                let _ = nats_client.publish(reply, payload.into()).await;
            }
        }
    }

    Ok(())
}
