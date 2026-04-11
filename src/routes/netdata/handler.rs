use crate::routes::netdata::{NetdataAlertStatus, NetdataPayload, NetdataReachabilitySeverity};
use crate::state::AppState;
use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use ntfy::{Payload, Priority};
use tracing::warn;
use url::Url;

pub async fn handle_netdata(
    State(state): State<AppState>,
    Path(topic): Path<String>,
    Json(netdata_payload): Json<NetdataPayload>,
) -> impl IntoResponse {
    let ntfy_payload = match &netdata_payload {
        NetdataPayload::Token(token) => Payload::new(&topic)
            .message(format!("{}\n\nToken: {}", token.message, token.token))
            .title(&token.title),
        NetdataPayload::Alert(alert) => {
            let priority = match alert.alert.state.status {
                NetdataAlertStatus::Critical => Priority::Max,
                NetdataAlertStatus::Warning => Priority::High,
                NetdataAlertStatus::Clear => Priority::Default,
            };

            let mut ntfy_payload = Payload::new(&topic)
                .message(&alert.message)
                .title(&alert.alert.rendered.info)
                .priority(priority)
                .tags(vec![alert.alert.config.classification.clone()]);

            if let Ok(url) = Url::parse(&alert.alert.url) {
                ntfy_payload = ntfy_payload.click(url);
            }
            ntfy_payload
        }
        NetdataPayload::Reachability(reach) => {
            let priority = match reach.severity {
                NetdataReachabilitySeverity::Critical => Priority::Max,
                NetdataReachabilitySeverity::Info => Priority::Default,
            };

            let hostname = reach
                .nodes
                .first()
                .map(|n| n.hostname.as_str())
                .unwrap_or("Unknown");

            let mut ntfy_payload = Payload::new(&topic)
                .message(&reach.message)
                .title(format!("Reachability: {}", hostname))
                .priority(priority)
                .tags(vec![reach.status.to_string()]);

            if let Ok(url) = Url::parse(&reach.url) {
                ntfy_payload = ntfy_payload.click(url);
            }
            ntfy_payload
        }
    };

    if let Err(e) = state.ntfy_client.send(&ntfy_payload).await {
        warn!("Failed to send notification to ntfy: {:?}", e);
    }

    StatusCode::ACCEPTED
}
