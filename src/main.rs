use anyhow::{anyhow, Result};
use rust_edge_agent::run;

#[tokio::main]
async fn main() -> Result<()> {
    // ---
    // ---
    tracing_subscriber::fmt().with_ansi(false).init();

    match run().await {
        Ok(_) => Ok(()),
        Err(err) => Err(anyhow!("Error:{err}")),
    }
}
