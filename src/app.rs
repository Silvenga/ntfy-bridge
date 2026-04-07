use crate::routes::{handle_netdata, health_check};
use crate::state::AppState;
use axum::{
    Router,
    extract::Request,
    routing::{get, post},
};
use std::sync::Arc;
use std::time::Duration;
use tower_governor::{GovernorLayer, governor::GovernorConfigBuilder};
use tower_http::{trace::TraceLayer, validate_request::ValidateRequestHeaderLayer};
use tracing::{Span, info};

pub fn app(
    state: AppState,
    api_token: Option<String>,
    rate_limit_per_second: u64,
    rate_limit_burst: u32,
) -> Router {
    let mut api_v1_routes = Router::new().route("/{channel}/netdata", post(handle_netdata));

    if let Some(token) = api_token {
        #[allow(deprecated)]
        // Fine for our usage.
        let layer = ValidateRequestHeaderLayer::bearer(&token);
        api_v1_routes = api_v1_routes.layer(layer);
    }

    let governor_conf = Arc::new(
        GovernorConfigBuilder::default()
            .per_second(rate_limit_per_second)
            .burst_size(rate_limit_burst)
            .use_headers()
            .finish()
            .expect("should have built governor config"),
    );

    Router::new()
        .route("/api/v1/health", get(health_check))
        .nest("/api/v1", api_v1_routes)
        .layer(GovernorLayer::new(governor_conf))
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(|request: &Request<_>| {
                    tracing::info_span!(
                        "http_request",
                        method = %request.method(),
                        uri = %request.uri(),
                    )
                })
                .on_request(|_request: &Request<_>, _span: &Span| {
                    info!("Started processing request");
                })
                .on_response(
                    |response: &axum::http::Response<_>, latency: Duration, _span: &Span| {
                        info!(
                            status = %response.status(),
                            latency = ?latency,
                            "Finished processing request"
                        );
                    },
                ),
        )
        .with_state(state)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ntfy::MockNtfyClient;
    use axum::body::Body;
    use axum::http::StatusCode;
    use http_body_util::BodyExt;
    use std::net::SocketAddr;
    use tower::{Service, ServiceExt};

    async fn test_app()
    -> impl Service<Request, Response = axum::response::Response, Error = std::convert::Infallible>
    + Clone {
        let mut mock_ntfy = MockNtfyClient::new();
        mock_ntfy.expect_send().returning(|_| Ok(()));
        let state = AppState {
            ntfy_client: Arc::new(mock_ntfy),
        };
        app(state, None, 100, 100)
            .into_make_service_with_connect_info::<SocketAddr>()
            .call(SocketAddr::from(([127, 0, 0, 1], 1234)))
            .await
            .expect("should have created service")
    }

    async fn test_app_with_auth(
        token: &str,
    ) -> impl Service<Request, Response = axum::response::Response, Error = std::convert::Infallible>
    + Clone {
        let mut mock_ntfy = MockNtfyClient::new();
        mock_ntfy.expect_send().returning(|_| Ok(()));
        let state = AppState {
            ntfy_client: Arc::new(mock_ntfy),
        };
        app(state, Some(token.to_string()), 100, 100)
            .into_make_service_with_connect_info::<SocketAddr>()
            .call(SocketAddr::from(([127, 0, 0, 1], 1234)))
            .await
            .expect("should have created service")
    }

    #[tokio::test]
    async fn when_get_health_then_should_return_ok() {
        let app = test_app().await;

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/health")
                    .body(Body::empty())
                    .expect("should have built health request"),
            )
            .await
            .expect("should have received health response");

        assert_eq!(response.status(), StatusCode::OK);
        let body = response
            .into_body()
            .collect()
            .await
            .expect("should have collected health body")
            .to_bytes();
        assert_eq!(&body[..], b"OK");
    }

    #[tokio::test]
    async fn when_get_health_with_auth_enabled_then_should_still_return_ok_without_token() {
        let app = test_app_with_auth("mytoken").await;

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/health")
                    .body(Body::empty())
                    .expect("should have built unauthenticated health request"),
            )
            .await
            .expect("should have received unauthenticated health response");

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn when_rate_limit_exceeded_then_should_return_too_many_requests() {
        let mut mock_ntfy = MockNtfyClient::new();
        mock_ntfy.expect_send().returning(|_| Ok(()));
        let state = AppState {
            ntfy_client: Arc::new(mock_ntfy),
        };
        // Rate limit 1 request per second, burst size 1
        let mut app = app(state, None, 1, 1).into_make_service_with_connect_info::<SocketAddr>();
        let addr = SocketAddr::from(([127, 0, 0, 1], 1234));

        // First request should pass
        let req1 = Request::builder()
            .uri("/api/v1/health")
            .body(Body::empty())
            .expect("should have built first request");
        let res1 = app
            .call(addr)
            .await
            .expect("should have created service for first request")
            .oneshot(req1)
            .await
            .expect("should have received response for first request");

        // Second request should fail
        let req2 = Request::builder()
            .uri("/api/v1/health")
            .body(Body::empty())
            .expect("should have built second request");
        let res2 = app
            .call(addr)
            .await
            .expect("should have created service for second request")
            .oneshot(req2)
            .await
            .expect("should have received response for second request");

        assert_eq!(res1.status(), StatusCode::OK);
        assert_eq!(res2.status(), StatusCode::TOO_MANY_REQUESTS);
        assert!(res2.headers().contains_key("retry-after"));
    }
}
