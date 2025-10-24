//! Error types for the Grok SDK

use std::fmt;

/// Result type alias for Grok operations
pub type Result<T> = std::result::Result<T, GrokError>;

/// Errors that can occur when using the Grok SDK
#[derive(Debug)]
pub enum GrokError {
    /// HTTP request failed
    Http(reqwest::Error),
    /// JSON serialization/deserialization failed
    Json(serde_json::Error),
    /// API returned an error response
    Api { status: u16, message: String },
    /// Invalid configuration or parameters
    InvalidConfig(String),
    /// Authentication failed
    Authentication(String),
    /// Rate limit exceeded
    RateLimit { retry_after: Option<u64> },
    /// Tool execution failed
    ToolExecution(String),
    /// Session operation failed
    Session(String),
    /// Collection operation failed
    Collection(String),
}

impl fmt::Display for GrokError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GrokError::Http(e) => write!(f, "HTTP error: {}", e),
            GrokError::Json(e) => write!(f, "JSON error: {}", e),
            GrokError::Api { status, message } => write!(f, "API error ({}): {}", status, message),
            GrokError::InvalidConfig(msg) => write!(f, "Invalid config: {}", msg),
            GrokError::Authentication(msg) => write!(f, "Authentication error: {}", msg),
            GrokError::RateLimit { retry_after } => {
                if let Some(seconds) = retry_after {
                    write!(f, "Rate limit exceeded, retry after {} seconds", seconds)
                } else {
                    write!(f, "Rate limit exceeded")
                }
            }
            GrokError::ToolExecution(msg) => write!(f, "Tool execution error: {}", msg),
            GrokError::Session(msg) => write!(f, "Session error: {}", msg),
            GrokError::Collection(msg) => write!(f, "Collection error: {}", msg),
        }
    }
}

impl std::error::Error for GrokError {}

impl From<reqwest::Error> for GrokError {
    fn from(err: reqwest::Error) -> Self {
        GrokError::Http(err)
    }
}

impl From<serde_json::Error> for GrokError {
    fn from(err: serde_json::Error) -> Self {
        GrokError::Json(err)
    }
}
