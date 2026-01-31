use super::registry::DeviceRegistry;
use crate::messaging::{self, CommandResponse, TelemetryMessage}; // CommandRequest, DeviceType
use anyhow::{anyhow, Result};
use async_nats::Client;
use clap::Parser;
use futures::StreamExt;
use std::time::Duration;

/// Edge agent for ARM64 embedded Linux systems.
#[derive(Parser, Debug)]
#[command(version, about)]
struct Args {
    // ---
    /// NATS server URL
    #[arg(long, env = "NATS_URL", default_value = "nats://localhost:4222")]
    nats_url: String,

    /// Device offline timeout in seconds
    #[arg(long, env = "DEVICE_TIMEOUT", default_value = "30")]
    device_timeout: u64,
}

/// Run the edge agent.
///
/// # Behavior
///
/// - Connects to NATS with retry on failure
/// - Subscribes to all device telemetry
/// - Subscribes to backend commands for routing
/// - Forwards telemetry to backend
/// - Routes commands to devices with timeout handling
pub async fn run() -> Result<()> {
    // ---
    let args = Args::parse();

    eprintln!("agent:: Starting edge agent...");
    eprintln!("agent:: NATS URL: {}", args.nats_url);
    eprintln!("agent:: Device timeout: {}s", args.device_timeout);

    let client = messaging::connect_with_retry(&args.nats_url).await;
    let timeout = Duration::from_secs(args.device_timeout);

    let mut registry = DeviceRegistry::new(timeout);

    let mut telemetry_sub = match client.subscribe(messaging::all_device_telemetry()).await {
        Ok(t) => t,
        Err(e) => {
            return Err(anyhow!(
                "agent::run: failed to subscribe to device telemetry::{e}"
            ));
        }
    };

    let mut command_sub = match client.subscribe(messaging::all_backend_commands()).await {
        Ok(c) => c,
        Err(e) => {
            return Err(anyhow!(
                "agent::run: failed to subscribe to backend commands:{e}"
            ));
        }
    };

    eprintln!("agent:: Edge agent running");

    loop {
        tokio::select! {
            Some(msg) = telemetry_sub.next() => {
                handle_telemetry(&client, &mut registry, msg).await?;
            }
            Some(msg) = command_sub.next() => {
                handle_command(&client, msg).await?;
            }
        };
    }
}

async fn handle_telemetry(
    client: &Client,
    registry: &mut DeviceRegistry,
    msg: async_nats::Message,
) -> Result<()> {
    // ---
    match serde_json::from_slice::<TelemetryMessage>(&msg.payload) {
        Ok(telemetry) => {
            registry.update(&telemetry);

            eprintln!(
                "agent::Telemetry: {} ({:?}) = {:.2} [devices: {}]",
                telemetry.device_id,
                telemetry.device_type,
                telemetry.value,
                registry.device_count()
            );

            let payload = serde_json::to_vec(&telemetry)?;
            let _ = client
                .publish(messaging::backend_telemetry(), payload.into())
                .await;
            Ok(())
        }
        Err(e) => Err(anyhow!("agent::Telemetry: Invalid telemetry payload: {e}")),
    }
}

async fn handle_command(client: &Client, msg: async_nats::Message) -> Result<()> {
    // ---
    let device_id = match msg.subject.strip_prefix("backend.command.") {
        Some(dev) => dev,
        None => {
            return Err(anyhow!(
                "agent::command: Error getting command from:{msg:?}"
            ));
        }
    };

    eprintln!("agent:: Command for device: {}", device_id);

    let device_subject = messaging::device_command(device_id);

    match client.request(device_subject, msg.payload.clone()).await {
        // ---
        Ok(response) => {
            if let Some(reply) = msg.reply {
                let _ = client.publish(reply, response.payload).await;
            }
        }
        Err(e) => {
            eprintln!("Command failed: {}", e);
            if let Some(reply) = msg.reply {
                let error_response = CommandResponse {
                    status: "error".to_string(),
                    message: Some(format!("Device unreachable: {}", e)),
                };
                let payload = serde_json::to_vec(&error_response)?;
                let _ = client.publish(reply, payload.into()).await;
            }
        }
    }
    Ok(())
}
