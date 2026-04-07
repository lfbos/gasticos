//! Transaction categorization job.
//!
//! This job processes uncategorized transactions and assigns categories
//! based on the Colombian merchant dictionary and rule-based patterns.

use anyhow::Result;
use categorizer::Categorizer;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use serde::{Deserialize, Serialize};
use shared::models::Category;
use shared::schema::categories;
use std::collections::HashMap;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

/// Job payload for categorization.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategorizeJobPayload {
    /// User ID to categorize transactions for.
    pub user_id: Uuid,
    /// Optional: specific transaction IDs to categorize.
    /// If empty, will categorize all uncategorized transactions.
    #[serde(default)]
    pub transaction_ids: Vec<Uuid>,
    /// Maximum number of transactions to process in one batch.
    #[serde(default = "default_batch_size")]
    pub batch_size: i64,
}

fn default_batch_size() -> i64 {
    500
}

/// Result of categorization job.
#[derive(Debug, Clone, Serialize)]
pub struct CategorizeJobResult {
    pub total_processed: usize,
    pub categorized: usize,
    pub uncategorized: usize,
    pub by_dictionary: usize,
    pub by_rules: usize,
}

/// Transaction struct for categorization (minimal fields needed).
#[derive(Debug, Clone, Queryable)]
struct TransactionForCategorization {
    id: Uuid,
    date: chrono::NaiveDate,
    description: String,
}

/// Process the categorization job.
pub async fn process_categorize_job(
    payload: CategorizeJobPayload,
    conn: &mut diesel_async::AsyncPgConnection,
) -> Result<CategorizeJobResult> {
    info!(
        "Starting categorization job for user {} with batch size {}",
        payload.user_id, payload.batch_size
    );

    // Build the category lookup map (key -> category_id)
    let category_map = build_category_map(conn, payload.user_id).await?;
    if category_map.is_empty() {
        warn!(
            "No system categories found for user {}. Skipping categorization.",
            payload.user_id
        );
        return Ok(CategorizeJobResult {
            total_processed: 0,
            categorized: 0,
            uncategorized: 0,
            by_dictionary: 0,
            by_rules: 0,
        });
    }

    // Fetch uncategorized transactions
    let transactions_to_process = fetch_uncategorized_transactions(conn, &payload).await?;

    let total_processed = transactions_to_process.len();
    info!(
        "Found {} uncategorized transactions to process",
        total_processed
    );

    if transactions_to_process.is_empty() {
        return Ok(CategorizeJobResult {
            total_processed: 0,
            categorized: 0,
            uncategorized: 0,
            by_dictionary: 0,
            by_rules: 0,
        });
    }

    // Initialize categorizer
    let categorizer = Categorizer::new();

    // Process and categorize transactions
    let mut categorized = 0;
    let mut uncategorized = 0;
    let mut by_dictionary = 0;
    let mut by_rules = 0;

    for txn in &transactions_to_process {
        if let Some(result) = categorizer.categorize(&txn.description) {
            // Look up the category_id from the map
            if let Some(&category_id) = category_map.get(&result.category_name) {
                // Update the transaction with the category
                match update_transaction_category(conn, txn.id, txn.date, category_id).await {
                    Ok(_) => {
                        categorized += 1;

                        // Track match type
                        if result.matched_by.as_deref() == Some("dictionary") {
                            by_dictionary += 1;
                        } else if result
                            .matched_by
                            .as_ref()
                            .map(|m| m.starts_with("rule:"))
                            .unwrap_or(false)
                        {
                            by_rules += 1;
                        }

                        debug!(
                            "Categorized transaction {} as {} (matched by {:?})",
                            txn.id, result.category_name, result.matched_by
                        );
                    }
                    Err(e) => {
                        error!("Failed to update transaction {}: {}", txn.id, e);
                        uncategorized += 1;
                    }
                }
            } else {
                warn!(
                    "Category '{}' not found in category map for user {}",
                    result.category_name, payload.user_id
                );
                uncategorized += 1;
            }
        } else {
            uncategorized += 1;
            debug!(
                "No category match for transaction {}: {}",
                txn.id, txn.description
            );
        }
    }

    info!(
        "Categorization complete: {}/{} categorized ({} dictionary, {} rules)",
        categorized, total_processed, by_dictionary, by_rules
    );

    Ok(CategorizeJobResult {
        total_processed,
        categorized,
        uncategorized,
        by_dictionary,
        by_rules,
    })
}

/// Build a map of category key -> category_id for the user.
/// Includes system categories and user-specific categories.
async fn build_category_map(
    conn: &mut diesel_async::AsyncPgConnection,
    user_id: Uuid,
) -> Result<HashMap<String, Uuid>> {
    let cats: Vec<Category> = categories::table
        .filter(
            categories::is_system
                .eq(true)
                .or(categories::user_id.eq(user_id)),
        )
        .select(Category::as_select())
        .load(conn)
        .await?;

    let mut map = HashMap::new();
    for cat in cats {
        // Use the key if available, otherwise use lowercase name
        let key = cat
            .key
            .clone()
            .unwrap_or_else(|| cat.name.to_lowercase().replace(' ', "_"));
        map.insert(key, cat.id);
    }

    debug!("Built category map with {} entries", map.len());
    Ok(map)
}

/// Fetch uncategorized transactions for the user.
async fn fetch_uncategorized_transactions(
    conn: &mut diesel_async::AsyncPgConnection,
    payload: &CategorizeJobPayload,
) -> Result<Vec<TransactionForCategorization>> {
    use shared::schema::transactions::dsl::*;

    let mut query = transactions
        .filter(user_id.eq(payload.user_id))
        .filter(category_id.is_null())
        .filter(is_user_categorized.eq(false))
        .into_boxed();

    // If specific transaction IDs are provided, filter by them
    if !payload.transaction_ids.is_empty() {
        query = query.filter(id.eq_any(&payload.transaction_ids));
    }

    let results: Vec<TransactionForCategorization> = query
        .select((id, date, description))
        .order(date.desc())
        .limit(payload.batch_size)
        .load(conn)
        .await?;

    Ok(results)
}

/// Update a transaction with the assigned category.
async fn update_transaction_category(
    conn: &mut diesel_async::AsyncPgConnection,
    txn_id: Uuid,
    txn_date: chrono::NaiveDate,
    cat_id: Uuid,
) -> Result<()> {
    use shared::schema::transactions::dsl::*;

    diesel::update(transactions.filter(id.eq(txn_id).and(date.eq(txn_date))))
        .set(category_id.eq(Some(cat_id)))
        .execute(conn)
        .await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_categorize_job_payload_default() {
        let json = r#"{"user_id": "550e8400-e29b-41d4-a716-446655440000"}"#;
        let payload: CategorizeJobPayload = serde_json::from_str(json).unwrap();

        assert_eq!(payload.batch_size, 500);
        assert!(payload.transaction_ids.is_empty());
    }

    #[test]
    fn test_categorize_job_payload_with_ids() {
        let json = r#"{
            "user_id": "550e8400-e29b-41d4-a716-446655440000",
            "transaction_ids": ["550e8400-e29b-41d4-a716-446655440001"],
            "batch_size": 100
        }"#;
        let payload: CategorizeJobPayload = serde_json::from_str(json).unwrap();

        assert_eq!(payload.batch_size, 100);
        assert_eq!(payload.transaction_ids.len(), 1);
    }

    #[test]
    fn test_categorize_job_result_serialization() {
        let result = CategorizeJobResult {
            total_processed: 100,
            categorized: 80,
            uncategorized: 20,
            by_dictionary: 50,
            by_rules: 30,
        };

        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("\"total_processed\":100"));
        assert!(json.contains("\"categorized\":80"));
    }
}
