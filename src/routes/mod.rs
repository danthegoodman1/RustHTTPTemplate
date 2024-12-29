// pub mod echo;
mod echo;
use axum::response::sse::{Event, Sse};
pub use echo::*;

use axum::{body::Bytes, response::IntoResponse};
use futures::stream::{self, Stream};
use std::convert::Infallible;
use std::time::Duration;
use tokio_stream::StreamExt;

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
