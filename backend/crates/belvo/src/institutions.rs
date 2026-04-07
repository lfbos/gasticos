//! Belvo Institutions API.

use crate::client::BelvoClient;
use crate::error::{BelvoError, Result};
use crate::models::{Institution, PaginatedResponse};

impl BelvoClient {
    /// Get institution details by code.
    pub async fn get_institution(&self, code: &str) -> Result<Institution> {
        let response: PaginatedResponse<Institution> = self
            .get(&format!("/api/institutions/?name={}", code))
            .await?;
        response
            .results
            .into_iter()
            .next()
            .ok_or_else(|| BelvoError::NotFound(format!("Institution not found: {}", code)))
    }
}

#[cfg(test)]
mod tests {
    use wiremock::matchers::{method, query_param};
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
    async fn test_get_institution() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(query_param("name", "bancolombia_retail_co"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "count": 1,
                "next": null,
                "previous": null,
                "results": [{
                    "id": 123,
                    "name": "bancolombia_retail_co",
                    "display_name": "Bancolombia",
                    "country_codes": ["CO"],
                    "type": "bank",
                    "country_code": "CO"
                }]
            })))
            .mount(&mock_server)
            .await;

        let client = create_test_client(&mock_server.uri());
        let result = client.get_institution("bancolombia_retail_co").await;

        assert!(result.is_ok());
        let institution = result.unwrap();
        assert_eq!(institution.name, "bancolombia_retail_co");
        assert_eq!(institution.display_name, Some("Bancolombia".to_string()));
    }

    #[tokio::test]
    async fn test_get_institution_not_found() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(query_param("name", "unknown_bank"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "count": 0,
                "next": null,
                "previous": null,
                "results": []
            })))
            .mount(&mock_server)
            .await;

        let client = create_test_client(&mock_server.uri());
        let result = client.get_institution("unknown_bank").await;

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("Institution not found"));
    }
}
