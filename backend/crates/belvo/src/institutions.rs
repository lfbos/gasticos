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
