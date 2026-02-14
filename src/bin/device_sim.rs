//! Device simulator for testing edge agent.
//!
//! Simulates sensors, actuators, or hybrid devices using mom-rpc over MQTT.
//! Each device registers with the agent on startup and responds to RPC calls.

use anyhow::{bail, Result};
use clap::Parser;
use mom_rpc::{RpcClient, RpcServer};
use rust_edge_agent::messaging::{
    create_transport_with_retry, ActuatorState, CommandRequest, CommandResponse, DeviceMode,
    DeviceType, RegisterRequest, RegisterResponse, TelemetryMessage,
};
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::sync::Mutex;

#[derive(Parser, Debug)]
#[command(version, about)]
struct Args {
    /// Unique device identifier (numeric)
    #[arg(long)]
    id: String,

    /// Device mode: sensor, actuator, or hybrid
    #[arg(long)]
    mode: String,

    /// Device type: temp, humidity, valve, propulsion
    #[arg(long, value_name = "TYPE")]
    r#type: String,

    /// Telemetry interval in seconds (for polling by agent)
    #[arg(long, env = "DEVICE_INTERVAL", default_value = "5")]
    interval: u64,

    /// MQTT broker URL
    #[arg(long, env = "MQTT_BROKER_URL", default_value = "mqtt://localhost:1883")]
    mqtt_broker: String,
}

/// Shared device state for sensor simulation.
struct DeviceState {
    device_id: String,
    device_type: DeviceType,
    current_value: f64,
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    let args = Args::parse();
    let device_type = parse_device_type(&args.r#type)?;
    let mode = parse_device_mode(&args.mode)?;

    eprintln!(
        "[device_sim] Starting: {} (mode: {:?}, type: {:?})",
        args.id, mode, device_type
    );

    // Determine service name based on mode
    let service_name = match mode {
        DeviceMode::Sensor => format!("sensor-{}", args.id),
        DeviceMode::Actuator => format!("actuator-{}", args.id),
        DeviceMode::Hybrid => format!("hybrid-{}", args.id),
    };

    // Create shared MQTT transport
    let transport_id = format!("{service_name}-transport");
    let transport = create_transport_with_retry(&args.mqtt_broker, &transport_id).await;

    // Create RPC server (receives calls from agent)
    let server = RpcServer::with_transport(transport.clone(), &service_name);

    // Create RPC client (calls agent for registration)
    let client_id = format!("{service_name}-client");
    let client = RpcClient::with_transport(transport.clone(), &client_id).await?;

    // Shared state for sensor value simulation
    let state = Arc::new(Mutex::new(DeviceState {
        device_id: args.id.clone(),
        device_type,
        current_value: 20.0,
    }));

    // Register methods based on mode
    match mode {
        DeviceMode::Sensor => {
            register_sensor_methods(&server, state.clone());
        }
        DeviceMode::Actuator => {
            register_actuator_methods(&server, state.clone());
        }
        DeviceMode::Hybrid => {
            register_sensor_methods(&server, state.clone());
            register_actuator_methods(&server, state.clone());
        }
    }

    // Spawn server to handle incoming RPC calls
    let server_handle = server.spawn();

    // Register with agent (with retry)
    eprintln!("[{service_name}] Registering with agent...");
    let register_req = RegisterRequest {
        device_id: args.id.clone(),
        device_type,
        mode,
    };

    let mut retry_count = 0;
    let max_retries = 10;
    let mut retry_delay = Duration::from_millis(500);

    loop {
        match client
            .request_to("agent", "register-device", register_req.clone())
            .await
        {
            Ok(resp) => {
                let response: RegisterResponse = resp;
                if response.accepted {
                    eprintln!("[{service_name}] Registration accepted");
                    break;
                } else {
                    eprintln!(
                        "[{}] Registration rejected: {:?}",
                        service_name, response.message
                    );
                    return Ok(());
                }
            }
            Err(e) => {
                retry_count += 1;
                if retry_count > max_retries {
                    eprintln!(
                        "[{service_name}] Registration failed after {max_retries} attempts: {e}",
                    );
                    return Ok(());
                }
                eprintln!(
                    "[{service_name}] Registration attempt {retry_count}/{max_retries}\
                     failed: {e}. Retrying in {retry_delay:?}...",
                );
                tokio::time::sleep(retry_delay).await;
                retry_delay = (retry_delay * 2).min(Duration::from_secs(5));
            }
        }
    }

    // Spawn sensor value simulation task (if sensor mode)
    if matches!(mode, DeviceMode::Sensor | DeviceMode::Hybrid) {
        let state_clone = state.clone();
        tokio::spawn(async move {
            simulate_sensor_values(state_clone, args.interval).await;
        });
    }

    eprintln!("[{service_name}] Device running (Ctrl+C to stop)");

    // Wait for server shutdown
    if let Err(e) = server_handle.await {
        eprintln!("[{service_name}] Server error: {e}");
    }

    Ok(())
}

/// Register RPC methods for sensor mode.
fn register_sensor_methods(server: &RpcServer, state: Arc<Mutex<DeviceState>>) {
    server.register("read-telemetry", move |_req: ()| {
        let state = state.clone();
        async move {
            let state = state.lock().await;
            let timestamp = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0);

            Ok(TelemetryMessage {
                device_id: state.device_id.clone(),
                device_type: state.device_type,
                timestamp,
                value: state.current_value,
            })
        }
    });
}

/// Register RPC methods for actuator mode.
fn register_actuator_methods(server: &RpcServer, state: Arc<Mutex<DeviceState>>) {
    // execute-command method
    let state_cmd = state.clone();
    server.register("execute-command", move |req: CommandRequest| {
        let state = state_cmd.clone();
        async move {
            let mut state = state.lock().await;
            state.current_value = req.target_value;

            eprintln!(
                "[{}] Executed command: set to {:.2}",
                state.device_id, req.target_value
            );

            Ok(CommandResponse {
                status: "ok".to_string(),
                message: Some(format!("Set to {:.2}", req.target_value)),
            })
        }
    });

    // read-state method
    server.register("read-state", move |_req: ()| {
        let state = state.clone();
        async move {
            let state = state.lock().await;
            let timestamp = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0);

            Ok(ActuatorState {
                current_value: state.current_value,
                timestamp,
            })
        }
    });
}

/// Simulate sensor value changes over time.
async fn simulate_sensor_values(state: Arc<Mutex<DeviceState>>, interval: u64) {
    loop {
        {
            let mut state = state.lock().await;
            // Random walk within bounds
            state.current_value += rand::random::<f64>() * 2.0 - 1.0;
            state.current_value = state.current_value.clamp(15.0, 30.0);
        }
        tokio::time::sleep(Duration::from_secs(interval)).await;
    }
}

fn parse_device_type(s: &str) -> anyhow::Result<DeviceType> {
    // --
    let device_type = match s {
        "temp" => DeviceType::Temperature,
        "humidity" => DeviceType::Humidity,
        "valve" => DeviceType::Valve,
        "propulsion" => DeviceType::Propulsion,
        _ => {
            bail!("Invalid device type: {s}");
        }
    };
    Ok(device_type)
}

fn parse_device_mode(s: &str) -> Result<DeviceMode> {
    // --
    let dev_mode = match s {
        "sensor" => DeviceMode::Sensor,
        "actuator" => DeviceMode::Actuator,
        "hybrid" => DeviceMode::Hybrid,
        _ => {
            bail!("Invalid device mode: {s}");
        }
    };
    Ok(dev_mode)
}
