//! Belvo sync job processor.
//!
//! This job syncs accounts and transactions from Belvo for a connected bank link.

use belvo::{models::AccountCategory, BelvoClient};
use chrono::{NaiveDate, Utc};
use diesel::prelude::*;
use diesel_async::{AsyncPgConnection, RunQueryDsl};
use serde::{Deserialize, Serialize};
use tracing::{error, info, warn};
use uuid::Uuid;

use shared::{
    models::{
        BelvoAccount, BelvoAccountType, BelvoLink, BelvoSyncLog, BelvoSyncStatus, NewBelvoAccount,
        NewBelvoSyncLog, NewTransaction,
    },
    schema::{belvo_accounts, belvo_links, belvo_sync_logs, transactions},
};

/// Default sync lookback period in days.
const DEFAULT_SYNC_DAYS: i64 = 90;

/// Belvo sync job payload.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BelvoSyncJobPayload {
    pub link_id: Uuid,
    pub user_id: Uuid,
    pub belvo_link_id: Uuid,
}

/// Process a Belvo sync job.
///
/// This fetches accounts and transactions from Belvo and stores them in our database.
pub async fn process_belvo_sync_job(
    payload: BelvoSyncJobPayload,
    conn: &mut AsyncPgConnection,
    belvo_client: &BelvoClient,
) -> anyhow::Result<()> {
    info!(
        "Processing Belvo sync job for link {} (user {})",
        payload.link_id, payload.user_id
    );

    // Create sync log entry
    let sync_log = create_sync_log(conn, payload.link_id).await?;
    info!("Created sync log {}", sync_log.id);

    // Update sync log to in_progress
    update_sync_status(conn, sync_log.id, BelvoSyncStatus::InProgress, None).await?;

    // Perform the sync
    let result = do_sync(conn, belvo_client, &payload, sync_log.id).await;

    // Update sync log based on result
    match &result {
        Ok((fetched, created)) => {
            info!("Sync completed: fetched={}, created={}", fetched, created);
            update_sync_complete(conn, sync_log.id, *fetched, *created).await?;

            // Update link's last_synced_at
            diesel::update(belvo_links::table.filter(belvo_links::id.eq(payload.link_id)))
                .set(belvo_links::last_synced_at.eq(Some(Utc::now())))
                .execute(conn)
                .await?;
        }
        Err(e) => {
            error!("Sync failed: {}", e);
            update_sync_status(
                conn,
                sync_log.id,
                BelvoSyncStatus::Failed,
                Some(e.to_string()),
            )
            .await?;
        }
    }

    result.map(|_| ())
}

/// Create a new sync log entry.
async fn create_sync_log(
    conn: &mut AsyncPgConnection,
    link_id: Uuid,
) -> anyhow::Result<BelvoSyncLog> {
    let today = Utc::now().date_naive();
    let date_from = today - chrono::Duration::days(DEFAULT_SYNC_DAYS);

    let new_log = NewBelvoSyncLog {
        link_id,
        status: BelvoSyncStatus::Pending,
        date_from: Some(date_from),
        date_to: Some(today),
    };

    let log = diesel::insert_into(belvo_sync_logs::table)
        .values(&new_log)
        .returning(BelvoSyncLog::as_returning())
        .get_result(conn)
        .await?;

    Ok(log)
}

/// Update sync log status.
async fn update_sync_status(
    conn: &mut AsyncPgConnection,
    log_id: Uuid,
    status: BelvoSyncStatus,
    error_message: Option<String>,
) -> anyhow::Result<()> {
    diesel::update(belvo_sync_logs::table.filter(belvo_sync_logs::id.eq(log_id)))
        .set((
            belvo_sync_logs::status.eq(status),
            belvo_sync_logs::error_message.eq(error_message),
        ))
        .execute(conn)
        .await?;
    Ok(())
}

/// Update sync log with completion stats.
async fn update_sync_complete(
    conn: &mut AsyncPgConnection,
    log_id: Uuid,
    fetched: i32,
    created: i32,
) -> anyhow::Result<()> {
    diesel::update(belvo_sync_logs::table.filter(belvo_sync_logs::id.eq(log_id)))
        .set((
            belvo_sync_logs::status.eq(BelvoSyncStatus::Completed),
            belvo_sync_logs::transactions_fetched.eq(Some(fetched)),
            belvo_sync_logs::transactions_created.eq(Some(created)),
            belvo_sync_logs::transactions_updated.eq(Some(0i32)),
            belvo_sync_logs::completed_at.eq(Some(Utc::now())),
        ))
        .execute(conn)
        .await?;
    Ok(())
}

/// Perform the actual sync operation.
async fn do_sync(
    conn: &mut AsyncPgConnection,
    belvo_client: &BelvoClient,
    payload: &BelvoSyncJobPayload,
    _sync_log_id: Uuid,
) -> anyhow::Result<(i32, i32)> {
    // Get the link to check status
    let link: BelvoLink = belvo_links::table
        .filter(belvo_links::id.eq(payload.link_id))
        .first(conn)
        .await?;

    // Sync accounts first
    info!("Fetching accounts from Belvo...");
    let belvo_accounts_list = belvo_client
        .retrieve_accounts(payload.belvo_link_id)
        .await?;
    info!("Fetched {} accounts from Belvo", belvo_accounts_list.len());

    // Store accounts
    let mut account_map: std::collections::HashMap<Uuid, Uuid> = std::collections::HashMap::new();
    for belvo_account in &belvo_accounts_list {
        let account_id = sync_account(conn, &link, belvo_account).await?;
        account_map.insert(belvo_account.id, account_id);
    }

    // Calculate date range for transactions
    let today = Utc::now().date_naive();
    let date_from = if let Some(last_sync) = link.last_synced_at {
        // Sync from last sync date (with some overlap for safety)
        last_sync.date_naive() - chrono::Duration::days(7)
    } else {
        // First sync: go back 90 days
        today - chrono::Duration::days(DEFAULT_SYNC_DAYS)
    };

    // Fetch transactions
    info!("Fetching transactions from {} to {}...", date_from, today);
    let belvo_transactions = belvo_client
        .retrieve_transactions(payload.belvo_link_id, date_from, today, None)
        .await?;
    let fetched = belvo_transactions.len() as i32;
    info!("Fetched {} transactions from Belvo", fetched);

    // Store transactions
    let mut created = 0;
    for tx in belvo_transactions {
        let account_id = account_map.get(&tx.account.id);
        match sync_transaction(conn, payload.user_id, account_id.copied(), &tx).await {
            Ok(SyncResult::Created) => created += 1,
            Ok(SyncResult::Skipped) => {}
            Err(e) => {
                warn!("Failed to sync transaction {}: {}", tx.id, e);
            }
        }
    }

    Ok((fetched, created))
}

/// Sync a single account from Belvo.
async fn sync_account(
    conn: &mut AsyncPgConnection,
    link: &BelvoLink,
    belvo_account: &belvo::models::Account,
) -> anyhow::Result<Uuid> {
    // Check if account already exists
    let existing: Option<BelvoAccount> = belvo_accounts::table
        .filter(belvo_accounts::belvo_account_id.eq(belvo_account.id))
        .first(conn)
        .await
        .ok();

    if let Some(existing) = existing {
        // Update balance
        diesel::update(belvo_accounts::table.filter(belvo_accounts::id.eq(existing.id)))
            .set((
                belvo_accounts::balance_current.eq(belvo_account.balance.current.clone()),
                belvo_accounts::balance_available.eq(belvo_account.balance.available.clone()),
                belvo_accounts::updated_at.eq(Utc::now()),
            ))
            .execute(conn)
            .await?;
        Ok(existing.id)
    } else {
        // Create new account
        let account_type = match belvo_account.category {
            AccountCategory::CheckingAccount => BelvoAccountType::Checking,
            AccountCategory::SavingsAccount => BelvoAccountType::Savings,
            AccountCategory::CreditCard => BelvoAccountType::CreditCard,
            AccountCategory::LoanAccount => BelvoAccountType::Loan,
            _ => BelvoAccountType::Other,
        };

        // Mask account number (only show last 4 digits)
        let number_masked = belvo_account.number.as_ref().map(|n| {
            if n.len() > 4 {
                format!("****{}", &n[n.len() - 4..])
            } else {
                "****".to_string()
            }
        });

        let new_account = NewBelvoAccount {
            link_id: link.id,
            belvo_account_id: belvo_account.id,
            name: belvo_account.name.clone(),
            number_masked,
            account_type,
            currency: belvo_account.currency.clone(),
            balance_current: belvo_account.balance.current.clone(),
            balance_available: belvo_account.balance.available.clone(),
        };

        let account: BelvoAccount = diesel::insert_into(belvo_accounts::table)
            .values(&new_account)
            .returning(BelvoAccount::as_returning())
            .get_result(conn)
            .await?;

        info!("Created new account {} for link {}", account.id, link.id);
        Ok(account.id)
    }
}

/// Result of syncing a single transaction.
enum SyncResult {
    Created,
    Skipped,
}

/// Sync a single transaction from Belvo.
async fn sync_transaction(
    conn: &mut AsyncPgConnection,
    user_id: Uuid,
    account_id: Option<Uuid>,
    tx: &belvo::models::Transaction,
) -> anyhow::Result<SyncResult> {
    // Get transaction date
    let date = tx
        .value_date
        .or(tx.accounting_date)
        .ok_or_else(|| anyhow::anyhow!("Transaction {} has no date", tx.id))?;

    // Check if transaction already exists by belvo_transaction_id
    // Since we can't use UNIQUE constraint, we need to check manually
    let existing: Option<(Uuid, NaiveDate)> = transactions::table
        .filter(transactions::belvo_transaction_id.eq(tx.id))
        .filter(transactions::user_id.eq(user_id))
        .select((transactions::id, transactions::date))
        .first(conn)
        .await
        .ok();

    if existing.is_some() {
        // Transaction already exists - skip (could update balance if needed)
        return Ok(SyncResult::Skipped);
    }

    // Determine if income or expense
    let is_income = tx.transaction_type == belvo::models::TransactionType::Inflow;

    // Build description
    let description = tx
        .description
        .clone()
        .unwrap_or_else(|| "Unknown".to_string());

    // Convert amount - Belvo returns positive values, we store negative for expenses
    let amount = if is_income {
        tx.amount.clone()
    } else {
        -tx.amount.clone()
    };

    let new_transaction = NewTransaction {
        user_id,
        statement_id: None, // Not from a statement upload
        category_id: None,  // Will be categorized by categorizer job
        date,
        description,
        amount,
        balance: tx.balance.clone(),
        reference: tx.reference.clone(),
        is_income,
        belvo_transaction_id: Some(tx.id),
        belvo_account_id: account_id,
        sequence: 0, // Belvo transactions have no sequence
    };

    diesel::insert_into(transactions::table)
        .values(&new_transaction)
        .execute(conn)
        .await?;

    Ok(SyncResult::Created)
}
