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

#[cfg(test)]
mod tests {
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

    fn sample_account_json(id: Uuid, link_id: Uuid) -> serde_json::Value {
        serde_json::json!({
            "id": id.to_string(),
            "link": link_id.to_string(),
            "category": "CHECKING_ACCOUNT",
            "currency": "COP",
            "balance": {
                "current": 1500000.50,
                "available": 1400000.00
            }
        })
    }

    #[tokio::test]
    async fn test_retrieve_accounts() {
        let mock_server = MockServer::start().await;
        let link_id = Uuid::new_v4();
        let account_id = Uuid::new_v4();

        Mock::given(method("POST"))
            .and(path("/api/accounts/"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_json(vec![sample_account_json(account_id, link_id)]),
            )
            .mount(&mock_server)
            .await;

        let client = create_test_client(&mock_server.uri());
        let result = client.retrieve_accounts(link_id).await;

        assert!(result.is_ok());
        let accounts = result.unwrap();
        assert_eq!(accounts.len(), 1);
        assert_eq!(accounts[0].currency, "COP");
    }

    #[tokio::test]
    async fn test_list_accounts_for_link() {
        let mock_server = MockServer::start().await;
        let link_id = Uuid::new_v4();
        let account_id = Uuid::new_v4();

        Mock::given(method("GET"))
            .and(query_param("link", link_id.to_string().as_str()))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "count": 1,
                "next": null,
                "previous": null,
                "results": [sample_account_json(account_id, link_id)]
            })))
            .mount(&mock_server)
            .await;

        let client = create_test_client(&mock_server.uri());
        let result = client.list_accounts_for_link(link_id).await;

        assert!(result.is_ok());
        let accounts = result.unwrap();
        assert_eq!(accounts.len(), 1);
    }

    #[tokio::test]
    async fn test_list_accounts_paginated() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/api/accounts/"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "count": 0,
                "next": null,
                "previous": null,
                "results": []
            })))
            .mount(&mock_server)
            .await;

        let client = create_test_client(&mock_server.uri());
        let result = client.list_accounts(None, None, None).await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap().count, 0);
    }

    #[tokio::test]
    async fn test_get_account() {
        let mock_server = MockServer::start().await;
        let account_id = Uuid::new_v4();
        let link_id = Uuid::new_v4();

        Mock::given(method("GET"))
            .and(path_regex(r"/api/accounts/[a-f0-9-]+/"))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(sample_account_json(account_id, link_id)),
            )
            .mount(&mock_server)
            .await;

        let client = create_test_client(&mock_server.uri());
        let result = client.get_account(account_id).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_delete_account() {
        let mock_server = MockServer::start().await;
        let account_id = Uuid::new_v4();

        Mock::given(method("DELETE"))
            .and(path_regex(r"/api/accounts/[a-f0-9-]+/"))
            .respond_with(ResponseTemplate::new(204))
            .mount(&mock_server)
            .await;

        let client = create_test_client(&mock_server.uri());
        let result = client.delete_account(account_id).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_list_accounts_with_page() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/api/accounts/"))
            .and(query_param("page", "2"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "count": 10,
                "next": null,
                "previous": null,
                "results": []
            })))
            .mount(&mock_server)
            .await;

        let client = create_test_client(&mock_server.uri());
        let result = client.list_accounts(Some(2), None, None).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_list_accounts_with_page_size() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/api/accounts/"))
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
        let result = client.list_accounts(None, Some(50), None).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_list_accounts_with_link_id() {
        let mock_server = MockServer::start().await;
        let link_id = Uuid::new_v4();

        Mock::given(method("GET"))
            .and(path("/api/accounts/"))
            .and(query_param("link", link_id.to_string().as_str()))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "count": 0,
                "next": null,
                "previous": null,
                "results": []
            })))
            .mount(&mock_server)
            .await;

        let client = create_test_client(&mock_server.uri());
        let result = client.list_accounts(None, None, Some(link_id)).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_list_accounts_with_all_params() {
        let mock_server = MockServer::start().await;
        let link_id = Uuid::new_v4();

        Mock::given(method("GET"))
            .and(path("/api/accounts/"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "count": 5,
                "next": null,
                "previous": null,
                "results": []
            })))
            .mount(&mock_server)
            .await;

        let client = create_test_client(&mock_server.uri());
        let result = client.list_accounts(Some(1), Some(10), Some(link_id)).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_list_accounts_page_size_capped_at_100() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/api/accounts/"))
            .and(query_param("page_size", "100"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "count": 0,
                "next": null,
                "previous": null,
                "results": []
            })))
            .mount(&mock_server)
            .await;

        let client = create_test_client(&mock_server.uri());
        // Should cap at 100 even if we pass 200
        let result = client.list_accounts(None, Some(200), None).await;

        assert!(result.is_ok());
    }
}
