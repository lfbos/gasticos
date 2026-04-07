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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_belvo_error_display() {
        let err = BelvoError::Api {
            code: "invalid_token".to_string(),
            message: "Token has expired".to_string(),
        };
        assert_eq!(
            err.to_string(),
            "Belvo API error: invalid_token - Token has expired"
        );
    }

    #[test]
    fn test_belvo_error_authentication() {
        let err = BelvoError::Authentication("Invalid credentials".to_string());
        assert_eq!(
            err.to_string(),
            "Authentication failed: Invalid credentials"
        );
    }

    #[test]
    fn test_belvo_error_not_found() {
        let err = BelvoError::NotFound("Link not found".to_string());
        assert_eq!(err.to_string(), "Resource not found: Link not found");
    }

    #[test]
    fn test_belvo_error_rate_limited() {
        let err = BelvoError::RateLimited { retry_after: 60 };
        assert_eq!(
            err.to_string(),
            "Rate limit exceeded, retry after 60 seconds"
        );
    }

    #[test]
    fn test_belvo_error_invalid_webhook_signature() {
        let err = BelvoError::InvalidWebhookSignature;
        assert_eq!(err.to_string(), "Invalid webhook signature");
    }

    #[test]
    fn test_belvo_error_configuration() {
        let err = BelvoError::Configuration("Missing API key".to_string());
        assert_eq!(err.to_string(), "Configuration error: Missing API key");
    }

    #[test]
    fn test_belvo_error_invalid_response() {
        let err = BelvoError::InvalidResponse("Unexpected format".to_string());
        assert_eq!(err.to_string(), "Invalid API response: Unexpected format");
    }

    #[test]
    fn test_belvo_api_error_conversion() {
        let api_error = BelvoApiError {
            code: Some("auth_error".to_string()),
            message: Some("Invalid token".to_string()),
            request_id: Some("req123".to_string()),
        };
        let err: BelvoError = api_error.into();
        match err {
            BelvoError::Api { code, message } => {
                assert_eq!(code, "auth_error");
                assert_eq!(message, "Invalid token");
            }
            _ => panic!("Expected Api error"),
        }
    }

    #[test]
    fn test_belvo_api_error_conversion_with_none() {
        let api_error = BelvoApiError {
            code: None,
            message: None,
            request_id: None,
        };
        let err: BelvoError = api_error.into();
        match err {
            BelvoError::Api { code, message } => {
                assert_eq!(code, "UNKNOWN");
                assert_eq!(message, "Unknown error");
            }
            _ => panic!("Expected Api error"),
        }
    }

    #[test]
    fn test_belvo_api_error_deserialization() {
        let json =
            r#"{"code": "invalid_link", "message": "Link not found", "request_id": "abc123"}"#;
        let api_error: BelvoApiError = serde_json::from_str(json).unwrap();
        assert_eq!(api_error.code, Some("invalid_link".to_string()));
        assert_eq!(api_error.message, Some("Link not found".to_string()));
        assert_eq!(api_error.request_id, Some("abc123".to_string()));
    }
}
