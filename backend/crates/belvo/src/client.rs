//! Belvo API HTTP client.

use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use reqwest::{Client, Response, StatusCode};
use tracing::{debug, error};

use crate::error::{BelvoApiError, BelvoError, Result};

/// Belvo API environment.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum BelvoEnvironment {
    #[default]
    Sandbox,
    Development,
    Production,
}

impl BelvoEnvironment {
    /// Get the base URL for this environment.
    pub fn base_url(&self) -> &'static str {
        match self {
            BelvoEnvironment::Sandbox => "https://sandbox.belvo.com",
            BelvoEnvironment::Development => "https://development.belvo.com",
            BelvoEnvironment::Production => "https://api.belvo.com",
        }
    }
}

impl std::str::FromStr for BelvoEnvironment {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "sandbox" => Ok(BelvoEnvironment::Sandbox),
            "development" => Ok(BelvoEnvironment::Development),
            "production" => Ok(BelvoEnvironment::Production),
            _ => Err(format!("Invalid environment: {}", s)),
        }
    }
}

/// Configuration for the Belvo client.
#[derive(Debug, Clone)]
pub struct BelvoConfig {
    pub secret_id: String,
    pub secret_password: String,
    pub environment: BelvoEnvironment,
}

impl BelvoConfig {
    /// Create a new configuration.
    pub fn new(secret_id: String, secret_password: String, environment: BelvoEnvironment) -> Self {
        Self {
            secret_id,
            secret_password,
            environment,
        }
    }

    /// Create configuration from environment variables.
    pub fn from_env() -> Result<Self> {
        let secret_id = std::env::var("BELVO_SECRET_ID")
            .map_err(|_| BelvoError::Configuration("BELVO_SECRET_ID not set".to_string()))?;
        let secret_password = std::env::var("BELVO_SECRET_PASSWORD")
            .map_err(|_| BelvoError::Configuration("BELVO_SECRET_PASSWORD not set".to_string()))?;
        let env_str = std::env::var("BELVO_ENV").unwrap_or_else(|_| "sandbox".to_string());
        let environment = env_str
            .parse::<BelvoEnvironment>()
            .map_err(|_| BelvoError::Configuration(format!("Invalid BELVO_ENV: {}", env_str)))?;

        Ok(Self {
            secret_id,
            secret_password,
            environment,
        })
    }
}

/// Belvo API client.
#[derive(Debug, Clone)]
pub struct BelvoClient {
    config: BelvoConfig,
    http: Client,
    base_url: String,
}

impl BelvoClient {
    /// Create a new Belvo client with the given configuration.
    pub fn new(config: BelvoConfig) -> Result<Self> {
        let http = Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .map_err(BelvoError::Request)?;

        let base_url = config.environment.base_url().to_string();

        Ok(Self {
            config,
            http,
            base_url,
        })
    }

    /// Create a new Belvo client from environment variables.
    pub fn from_env() -> Result<Self> {
        let config = BelvoConfig::from_env()?;
        Self::new(config)
    }

    /// Get the base URL.
    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    /// Set the base URL (for testing).
    #[cfg(test)]
    pub fn set_base_url(&mut self, url: String) {
        self.base_url = url;
    }

    /// Get the secret ID (for widget token).
    pub fn secret_id(&self) -> &str {
        &self.config.secret_id
    }

    /// Get the secret password (for widget token).
    pub fn secret_password(&self) -> &str {
        &self.config.secret_password
    }

    /// Create Basic Auth header value.
    fn basic_auth(&self) -> String {
        let credentials = format!("{}:{}", self.config.secret_id, self.config.secret_password);
        let encoded = BASE64.encode(credentials.as_bytes());
        format!("Basic {}", encoded)
    }

    /// Make a GET request to the Belvo API.
    pub async fn get<T: serde::de::DeserializeOwned>(&self, path: &str) -> Result<T> {
        let url = format!("{}{}", self.base_url, path);
        debug!("GET {}", url);

        let response = self
            .http
            .get(&url)
            .header("Authorization", self.basic_auth())
            .header("Accept", "application/json")
            .send()
            .await?;

        self.handle_response(response).await
    }

    /// Make a POST request to the Belvo API.
    pub async fn post<T: serde::de::DeserializeOwned, B: serde::Serialize>(
        &self,
        path: &str,
        body: &B,
    ) -> Result<T> {
        let url = format!("{}{}", self.base_url, path);

        // Log request details for debugging
        if let Ok(body_json) = serde_json::to_string(body) {
            debug!("POST {} with body: {}", url, body_json);
        } else {
            debug!("POST {}", url);
        }

        let response = self
            .http
            .post(&url)
            .header("Authorization", self.basic_auth())
            .header("Accept", "application/json")
            .header("Content-Type", "application/json")
            .json(body)
            .send()
            .await?;

        self.handle_response(response).await
    }

    /// Make a DELETE request to the Belvo API.
    pub async fn delete(&self, path: &str) -> Result<()> {
        let url = format!("{}{}", self.base_url, path);
        debug!("DELETE {}", url);

        let response = self
            .http
            .delete(&url)
            .header("Authorization", self.basic_auth())
            .send()
            .await?;

        match response.status() {
            StatusCode::NO_CONTENT | StatusCode::OK => Ok(()),
            status => {
                let error = self.parse_error(response).await;
                error!("DELETE {} failed with status {}: {:?}", url, status, error);
                Err(error)
            }
        }
    }

    /// Make a PATCH request to the Belvo API.
    pub async fn patch<T: serde::de::DeserializeOwned, B: serde::Serialize>(
        &self,
        path: &str,
        body: &B,
    ) -> Result<T> {
        let url = format!("{}{}", self.base_url, path);
        debug!("PATCH {}", url);

        let response = self
            .http
            .patch(&url)
            .header("Authorization", self.basic_auth())
            .header("Accept", "application/json")
            .header("Content-Type", "application/json")
            .json(body)
            .send()
            .await?;

        self.handle_response(response).await
    }

    /// Handle API response.
    async fn handle_response<T: serde::de::DeserializeOwned>(
        &self,
        response: Response,
    ) -> Result<T> {
        let status = response.status();

        if status.is_success() {
            let body = response.text().await?;
            debug!("Response body: {}", &body[..body.len().min(500)]);
            serde_json::from_str(&body).map_err(|e| {
                error!("Failed to parse response: {}", e);
                BelvoError::InvalidResponse(format!("Failed to parse response: {}", e))
            })
        } else {
            Err(self.parse_error(response).await)
        }
    }

    /// Parse error response.
    async fn parse_error(&self, response: Response) -> BelvoError {
        let status = response.status();

        // Check for rate limiting
        if status == StatusCode::TOO_MANY_REQUESTS {
            let retry_after = response
                .headers()
                .get("Retry-After")
                .and_then(|v| v.to_str().ok())
                .and_then(|s| s.parse().ok())
                .unwrap_or(60);
            return BelvoError::RateLimited { retry_after };
        }

        // Check for not found
        if status == StatusCode::NOT_FOUND {
            return BelvoError::NotFound("Resource not found".to_string());
        }

        // Check for authentication errors
        if status == StatusCode::UNAUTHORIZED {
            return BelvoError::Authentication("Invalid credentials".to_string());
        }

        // Get the response body for detailed logging
        let body = match response.text().await {
            Ok(text) => {
                error!("Belvo API error response (status {}): {}", status, text);
                text
            }
            Err(e) => {
                error!("Failed to read error response body: {}", e);
                return BelvoError::Api {
                    code: status.as_str().to_string(),
                    message: format!("HTTP error: {} (failed to read body)", status),
                };
            }
        };

        // Try to parse error body as JSON
        match serde_json::from_str::<BelvoApiError>(&body) {
            Ok(api_error) => api_error.into(),
            Err(_) => BelvoError::Api {
                code: status.as_str().to_string(),
                message: format!("HTTP error: {} - {}", status, body),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::matchers::{header, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

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

    #[test]
    fn test_environment_urls() {
        assert_eq!(
            BelvoEnvironment::Sandbox.base_url(),
            "https://sandbox.belvo.com"
        );
        assert_eq!(
            BelvoEnvironment::Production.base_url(),
            "https://api.belvo.com"
        );
        assert_eq!(
            BelvoEnvironment::Development.base_url(),
            "https://development.belvo.com"
        );
    }

    #[test]
    fn test_environment_from_str() {
        assert_eq!(
            "sandbox".parse::<BelvoEnvironment>().unwrap(),
            BelvoEnvironment::Sandbox
        );
        assert_eq!(
            "PRODUCTION".parse::<BelvoEnvironment>().unwrap(),
            BelvoEnvironment::Production
        );
        assert_eq!(
            "Development".parse::<BelvoEnvironment>().unwrap(),
            BelvoEnvironment::Development
        );
        assert!("invalid".parse::<BelvoEnvironment>().is_err());
    }

    #[test]
    fn test_environment_default() {
        assert_eq!(BelvoEnvironment::default(), BelvoEnvironment::Sandbox);
    }

    #[test]
    fn test_config_new() {
        let config = BelvoConfig::new(
            "my_id".to_string(),
            "my_password".to_string(),
            BelvoEnvironment::Production,
        );
        assert_eq!(config.secret_id, "my_id");
        assert_eq!(config.secret_password, "my_password");
        assert_eq!(config.environment, BelvoEnvironment::Production);
    }

    #[test]
    fn test_client_new() {
        let config = BelvoConfig::new(
            "test_id".to_string(),
            "test_password".to_string(),
            BelvoEnvironment::Sandbox,
        );
        let client = BelvoClient::new(config).unwrap();
        assert_eq!(client.base_url(), "https://sandbox.belvo.com");
        assert_eq!(client.secret_id(), "test_id");
        assert_eq!(client.secret_password(), "test_password");
    }

    #[test]
    fn test_basic_auth_header() {
        let config = BelvoConfig::new(
            "test_id".to_string(),
            "test_password".to_string(),
            BelvoEnvironment::Sandbox,
        );
        let client = BelvoClient::new(config).unwrap();
        let auth = client.basic_auth();
        // "test_id:test_password" base64 encoded = "dGVzdF9pZDp0ZXN0X3Bhc3N3b3Jk"
        assert_eq!(auth, "Basic dGVzdF9pZDp0ZXN0X3Bhc3N3b3Jk");
    }

    #[tokio::test]
    async fn test_get_request() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/api/test/"))
            .and(header(
                "Authorization",
                "Basic dGVzdF9pZDp0ZXN0X3Bhc3N3b3Jk",
            ))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": "test"
            })))
            .mount(&mock_server)
            .await;

        let client = create_test_client(&mock_server.uri());
        let result: Result<serde_json::Value> = client.get("/api/test/").await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap()["data"], "test");
    }

    #[tokio::test]
    async fn test_post_request() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/api/test/"))
            .and(header("Content-Type", "application/json"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "created": true
            })))
            .mount(&mock_server)
            .await;

        let client = create_test_client(&mock_server.uri());
        let body = serde_json::json!({"name": "test"});
        let result: Result<serde_json::Value> = client.post("/api/test/", &body).await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap()["created"], true);
    }

    #[tokio::test]
    async fn test_delete_request() {
        let mock_server = MockServer::start().await;

        Mock::given(method("DELETE"))
            .and(path("/api/test/123/"))
            .respond_with(ResponseTemplate::new(204))
            .mount(&mock_server)
            .await;

        let client = create_test_client(&mock_server.uri());
        let result = client.delete("/api/test/123/").await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_patch_request() {
        let mock_server = MockServer::start().await;

        Mock::given(method("PATCH"))
            .and(path("/api/test/123/"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "updated": true
            })))
            .mount(&mock_server)
            .await;

        let client = create_test_client(&mock_server.uri());
        let body = serde_json::json!({"name": "updated"});
        let result: Result<serde_json::Value> = client.patch("/api/test/123/", &body).await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap()["updated"], true);
    }

    #[tokio::test]
    async fn test_rate_limit_error() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/api/test/"))
            .respond_with(ResponseTemplate::new(429).insert_header("Retry-After", "120"))
            .mount(&mock_server)
            .await;

        let client = create_test_client(&mock_server.uri());
        let result: Result<serde_json::Value> = client.get("/api/test/").await;

        assert!(result.is_err());
        match result.unwrap_err() {
            BelvoError::RateLimited { retry_after } => assert_eq!(retry_after, 120),
            e => panic!("Expected RateLimited error, got {:?}", e),
        }
    }

    #[tokio::test]
    async fn test_not_found_error() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/api/test/"))
            .respond_with(ResponseTemplate::new(404))
            .mount(&mock_server)
            .await;

        let client = create_test_client(&mock_server.uri());
        let result: Result<serde_json::Value> = client.get("/api/test/").await;

        assert!(result.is_err());
        match result.unwrap_err() {
            BelvoError::NotFound(_) => {}
            e => panic!("Expected NotFound error, got {:?}", e),
        }
    }

    #[tokio::test]
    async fn test_authentication_error() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/api/test/"))
            .respond_with(ResponseTemplate::new(401))
            .mount(&mock_server)
            .await;

        let client = create_test_client(&mock_server.uri());
        let result: Result<serde_json::Value> = client.get("/api/test/").await;

        assert!(result.is_err());
        match result.unwrap_err() {
            BelvoError::Authentication(_) => {}
            e => panic!("Expected Authentication error, got {:?}", e),
        }
    }

    #[tokio::test]
    async fn test_api_error_response() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/api/test/"))
            .respond_with(ResponseTemplate::new(400).set_body_json(serde_json::json!({
                "code": "invalid_request",
                "message": "Bad request parameters"
            })))
            .mount(&mock_server)
            .await;

        let client = create_test_client(&mock_server.uri());
        let result: Result<serde_json::Value> = client.get("/api/test/").await;

        assert!(result.is_err());
        match result.unwrap_err() {
            BelvoError::Api { code, message } => {
                assert_eq!(code, "invalid_request");
                assert_eq!(message, "Bad request parameters");
            }
            e => panic!("Expected Api error, got {:?}", e),
        }
    }

    #[tokio::test]
    async fn test_delete_with_ok_status() {
        let mock_server = MockServer::start().await;

        Mock::given(method("DELETE"))
            .and(path("/api/test/123/"))
            .respond_with(ResponseTemplate::new(200))
            .mount(&mock_server)
            .await;

        let client = create_test_client(&mock_server.uri());
        let result = client.delete("/api/test/123/").await;

        assert!(result.is_ok());
    }
}
