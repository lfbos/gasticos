//! Transaction categorization engine for Gastico.

use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

/// Pre-defined system categories for Colombian context.
/// Uses English identifiers internally; UI displays localized names via i18n.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SystemCategory {
    Housing,
    Groceries,
    Restaurants,
    Transportation,
    Health,
    Education,
    Entertainment,
    Clothing,
    Financial,
    Technology,
    Subscriptions,
    Other,
}

impl SystemCategory {
    /// Returns the lowercase English key for database storage and API responses.
    pub fn as_key(&self) -> &str {
        match self {
            SystemCategory::Housing => "housing",
            SystemCategory::Groceries => "groceries",
            SystemCategory::Restaurants => "restaurants",
            SystemCategory::Transportation => "transportation",
            SystemCategory::Health => "health",
            SystemCategory::Education => "education",
            SystemCategory::Entertainment => "entertainment",
            SystemCategory::Clothing => "clothing",
            SystemCategory::Financial => "financial",
            SystemCategory::Technology => "technology",
            SystemCategory::Subscriptions => "subscriptions",
            SystemCategory::Other => "other",
        }
    }

    /// Returns all system categories.
    pub fn all() -> &'static [SystemCategory] {
        &[
            SystemCategory::Housing,
            SystemCategory::Groceries,
            SystemCategory::Restaurants,
            SystemCategory::Transportation,
            SystemCategory::Health,
            SystemCategory::Education,
            SystemCategory::Entertainment,
            SystemCategory::Clothing,
            SystemCategory::Financial,
            SystemCategory::Technology,
            SystemCategory::Subscriptions,
            SystemCategory::Other,
        ]
    }

    /// Parses a category from its key string.
    pub fn from_key(key: &str) -> Option<Self> {
        match key {
            "housing" => Some(SystemCategory::Housing),
            "groceries" => Some(SystemCategory::Groceries),
            "restaurants" => Some(SystemCategory::Restaurants),
            "transportation" => Some(SystemCategory::Transportation),
            "health" => Some(SystemCategory::Health),
            "education" => Some(SystemCategory::Education),
            "entertainment" => Some(SystemCategory::Entertainment),
            "clothing" => Some(SystemCategory::Clothing),
            "financial" => Some(SystemCategory::Financial),
            "technology" => Some(SystemCategory::Technology),
            "subscriptions" => Some(SystemCategory::Subscriptions),
            "other" => Some(SystemCategory::Other),
            _ => None,
        }
    }
}

/// A categorization result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategorizationResult {
    pub category_id: Uuid,
    pub category_name: String,
    pub confidence: f32,
    pub matched_by: Option<String>,
}

/// Errors that can occur during categorization
#[derive(Debug, Error)]
pub enum CategorizationError {
    #[error("Invalid rule pattern: {0}")]
    InvalidPattern(String),

    #[error("Categorization error: {0}")]
    Other(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_system_categories() {
        assert_eq!(SystemCategory::Groceries.as_key(), "groceries");
        assert_eq!(SystemCategory::Housing.as_key(), "housing");
        assert_eq!(SystemCategory::all().len(), 12);
    }

    #[test]
    fn test_category_from_key() {
        assert_eq!(
            SystemCategory::from_key("groceries"),
            Some(SystemCategory::Groceries)
        );
        assert_eq!(
            SystemCategory::from_key("transportation"),
            Some(SystemCategory::Transportation)
        );
        assert_eq!(SystemCategory::from_key("invalid"), None);
    }

    #[test]
    fn test_category_roundtrip() {
        for category in SystemCategory::all() {
            let key = category.as_key();
            let parsed = SystemCategory::from_key(key);
            assert_eq!(parsed, Some(*category));
        }
    }
}
