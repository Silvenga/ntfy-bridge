use axum::extract::{ConnectInfo, Request};
use axum::response::Response;
use axum_client_ip::ClientIp;
use futures_util::future::BoxFuture;
use std::net::SocketAddr;
use std::task::{Context, Poll};
use tokio::time::Instant;
use tower::{Layer, Service};
use tracing::info;

#[derive(Clone)]
pub struct LoggingLayer;

impl<S> Layer<S> for LoggingLayer {
    type Service = LoggingMiddleware<S>;

    fn layer(&self, service: S) -> Self::Service {
        LoggingMiddleware { inner: service }
    }
}

#[derive(Clone)]
pub struct LoggingMiddleware<S> {
    inner: S,
}

impl<S, ReqBody, ResBody> Service<Request<ReqBody>> for LoggingMiddleware<S>
where
    S: Service<Request<ReqBody>, Response = Response<ResBody>> + Clone + Send + 'static,
    S::Future: Send + 'static,
    ReqBody: Send + 'static,
    ResBody: Send + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request<ReqBody>) -> Self::Future {
        let mut inner = self.inner.clone();
        let start = Instant::now();

        let method = req.method().to_string();
        let path_and_query = req
            .uri()
            .path_and_query()
            .map(|pq| pq.to_string())
            .unwrap_or_else(|| req.uri().path().to_string());
        let extensions = req.extensions();
        let client_ip_captured = extensions
            .get::<ClientIp>()
            .map(|ip| ip.0.to_string())
            .or_else(|| {
                extensions
                    .get::<ConnectInfo<SocketAddr>>()
                    .map(|ci| ci.0.ip().to_string())
            });

        Box::pin(async move {
            let res = inner.call(req).await?;
            let latency = start.elapsed();
            let status = res.status().as_u16();

            let client_ip = client_ip_captured.unwrap_or_else(|| "unknown".to_string());

            info!(
                "{} {} -> {} ({}, {})",
                method,
                path_and_query,
                status,
                humantime::format_duration(latency),
                client_ip
            );

            Ok(res)
        })
    }
}
