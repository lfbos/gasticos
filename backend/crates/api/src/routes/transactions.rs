//! Transaction routes.
//!
//! Endpoints:
//! - GET  /transactions           - List transactions with pagination and filters
//! - GET  /transactions/{id}      - Get a single transaction
//! - PATCH /transactions/{id}     - Update transaction category

use actix_web::{get, patch, web, HttpResponse, Responder};
use chrono::NaiveDate;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use serde::{Deserialize, Serialize};
use shared::{
    error::AppError,
    models::Transaction,
    schema::{categories, statements, transactions},
    DbPool,
};
use uuid::Uuid;

use crate::extractors::AuthUser;

/// Convert BigDecimal to f64 using string parsing.
fn decimal_to_f64(d: &bigdecimal::BigDecimal) -> f64 {
    d.to_string().parse().unwrap_or(0.0)
}

// ============================================================================
// Response DTOs
// ============================================================================

/// Transaction response DTO.
#[derive(Debug, Serialize)]
pub struct TransactionResponse {
    pub id: Uuid,
    pub date: String,
    pub description: String,
    pub amount: f64,
    pub balance: Option<f64>,
    pub reference: Option<String>,
    pub is_income: bool,
    pub category_id: Option<Uuid>,
    pub category_name: Option<String>,
    pub category_key: Option<String>,
    pub category_color: Option<String>,
    pub category_icon: Option<String>,
    pub is_user_categorized: bool,
    pub bank: Option<String>,
    pub created_at: String,
}

impl TransactionResponse {
    fn from_row(
        txn: Transaction,
        category_name: Option<String>,
        category_key: Option<String>,
        category_color: Option<String>,
        category_icon: Option<String>,
        bank: Option<String>,
    ) -> Self {
        Self {
            id: txn.id,
            date: txn.date.format("%Y-%m-%d").to_string(),
            description: txn.description,
            amount: decimal_to_f64(&txn.amount),
            balance: txn.balance.as_ref().map(decimal_to_f64),
            reference: txn.reference,
            is_income: txn.is_income,
            category_id: txn.category_id,
            category_name,
            category_key,
            category_color,
            category_icon,
            is_user_categorized: txn.is_user_categorized,
            bank,
            created_at: txn.created_at.to_rfc3339(),
        }
    }
}

/// Paginated transactions response.
#[derive(Debug, Serialize)]
pub struct PaginatedTransactionsResponse {
    pub data: Vec<TransactionResponse>,
    pub meta: PaginationMeta,
}

/// Pagination metadata.
#[derive(Debug, Serialize)]
pub struct PaginationMeta {
    pub total: i64,
    pub page: i64,
    pub per_page: i64,
    pub total_pages: i64,
}

// ============================================================================
// Request DTOs
// ============================================================================

/// Query parameters for listing transactions.
#[derive(Debug, Deserialize)]
pub struct ListTransactionsQuery {
    /// Page number (1-indexed)
    pub page: Option<i64>,
    /// Items per page (max 100)
    pub per_page: Option<i64>,
    /// Filter by category IDs (comma-separated)
    pub category_ids: Option<String>,
    /// Filter by start date (YYYY-MM-DD)
    pub date_from: Option<String>,
    /// Filter by end date (YYYY-MM-DD)
    pub date_to: Option<String>,
    /// Filter by income/expense
    pub is_income: Option<bool>,
    /// Search in description
    pub search: Option<String>,
    /// Sort order: "asc" or "desc" (default: desc)
    pub sort_order: Option<String>,
    /// Filter by bank name (from statement)
    pub bank: Option<String>,
}

/// Request to update a transaction's category.
#[derive(Debug, Deserialize)]
pub struct UpdateTransactionCategoryRequest {
    pub category_id: Option<Uuid>,
}

// ============================================================================
// Handlers
// ============================================================================

/// List transactions with pagination and filters.
///
/// GET /api/v1/transactions
#[get("/transactions")]
pub async fn list_transactions(
    auth_user: AuthUser,
    query: web::Query<ListTransactionsQuery>,
    pool: web::Data<DbPool>,
) -> Result<impl Responder, AppError> {
    let mut conn = pool.get().await?;

    let page = query.page.unwrap_or(1).max(1);
    let per_page = query.per_page.unwrap_or(20).clamp(1, 100);
    let offset = (page - 1) * per_page;

    // Get statement IDs for bank filter (needed for both count and data queries)
    let bank_statement_ids: Option<Vec<Uuid>> = if let Some(ref bank) = query.bank {
        Some(
            statements::table
                .filter(statements::user_id.eq(auth_user.user_id))
                .filter(statements::bank.eq(bank))
                .select(statements::id)
                .load(&mut conn)
                .await
                .map_err(|e| AppError::Database(e.to_string()))?,
        )
    } else {
        None
    };

    // Build base query for data
    let mut base_query = transactions::table
        .filter(transactions::user_id.eq(auth_user.user_id))
        .into_boxed();

    // Build count query with same filters
    let mut count_query = transactions::table
        .filter(transactions::user_id.eq(auth_user.user_id))
        .into_boxed();

    // Apply filters to both queries
    if let Some(ref cat_ids_str) = query.category_ids {
        // Parse comma-separated UUIDs
        let cat_ids: Vec<Uuid> = cat_ids_str
            .split(',')
            .filter_map(|s| Uuid::parse_str(s.trim()).ok())
            .collect();
        if !cat_ids.is_empty() {
            base_query = base_query.filter(transactions::category_id.eq_any(cat_ids.clone()));
            count_query = count_query.filter(transactions::category_id.eq_any(cat_ids));
        }
    }

    if let Some(ref date_from) = query.date_from {
        if let Ok(date) = NaiveDate::parse_from_str(date_from, "%Y-%m-%d") {
            base_query = base_query.filter(transactions::date.ge(date));
            count_query = count_query.filter(transactions::date.ge(date));
        }
    }

    if let Some(ref date_to) = query.date_to {
        if let Ok(date) = NaiveDate::parse_from_str(date_to, "%Y-%m-%d") {
            base_query = base_query.filter(transactions::date.le(date));
            count_query = count_query.filter(transactions::date.le(date));
        }
    }

    if let Some(is_income) = query.is_income {
        base_query = base_query.filter(transactions::is_income.eq(is_income));
        count_query = count_query.filter(transactions::is_income.eq(is_income));
    }

    if let Some(ref search) = query.search {
        let pattern = format!("%{}%", search.to_uppercase());
        base_query = base_query.filter(transactions::description.ilike(pattern.clone()));
        count_query = count_query.filter(transactions::description.ilike(pattern));
    }

    // Apply bank filter
    if let Some(ref stmt_ids) = bank_statement_ids {
        base_query = base_query.filter(transactions::statement_id.eq_any(stmt_ids.clone()));
        count_query = count_query.filter(transactions::statement_id.eq_any(stmt_ids.clone()));
    }

    // Get total count for pagination (with filters applied)
    let total: i64 = count_query
        .count()
        .get_result(&mut conn)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    // Get transactions with category info
    let sort_asc = query.sort_order.as_deref() == Some("asc");
    let txns: Vec<Transaction> = if sort_asc {
        base_query
            .order(transactions::date.asc())
            .offset(offset)
            .limit(per_page)
            .load(&mut conn)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?
    } else {
        base_query
            .order(transactions::date.desc())
            .offset(offset)
            .limit(per_page)
            .load(&mut conn)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?
    };

    // Get category info for each transaction
    let category_ids: Vec<Uuid> = txns.iter().filter_map(|t| t.category_id).collect();

    let cats: Vec<shared::models::Category> = if !category_ids.is_empty() {
        categories::table
            .filter(categories::id.eq_any(&category_ids))
            .load(&mut conn)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?
    } else {
        vec![]
    };

    // Get statement info (bank names) for each transaction
    let statement_ids: Vec<Uuid> = txns.iter().filter_map(|t| t.statement_id).collect();

    let stmts: Vec<shared::models::Statement> = if !statement_ids.is_empty() {
        statements::table
            .filter(statements::id.eq_any(&statement_ids))
            .load(&mut conn)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?
    } else {
        vec![]
    };

    // Build response
    let responses: Vec<TransactionResponse> = txns
        .into_iter()
        .map(|txn| {
            let cat = txn
                .category_id
                .and_then(|cid| cats.iter().find(|c| c.id == cid));
            let bank = txn
                .statement_id
                .and_then(|sid| stmts.iter().find(|s| s.id == sid))
                .map(|s| s.bank.clone());
            TransactionResponse::from_row(
                txn,
                cat.map(|c| c.name.clone()),
                cat.and_then(|c| c.key.clone()),
                cat.and_then(|c| c.color.clone()),
                cat.and_then(|c| c.icon.clone()),
                bank,
            )
        })
        .collect();

    let total_pages = (total as f64 / per_page as f64).ceil() as i64;

    Ok(HttpResponse::Ok().json(PaginatedTransactionsResponse {
        data: responses,
        meta: PaginationMeta {
            total,
            page,
            per_page,
            total_pages,
        },
    }))
}

/// Get a single transaction.
///
/// GET /api/v1/transactions/{id}
#[get("/transactions/{id}")]
pub async fn get_transaction(
    auth_user: AuthUser,
    path: web::Path<Uuid>,
    pool: web::Data<DbPool>,
) -> Result<impl Responder, AppError> {
    let txn_id = path.into_inner();
    let mut conn = pool.get().await?;

    let txn: Transaction = transactions::table
        .filter(transactions::id.eq(txn_id))
        .filter(transactions::user_id.eq(auth_user.user_id))
        .first(&mut conn)
        .await
        .map_err(|e| match e {
            diesel::result::Error::NotFound => {
                AppError::NotFound("Transaction not found".to_string())
            }
            _ => AppError::Database(e.to_string()),
        })?;

    // Get category info if exists
    let cat: Option<shared::models::Category> = if let Some(cat_id) = txn.category_id {
        categories::table
            .filter(categories::id.eq(cat_id))
            .first(&mut conn)
            .await
            .optional()
            .map_err(|e| AppError::Database(e.to_string()))?
    } else {
        None
    };

    // Get bank info from statement if exists
    let bank: Option<String> = if let Some(stmt_id) = txn.statement_id {
        statements::table
            .filter(statements::id.eq(stmt_id))
            .select(statements::bank)
            .first(&mut conn)
            .await
            .optional()
            .map_err(|e| AppError::Database(e.to_string()))?
    } else {
        None
    };

    let response = TransactionResponse::from_row(
        txn,
        cat.as_ref().map(|c| c.name.clone()),
        cat.as_ref().and_then(|c| c.key.clone()),
        cat.as_ref().and_then(|c| c.color.clone()),
        cat.as_ref().and_then(|c| c.icon.clone()),
        bank,
    );

    Ok(HttpResponse::Ok().json(response))
}

/// Update a transaction's category.
///
/// PATCH /api/v1/transactions/{id}
#[patch("/transactions/{id}")]
pub async fn update_transaction_category(
    auth_user: AuthUser,
    path: web::Path<Uuid>,
    body: web::Json<UpdateTransactionCategoryRequest>,
    pool: web::Data<DbPool>,
) -> Result<impl Responder, AppError> {
    let txn_id = path.into_inner();
    let mut conn = pool.get().await?;

    // Verify transaction exists and belongs to user
    let txn: Transaction = transactions::table
        .filter(transactions::id.eq(txn_id))
        .filter(transactions::user_id.eq(auth_user.user_id))
        .first(&mut conn)
        .await
        .map_err(|e| match e {
            diesel::result::Error::NotFound => {
                AppError::NotFound("Transaction not found".to_string())
            }
            _ => AppError::Database(e.to_string()),
        })?;

    // If category_id is provided, verify it exists and user has access
    if let Some(cat_id) = body.category_id {
        let cat_exists: bool = categories::table
            .filter(categories::id.eq(cat_id))
            .filter(
                categories::is_system
                    .eq(true)
                    .or(categories::user_id.eq(auth_user.user_id)),
            )
            .count()
            .get_result::<i64>(&mut conn)
            .await
            .map(|c| c > 0)
            .map_err(|e| AppError::Database(e.to_string()))?;

        if !cat_exists {
            return Err(AppError::NotFound("Category not found".to_string()));
        }
    }

    // Update the transaction
    diesel::update(
        transactions::table
            .filter(transactions::id.eq(txn_id))
            .filter(transactions::date.eq(txn.date)),
    )
    .set((
        transactions::category_id.eq(body.category_id),
        transactions::is_user_categorized.eq(body.category_id.is_some()),
    ))
    .execute(&mut conn)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?;

    // Get updated transaction with category info
    let updated_txn: Transaction = transactions::table
        .filter(transactions::id.eq(txn_id))
        .filter(transactions::user_id.eq(auth_user.user_id))
        .first(&mut conn)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    let cat: Option<shared::models::Category> = if let Some(cat_id) = updated_txn.category_id {
        categories::table
            .filter(categories::id.eq(cat_id))
            .first(&mut conn)
            .await
            .optional()
            .map_err(|e| AppError::Database(e.to_string()))?
    } else {
        None
    };

    // Get bank info from statement if exists
    let bank: Option<String> = if let Some(stmt_id) = updated_txn.statement_id {
        statements::table
            .filter(statements::id.eq(stmt_id))
            .select(statements::bank)
            .first(&mut conn)
            .await
            .optional()
            .map_err(|e| AppError::Database(e.to_string()))?
    } else {
        None
    };

    let response = TransactionResponse::from_row(
        updated_txn,
        cat.as_ref().map(|c| c.name.clone()),
        cat.as_ref().and_then(|c| c.key.clone()),
        cat.as_ref().and_then(|c| c.color.clone()),
        cat.as_ref().and_then(|c| c.icon.clone()),
        bank,
    );

    Ok(HttpResponse::Ok().json(response))
}

/// Response with list of banks.
#[derive(Debug, Serialize)]
pub struct BanksResponse {
    pub banks: Vec<String>,
}

/// Get list of banks for the current user.
///
/// GET /api/v1/transactions/banks
#[get("/transactions/banks")]
pub async fn get_banks(
    auth_user: AuthUser,
    pool: web::Data<DbPool>,
) -> Result<impl Responder, AppError> {
    let mut conn = pool.get().await?;

    // Get distinct bank names from user's statements
    let banks: Vec<String> = statements::table
        .filter(statements::user_id.eq(auth_user.user_id))
        .select(statements::bank)
        .distinct()
        .order(statements::bank.asc())
        .load(&mut conn)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    Ok(HttpResponse::Ok().json(BanksResponse { banks }))
}

/// Configure transaction routes.
pub fn transaction_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(get_banks) // Must be before list_transactions to avoid route conflict
        .service(list_transactions)
        .service(get_transaction)
        .service(update_transaction_category);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_list_transactions_query_defaults() {
        let json = "{}";
        let query: ListTransactionsQuery = serde_json::from_str(json).unwrap();

        assert!(query.page.is_none());
        assert!(query.per_page.is_none());
        assert!(query.category_ids.is_none());
        assert!(query.date_from.is_none());
        assert!(query.date_to.is_none());
        assert!(query.is_income.is_none());
        assert!(query.search.is_none());
        assert!(query.bank.is_none());
    }

    #[test]
    fn test_list_transactions_query_with_filters() {
        let json = r#"{
            "page": 2,
            "per_page": 50,
            "category_ids": "550e8400-e29b-41d4-a716-446655440000,660e8400-e29b-41d4-a716-446655440001",
            "date_from": "2024-01-01",
            "date_to": "2024-12-31",
            "is_income": false,
            "search": "mercado",
            "bank": "Bancolombia"
        }"#;
        let query: ListTransactionsQuery = serde_json::from_str(json).unwrap();

        assert_eq!(query.page, Some(2));
        assert_eq!(query.per_page, Some(50));
        assert_eq!(
            query.category_ids,
            Some(
                "550e8400-e29b-41d4-a716-446655440000,660e8400-e29b-41d4-a716-446655440001"
                    .to_string()
            )
        );
        assert_eq!(query.date_from, Some("2024-01-01".to_string()));
        assert_eq!(query.date_to, Some("2024-12-31".to_string()));
        assert_eq!(query.is_income, Some(false));
        assert_eq!(query.search, Some("mercado".to_string()));
        assert_eq!(query.bank, Some("Bancolombia".to_string()));
    }

    #[test]
    fn test_banks_response_serialization() {
        let response = BanksResponse {
            banks: vec!["Bancolombia".to_string(), "Nequi".to_string()],
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"banks\""));
        assert!(json.contains("Bancolombia"));
        assert!(json.contains("Nequi"));
    }

    #[test]
    fn test_update_transaction_request() {
        let json = r#"{"category_id": "550e8400-e29b-41d4-a716-446655440000"}"#;
        let request: UpdateTransactionCategoryRequest = serde_json::from_str(json).unwrap();
        assert!(request.category_id.is_some());

        let json_null = r#"{"category_id": null}"#;
        let request_null: UpdateTransactionCategoryRequest =
            serde_json::from_str(json_null).unwrap();
        assert!(request_null.category_id.is_none());
    }

    #[test]
    fn test_pagination_meta_serialization() {
        let meta = PaginationMeta {
            total: 100,
            page: 2,
            per_page: 20,
            total_pages: 5,
        };

        let json = serde_json::to_string(&meta).unwrap();
        assert!(json.contains("\"total\":100"));
        assert!(json.contains("\"page\":2"));
        assert!(json.contains("\"per_page\":20"));
        assert!(json.contains("\"total_pages\":5"));
    }
}
