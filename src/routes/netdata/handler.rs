use crate::routes::netdata::models::NetdataPayload;
use crate::state::AppState;
use axum::{
    Json,
    extract::{Path, State},
};
use ntfy::Payload;
use tracing::info;

pub async fn handle_netdata(
    State(state): State<AppState>,
    Path(channel): Path<String>,
    Json(payload): Json<NetdataPayload>,
) -> Json<NetdataPayload> {
    info!(
        "Received netdata webhook for channel {}: {:?}",
        channel, payload
    );

    let ntfy_payload = match &payload {
        NetdataPayload::Alert(alert) => Payload::new(&channel)
            .message(&alert.message)
            .title(&alert.alert)
            .tags(vec![alert.severity.clone(), alert.class.clone()]),
        NetdataPayload::Reachability(reach) => Payload::new(&channel)
            .message(&reach.message)
            .title(format!("Reachability: {}", reach.host))
            .tags(vec![reach.severity.clone(), reach.status.text.clone()]),
    };

    if let Err(e) = state.ntfy_client.send(&ntfy_payload).await {
        tracing::error!("Failed to send notification to ntfy: {:?}", e);
    }

    Json(payload)
}
