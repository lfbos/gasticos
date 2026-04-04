//! Database models for Diesel ORM.
//!
//! These models map to database tables defined in the schema.

use chrono::{DateTime, NaiveDate, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::schema::{categories, statements, transactions, users};

// ============================================================================
// User
// ============================================================================

/// User model for reading from database
#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize)]
#[diesel(table_name = users)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct User {
    pub id: Uuid,
    pub email: String,
    #[serde(skip_serializing)]
    pub password_hash: String,
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// New user for inserting into database
#[derive(Debug, Insertable, Deserialize)]
#[diesel(table_name = users)]
pub struct NewUser {
    pub email: String,
    pub password_hash: String,
    pub name: String,
}

// ============================================================================
// Category
// ============================================================================

/// Category model for reading from database
#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize)]
#[diesel(table_name = categories)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Category {
    pub id: Uuid,
    pub user_id: Option<Uuid>,
    pub name: String,
    pub key: Option<String>,
    pub icon: Option<String>,
    pub color: Option<String>,
    pub is_system: bool,
    pub created_at: DateTime<Utc>,
}

/// New category for inserting into database
#[derive(Debug, Insertable, Deserialize)]
#[diesel(table_name = categories)]
pub struct NewCategory {
    pub user_id: Option<Uuid>,
    pub name: String,
    pub key: Option<String>,
    pub icon: Option<String>,
    pub color: Option<String>,
    pub is_system: bool,
}

// ============================================================================
// Statement
// ============================================================================

/// Statement status enum matching PostgreSQL enum
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, diesel_derive_enum::DbEnum)]
#[ExistingTypePath = "crate::schema::sql_types::StatementStatus"]
pub enum StatementStatus {
    Pending,
    Processing,
    Completed,
    Failed,
}

/// Bank statement model for reading from database
#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize)]
#[diesel(table_name = statements)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Statement {
    pub id: Uuid,
    pub user_id: Uuid,
    pub bank: String,
    pub filename: String,
    pub file_path: Option<String>,
    pub file_size: i64,
    pub status: StatementStatus,
    pub transaction_count: Option<i32>,
    pub error_message: Option<String>,
    pub created_at: DateTime<Utc>,
    pub processed_at: Option<DateTime<Utc>>,
}

/// New statement for inserting into database
#[derive(Debug, Insertable)]
#[diesel(table_name = statements)]
pub struct NewStatement {
    pub user_id: Uuid,
    pub bank: String,
    pub filename: String,
    pub file_path: Option<String>,
    pub file_size: i64,
}

// ============================================================================
// Transaction
// ============================================================================

/// Transaction model for reading from database
#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize)]
#[diesel(table_name = transactions)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Transaction {
    pub id: Uuid,
    pub user_id: Uuid,
    pub statement_id: Option<Uuid>,
    pub category_id: Option<Uuid>,
    pub date: NaiveDate,
    pub description: String,
    pub amount: bigdecimal::BigDecimal,
    pub balance: Option<bigdecimal::BigDecimal>,
    pub reference: Option<String>,
    pub is_income: bool,
    pub is_user_categorized: bool,
    pub created_at: DateTime<Utc>,
}

/// New transaction for inserting into database
#[derive(Debug, Insertable)]
#[diesel(table_name = transactions)]
pub struct NewTransaction {
    pub user_id: Uuid,
    pub statement_id: Option<Uuid>,
    pub category_id: Option<Uuid>,
    pub date: NaiveDate,
    pub description: String,
    pub amount: bigdecimal::BigDecimal,
    pub balance: Option<bigdecimal::BigDecimal>,
    pub reference: Option<String>,
    pub is_income: bool,
}

// ============================================================================
// DTOs (Data Transfer Objects for API)
// ============================================================================

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
