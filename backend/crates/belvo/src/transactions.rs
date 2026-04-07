//! Transaction retrieval from Belvo API.

use chrono::NaiveDate;
use uuid::Uuid;

use crate::client::BelvoClient;
use crate::error::Result;
use crate::models::{PaginatedResponse, RetrieveTransactionsRequest, Transaction};

impl BelvoClient {
    /// Retrieve transactions for a link within a date range.
    ///
    /// This fetches transaction data from the bank through Belvo.
    /// The data is retrieved in real-time from the bank.
    ///
    /// # Arguments
    /// * `link_id` - The link ID to retrieve transactions for
    /// * `date_from` - Start date (inclusive)
    /// * `date_to` - End date (inclusive)
    /// * `account_id` - Optional account ID to filter transactions
    pub async fn retrieve_transactions(
        &self,
        link_id: Uuid,
        date_from: NaiveDate,
        date_to: NaiveDate,
        account_id: Option<Uuid>,
    ) -> Result<Vec<Transaction>> {
        let request = RetrieveTransactionsRequest {
            link: link_id,
            date_from,
            date_to,
            account: account_id,
            save_data: Some(true),
        };

        self.post("/api/transactions/", &request).await
    }

    /// List stored transactions with filters and pagination.
    ///
    /// This returns transactions that have been previously retrieved and stored.
    pub async fn list_transactions(
        &self,
        page: Option<i32>,
        page_size: Option<i32>,
        link_id: Option<Uuid>,
        account_id: Option<Uuid>,
        date_from: Option<NaiveDate>,
        date_to: Option<NaiveDate>,
    ) -> Result<PaginatedResponse<Transaction>> {
        let mut path = "/api/transactions/".to_string();
        let mut params = Vec::new();

        if let Some(p) = page {
            params.push(format!("page={}", p));
        }
        if let Some(ps) = page_size {
            params.push(format!("page_size={}", ps.min(100)));
        }
        if let Some(id) = link_id {
            params.push(format!("link={}", id));
        }
        if let Some(id) = account_id {
            params.push(format!("account={}", id));
        }
        if let Some(d) = date_from {
            params.push(format!("value_date__gte={}", d));
        }
        if let Some(d) = date_to {
            params.push(format!("value_date__lte={}", d));
        }

        if !params.is_empty() {
            path.push('?');
            path.push_str(&params.join("&"));
        }

        self.get(&path).await
    }

    /// List all transactions for a link within a date range.
    ///
    /// This handles pagination automatically and returns all transactions.
    pub async fn list_all_transactions_for_link(
        &self,
        link_id: Uuid,
        date_from: NaiveDate,
        date_to: NaiveDate,
    ) -> Result<Vec<Transaction>> {
        let mut all_transactions = Vec::new();
        let mut page = 1;
        let page_size = 100;

        loop {
            let response = self
                .list_transactions(
                    Some(page),
                    Some(page_size),
                    Some(link_id),
                    None,
                    Some(date_from),
                    Some(date_to),
                )
                .await?;

            all_transactions.extend(response.results);

            if response.next.is_none() {
                break;
            }
            page += 1;
        }

        Ok(all_transactions)
    }

    /// Get a specific transaction by ID.
    pub async fn get_transaction(&self, transaction_id: Uuid) -> Result<Transaction> {
        self.get(&format!("/api/transactions/{}/", transaction_id))
            .await
    }

    /// Delete stored transaction data.
    #[allow(dead_code)]
    pub async fn delete_transaction(&self, transaction_id: Uuid) -> Result<()> {
        self.delete(&format!("/api/transactions/{}/", transaction_id))
            .await
    }
}
