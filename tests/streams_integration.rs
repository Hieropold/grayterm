/// Integration tests for the `streams list` command against a wiremock server.
///
/// <purpose-start>
/// Verifies that GraylogClient::streams_list GETs the correct endpoint with
/// the correct Authorization header, and that server error status codes are
/// surfaced as errors.
/// [initial-implementation.md]
/// <purpose-end>
use grayterm::api::GraylogClient;
use grayterm::output::OutputFormat;
use serde_json::json;
use wiremock::{
    matchers::{method, path},
    Mock, MockServer, ResponseTemplate,
};

fn streams_payload() -> serde_json::Value {
    json!({
        "streams": [
            {"id": "abc123", "title": "My Stream", "disabled": false},
            {"id": "def456", "title": "Another Stream", "disabled": false}
        ],
        "total": 2
    })
}

#[tokio::test]
async fn streams_list_gets_api_streams() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/streams"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&streams_payload()))
        .mount(&server)
        .await;

    let client = GraylogClient::new(server.uri(), "tok".into()).unwrap();
    client.streams_list(OutputFormat::Text).await.unwrap();

    let received = server.received_requests().await.unwrap();
    assert_eq!(received.len(), 1);
    assert_eq!(received[0].method.as_str(), "GET");
    assert_eq!(received[0].url.path(), "/api/streams");
}

#[tokio::test]
async fn streams_list_sends_basic_authorization_header() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/streams"))
        .and(|req: &wiremock::Request| {
            req.headers
                .get("authorization")
                .and_then(|v| v.to_str().ok())
                .is_some_and(|v| v.starts_with("Basic "))
        })
        .respond_with(ResponseTemplate::new(200).set_body_json(&streams_payload()))
        .mount(&server)
        .await;

    let client = GraylogClient::new(server.uri(), "tok".into()).unwrap();
    client.streams_list(OutputFormat::Text).await.unwrap();
}

#[tokio::test]
async fn streams_list_json_mode_requests_application_json() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/streams"))
        .and(wiremock::matchers::header("accept", "application/json"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&streams_payload()))
        .mount(&server)
        .await;

    let client = GraylogClient::new(server.uri(), "tok".into()).unwrap();
    client.streams_list(OutputFormat::Json).await.unwrap();
}

#[tokio::test]
async fn streams_list_returns_error_on_401() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/streams"))
        .respond_with(ResponseTemplate::new(401).set_body_string("Unauthorized"))
        .mount(&server)
        .await;

    let client = GraylogClient::new(server.uri(), "bad-token".into()).unwrap();
    let err = client.streams_list(OutputFormat::Text).await.unwrap_err();
    assert!(err.to_string().contains("401"), "error was: {err}");
}

#[tokio::test]
async fn streams_list_handles_empty_streams_list() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/streams"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(&json!({"streams": [], "total": 0})),
        )
        .mount(&server)
        .await;

    let client = GraylogClient::new(server.uri(), "tok".into()).unwrap();
    // Should complete without error even with no streams.
    client.streams_list(OutputFormat::Text).await.unwrap();
}
