//! mom-rpc transport helpers for MQTT-based device communication.
//!
//! Provides connection management, retry logic, and transport creation
//! for both RPC clients and servers in the edge agent system.

use anyhow::Result;
use mom_rpc::{create_transport, RpcConfig, TransportPtr};
use std::time::Duration;
use tokio::time::sleep;

/// Create MQTT transport with exponential backoff retry.
///
/// # Arguments
///
/// * `broker_url` - MQTT broker URL (e.g., "mqtt://localhost:1883")
/// * `client_id` - Unique client identifier for MQTT connection
///
/// # Behavior
///
/// - Retries with exponential backoff: 100ms, 200ms, 400ms, ..., 30s (max)
/// - Continues retrying indefinitely until connection succeeds
/// - Logs connection failures but does not crash
///
/// # Returns
///
/// A shared transport instance that can be cloned for both RpcClient and RpcServer.
///
/// # Example
///
/// ```no_run
/// use rust_edge_agent::messaging::create_transport_with_retry;
///
/// let transport = create_transport_with_retry(
///     "mqtt://localhost:1883",
///     "device-sensor-1"
/// ).await;
///
/// // Share transport between client and server
/// let server = RpcServer::with_transport(transport.clone(), "sensor-1");
/// let client = RpcClient::with_transport(transport.clone(), "sensor-1-client").await?;
/// ```
pub async fn create_transport_with_retry(broker_url: &str, client_id: &str) -> TransportPtr {
    let mut backoff = Duration::from_millis(100);

    loop {
        let config = RpcConfig::with_broker(broker_url, client_id);

        match create_transport(&config).await {
            Ok(transport) => {
                eprintln!("Connected to MQTT broker at {broker_url} (client_id: {client_id})",);
                return transport;
            }
            Err(e) => {
                eprintln!("MQTT connection failed: {e}, retrying in {backoff:?}...",);
                sleep(backoff).await;
                backoff = (backoff * 2).min(Duration::from_secs(30));
            }
        }
    }
}

/// Create MQTT transport without retry.
///
/// # Arguments
///
/// * `broker_url` - MQTT broker URL (e.g., "mqtt://localhost:1883")
/// * `client_id` - Unique client identifier for MQTT connection
///
/// # Errors
///
/// Returns error if initial connection fails. Use `create_transport_with_retry`
/// for resilient connection establishment.
pub async fn create_transport_once(broker_url: &str, client_id: &str) -> Result<TransportPtr> {
    let config = RpcConfig::with_broker(broker_url, client_id);
    let transport = create_transport(&config).await?;
    Ok(transport)
}
