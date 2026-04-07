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
