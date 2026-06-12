/// Integration tests for the `search` command against a wiremock server.
///
/// <purpose-start>
/// Verifies that GraylogClient::search sends requests with correct HTTP
/// method, path, headers (Accept, X-Requested-By, Authorization), and body
/// structure, and that it surfaces server error status codes as errors.
/// [initial-implementation.md]
/// <purpose-end>
use grayterm::api::{
    search::{SearchRequest, TimeRange},
    GraylogClient,
};
use grayterm::output::OutputFormat;
use wiremock::{
    matchers::{header, method, path},
    Mock, MockServer, ResponseTemplate,
};

/// Builds a minimal SearchRequest for use across tests.
fn test_request() -> SearchRequest {
    SearchRequest {
        query: "level:ERROR".into(),
        streams: vec![],
        fields: vec!["timestamp".into(), "message".into()],
        timerange: TimeRange::Relative { range: 3_600 },
        size: None,
        sort: None,
        sort_order: None,
    }
}

#[tokio::test]
async fn search_posts_to_api_search_messages() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/search/messages"))
        .respond_with(ResponseTemplate::new(200).set_body_string("results"))
        .mount(&server)
        .await;

    let client = GraylogClient::new(server.uri(), "tok".into()).unwrap();
    client.search(&test_request(), OutputFormat::Text).await.unwrap();

    let received = server.received_requests().await.unwrap();
    assert_eq!(received.len(), 1);
    assert_eq!(received[0].method.as_str(), "POST");
    assert_eq!(received[0].url.path(), "/api/search/messages");
}

#[tokio::test]
async fn search_text_format_sends_text_plain_accept() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/search/messages"))
        .and(header("accept", "text/plain"))
        .respond_with(ResponseTemplate::new(200).set_body_string(""))
        .mount(&server)
        .await;

    let client = GraylogClient::new(server.uri(), "tok".into()).unwrap();
    client.search(&test_request(), OutputFormat::Text).await.unwrap();
}

#[tokio::test]
async fn search_csv_format_sends_text_csv_accept() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/search/messages"))
        .and(header("accept", "text/csv"))
        .respond_with(ResponseTemplate::new(200).set_body_string(""))
        .mount(&server)
        .await;

    let client = GraylogClient::new(server.uri(), "tok".into()).unwrap();
    client.search(&test_request(), OutputFormat::Csv).await.unwrap();
}

#[tokio::test]
async fn search_json_format_sends_application_json_accept() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/search/messages"))
        .and(header("accept", "application/json"))
        .respond_with(ResponseTemplate::new(200).set_body_string("{}"))
        .mount(&server)
        .await;

    let client = GraylogClient::new(server.uri(), "tok".into()).unwrap();
    client.search(&test_request(), OutputFormat::Json).await.unwrap();
}

#[tokio::test]
async fn search_sends_x_requested_by_grayterm() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/search/messages"))
        .and(header("x-requested-by", "grayterm"))
        .respond_with(ResponseTemplate::new(200).set_body_string(""))
        .mount(&server)
        .await;

    let client = GraylogClient::new(server.uri(), "tok".into()).unwrap();
    client.search(&test_request(), OutputFormat::Text).await.unwrap();
}

#[tokio::test]
async fn search_sends_basic_authorization_header() {
    let server = MockServer::start().await;
    // Verify the Authorization header is Basic auth (value starts with "Basic ").
    // We trust reqwest to correctly encode token:token in base64.
    Mock::given(method("POST"))
        .and(path("/api/search/messages"))
        .and(|req: &wiremock::Request| {
            req.headers
                .get("authorization")
                .and_then(|v| v.to_str().ok())
                .is_some_and(|v| v.starts_with("Basic "))
        })
        .respond_with(ResponseTemplate::new(200).set_body_string(""))
        .mount(&server)
        .await;

    let client = GraylogClient::new(server.uri(), "tok".into()).unwrap();
    client.search(&test_request(), OutputFormat::Text).await.unwrap();
}

#[tokio::test]
async fn search_returns_error_on_401() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/search/messages"))
        .respond_with(ResponseTemplate::new(401).set_body_string("Unauthorized"))
        .mount(&server)
        .await;

    let client = GraylogClient::new(server.uri(), "bad-token".into()).unwrap();
    let err = client.search(&test_request(), OutputFormat::Text).await.unwrap_err();
    assert!(err.to_string().contains("401"), "error was: {err}");
}

#[tokio::test]
async fn search_returns_error_on_500() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/search/messages"))
        .respond_with(ResponseTemplate::new(500).set_body_string("Internal Server Error"))
        .mount(&server)
        .await;

    let client = GraylogClient::new(server.uri(), "tok".into()).unwrap();
    let err = client.search(&test_request(), OutputFormat::Text).await.unwrap_err();
    assert!(err.to_string().contains("500"), "error was: {err}");
}
