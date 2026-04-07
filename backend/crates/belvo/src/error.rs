//! Error types for the Belvo client.

use thiserror::Error;

/// Result type alias using BelvoError.
pub type Result<T> = std::result::Result<T, BelvoError>;

/// Errors that can occur when interacting with the Belvo API.
#[derive(Error, Debug)]
pub enum BelvoError {
    /// HTTP request failed.
    #[error("HTTP request failed: {0}")]
    Request(#[from] reqwest::Error),

    /// JSON serialization/deserialization failed.
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// API returned an error response.
    #[error("Belvo API error: {code} - {message}")]
    Api { code: String, message: String },

    /// Authentication failed.
    #[error("Authentication failed: {0}")]
    Authentication(String),

    /// Resource not found.
    #[error("Resource not found: {0}")]
    NotFound(String),

    /// Rate limit exceeded.
    #[error("Rate limit exceeded, retry after {retry_after} seconds")]
    RateLimited { retry_after: u64 },

    /// Invalid webhook signature.
    #[error("Invalid webhook signature")]
    InvalidWebhookSignature,

    /// Configuration error.
    #[error("Configuration error: {0}")]
    Configuration(String),

    /// Invalid response from API.
    #[error("Invalid API response: {0}")]
    InvalidResponse(String),
}

/// Belvo API error response structure.
#[derive(Debug, serde::Deserialize)]
pub struct BelvoApiError {
    pub code: Option<String>,
    pub message: Option<String>,
    pub request_id: Option<String>,
}

impl From<BelvoApiError> for BelvoError {
    fn from(err: BelvoApiError) -> Self {
        BelvoError::Api {
            code: err.code.unwrap_or_else(|| "UNKNOWN".to_string()),
            message: err.message.unwrap_or_else(|| "Unknown error".to_string()),
        }
    }
}
