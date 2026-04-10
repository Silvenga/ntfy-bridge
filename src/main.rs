mod app;
mod config;
mod logging;
mod ntfy;
mod routes;
mod state;

use crate::app::AppBuilder;
use crate::config::Config;
use crate::ntfy::NtfyClientBuilder;
use std::panic;
use std::str::FromStr;
use tracing::{Level, info, subscriber, warn};
use tracing_subscriber::FmtSubscriber;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = Config::load();

    let level = Level::from_str(config.log_level()).unwrap_or(Level::INFO);

    let subscriber = FmtSubscriber::builder().with_max_level(level).finish();

    subscriber::set_global_default(subscriber)?;

    panic::set_hook(Box::new(tracing_panic::panic_hook));

    info!("Loaded config: {}", config);

    if config.api_token().is_none() {
        warn!("No API token configured. Server is running without authentication.");
    }

    info!("Starting ntfy-bridge server...");

    let ntfy_client =
        NtfyClientBuilder::new(config.ntfy_url(), config.ntfy_credentials()).build()?;

    let app = AppBuilder::new(ntfy_client, config.listen_addr().parse()?)
        .with_api_token(config.api_token().map(|s| s.to_owned()))
        .with_rate_limit(config.rate_limit_per_second(), config.rate_limit_burst())
        .with_use_x_forwarded_for(config.use_x_forwarded_for())
        .with_base_path(config.base_path().to_owned())
        .build()?;

    app.serve().await?;

    Ok(())
}
