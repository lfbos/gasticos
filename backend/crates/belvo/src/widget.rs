//! Widget token generation for Belvo Connect widget.

use crate::client::BelvoClient;
use crate::error::Result;
use crate::models::{
    CreateWidgetTokenRequest, WidgetBranding, WidgetCallbackUrls, WidgetConfig, WidgetToken,
};

impl BelvoClient {
    /// Create a widget access token for the Belvo Connect widget.
    ///
    /// This token is used to initialize the Belvo Connect widget in the frontend,
    /// allowing users to securely connect their bank accounts.
    pub async fn create_widget_token(&self) -> Result<WidgetToken> {
        let request = CreateWidgetTokenRequest {
            id: self.secret_id().to_string(),
            password: self.secret_password().to_string(),
            scopes: Some("read_institutions,write_links,read_links".to_string()),
            link_id: None,
            widget: None,
        };

        self.post("/api/token/", &request).await
    }

    /// Create a widget access token with custom configuration.
    ///
    /// Use this to customize the widget appearance and behavior.
    pub async fn create_widget_token_with_config(
        &self,
        company_name: Option<&str>,
        company_logo: Option<&str>,
        institutions: Option<Vec<String>>,
        callback_success: Option<&str>,
        callback_exit: Option<&str>,
        callback_event: Option<&str>,
    ) -> Result<WidgetToken> {
        let branding = if company_name.is_some() || company_logo.is_some() {
            Some(WidgetBranding {
                company_name: company_name.map(String::from),
                company_logo: company_logo.map(String::from),
            })
        } else {
            None
        };

        let callback_urls =
            if callback_success.is_some() || callback_exit.is_some() || callback_event.is_some() {
                Some(WidgetCallbackUrls {
                    success: callback_success.map(String::from),
                    exit: callback_exit.map(String::from),
                    event: callback_event.map(String::from),
                })
            } else {
                None
            };

        let widget = if branding.is_some() || callback_urls.is_some() || institutions.is_some() {
            Some(WidgetConfig {
                branding,
                callback_urls,
                institutions,
            })
        } else {
            None
        };

        let request = CreateWidgetTokenRequest {
            id: self.secret_id().to_string(),
            password: self.secret_password().to_string(),
            scopes: Some("read_institutions,write_links,read_links".to_string()),
            link_id: None,
            widget,
        };

        self.post("/api/token/", &request).await
    }

    /// Create a widget token for updating an existing link.
    ///
    /// Use this when a link requires token renewal or credential update.
    pub async fn create_widget_token_for_link(&self, link_id: &str) -> Result<WidgetToken> {
        let request = CreateWidgetTokenRequest {
            id: self.secret_id().to_string(),
            password: self.secret_password().to_string(),
            scopes: Some("read_institutions,write_links,read_links".to_string()),
            link_id: Some(link_id.to_string()),
            widget: None,
        };

        self.post("/api/token/", &request).await
    }
}

#[cfg(test)]
mod tests {
    use wiremock::matchers::{method, path};
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
        // Override base URL for testing
        client.set_base_url(base_url.to_string());
        client
    }

    #[tokio::test]
    async fn test_create_widget_token() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/api/token/"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "access": "test_access_token",
                "refresh": "test_refresh_token"
            })))
            .mount(&mock_server)
            .await;

        let client = create_test_client(&mock_server.uri());
        let result = client.create_widget_token().await;

        assert!(result.is_ok());
        let token = result.unwrap();
        assert_eq!(token.access, "test_access_token");
        assert_eq!(token.refresh, "test_refresh_token");
    }

    #[tokio::test]
    async fn test_create_widget_token_with_config() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/api/token/"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "access": "config_access_token",
                "refresh": "config_refresh_token"
            })))
            .mount(&mock_server)
            .await;

        let client = create_test_client(&mock_server.uri());
        let result = client
            .create_widget_token_with_config(
                Some("Gasticos"),
                Some("https://example.com/logo.png"),
                Some(vec!["bancolombia_retail_co".to_string()]),
                None,
                None,
                None,
            )
            .await;

        assert!(result.is_ok());
        let token = result.unwrap();
        assert_eq!(token.access, "config_access_token");
    }

    #[tokio::test]
    async fn test_create_widget_token_for_link() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/api/token/"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "access": "link_access_token",
                "refresh": "link_refresh_token"
            })))
            .mount(&mock_server)
            .await;

        let client = create_test_client(&mock_server.uri());
        let result = client.create_widget_token_for_link("link_123").await;

        assert!(result.is_ok());
        let token = result.unwrap();
        assert_eq!(token.access, "link_access_token");
    }

    #[tokio::test]
    async fn test_create_widget_token_error() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/api/token/"))
            .respond_with(ResponseTemplate::new(401).set_body_json(serde_json::json!({
                "code": "authentication_failed",
                "message": "Invalid credentials"
            })))
            .mount(&mock_server)
            .await;

        let client = create_test_client(&mock_server.uri());
        let result = client.create_widget_token().await;

        assert!(result.is_err());
    }
}
