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
