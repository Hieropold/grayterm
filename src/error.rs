/// Error types for the Graylog API layer.
///
/// <purpose-start>
/// Provides typed errors for HTTP and API-level failures so callers can
/// distinguish network failures from server-side application errors such as
/// authentication failures or unexpected status codes.
/// [initial-implementation.md]
/// <purpose-end>
///
/// <side-effects-start>
/// None.
/// <side-effects-end>
#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    /// An underlying HTTP transport or connection error.
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    /// The server returned a non-2xx status code.
    #[error("Server error {status}: {body}")]
    Status { status: u16, body: String },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn status_error_includes_code_in_message() {
        let err = ApiError::Status {
            status: 401,
            body: "Unauthorized".to_string(),
        };
        let msg = err.to_string();
        assert!(msg.contains("401"), "message was: {msg}");
        assert!(msg.contains("Unauthorized"), "message was: {msg}");
    }
}
