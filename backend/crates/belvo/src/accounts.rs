//! Account retrieval from Belvo API.

use uuid::Uuid;

use crate::client::BelvoClient;
use crate::error::Result;
use crate::models::{Account, PaginatedResponse, RetrieveAccountsRequest};

impl BelvoClient {
    /// Retrieve accounts for a link.
    ///
    /// This fetches account data from the bank through Belvo.
    /// The data is retrieved in real-time from the bank.
    pub async fn retrieve_accounts(&self, link_id: Uuid) -> Result<Vec<Account>> {
        let request = RetrieveAccountsRequest {
            link: link_id,
            save_data: Some(true),
        };

        self.post("/api/accounts/", &request).await
    }

    /// List all stored accounts for a link.
    ///
    /// This returns accounts that have been previously retrieved and stored.
    pub async fn list_accounts_for_link(&self, link_id: Uuid) -> Result<Vec<Account>> {
        let path = format!("/api/accounts/?link={}", link_id);
        let response: PaginatedResponse<Account> = self.get(&path).await?;
        Ok(response.results)
    }

    /// List all accounts with pagination.
    pub async fn list_accounts(
        &self,
        page: Option<i32>,
        page_size: Option<i32>,
        link_id: Option<Uuid>,
    ) -> Result<PaginatedResponse<Account>> {
        let mut path = "/api/accounts/".to_string();
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

        if !params.is_empty() {
            path.push('?');
            path.push_str(&params.join("&"));
        }

        self.get(&path).await
    }

    /// Get a specific account by ID.
    pub async fn get_account(&self, account_id: Uuid) -> Result<Account> {
        self.get(&format!("/api/accounts/{}/", account_id)).await
    }

    /// Delete stored account data.
    #[allow(dead_code)]
    pub async fn delete_account(&self, account_id: Uuid) -> Result<()> {
        self.delete(&format!("/api/accounts/{}/", account_id)).await
    }
}
