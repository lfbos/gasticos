//! Transaction categorization engine for Gastico.

use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

/// Pre-defined system categories for Colombian context
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SystemCategory {
    Vivienda,
    Mercado,
    Restaurantes,
    Transporte,
    Salud,
    Educacion,
    Entretenimiento,
    Ropa,
    Financiero,
    Tecnologia,
    Suscripciones,
    Otros,
}

impl SystemCategory {
    pub fn as_str(&self) -> &str {
        match self {
            SystemCategory::Vivienda => "Vivienda",
            SystemCategory::Mercado => "Mercado",
            SystemCategory::Restaurantes => "Restaurantes",
            SystemCategory::Transporte => "Transporte",
            SystemCategory::Salud => "Salud",
            SystemCategory::Educacion => "Educación",
            SystemCategory::Entretenimiento => "Entretenimiento",
            SystemCategory::Ropa => "Ropa",
            SystemCategory::Financiero => "Financiero",
            SystemCategory::Tecnologia => "Tecnología",
            SystemCategory::Suscripciones => "Suscripciones",
            SystemCategory::Otros => "Otros",
        }
    }

    pub fn all() -> &'static [SystemCategory] {
        &[
            SystemCategory::Vivienda,
            SystemCategory::Mercado,
            SystemCategory::Restaurantes,
            SystemCategory::Transporte,
            SystemCategory::Salud,
            SystemCategory::Educacion,
            SystemCategory::Entretenimiento,
            SystemCategory::Ropa,
            SystemCategory::Financiero,
            SystemCategory::Tecnologia,
            SystemCategory::Suscripciones,
            SystemCategory::Otros,
        ]
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
        assert_eq!(SystemCategory::Mercado.as_str(), "Mercado");
        assert_eq!(SystemCategory::all().len(), 12);
    }
}
