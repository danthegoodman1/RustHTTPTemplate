use axum::routing::post;
use axum::{middleware, ServiceExt};
use grpc::helloworld::helloworld::greeter_server;

use std::sync::Arc;
use std::time::Duration;
use tonic::transport::Server;

use axum::{
    error_handling::HandleErrorLayer,
    extract::DefaultBodyLimit,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use tower::{buffer::BufferLayer, steer::Steer, BoxError, ServiceBuilder, ServiceExt};
use tracing::{error, info};

mod grpc;
mod rate_limiter;
mod routes;
use rate_limiter::{ip_rate_limiter, RateLimiter};

#[derive(Clone)]
struct AppState {
    rate_limiter: Arc<RateLimiter>,
}

pub async fn start(addr: &str) {
    let state = AppState {
        rate_limiter: Arc::new(RateLimiter::new(10, Duration::from_secs(60))), // 10 requests per minute
    };

    let greeter_service = grpc::helloworld::MyGreeter::default();
    let grpc_service = Server::builder()
        .add_service(greeter_server::GreeterServer::new(greeter_service))
        .into_service()
        .map_response(|r| axum::response::Response::new(r.into_body()))
        .boxed_clone()
        .into_make_service();

    let app = axum::Router::new()
        .route("/echo/json", post(routes::echo_json))
        // .route(
        //     "/:key",
        //     get(routes::get::get_key).post(routes::post::write_key),
        // )
        .layer(
            ServiceBuilder::new()
                // https://github.com/tokio-rs/axum/discussions/987
                .layer(HandleErrorLayer::new(|err: BoxError| async move {
                    // turns layer errors into HTTP errors
                    error!("Unhandled error: {}", err);
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        format!("Unhandled error: {}", err),
                    )
                }))
                .layer(BufferLayer::new(1024))
                .layer(DefaultBodyLimit::max(1_000_000))
                // also see https://docs.rs/tower-http/0.6.1/tower_http/request_id/index.html#example
                .layer(tower::timeout::TimeoutLayer::new(Duration::from_secs(60))) // 30 second timeout
                .layer(middleware::from_fn_with_state(
                    state.clone(),
                    ip_rate_limiter,
                )),
        )
        .with_state(state)
        .into_make_service();

    let http_grpc = Steer::new(
        vec![app, grpc_service],
        |req: &axum::http::Request<_>, _svcs: &[_]| {
            if req
                .headers()
                .get(axum::http::header::CONTENT_TYPE)
                .map(|v| v.as_bytes())
                == Some(b"application/grpc")
            {
                1
            } else {
                0
            }
        },
    );

    info!("Starting on {}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();

    axum::serve(listener, http_grpc).await.unwrap();
}

// Make our own error that wraps `anyhow::Error`.
pub enum AppError {
    Anyhow(anyhow::Error),
    CustomCode(anyhow::Error, axum::http::StatusCode),
    RateLimited(anyhow::Error),
}

// Tell axum how to convert `AppError` into a response.
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        match self {
            AppError::Anyhow(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Something went wrong: {}", e),
            ),
            AppError::CustomCode(e, code) => (code, format!("{}", e)),
            AppError::RateLimited(e) => (
                StatusCode::TOO_MANY_REQUESTS,
                format!("Rate limit exceeded: {}", e),
            ),
        }
        .into_response()
    }
}

// This enables using `?` on functions that return `Result<_, anyhow::Error>` to turn them into
// `Result<_, AppError>`. That way you don't need to do that manually.
impl<E> From<E> for AppError
where
    E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        Self::Anyhow(err.into())
    }
}

impl AppError {
    pub fn rate_limited() -> Self {
        Self::CustomCode(
            anyhow::anyhow!("Rate limit exceeded"),
            StatusCode::TOO_MANY_REQUESTS,
        )
    }
}