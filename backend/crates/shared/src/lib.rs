//! Shared models, DTOs, and utilities for Gastico.

use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// User model
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    #[serde(skip_serializing)]
    pub password_hash: String,
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Category model
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct Category {
    pub id: Uuid,
    pub user_id: Option<Uuid>,
    pub name: String,
    pub icon: Option<String>,
    pub color: Option<String>,
    pub is_system: bool,
    pub created_at: DateTime<Utc>,
}

/// Bank statement model
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct Statement {
    pub id: Uuid,
    pub user_id: Uuid,
    pub bank: String,
    pub filename: String,
    pub file_size: i64,
    pub status: String,
    pub transaction_count: Option<i32>,
    pub error_message: Option<String>,
    pub created_at: DateTime<Utc>,
    pub processed_at: Option<DateTime<Utc>>,
}

/// Transaction model
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct Transaction {
    pub id: Uuid,
    pub user_id: Uuid,
    pub statement_id: Uuid,
    pub category_id: Option<Uuid>,
    pub date: NaiveDate,
    pub description: String,
    pub amount: Decimal,
    pub balance: Option<Decimal>,
    pub reference: Option<String>,
    pub is_user_categorized: bool,
    pub created_at: DateTime<Utc>,
}

/// Pagination metadata for API responses
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginationMeta {
    pub total: i64,
    pub page: i32,
    pub per_page: i32,
    pub total_pages: i32,
}

/// Standard paginated response wrapper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginatedResponse<T> {
    pub data: Vec<T>,
    pub meta: PaginationMeta,
}

/// Standard error response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub error: ErrorDetail,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorDetail {
    pub code: String,
    pub message: String,
}

impl ErrorResponse {
    pub fn new(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            error: ErrorDetail {
                code: code.into(),
                message: message.into(),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_response() {
        let err = ErrorResponse::new("INVALID_FILE", "The uploaded file is not supported");
        assert_eq!(err.error.code, "INVALID_FILE");
    }
}
