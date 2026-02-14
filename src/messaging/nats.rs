use async_nats::Client;
use std::time::Duration;

/// Connect to NATS with exponential backoff retry.
///
/// # Behavior
///
/// - Retries with exponential backoff: 1s, 2s, 4s, 8s, 16s, 30s (max)
/// - Continues retrying indefinitely until connection succeeds
/// - Logs connection failures but does not crash
///
/// This implements resilient edge behavior for UI-less embedded systems.
pub async fn connect_with_retry(url: &str) -> Client {
    // ---
    let mut retry_count = 0;

    loop {
        match async_nats::connect(url).await {
            Ok(client) => {
                eprintln!("Connected to NATS at {url}");
                return client;
            }
            Err(e) => {
                let delay = if retry_count < 5 {
                    1 << retry_count
                } else {
                    30_u64
                };

                eprintln!("NATS connection failed: {e}, retrying in {delay}s...",);
                tokio::time::sleep(Duration::from_secs(delay)).await;
                retry_count += 1;
            }
        }
    }
}
