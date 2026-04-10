use crate::routes::netdata::models::NetdataPayload;
use crate::state::AppState;
use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use ntfy::Payload;
use tracing::{error, info};

pub async fn handle_netdata(
    State(state): State<AppState>,
    Path(topic): Path<String>,
    Json(payload): Json<NetdataPayload>,
) -> impl IntoResponse {
    info!(
        "Received netdata webhook for topic {}: {:?}",
        topic, payload
    );

    let ntfy_payload = match &payload {
        NetdataPayload::Token(token) => Payload::new(&topic)
            .message(format!("{} Token: {}", token.message, token.token))
            .title(&token.title),
        NetdataPayload::Alert(alert) => Payload::new(&topic)
            .message(&alert.message)
            .title(&alert.alert)
            .tags(vec![alert.severity.clone(), alert.class.clone()]),
        NetdataPayload::Reachability(reach) => Payload::new(&topic)
            .message(&reach.message)
            .title(format!("Reachability: {}", reach.host))
            .tags(vec![reach.severity.clone(), reach.status.text.clone()]),
    };

    if let Err(e) = state.ntfy_client.send(&ntfy_payload).await {
        error!("Failed to send notification to ntfy: {:?}", e);
    }

    StatusCode::ACCEPTED
}
