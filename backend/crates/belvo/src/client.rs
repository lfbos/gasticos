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
        let environment = env_str.parse::<BelvoEnvironment>()
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
        debug!("POST {}", url);

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

        // Try to parse error body
        match response.json::<BelvoApiError>().await {
            Ok(api_error) => api_error.into(),
            Err(_) => BelvoError::Api {
                code: status.as_str().to_string(),
                message: format!("HTTP error: {}", status),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
        assert!("invalid".parse::<BelvoEnvironment>().is_err());
    }
}
