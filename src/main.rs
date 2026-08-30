//! Sunnypot's entrypoint.

use std::sync::Arc;

use sunnypot::app::App;
use sunnypot::capture::StdoutSink;
use sunnypot::clock::SystemClock;
use sunnypot::config::Config;

const DEFAULT_CONFIG_PATH: &str = "sunnypot.toml";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| DEFAULT_CONFIG_PATH.to_owned());
    let config = Config::load(&path)?;

    let mut app = App::bind(config, Arc::new(SystemClock), Arc::new(StdoutSink::new())).await?;
    eprintln!("sunnypot: modbus surface on {}", app.modbus_addr());

    app.wait().await;
    Ok(())
}
