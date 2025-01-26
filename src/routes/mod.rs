// pub mod echo;
mod echo;
use axum::{
    extract::State,
    response::sse::{Event, Sse},
    Json,
};
pub use echo::*;

use axum::{body::Bytes, response::IntoResponse};
use futures::stream::{self, Stream};
use std::convert::Infallible;
use std::time::Duration;
use tokio_stream::StreamExt;
use tracing::Value;

use axum::{extract::Request, http::StatusCode};

use crate::{
    json_rpc::{JsonRpcError, JsonRpcRequest, JsonRpcResponse},
    AppError, AppState,
};

pub async fn sse_res() -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    // Create a stream that yields bytes every 100ms

    let bytes = Bytes::from("Hello, World!\n").to_vec();
    let chunks: Vec<_> = bytes.chunks(3).map(|x| x.to_vec()).collect();

    let s = stream::iter(
        chunks
            .into_iter()
            .map(|x| Ok(Event::default().data(String::from_utf8_lossy(x.as_ref())))),
    )
    .throttle(Duration::from_millis(100))
    .take(10); // Limit to 10 messages

    // Convert the stream into a response
    axum::response::sse::Sse::new(s)
}

pub async fn stream_res() -> impl IntoResponse {
    // Create a stream that yields bytes every 100ms

    // Have to make our own chunks because of the borrow checker
    let bytes = Bytes::from("Hello, World!\n").to_vec();
    let chunks: Vec<_> = bytes.chunks(3).map(|x| x.to_vec()).collect();

    let s = stream::iter(chunks.into_iter().map(|x| Ok::<_, std::io::Error>(x)))
        .throttle(Duration::from_millis(100));

    // Convert the stream into a response
    axum::response::Response::builder()
        .header("Content-Type", "application/octet-stream")
        .body(axum::body::Body::from_stream(s))
        .unwrap()
}

pub async fn stream_handler(request: Request) -> Result<String, (StatusCode, String)> {
    // Convert request body into a stream
    let body_stream = request.into_body().into_data_stream();

    // Collect all chunks into a vector
    let chunks: Vec<_> = body_stream
        .map(|result| {
            result.map_err(|err| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Failed to read stream: {}", err),
                )
            })
        })
        .collect::<Result<Vec<Bytes>, _>>()
        .await?;

    // Concatenate all chunks and convert to string
    let full_body = chunks.concat();
    println!("full_body: {:?}", String::from_utf8(full_body.to_vec()));
    String::from_utf8(full_body.to_vec())
        .map_err(|err| (StatusCode::BAD_REQUEST, format!("Invalid UTF-8: {}", err)))
}

pub async fn json_rpc(
    State(state): State<AppState>, // state must be listed first in params
    Json(payload): Json<JsonRpcRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    match state.registry.handle_request(payload).await {
        Ok(response) => Ok(Json(serde_json::to_value(response).unwrap())),
        Err(error) => Ok(Json(serde_json::to_value(error).unwrap())),
    }
}
