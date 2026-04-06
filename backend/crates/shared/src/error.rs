//! Application error types.

use actix_web::{http::StatusCode, HttpResponse, ResponseError};
use serde::Serialize;
use std::fmt;

/// Application error type that implements ResponseError for Actix
#[derive(Debug)]
pub enum AppError {
    /// Resource not found
    NotFound(String),

    /// Bad request / validation error
    BadRequest(String),

    /// Authentication required
    Unauthorized,

    /// Insufficient permissions
    Forbidden,

    /// Database error
    Database(String),

    /// Internal server error
    Internal(String),

    /// Invalid email or password
    InvalidCredentials,

    /// JWT token has expired
    TokenExpired,

    /// JWT token is invalid or malformed
    TokenInvalid,

    /// Email already registered
    EmailAlreadyExists,

    /// Password doesn't meet strength requirements
    PasswordTooWeak(String),
}

#[derive(Serialize)]
struct ErrorBody {
    error: ErrorDetail,
}

#[derive(Serialize)]
struct ErrorDetail {
    code: String,
    message: String,
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::NotFound(msg) => write!(f, "Not found: {}", msg),
            AppError::BadRequest(msg) => write!(f, "Bad request: {}", msg),
            AppError::Unauthorized => write!(f, "Unauthorized"),
            AppError::Forbidden => write!(f, "Forbidden"),
            AppError::Database(msg) => write!(f, "Database error: {}", msg),
            AppError::Internal(msg) => write!(f, "Internal error: {}", msg),
            AppError::InvalidCredentials => write!(f, "Invalid credentials"),
            AppError::TokenExpired => write!(f, "Token expired"),
            AppError::TokenInvalid => write!(f, "Invalid token"),
            AppError::EmailAlreadyExists => write!(f, "Email already registered"),
            AppError::PasswordTooWeak(msg) => write!(f, "Password too weak: {}", msg),
        }
    }
}

impl ResponseError for AppError {
    fn status_code(&self) -> StatusCode {
        match self {
            AppError::NotFound(_) => StatusCode::NOT_FOUND,
            AppError::BadRequest(_) => StatusCode::BAD_REQUEST,
            AppError::Unauthorized => StatusCode::UNAUTHORIZED,
            AppError::Forbidden => StatusCode::FORBIDDEN,
            AppError::Database(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::InvalidCredentials => StatusCode::UNAUTHORIZED,
            AppError::TokenExpired => StatusCode::UNAUTHORIZED,
            AppError::TokenInvalid => StatusCode::UNAUTHORIZED,
            AppError::EmailAlreadyExists => StatusCode::CONFLICT,
            AppError::PasswordTooWeak(_) => StatusCode::BAD_REQUEST,
        }
    }

    fn error_response(&self) -> HttpResponse {
        let (code, message) = match self {
            AppError::NotFound(msg) => ("NOT_FOUND", msg.clone()),
            AppError::BadRequest(msg) => ("BAD_REQUEST", msg.clone()),
            AppError::Unauthorized => ("UNAUTHORIZED", "Authentication required".to_string()),
            AppError::Forbidden => ("FORBIDDEN", "Insufficient permissions".to_string()),
            AppError::Database(_) => ("DATABASE_ERROR", "A database error occurred".to_string()),
            AppError::Internal(_) => ("INTERNAL_ERROR", "An internal error occurred".to_string()),
            AppError::InvalidCredentials => (
                "INVALID_CREDENTIALS",
                "Invalid email or password".to_string(),
            ),
            AppError::TokenExpired => ("TOKEN_EXPIRED", "Token has expired".to_string()),
            AppError::TokenInvalid => ("TOKEN_INVALID", "Invalid or malformed token".to_string()),
            AppError::EmailAlreadyExists => {
                ("EMAIL_EXISTS", "Email is already registered".to_string())
            }
            AppError::PasswordTooWeak(msg) => ("PASSWORD_WEAK", msg.clone()),
        };

        HttpResponse::build(self.status_code()).json(ErrorBody {
            error: ErrorDetail {
                code: code.to_string(),
                message,
            },
        })
    }
}

impl From<diesel::result::Error> for AppError {
    fn from(err: diesel::result::Error) -> Self {
        match err {
            diesel::result::Error::NotFound => AppError::NotFound("Resource not found".to_string()),
            _ => AppError::Database(err.to_string()),
        }
    }
}

impl From<diesel_async::pooled_connection::deadpool::PoolError> for AppError {
    fn from(err: diesel_async::pooled_connection::deadpool::PoolError) -> Self {
        AppError::Database(format!("Connection pool error: {}", err))
    }
}
