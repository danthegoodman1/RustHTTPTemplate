use futures_util::stream::StreamExt;
use reqwest::Client;
use serde_json::json;

const BASE_URL: &str = "http://localhost:8080";

#[tokio::test]
async fn test_echo_json() {
    let client = Client::new();
    let test_json = json!({
        "message": "hello",
        "number": 42
    });

    let response = client
        .post(format!("{}/echo/json", BASE_URL))
        .json(&test_json)
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), 200);
    let response_json = response.json::<serde_json::Value>().await.unwrap();
    assert_eq!(response_json, test_json);
}

#[tokio::test]
async fn test_sse_endpoint() {
    let client = Client::new();
    let response = client
        .get(format!("{}/sse", BASE_URL))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), 200);
    assert_eq!(
        response.headers().get("content-type").unwrap(),
        "text/event-stream"
    );

    let mut body = response.bytes_stream();
    let mut accumulated = String::new();

    while let Some(chunk) = body.next().await {
        let chunk = chunk.unwrap();
        let text = String::from_utf8_lossy(&chunk);
        accumulated.push_str(&text);
    }

    assert!(accumulated.contains("data: Hello, World!"));
}

#[tokio::test]
async fn test_stream_endpoint() {
    let client = Client::new();
    let response = client
        .get(format!("{}/stream", BASE_URL))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), 200);
    assert_eq!(
        response.headers().get("content-type").unwrap(),
        "application/octet-stream"
    );

    let body = response.text().await.unwrap();
    assert_eq!(body, "Hello, World!\n");
}