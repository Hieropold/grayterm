use crate::api::search::SearchRequest;
use crate::api::streams::{format_streams_text, StreamsResponse};
use crate::error::ApiError;
use crate::output::OutputFormat;
use anyhow::Result;
use futures_util::StreamExt;
use tokio::io::AsyncWriteExt;

/// HTTP client for the Graylog REST API.
///
/// <purpose-start>
/// Encapsulates the reqwest client, base URL, and token so that all
/// authentication and header boilerplate is in one place. Callers use
/// high-level methods (search, streams_list) instead of constructing
/// requests manually.
/// [initial-implementation.md]
/// <purpose-end>
///
/// <side-effects-start>
/// Holds a reqwest::Client which maintains a connection pool internally.
/// <side-effects-end>
pub struct GraylogClient {
    client: reqwest::Client,
    base_url: String,
    token: String,
}

impl GraylogClient {
    /// Creates a new GraylogClient.
    ///
    /// <purpose-start>
    /// Initialises the underlying reqwest client with rustls TLS and stores
    /// the base URL and access token for subsequent API calls.
    /// [initial-implementation.md]
    /// <purpose-end>
    ///
    /// <inputs-start>
    /// - `base_url`: Graylog server root URL (e.g. "https://graylog.example.com").
    ///   Must not include a trailing slash.
    /// - `token`: Graylog personal access token.
    /// <inputs-end>
    ///
    /// <outputs-start>
    /// Returns a GraylogClient or an error if the reqwest Client cannot be
    /// constructed (e.g. invalid TLS configuration).
    /// <outputs-end>
    ///
    /// <side-effects-start>
    /// Allocates a reqwest connection pool.
    /// <side-effects-end>
    pub fn new(base_url: String, token: String) -> Result<Self> {
        let client = reqwest::Client::builder()
            .build()
            .map_err(ApiError::Http)?;
        Ok(Self { client, base_url, token })
    }

    /// Searches Graylog messages and streams the response body to stdout.
    ///
    /// <purpose-start>
    /// Executes a POST /api/search/messages request and forwards the
    /// server-rendered response body directly to stdout with zero
    /// intermediate buffering. The Graylog server renders the output in the
    /// format requested via the Accept header (text/csv/json), so grayterm
    /// does not need to parse or reformat search results.
    /// [initial-implementation.md]
    /// <purpose-end>
    ///
    /// <inputs-start>
    /// - `req`: Search parameters to include in the request body.
    /// - `format`: Controls the Accept header and thereby the server's
    ///   rendering format.
    /// <inputs-end>
    ///
    /// <outputs-start>
    /// Returns Ok(()) after the response body has been written to stdout,
    /// or a descriptive error for HTTP/server failures.
    /// <outputs-end>
    ///
    /// <side-effects-start>
    /// - Makes an HTTP POST to the Graylog server.
    /// - Writes response bytes to stdout.
    /// <side-effects-end>
    pub async fn search(&self, req: &SearchRequest, format: OutputFormat) -> Result<()> {
        let url = format!("{}/api/search/messages", self.base_url);
        let body = req.to_body();

        let response = self
            .client
            .post(&url)
            .basic_auth(&self.token, Some("token"))
            .header("X-Requested-By", "grayterm")
            .header("Accept", format.accept_header())
            .json(&body)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status().as_u16();
            let body_text = response.text().await.unwrap_or_default();
            return Err(ApiError::Status { status, body: body_text }.into());
        }

        let mut stream = response.bytes_stream();
        let mut stdout = tokio::io::stdout();
        while let Some(chunk) = stream.next().await {
            let bytes = chunk.map_err(ApiError::Http)?;
            stdout.write_all(&bytes).await?;
        }
        stdout.flush().await?;
        Ok(())
    }

    /// Lists Graylog streams and writes the result to stdout.
    ///
    /// <purpose-start>
    /// Fetches GET /api/streams and writes to stdout. In text mode each
    /// stream is rendered as a "<id>\t<title>" line for scripting. In json
    /// mode the raw server response bytes are forwarded unchanged.
    /// [initial-implementation.md]
    /// <purpose-end>
    ///
    /// <inputs-start>
    /// - `format`: OutputFormat::Text produces id-tab-title lines;
    ///             OutputFormat::Json streams the raw JSON bytes.
    ///             OutputFormat::Csv is treated as Text (streams have no
    ///             CSV representation).
    /// <inputs-end>
    ///
    /// <outputs-start>
    /// Returns Ok(()) on success, or an error for HTTP or parse failures.
    /// <outputs-end>
    ///
    /// <side-effects-start>
    /// - Makes an HTTP GET to the Graylog server.
    /// - Writes to stdout.
    /// <side-effects-end>
    pub async fn streams_list(&self, format: OutputFormat) -> Result<()> {
        let url = format!("{}/api/streams", self.base_url);

        let response = self
            .client
            .get(&url)
            .basic_auth(&self.token, Some("token"))
            .header("Accept", "application/json")
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status().as_u16();
            let body_text = response.text().await.unwrap_or_default();
            return Err(ApiError::Status { status, body: body_text }.into());
        }

        match format {
            OutputFormat::Text | OutputFormat::Csv => {
                let data: StreamsResponse = response.json().await?;
                let text = format_streams_text(&data.streams);
                println!("{text}");
            }
            OutputFormat::Json => {
                let bytes = response.bytes().await?;
                let mut stdout = tokio::io::stdout();
                stdout.write_all(&bytes).await?;
                stdout.flush().await?;
            }
        }
        Ok(())
    }
}
