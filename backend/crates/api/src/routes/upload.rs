//! Upload routes for parsing bank statement PDFs.
//!
//! Endpoints:
//! - POST /upload/pdf - Upload and parse a bank statement PDF

use actix_multipart::Multipart;
use actix_web::{post, web, HttpResponse, Responder};
use bigdecimal::BigDecimal;
use categorizer::Categorizer;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use shared::{
    error::AppError,
    models::{Category, NewStatement, NewTransaction, StatementStatus},
    schema::{categories, statements, transactions},
    DbPool,
};
use std::collections::HashMap;
use std::str::FromStr;
use tracing::info;
use uuid::Uuid;

use crate::extractors::AuthUser;

// ============================================================================
// Response DTOs
// ============================================================================

/// Response after uploading and parsing a PDF.
#[derive(Debug, Serialize)]
pub struct UploadResponse {
    /// Bank name detected from the statement
    pub bank: String,
    /// Number of transactions parsed
    pub transactions_parsed: usize,
    /// Number of transactions inserted (new)
    pub transactions_inserted: usize,
    /// Number of duplicate transactions skipped
    pub transactions_skipped: usize,
}

// ============================================================================
// Request DTOs
// ============================================================================

/// Query parameters for PDF upload (password).
#[derive(Debug, Deserialize)]
pub struct UploadQuery {
    /// Password for the encrypted PDF
    pub password: String,
}

// ============================================================================
// Handlers
// ============================================================================

/// Maximum file size (10 MB)
const MAX_FILE_SIZE: usize = 10 * 1024 * 1024;

/// Upload and parse a bank statement PDF.
///
/// POST /api/v1/upload/pdf?password=xxx
///
/// The request body should be multipart/form-data with a single file field named "file".
#[post("/upload/pdf")]
pub async fn upload_pdf(
    auth_user: AuthUser,
    query: web::Query<UploadQuery>,
    mut payload: Multipart,
    pool: web::Data<DbPool>,
) -> Result<impl Responder, AppError> {
    let password = &query.password;

    // Read the file from multipart
    let mut file_bytes: Vec<u8> = Vec::new();

    while let Some(item) = payload.next().await {
        let mut field =
            item.map_err(|e| AppError::BadRequest(format!("Error al leer archivo: {}", e)))?;

        // Only process the first file
        if file_bytes.is_empty() {
            while let Some(chunk) = field.next().await {
                let data = chunk
                    .map_err(|e| AppError::BadRequest(format!("Error al leer archivo: {}", e)))?;

                if file_bytes.len() + data.len() > MAX_FILE_SIZE {
                    return Err(AppError::BadRequest(
                        "Archivo muy grande (máximo 10 MB)".to_string(),
                    ));
                }

                file_bytes.extend_from_slice(&data);
            }
        }
    }

    if file_bytes.is_empty() {
        return Err(AppError::BadRequest(
            "No se proporcionó ningún archivo".to_string(),
        ));
    }

    info!(
        "Received PDF upload: {} bytes for user {}, password length: {}",
        file_bytes.len(),
        auth_user.user_id,
        password.len()
    );
    tracing::debug!("Password received: '{}'", password);

    // Parse the PDF
    let (bank, parsed_transactions) =
        parser::parse_statement(&file_bytes, password).map_err(|e| {
            tracing::error!("PDF parsing error: {:?}", e);
            match e {
                parser::ParserError::InvalidPassword => {
                    AppError::BadRequest("Contraseña incorrecta".to_string())
                }
                parser::ParserError::UnsupportedFormat => {
                    AppError::BadRequest("Formato de banco no soportado".to_string())
                }
                parser::ParserError::NoTransactionsFound => AppError::BadRequest(
                    "No se encontraron transacciones en el extracto".to_string(),
                ),
                _ => AppError::BadRequest(format!("Error al procesar el PDF: {}", e)),
            }
        })?;

    info!(
        "Parsed {} transactions from {} statement",
        parsed_transactions.len(),
        bank
    );

    // Insert transactions into database
    let mut conn = pool.get().await?;

    // Create statement record to track the bank source
    let new_statement = NewStatement {
        user_id: auth_user.user_id,
        bank: bank.clone(),
        filename: "upload.pdf".to_string(), // We don't have the original filename
        file_path: None,
        file_size: file_bytes.len() as i64,
    };

    let statement_id: Uuid = diesel::insert_into(statements::table)
        .values(&new_statement)
        .returning(statements::id)
        .get_result(&mut conn)
        .await
        .map_err(|e| AppError::Database(format!("Failed to create statement: {}", e)))?;

    let mut inserted = 0;
    let mut skipped = 0;

    for ptx in &parsed_transactions {
        // Convert Decimal to BigDecimal
        let amount = BigDecimal::from_str(&ptx.amount.to_string())
            .map_err(|e| AppError::Internal(format!("Failed to convert amount: {}", e)))?;

        let balance = ptx
            .balance
            .as_ref()
            .map(|b| BigDecimal::from_str(&b.to_string()).unwrap_or_else(|_| BigDecimal::from(0)));

        let new_txn = NewTransaction {
            user_id: auth_user.user_id,
            statement_id: Some(statement_id),
            category_id: None,
            date: ptx.date,
            description: ptx.description.clone(),
            amount,
            balance,
            reference: ptx.reference.clone(),
            is_income: ptx.is_income,
            belvo_transaction_id: None,
            belvo_account_id: None,
            sequence: ptx.sequence,
        };

        // Try to insert, handle duplicates gracefully
        // A transaction is considered duplicate if same user, date, description, and amount
        let result = diesel::insert_into(transactions::table)
            .values(&new_txn)
            .on_conflict_do_nothing()
            .execute(&mut conn)
            .await;

        match result {
            Ok(rows) => {
                if rows > 0 {
                    inserted += 1;
                } else {
                    skipped += 1;
                }
            }
            Err(e) => {
                // Log but continue - don't fail the whole upload for one transaction
                tracing::warn!("Failed to insert transaction: {}", e);
                skipped += 1;
            }
        }
    }

    info!(
        "Inserted {} transactions, skipped {} duplicates for user {}",
        inserted, skipped, auth_user.user_id
    );

    // Update statement with transaction count and status
    diesel::update(statements::table.filter(statements::id.eq(statement_id)))
        .set((
            statements::status.eq(StatementStatus::Completed),
            statements::transaction_count.eq(inserted as i32),
            statements::processed_at.eq(diesel::dsl::now),
        ))
        .execute(&mut conn)
        .await
        .map_err(|e| AppError::Database(format!("Failed to update statement: {}", e)))?;

    // Categorize all uncategorized transactions for this user
    let categorized = categorize_uncategorized(&mut conn, auth_user.user_id).await;
    info!("Categorized {} transactions", categorized);

    Ok(HttpResponse::Ok().json(UploadResponse {
        bank,
        transactions_parsed: parsed_transactions.len(),
        transactions_inserted: inserted,
        transactions_skipped: skipped,
    }))
}

/// Categorize all uncategorized transactions for a user.
async fn categorize_uncategorized(
    conn: &mut diesel_async::AsyncPgConnection,
    user_id: Uuid,
) -> usize {
    use shared::models::Transaction;

    // Load system categories from database
    let system_cats: Vec<Category> = match categories::table
        .filter(categories::is_system.eq(true))
        .load(conn)
        .await
    {
        Ok(cats) => cats,
        Err(e) => {
            tracing::warn!("Failed to load system categories: {}", e);
            return 0;
        }
    };

    // Build a map from category key to category id
    let category_map: HashMap<String, Uuid> = system_cats
        .into_iter()
        .filter_map(|c| c.key.map(|key| (key, c.id)))
        .collect();

    // Load uncategorized transactions for this user
    let uncategorized: Vec<Transaction> = match transactions::table
        .filter(transactions::user_id.eq(user_id))
        .filter(transactions::category_id.is_null())
        .filter(transactions::is_user_categorized.eq(false))
        .load(conn)
        .await
    {
        Ok(txns) => txns,
        Err(e) => {
            tracing::warn!("Failed to load uncategorized transactions: {}", e);
            return 0;
        }
    };

    let categorizer = Categorizer::new();
    let mut categorized_count = 0;

    for txn in &uncategorized {
        // Try to categorize by description first
        let category_key = if let Some(result) = categorizer.categorize(&txn.description) {
            result.category_name
        } else if txn.is_income {
            // Fallback: if amount is positive (income), categorize as Income
            "income".to_string()
        } else {
            // Fallback: categorize as Other
            "other".to_string()
        };

        if let Some(category_id) = category_map.get(&category_key) {
            // Update the transaction with the category
            let update_result = diesel::update(
                transactions::table
                    .filter(transactions::id.eq(txn.id))
                    .filter(transactions::date.eq(txn.date)),
            )
            .set(transactions::category_id.eq(*category_id))
            .execute(conn)
            .await;

            if update_result.is_ok() {
                categorized_count += 1;
            }
        }
    }

    categorized_count
}

/// Configure upload routes.
pub fn upload_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(upload_pdf);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_upload_response_serialization() {
        let response = UploadResponse {
            bank: "Bancolombia".to_string(),
            transactions_parsed: 50,
            transactions_inserted: 45,
            transactions_skipped: 5,
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"bank\":\"Bancolombia\""));
        assert!(json.contains("\"transactions_parsed\":50"));
        assert!(json.contains("\"transactions_inserted\":45"));
        assert!(json.contains("\"transactions_skipped\":5"));
    }

    #[test]
    fn test_upload_query_deserialization() {
        let json = r#"{"password": "123456"}"#;
        let query: UploadQuery = serde_json::from_str(json).unwrap();
        assert_eq!(query.password, "123456");
    }
}
