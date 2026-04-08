//! Transaction categorization engine for Gasticos.
//!
//! This crate provides automatic categorization of bank transactions using:
//! 1. A Colombian merchant dictionary for exact/partial matches
//! 2. Regex-based rules for pattern matching
//!
//! # Example
//!
//! ```rust
//! use categorizer::Categorizer;
//!
//! let categorizer = Categorizer::new();
//! let result = categorizer.categorize("NETFLIX.COM");
//! assert!(result.is_some());
//! ```

pub mod dictionary;
pub mod rules;

use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

pub use dictionary::lookup_merchant;
pub use rules::match_rules;

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

/// Main categorization service that combines dictionary lookup and rule matching.
///
/// Categorization strategy (in order of priority):
/// 1. Dictionary exact match (highest confidence)
/// 2. Dictionary partial match
/// 3. Rule-based pattern matching
/// 4. Falls back to `Other` category if no match found
#[derive(Debug, Default)]
pub struct Categorizer {
    /// Minimum confidence threshold for accepting a categorization
    min_confidence: f32,
}

impl Categorizer {
    /// Create a new categorizer with default settings.
    pub fn new() -> Self {
        Self {
            min_confidence: 0.5,
        }
    }

    /// Create a new categorizer with a custom confidence threshold.
    pub fn with_min_confidence(min_confidence: f32) -> Self {
        Self { min_confidence }
    }

    /// Categorize a transaction description.
    ///
    /// Returns a `CategorizationResult` with the best matching category,
    /// or `None` if no match is found above the minimum confidence threshold.
    pub fn categorize(&self, description: &str) -> Option<CategorizationResult> {
        let normalized = description.trim().to_uppercase();

        // Strategy 1: Dictionary lookup (highest priority)
        if let Some(category) = dictionary::lookup_merchant(&normalized) {
            return Some(CategorizationResult {
                category_id: Uuid::nil(), // Will be resolved by caller
                category_name: category.as_key().to_string(),
                confidence: 1.0, // Dictionary matches are high confidence
                matched_by: Some("dictionary".to_string()),
            });
        }

        // Strategy 2: Rule-based pattern matching
        if let Some((category, confidence, rule_name)) = rules::match_rules(&normalized) {
            if confidence >= self.min_confidence {
                return Some(CategorizationResult {
                    category_id: Uuid::nil(),
                    category_name: category.as_key().to_string(),
                    confidence,
                    matched_by: Some(format!("rule:{}", rule_name)),
                });
            }
        }

        // No match found above threshold
        None
    }

    /// Categorize a transaction, falling back to `Other` if no match.
    pub fn categorize_or_other(&self, description: &str) -> CategorizationResult {
        self.categorize(description)
            .unwrap_or_else(|| CategorizationResult {
                category_id: Uuid::nil(),
                category_name: SystemCategory::Other.as_key().to_string(),
                confidence: 0.0,
                matched_by: None,
            })
    }

    /// Categorize multiple transactions in batch.
    pub fn categorize_batch(
        &self,
        descriptions: &[&str],
    ) -> Vec<(String, Option<CategorizationResult>)> {
        descriptions
            .iter()
            .map(|desc| (desc.to_string(), self.categorize(desc)))
            .collect()
    }

    /// Get statistics about categorization coverage.
    pub fn categorize_batch_with_stats(&self, descriptions: &[&str]) -> CategorizationStats {
        let results = self.categorize_batch(descriptions);
        let total = results.len();
        let categorized = results.iter().filter(|(_, r)| r.is_some()).count();

        let by_dictionary = results
            .iter()
            .filter(|(_, r)| {
                r.as_ref()
                    .map(|r| r.matched_by.as_deref() == Some("dictionary"))
                    .unwrap_or(false)
            })
            .count();

        let by_rules = results
            .iter()
            .filter(|(_, r)| {
                r.as_ref()
                    .map(|r| {
                        r.matched_by
                            .as_ref()
                            .map(|m| m.starts_with("rule:"))
                            .unwrap_or(false)
                    })
                    .unwrap_or(false)
            })
            .count();

        CategorizationStats {
            total,
            categorized,
            uncategorized: total - categorized,
            by_dictionary,
            by_rules,
            coverage_percent: if total > 0 {
                (categorized as f32 / total as f32) * 100.0
            } else {
                0.0
            },
        }
    }
}

/// Statistics about batch categorization results.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategorizationStats {
    pub total: usize,
    pub categorized: usize,
    pub uncategorized: usize,
    pub by_dictionary: usize,
    pub by_rules: usize,
    pub coverage_percent: f32,
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

    // Categorizer tests
    #[test]
    fn test_categorizer_new() {
        let categorizer = Categorizer::new();
        assert_eq!(categorizer.min_confidence, 0.5);
    }

    #[test]
    fn test_categorizer_with_min_confidence() {
        let categorizer = Categorizer::with_min_confidence(0.8);
        assert_eq!(categorizer.min_confidence, 0.8);
    }

    #[test]
    fn test_categorize_by_dictionary() {
        let categorizer = Categorizer::new();

        // Netflix should match dictionary
        let result = categorizer.categorize("NETFLIX.COM").unwrap();
        assert_eq!(result.category_name, "subscriptions");
        assert_eq!(result.confidence, 1.0);
        assert_eq!(result.matched_by.as_deref(), Some("dictionary"));
    }

    #[test]
    fn test_categorize_by_rules() {
        let categorizer = Categorizer::new();

        // This should match rules (GRAVAMEN pattern - not in dictionary)
        let result = categorizer
            .categorize("GRAVAMEN MOVIMIENTO FINANCIERO")
            .unwrap();
        assert_eq!(result.category_name, "financial");
        assert!(result.matched_by.as_ref().unwrap().starts_with("rule:"));
    }

    #[test]
    fn test_categorize_no_match() {
        let categorizer = Categorizer::new();

        // Random text shouldn't match
        let result = categorizer.categorize("XYZABC123RANDOM");
        assert!(result.is_none());
    }

    #[test]
    fn test_categorize_or_other() {
        let categorizer = Categorizer::new();

        // No match should return Other
        let result = categorizer.categorize_or_other("XYZABC123RANDOM");
        assert_eq!(result.category_name, "other");
        assert_eq!(result.confidence, 0.0);
        assert!(result.matched_by.is_none());

        // Match should return the match
        let result = categorizer.categorize_or_other("UBER*TRIP");
        assert_eq!(result.category_name, "transportation");
    }

    #[test]
    fn test_categorize_batch() {
        let categorizer = Categorizer::new();
        let descriptions = ["NETFLIX", "UBER", "RANDOM123"];

        let results = categorizer.categorize_batch(&descriptions);
        assert_eq!(results.len(), 3);
        assert!(results[0].1.is_some()); // Netflix
        assert!(results[1].1.is_some()); // Uber
        assert!(results[2].1.is_none()); // Random
    }

    #[test]
    fn test_categorize_batch_with_stats() {
        let categorizer = Categorizer::new();
        let descriptions = [
            "NETFLIX",         // dictionary
            "ALMACENES EXITO", // dictionary
            "GRAVAMEN FINANC", // rules (GRAVAMEN not in dictionary)
            "RANDOM123",       // no match
        ];

        let stats = categorizer.categorize_batch_with_stats(&descriptions);
        assert_eq!(stats.total, 4);
        assert_eq!(stats.categorized, 3);
        assert_eq!(stats.uncategorized, 1);
        assert_eq!(stats.by_dictionary, 2);
        assert_eq!(stats.by_rules, 1);
        assert!((stats.coverage_percent - 75.0).abs() < 0.01);
    }

    #[test]
    fn test_categorize_case_insensitive() {
        let categorizer = Categorizer::new();

        // All should match regardless of case
        let r1 = categorizer.categorize("netflix").unwrap();
        let r2 = categorizer.categorize("NETFLIX").unwrap();
        let r3 = categorizer.categorize("Netflix").unwrap();

        assert_eq!(r1.category_name, r2.category_name);
        assert_eq!(r2.category_name, r3.category_name);
    }

    #[test]
    fn test_categorize_colombian_merchants() {
        let categorizer = Categorizer::new();

        // Test various Colombian merchants
        let test_cases = [
            ("EXITO MEDELLIN", "groceries"),
            ("D1 BOGOTA", "groceries"),
            ("RAPPI*DOMICILIO", "restaurants"),
            ("TRANSMILENIO SITP", "transportation"),
            ("DROGUERIA LA REBAJA", "health"),
            ("EPM FACTURA", "housing"),
            ("PLATZI SUBSCRIPTION", "education"),
        ];

        for (description, expected_category) in test_cases {
            let result = categorizer.categorize_or_other(description);
            assert_eq!(
                result.category_name, expected_category,
                "Failed for: {}",
                description
            );
        }
    }

    #[test]
    fn test_high_confidence_threshold() {
        let categorizer = Categorizer::with_min_confidence(0.95);

        // Dictionary matches (1.0 confidence) should still pass
        let result = categorizer.categorize("NETFLIX");
        assert!(result.is_some());

        // Low-priority rules might not pass
        // (depends on the specific rule priorities)
    }

    #[test]
    fn test_categorization_stats_empty() {
        let categorizer = Categorizer::new();
        let empty: &[&str] = &[];

        let stats = categorizer.categorize_batch_with_stats(empty);
        assert_eq!(stats.total, 0);
        assert_eq!(stats.coverage_percent, 0.0);
    }
}
