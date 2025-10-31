//! Error types for the Grok SDK
//!
//! This module provides comprehensive error handling for the SDK using `thiserror`.
//! All errors implement `std::error::Error` and provide detailed context.

use thiserror::Error;

/// Result type alias for Grok operations
pub type Result<T> = std::result::Result<T, GrokError>;

/// Errors that can occur when using the Grok SDK
#[derive(Debug, Error)]
pub enum GrokError {
    /// HTTP request failed
    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),

    /// JSON serialization/deserialization failed
    #[error("JSON processing failed: {0}")]
    Json(#[from] serde_json::Error),

    /// API returned an error response
    #[error("API error (status {status}): {message}")]
    Api {
        /// HTTP status code
        status: u16,
        /// Error message from the API
        message: String,
    },

    /// Invalid configuration or parameters
    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),

    /// Authentication failed
    #[error("Authentication failed: {0}")]
    Authentication(String),

    /// Rate limit exceeded
    #[error("Rate limit exceeded{}", match .retry_after {
        Some(seconds) => format!(", retry after {} seconds", seconds),
        None => String::new(),
    })]
    RateLimit {
        /// Seconds until retry is allowed
        retry_after: Option<u64>,
    },

    /// Tool execution failed
    #[error("Tool execution failed: {0}")]
    ToolExecution(String),

    /// Session operation failed
    #[error("Session operation failed: {0}")]
    Session(String),

    /// Collection operation failed
    #[error("Collection operation failed: {0}")]
    Collection(String),

    /// Database operation failed
    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),

    /// JSON Schema validation failed
    #[error("JSON Schema validation failed: {0}")]
    SchemaValidation(String),

    /// Invalid API key format
    #[error("Invalid API key: {0}")]
    InvalidApiKey(String),
}
