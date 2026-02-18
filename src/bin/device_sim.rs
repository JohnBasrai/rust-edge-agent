//! Device simulator for testing edge agent.
//!
//! Simulates sensors, actuators, or hybrid devices using mom-rpc over MQTT.
//! Each device registers with the agent on startup and responds to RPC calls.

use anyhow::{bail, Result};
use clap::Parser;
use mom_rpc::{RpcBroker, RpcBrokerBuilder, TransportBuilder};
use rust_edge_agent::messaging::{
    // ---
    ActuatorState,
    CommandRequest,
    CommandResponse,
    DeviceMode,
    DeviceType,
    RegisterRequest,
    RegisterResponse,
    TelemetryMessage,
};
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::sync::Mutex;

#[derive(Parser, Debug)]
#[command(version, about)]
struct Args {
    // ---
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
    // ---
    device_id: String,
    device_type: DeviceType,
    current_value: f64,
}

#[tokio::main]
async fn main() -> Result<()> {
    // ---
    tracing_subscriber::fmt().with_ansi(false).init();

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
    let transport = TransportBuilder::new()
        .uri(&args.mqtt_broker)
        .node_id(&service_name)
        .full_duplex()
        .build()
        .await?;

    // Create bidirectional RPC broker (handles both incoming agent calls
    // and outbound registration request on a single MQTT connection)
    let broker = RpcBrokerBuilder::new(transport)
        .retry_max_attempts(1000)
        .retry_initial_delay(Duration::from_millis(200))
        .retry_max_delay(Duration::from_secs(5))
        .request_total_timeout(Duration::from_secs(3600))
        .build()?;
    // max delay is 1000 x 5 => 5000 seconds, but request_total_timeout will
    // shorten it to 3600.  The request_to_with_timeout will increase
    // request_to_with_timeout.

    // Shared state for sensor value simulation
    let state = Arc::new(Mutex::new(DeviceState {
        device_id: args.id.clone(),
        device_type,
        current_value: 20.0,
    }));

    // Register methods based on mode
    match mode {
        DeviceMode::Sensor => {
            register_sensor_methods(&broker, state.clone())?;
        }
        DeviceMode::Actuator => {
            register_actuator_methods(&broker, state.clone())?;
        }
        DeviceMode::Hybrid => {
            register_sensor_methods(&broker, state.clone())?;
            register_actuator_methods(&broker, state.clone())?;
        }
    }

    // Spawn broker receive loop
    let broker_handle = broker.clone().spawn()?;

    // Register with agent (with retry)
    eprintln!("[{service_name}] Registering with agent...");
    let register_req = RegisterRequest {
        device_id: args.id.clone(),
        device_type,
        mode,
    };

    // Register with agent using a very long timeout. In real deployments, the
    // agent and devices start independently with no guaranteed ordering.  For
    // example, in a vehicle telematics system, sensor nodes on the CAN bus may
    // power up before the gateway agent has finished booting or reconnected
    // after a network flap. Rather than failing fast, devices wait patiently
    // for the agent to become reachable. The broker handles retries internally;
    // the long ceiling here covers realistic startup delays without requiring
    // external orchestration.
    eprintln!("[{service_name}] Registering with agent...");
    let timeout_seconds = 5000;
    let response: RegisterResponse = broker
        // Overriding default timeout.
        .request_to_with_timeout(
            "agent",
            "register-device",
            register_req.clone(),
            Duration::from_secs(timeout_seconds),
        )
        .await
        .map_err(|err| {
            anyhow::anyhow!(
                "[{service_name}] Registration failed after {timeout_seconds} seconds: {err}"
            )
        })?;

    if !response.accepted {
        bail!(
            "[{service_name}] Registration rejected: {:?}",
            response.message
        );
    };
    eprintln!("[{service_name}] Registration accepted");

    // Spawn sensor value simulation task (if sensor mode)
    if matches!(mode, DeviceMode::Sensor | DeviceMode::Hybrid) {
        let state_clone = state.clone();
        tokio::spawn(async move {
            simulate_sensor_values(state_clone, args.interval).await;
        });
    }

    eprintln!("[{service_name}] Device running (Ctrl+C to stop)");

    // Wait for broker shutdown
    if let Err(e) = broker_handle.await {
        eprintln!("[{service_name}] Broker error: {e}");
    }

    Ok(())
}

/// Register RPC methods for sensor mode.
fn register_sensor_methods(broker: &RpcBroker, state: Arc<Mutex<DeviceState>>) -> Result<()> {
    // ---

    broker.register_rpc_handler("read-telemetry", move |_req: ()| {
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
    })?;
    Ok(())
}

/// Register RPC methods for actuator mode.
fn register_actuator_methods(broker: &RpcBroker, state: Arc<Mutex<DeviceState>>) -> Result<()> {
    // ---
    // execute-command method
    let state_cmd = state.clone();
    broker.register_rpc_handler("execute-command", move |req: CommandRequest| {
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
    })?;

    // read-state method
    broker.register_rpc_handler("read-state", move |_req: ()| {
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
    })?;
    Ok(())
}

/// Simulate sensor value changes over time.
async fn simulate_sensor_values(state: Arc<Mutex<DeviceState>>, interval: u64) {
    // ---

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
