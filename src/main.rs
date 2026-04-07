mod app;
mod config;
mod ntfy;
mod routes;
mod state;

use ntfy::NtfyHttpClientBuilder;
use state::AppState;
use std::{net::SocketAddr, panic, sync::Arc};
use tracing::{Level, info, warn};
use tracing_subscriber::FmtSubscriber;

#[tokio::main]
async fn main() {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::TRACE)
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    panic::set_hook(Box::new(tracing_panic::panic_hook));

    let config = config::Config::load();

    if config.api_token().is_none() {
        warn!("No API token configured. Server is running without authentication.");
    }

    info!("Starting ntfy-bridge server...");

    let mut ntfy_builder = NtfyHttpClientBuilder::new(config.ntfy_url());
    if let Some((username, password)) = config.ntfy_credentials() {
        ntfy_builder = ntfy_builder.with_credentials(username, password);
    } else if let Some(token) = config.ntfy_token() {
        ntfy_builder = ntfy_builder.with_token(token);
    }
    let ntfy_client = Arc::new(ntfy_builder.build().expect("failed to build ntfy client"));

    let state = AppState { ntfy_client };

    let api_token = config.api_token().map(|s| s.to_owned());
    let rate_limit_per_second = config.rate_limit_per_second();
    let rate_limit_burst = config.rate_limit_burst();
    let app = app::app(state, api_token, rate_limit_per_second, rate_limit_burst);

    let addr: SocketAddr = config
        .listen_addr()
        .parse()
        .expect("invalid listen address");
    info!("Listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("should have bound to address");
    axum::serve(listener, app)
        .await
        .expect("should have started axum server");
}
