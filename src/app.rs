use crate::logging::LoggingLayer;
use crate::ntfy::NtfyClientShared;
use crate::routes::{handle_netdata, health_check, robots_txt};
use crate::state::AppState;
use anyhow::Context;
use axum::http::header::{CACHE_CONTROL, EXPIRES, SERVER};
use axum::http::{HeaderValue, StatusCode};
use axum::routing::{get, post};
use axum::{Router, serve};
use axum_client_ip::ClientIpSource;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;
use tower_governor::{GovernorLayer, governor::GovernorConfigBuilder};
use tower_http::set_header::SetResponseHeaderLayer;
use tower_http::validate_request::ValidateRequestHeaderLayer;
use tracing::info;

pub struct App {
    router: Router,
    listen_addr: SocketAddr,
}

impl App {
    pub async fn serve(self) -> anyhow::Result<()> {
        info!("Listening on {}", self.listen_addr);
        let listener = TcpListener::bind(self.listen_addr)
            .await
            .context("should have bound to address")?;
        serve(
            listener,
            self.router
                .into_make_service_with_connect_info::<SocketAddr>(),
        )
        .await
        .context("should have started axum server")?;

        Ok(())
    }
}

pub struct AppBuilder {
    ntfy_client: NtfyClientShared,
    api_token: Option<String>,
    rate_limit_per_second: u64,
    rate_limit_burst: u32,
    use_x_forwarded_for: bool,
    base_path: String,
    listen_addr: SocketAddr,
}

impl AppBuilder {
    pub fn new(ntfy_client: NtfyClientShared, listen_addr: SocketAddr) -> Self {
        Self {
            ntfy_client,
            api_token: None,
            rate_limit_per_second: 2,
            rate_limit_burst: 5,
            use_x_forwarded_for: false,
            base_path: "api".to_string(),
            listen_addr,
        }
    }

    pub fn with_api_token(mut self, api_token: Option<String>) -> Self {
        self.api_token = api_token;
        self
    }

    pub fn with_rate_limit(mut self, per_second: u64, burst: u32) -> Self {
        self.rate_limit_per_second = per_second;
        self.rate_limit_burst = burst;
        self
    }

    pub fn with_use_x_forwarded_for(mut self, use_x_forwarded_for: bool) -> Self {
        self.use_x_forwarded_for = use_x_forwarded_for;
        self
    }

    pub fn with_base_path(mut self, base_path: impl Into<String>) -> Self {
        self.base_path = base_path.into();
        self
    }

    pub fn build(self) -> anyhow::Result<App> {
        let mut api_routes = Router::new().route("/{topic}/netdata", post(handle_netdata));

        if let Some(token) = self.api_token {
            #[allow(deprecated)]
            // Fine for our usage.
            let layer = ValidateRequestHeaderLayer::bearer(&token);
            api_routes = api_routes.layer(layer);
        }

        let governor_conf = Arc::new(
            GovernorConfigBuilder::default()
                .per_second(self.rate_limit_per_second)
                .burst_size(self.rate_limit_burst)
                .use_headers()
                .finish()
                .context("should have built governor config")?,
        );

        let state = AppState {
            ntfy_client: self.ntfy_client,
        };

        let client_ip_source = if self.use_x_forwarded_for {
            ClientIpSource::RightmostXForwardedFor
        } else {
            ClientIpSource::ConnectInfo
        };

        let router = Router::new()
            .route(
                "/robots.txt",
                get(robots_txt).layer(Self::cache_header_layer(86400)),
            )
            .route(&format!("/{}/v1/health", self.base_path), get(health_check))
            .nest(&format!("/{}/v1", self.base_path), api_routes)
            .fallback(move || async move {
                (
                    StatusCode::NOT_FOUND,
                    [
                        (CACHE_CONTROL, format!("public, max-age={}", 3600)),
                        (EXPIRES, "3600".to_owned()),
                    ],
                )
            })
            .layer(GovernorLayer::new(governor_conf))
            .layer(LoggingLayer)
            .layer(SetResponseHeaderLayer::if_not_present(
                CACHE_CONTROL,
                HeaderValue::from_static("no-store, no-cache, must-revalidate, proxy-revalidate"),
            ))
            .layer(SetResponseHeaderLayer::if_not_present(
                EXPIRES,
                HeaderValue::from_static("0"),
            ))
            .layer(SetResponseHeaderLayer::overriding(
                SERVER,
                HeaderValue::from_static("ntfy-bridge"),
            ))
            .layer(client_ip_source.into_extension())
            .with_state(state);

        Ok(App {
            router,
            listen_addr: self.listen_addr,
        })
    }

    fn cache_header_layer(
        seconds: u64,
    ) -> (
        SetResponseHeaderLayer<HeaderValue>,
        SetResponseHeaderLayer<HeaderValue>,
    ) {
        let cache_control = format!("public, max-age={}", seconds);
        let expires = seconds.to_string();

        (
            SetResponseHeaderLayer::overriding(
                CACHE_CONTROL,
                HeaderValue::try_from(cache_control).expect("should have valid cache control"),
            ),
            SetResponseHeaderLayer::overriding(
                EXPIRES,
                HeaderValue::try_from(expires).expect("should have valid expires"),
            ),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ntfy::MockNtfyClient;
    use axum::body::Body;
    use axum::extract::Request;
    use axum::http::StatusCode;
    use http_body_util::BodyExt;
    use std::net::SocketAddr;
    use tower::{Service, ServiceExt};

    async fn build_test_app()
    -> impl Service<Request, Response = axum::response::Response, Error = std::convert::Infallible>
    + Clone {
        let mut mock_ntfy = MockNtfyClient::new();
        mock_ntfy.expect_send().returning(|_| Ok(()));

        AppBuilder::new(
            Arc::new(mock_ntfy),
            SocketAddr::from(([127, 0, 0, 1], 8080)),
        )
        .with_rate_limit(100, 100)
        .build()
        .expect("should have built test app")
        .router
        .into_make_service_with_connect_info::<SocketAddr>()
        .call(SocketAddr::from(([127, 0, 0, 1], 1234)))
        .await
        .expect("should have created service")
    }

    async fn build_test_app_with_ntfy_auth(
        token: &str,
    ) -> impl Service<Request, Response = axum::response::Response, Error = std::convert::Infallible>
    + Clone {
        let mut mock_ntfy = MockNtfyClient::new();
        mock_ntfy.expect_send().returning(|_| Ok(()));

        AppBuilder::new(
            Arc::new(mock_ntfy),
            SocketAddr::from(([127, 0, 0, 1], 8080)),
        )
        .with_api_token(Some(token.to_string()))
        .with_rate_limit(100, 100)
        .build()
        .expect("should have built test app with auth")
        .router
        .into_make_service_with_connect_info::<SocketAddr>()
        .call(SocketAddr::from(([127, 0, 0, 1], 1234)))
        .await
        .expect("should have created service")
    }

    #[tokio::test]
    async fn when_get_health_then_should_return_ok() {
        let app = build_test_app().await;

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
        assert_eq!(
            response.headers().get(SERVER),
            Some(&HeaderValue::from_static("ntfy-bridge"))
        );
        assert_eq!(
            response.headers().get(CACHE_CONTROL),
            Some(&HeaderValue::from_static(
                "no-store, no-cache, must-revalidate, proxy-revalidate"
            ))
        );
        assert_eq!(
            response.headers().get(EXPIRES),
            Some(&HeaderValue::from_static("0"))
        );
        let body = response
            .into_body()
            .collect()
            .await
            .expect("should have collected health body")
            .to_bytes();
        assert_eq!(&body[..], b"OK");
    }

    #[tokio::test]
    async fn when_get_health_with_ntfy_auth_enabled_then_should_still_return_ok_without_token() {
        let app = build_test_app_with_ntfy_auth("mytoken").await;

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
    async fn when_post_to_protected_route_with_valid_ntfy_auth_then_should_return_not_found() {
        let token = "mytoken";
        let app = build_test_app_with_ntfy_auth(token).await;

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/non-existent")
                    .header("Authorization", format!("Bearer {}", token))
                    .body(Body::empty())
                    .expect("should have built authenticated request"),
            )
            .await
            .expect("should have received authenticated response");

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
        assert_eq!(
            response.headers().get(CACHE_CONTROL),
            Some(&HeaderValue::from_static("public, max-age=3600"))
        );
        assert_eq!(
            response.headers().get(EXPIRES),
            Some(&HeaderValue::from_static("3600"))
        );
    }

    #[tokio::test]
    async fn when_get_robots_txt_then_should_return_disallow_all() {
        let app = build_test_app().await;

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/robots.txt")
                    .body(Body::empty())
                    .expect("should have built robots.txt request"),
            )
            .await
            .expect("should have received robots.txt response");

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            response.headers().get(CACHE_CONTROL),
            Some(&HeaderValue::from_static("public, max-age=86400"))
        );
        let body = response
            .into_body()
            .collect()
            .await
            .expect("should have collected robots.txt body")
            .to_bytes();
        assert_eq!(&body[..], b"User-agent: *\nDisallow: /");
    }

    #[tokio::test]
    async fn when_rate_limit_exceeded_then_should_return_too_many_requests() {
        let mut mock_ntfy = MockNtfyClient::new();
        mock_ntfy.expect_send().returning(|_| Ok(()));

        // Rate limit 1 request per second, burst size 1
        let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
        let mut app = AppBuilder::new(Arc::new(mock_ntfy), addr)
            .with_rate_limit(1, 1)
            .build()
            .expect("should have built rate limited test app")
            .router
            .into_make_service_with_connect_info::<SocketAddr>();
        let client_addr = SocketAddr::from(([127, 0, 0, 1], 1234));

        // First request should pass
        let req1 = Request::builder()
            .uri("/api/v1/health")
            .body(Body::empty())
            .expect("should have built first request");
        let res1 = app
            .call(client_addr)
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
            .call(client_addr)
            .await
            .expect("should have created service for second request")
            .oneshot(req2)
            .await
            .expect("should have received response for second request");

        assert_eq!(res1.status(), StatusCode::OK);
        assert_eq!(res2.status(), StatusCode::TOO_MANY_REQUESTS);
        assert!(res2.headers().contains_key("retry-after"));
    }

    #[tokio::test]
    async fn when_custom_base_path_then_should_be_available_at_custom_path() {
        let mut mock_ntfy = MockNtfyClient::new();
        mock_ntfy.expect_send().returning(|_| Ok(()));

        let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
        let app = AppBuilder::new(Arc::new(mock_ntfy), addr)
            .with_base_path("bridge".to_string())
            .build()
            .expect("should have built test app with custom base path")
            .router
            .into_make_service_with_connect_info::<SocketAddr>()
            .call(SocketAddr::from(([127, 0, 0, 1], 1234)))
            .await
            .expect("should have created service");

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/bridge/v1/health")
                    .body(Body::empty())
                    .expect("should have built health request"),
            )
            .await
            .expect("should have received health response");

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn when_custom_base_path_then_old_path_should_not_be_available() {
        let mut mock_ntfy = MockNtfyClient::new();
        mock_ntfy.expect_send().returning(|_| Ok(()));

        let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
        let app = AppBuilder::new(Arc::new(mock_ntfy), addr)
            .with_base_path("bridge".to_string())
            .build()
            .expect("should have built test app with custom base path")
            .router
            .into_make_service_with_connect_info::<SocketAddr>()
            .call(SocketAddr::from(([127, 0, 0, 1], 1234)))
            .await
            .expect("should have created service");

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/health")
                    .body(Body::empty())
                    .expect("should have built health request"),
            )
            .await
            .expect("should have received health response");

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }
}
