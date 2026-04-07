//! Link management for Belvo API.

use uuid::Uuid;

use crate::client::BelvoClient;
use crate::error::Result;
use crate::models::{Link, PaginatedResponse};

impl BelvoClient {
    /// Get a specific link by ID.
    pub async fn get_link(&self, link_id: Uuid) -> Result<Link> {
        self.get(&format!("/api/links/{}/", link_id)).await
    }

    /// List all links with pagination.
    ///
    /// # Arguments
    /// * `page` - Page number (1-indexed)
    /// * `page_size` - Number of results per page (max 100)
    pub async fn list_links(
        &self,
        page: Option<i32>,
        page_size: Option<i32>,
    ) -> Result<PaginatedResponse<Link>> {
        let mut path = "/api/links/".to_string();
        let mut params = Vec::new();

        if let Some(p) = page {
            params.push(format!("page={}", p));
        }
        if let Some(ps) = page_size {
            params.push(format!("page_size={}", ps.min(100)));
        }

        if !params.is_empty() {
            path.push('?');
            path.push_str(&params.join("&"));
        }

        self.get(&path).await
    }

    /// Delete a link by ID.
    ///
    /// This permanently removes the link and all associated data from Belvo.
    pub async fn delete_link(&self, link_id: Uuid) -> Result<()> {
        self.delete(&format!("/api/links/{}/", link_id)).await
    }

    /// Update a link's credentials.
    ///
    /// Use this to update the stored credentials for a link.
    #[allow(dead_code)]
    pub async fn update_link(
        &self,
        link_id: Uuid,
        password: &str,
        password2: Option<&str>,
        token: Option<&str>,
    ) -> Result<Link> {
        #[derive(serde::Serialize)]
        struct UpdateLinkRequest<'a> {
            password: &'a str,
            #[serde(skip_serializing_if = "Option::is_none")]
            password2: Option<&'a str>,
            #[serde(skip_serializing_if = "Option::is_none")]
            token: Option<&'a str>,
        }

        let request = UpdateLinkRequest {
            password,
            password2,
            token,
        };

        self.patch(&format!("/api/links/{}/", link_id), &request)
            .await
    }
}

#[cfg(test)]
mod tests {
    use uuid::Uuid;
    use wiremock::matchers::{method, path, path_regex};
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

    #[tokio::test]
    async fn test_get_link() {
        let mock_server = MockServer::start().await;
        let link_id = Uuid::new_v4();

        Mock::given(method("GET"))
            .and(path_regex(r"/api/links/[a-f0-9-]+/"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "id": link_id.to_string(),
                "institution": "bancolombia_retail_co",
                "access_mode": "recurrent",
                "status": "valid"
            })))
            .mount(&mock_server)
            .await;

        let client = create_test_client(&mock_server.uri());
        let result = client.get_link(link_id).await;

        assert!(result.is_ok());
        let link = result.unwrap();
        assert_eq!(link.institution, "bancolombia_retail_co");
        assert_eq!(link.status, crate::models::LinkStatus::Valid);
    }

    #[tokio::test]
    async fn test_list_links() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/api/links/"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "count": 2,
                "next": null,
                "previous": null,
                "results": [
                    {
                        "id": Uuid::new_v4().to_string(),
                        "institution": "bancolombia_retail_co",
                        "access_mode": "recurrent",
                        "status": "valid"
                    },
                    {
                        "id": Uuid::new_v4().to_string(),
                        "institution": "nequi_co",
                        "access_mode": "single",
                        "status": "valid"
                    }
                ]
            })))
            .mount(&mock_server)
            .await;

        let client = create_test_client(&mock_server.uri());
        let result = client.list_links(None, None).await;

        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.count, 2);
        assert_eq!(response.results.len(), 2);
    }

    #[tokio::test]
    async fn test_list_links_with_pagination() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(wiremock::matchers::query_param("page", "1"))
            .and(wiremock::matchers::query_param("page_size", "5"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "count": 10,
                "next": "https://api.belvo.com/api/links/?page=2",
                "previous": null,
                "results": []
            })))
            .mount(&mock_server)
            .await;

        let client = create_test_client(&mock_server.uri());
        let result = client.list_links(Some(1), Some(5)).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_delete_link() {
        let mock_server = MockServer::start().await;
        let link_id = Uuid::new_v4();

        Mock::given(method("DELETE"))
            .and(path_regex(r"/api/links/[a-f0-9-]+/"))
            .respond_with(ResponseTemplate::new(204))
            .mount(&mock_server)
            .await;

        let client = create_test_client(&mock_server.uri());
        let result = client.delete_link(link_id).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_get_link_not_found() {
        let mock_server = MockServer::start().await;
        let link_id = Uuid::new_v4();

        Mock::given(method("GET"))
            .and(path_regex(r"/api/links/[a-f0-9-]+/"))
            .respond_with(ResponseTemplate::new(404).set_body_json(serde_json::json!({
                "code": "not_found",
                "message": "Link not found"
            })))
            .mount(&mock_server)
            .await;

        let client = create_test_client(&mock_server.uri());
        let result = client.get_link(link_id).await;

        assert!(result.is_err());
    }
}
