//! Device simulator for testing edge agent.
//!
//! Simulates sensors, actuators, or hybrid devices publishing telemetry
//! and responding to commands via NATS.

use anyhow::Result;
use clap::Parser;
use futures::StreamExt;
use rust_edge_agent::{self, CommandRequest, CommandResponse, DeviceType, TelemetryMessage};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

#[derive(Parser, Debug)]
#[command(version, about)]
struct Args {
    // ---
    /// Unique device identifier
    #[arg(long)]
    id: String,

    /// Device mode: sensor, actuator, or hybrid
    #[arg(long)]
    mode: String,

    /// Device type: temp, humidity, valve, propulsion
    #[arg(long, value_name = "TYPE")]
    r#type: String,

    /// Telemetry interval in seconds
    #[arg(long, env = "DEVICE_INTERVAL", default_value = "5")]
    interval: u64,

    /// NATS server URL
    #[arg(long, env = "NATS_URL", default_value = "nats://localhost:4222")]
    nats_url: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    // ---
    let args = Args::parse();

    let device_type = parse_device_type(&args.r#type);
    let mode = args.mode.as_str();

    eprintln!(
        "Device simulator: {} (mode: {mode}, type: {:?})",
        args.id, device_type
    );

    let client = rust_edge_agent::connect_with_retry(&args.nats_url).await;

    match mode {
        "sensor" => run_sensor(&client, &args.id, device_type, args.interval).await,
        "actuator" => run_actuator(&client, &args.id, device_type).await,
        "hybrid" => {
            let client_clone = client.clone();
            let id_clone = args.id.clone();

            tokio::spawn(async move {
                // ---
                let _status =
                    run_sensor(&client_clone, &id_clone, device_type, args.interval).await;
            });

            run_actuator(&client, &args.id, device_type).await
        }
        _ => {
            eprintln!("Invalid mode: {}", mode);
            std::process::exit(1);
        }
    }
}

fn parse_device_type(s: &str) -> DeviceType {
    // ---
    match s {
        "temp" => DeviceType::Temperature,
        "humidity" => DeviceType::Humidity,
        "valve" => DeviceType::Valve,
        "propulsion" => DeviceType::Propulsion,
        _ => {
            eprintln!("Invalid device type: {}", s);
            std::process::exit(1);
        }
    }
}

async fn run_sensor(
    client: &async_nats::Client,
    device_id: &str,
    device_type: DeviceType,
    interval: u64,
) -> Result<()> {
    // ---
    let subject = rust_edge_agent::device_telemetry(device_id);
    let mut value = 20.0;

    loop {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        value += rand::random::<f64>() * 2.0 - 1.0;
        value = value.clamp(15.0, 30.0);

        let telemetry = TelemetryMessage {
            device_id: device_id.to_string(),
            device_type,
            timestamp,
            value,
        };

        let payload = serde_json::to_vec(&telemetry)?;
        let _ = client.publish(subject.clone(), payload.into()).await;

        eprintln!("[{}] Published: {:.2}", device_id, value);

        tokio::time::sleep(Duration::from_secs(interval)).await;
    }
}

async fn run_actuator(
    client: &async_nats::Client,
    device_id: &str,
    _device_type: DeviceType,
) -> anyhow::Result<()> {
    // ---
    let subject = rust_edge_agent::device_command(device_id);

    let mut sub = client.subscribe(subject).await?;

    eprintln!("[{}] Listening for commands", device_id);

    while let Some(msg) = sub.next().await {
        // ---
        match serde_json::from_slice::<CommandRequest>(&msg.payload) {
            Ok(cmd) => {
                eprintln!(
                    "[{}] Received command: target_value = {:.2}",
                    device_id, cmd.target_value
                );

                let response = CommandResponse {
                    status: "ok".to_string(),
                    message: Some(format!("Set to {:.2}", cmd.target_value)),
                };

                if let Some(reply) = msg.reply {
                    let payload = serde_json::to_vec(&response)?;
                    let _ = client.publish(reply, payload.into()).await;
                }
            }
            Err(e) => {
                eprintln!("[{}] Invalid command: {}", device_id, e);
                tokio::time::sleep(Duration::from_secs(1_u64)).await;
            }
        }
    }
    Ok(())
}
