use async_nats::Client;
use futures::StreamExt;
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Control request payload.
#[derive(Deserialize)]
#[allow(dead_code)]
struct ControlRequest {
    // ---
    command: String,
}

/// Control response payload.
#[derive(Serialize)]
#[allow(dead_code)]
struct ControlResponse {
    // ---
    status: &'static str,
}

/// Start the NATS control request/reply handler.
///
/// Listens for control commands and responds synchronously.
#[allow(dead_code)]
pub async fn start_control(client: Client) {
    // ---
    let mut sub = client
        .subscribe("edge.control")
        .await
        .expect("failed to subscribe to control subject");

    tokio::spawn(async move {
        // ---
        while let Some(msg) = sub.next().await {
            // ---
            let _req: ControlRequest =
                serde_json::from_slice(&msg.payload).expect("invalid control payload");

            let resp = ControlResponse { status: "ok" };
            if let Some(reply) = msg.reply {
                let _ = client
                    .publish(reply, serde_json::to_vec(&resp).unwrap().into())
                    .await;
            }
        }
    });
}

/// Start periodic heartbeat telemetry publication.
#[allow(dead_code)]
pub async fn start_heartbeat(client: Client) {
    // ---
    tokio::spawn(async move {
        // ---
        loop {
            let payload = serde_json::json!({
                "arch": std::env::consts::ARCH,
            });

            let _ = client
                .publish("edge.heartbeat", payload.to_string().into())
                .await;

            tokio::time::sleep(Duration::from_secs(5)).await;
        }
    });
}
