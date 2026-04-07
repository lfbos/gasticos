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

#[cfg(test)]
mod tests {
    use chrono::NaiveDate;
    use uuid::Uuid;
    use wiremock::matchers::{method, path, path_regex, query_param};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    use crate::client::{BelvoConfig, BelvoEnvironment};
    use crate::BelvoClient;

    fn create_test_client(base_url: &str) -> BelvoClient {
        let config = BelvoConfig {
            secret_id: "test_id".to_string(),
            secret_password: "test_password".to_string(),
            environment: BelvoEnvironment::Sandbox,
        };
        let mut client = BelvoClient::new(config).unwrap();
        client.set_base_url(base_url.to_string());
        client
    }

    fn sample_transaction_json(id: Uuid, account_id: Uuid) -> serde_json::Value {
        serde_json::json!({
            "id": id.to_string(),
            "account": {
                "id": account_id.to_string(),
                "link": Uuid::new_v4().to_string(),
                "category": "CHECKING_ACCOUNT",
                "currency": "COP",
                "balance": {
                    "current": 1000000.0,
                    "available": 900000.0
                }
            },
            "value_date": "2024-01-15",
            "accounting_date": "2024-01-15",
            "amount": -50000.0,
            "balance": 950000.0,
            "currency": "COP",
            "description": "COMPRA SUPERMERCADO",
            "reference": "REF123",
            "type": "OUTFLOW",
            "status": "PROCESSED",
            "category": "Online Platforms / Leisure"
        })
    }

    #[tokio::test]
    async fn test_retrieve_transactions() {
        let mock_server = MockServer::start().await;
        let link_id = Uuid::new_v4();
        let transaction_id = Uuid::new_v4();
        let account_id = Uuid::new_v4();

        Mock::given(method("POST"))
            .and(path("/api/transactions/"))
            .respond_with(ResponseTemplate::new(200).set_body_json(vec![
                sample_transaction_json(transaction_id, account_id)
            ]))
            .mount(&mock_server)
            .await;

        let client = create_test_client(&mock_server.uri());
        let result = client
            .retrieve_transactions(
                link_id,
                NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
                NaiveDate::from_ymd_opt(2024, 1, 31).unwrap(),
                None,
            )
            .await;

        assert!(result.is_ok());
        let transactions = result.unwrap();
        assert_eq!(transactions.len(), 1);
        assert_eq!(transactions[0].currency, "COP");
    }

    #[tokio::test]
    async fn test_retrieve_transactions_with_account() {
        let mock_server = MockServer::start().await;
        let link_id = Uuid::new_v4();
        let account_id = Uuid::new_v4();
        let transaction_id = Uuid::new_v4();

        Mock::given(method("POST"))
            .and(path("/api/transactions/"))
            .respond_with(ResponseTemplate::new(200).set_body_json(vec![
                sample_transaction_json(transaction_id, account_id)
            ]))
            .mount(&mock_server)
            .await;

        let client = create_test_client(&mock_server.uri());
        let result = client
            .retrieve_transactions(
                link_id,
                NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
                NaiveDate::from_ymd_opt(2024, 1, 31).unwrap(),
                Some(account_id),
            )
            .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_list_transactions() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/api/transactions/"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "count": 0,
                "next": null,
                "previous": null,
                "results": []
            })))
            .mount(&mock_server)
            .await;

        let client = create_test_client(&mock_server.uri());
        let result = client.list_transactions(None, None, None, None, None, None).await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap().count, 0);
    }

    #[tokio::test]
    async fn test_list_transactions_with_filters() {
        let mock_server = MockServer::start().await;
        let link_id = Uuid::new_v4();

        Mock::given(method("GET"))
            .and(query_param("link", link_id.to_string().as_str()))
            .and(query_param("page", "1"))
            .and(query_param("page_size", "50"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "count": 0,
                "next": null,
                "previous": null,
                "results": []
            })))
            .mount(&mock_server)
            .await;

        let client = create_test_client(&mock_server.uri());
        let result = client
            .list_transactions(Some(1), Some(50), Some(link_id), None, None, None)
            .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_get_transaction() {
        let mock_server = MockServer::start().await;
        let transaction_id = Uuid::new_v4();
        let account_id = Uuid::new_v4();

        Mock::given(method("GET"))
            .and(path_regex(r"/api/transactions/[a-f0-9-]+/"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_json(sample_transaction_json(transaction_id, account_id)),
            )
            .mount(&mock_server)
            .await;

        let client = create_test_client(&mock_server.uri());
        let result = client.get_transaction(transaction_id).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_delete_transaction() {
        let mock_server = MockServer::start().await;
        let transaction_id = Uuid::new_v4();

        Mock::given(method("DELETE"))
            .and(path_regex(r"/api/transactions/[a-f0-9-]+/"))
            .respond_with(ResponseTemplate::new(204))
            .mount(&mock_server)
            .await;

        let client = create_test_client(&mock_server.uri());
        let result = client.delete_transaction(transaction_id).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_list_all_transactions_for_link() {
        let mock_server = MockServer::start().await;
        let link_id = Uuid::new_v4();
        let transaction_id = Uuid::new_v4();
        let account_id = Uuid::new_v4();

        // First page with next
        Mock::given(method("GET"))
            .and(query_param("page", "1"))
            .and(query_param("link", link_id.to_string().as_str()))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "count": 2,
                "next": "https://api.belvo.com/api/transactions/?page=2",
                "previous": null,
                "results": [sample_transaction_json(transaction_id, account_id)]
            })))
            .mount(&mock_server)
            .await;

        // Second page without next
        Mock::given(method("GET"))
            .and(query_param("page", "2"))
            .and(query_param("link", link_id.to_string().as_str()))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "count": 2,
                "next": null,
                "previous": "https://api.belvo.com/api/transactions/?page=1",
                "results": [sample_transaction_json(Uuid::new_v4(), account_id)]
            })))
            .mount(&mock_server)
            .await;

        let client = create_test_client(&mock_server.uri());
        let result = client
            .list_all_transactions_for_link(
                link_id,
                NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
                NaiveDate::from_ymd_opt(2024, 1, 31).unwrap(),
            )
            .await;

        assert!(result.is_ok());
        let transactions = result.unwrap();
        assert_eq!(transactions.len(), 2);
    }
}
