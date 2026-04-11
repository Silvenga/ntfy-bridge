use axum::body::Bytes;
use axum::http::{HeaderMap, StatusCode};
use axum::response::IntoResponse;
use tracing::info;

pub async fn handle_dump(headers: HeaderMap, body: Bytes) -> impl IntoResponse {
    info!("Headers: {:?}", headers);

    if let Ok(text) = str::from_utf8(&body) {
        info!("{}", text);
    } else {
        info!("Body is not valid UTF-8: {:?}", body);
    }

    StatusCode::ACCEPTED
}
