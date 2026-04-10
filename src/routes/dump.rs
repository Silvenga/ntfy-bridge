use axum::body::Bytes;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use tracing::info;

pub async fn handle_dump(body: Bytes) -> impl IntoResponse {
    if let Ok(text) = str::from_utf8(&body) {
        info!("{}", text);
    } else {
        info!("Body is not valid UTF-8: {:?}", body);
    }

    StatusCode::ACCEPTED
}
