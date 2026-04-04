//! Bank statement parsing engine for Gastico.
//!
//! This crate provides parsing logic for Colombian bank statement formats.

use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Supported file types for bank statements
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileType {
    Csv,
    Excel,
    Pdf,
}

/// A parsed transaction from a bank statement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedTransaction {
    pub date: NaiveDate,
    pub description: String,
    pub amount: Decimal,
    pub balance: Option<Decimal>,
    pub reference: Option<String>,
}

/// Errors that can occur during parsing
#[derive(Debug, Error)]
pub enum ParseError {
    #[error("Unsupported file format")]
    UnsupportedFormat,

    #[error("Unable to detect bank from file content")]
    UnknownBank,

    #[error("Invalid file encoding: {0}")]
    InvalidEncoding(String),

    #[error("Failed to parse date: {0}")]
    InvalidDate(String),

    #[error("Failed to parse amount: {0}")]
    InvalidAmount(String),

    #[error("File is empty or contains no transactions")]
    EmptyFile,

    #[error("Parse error: {0}")]
    Other(String),
}

pub type ParseResult<T> = Result<T, ParseError>;

/// Trait that all bank parsers must implement
pub trait BankParser: Send + Sync {
    fn bank_name(&self) -> &str;
    fn can_parse(&self, content: &[u8], file_type: FileType) -> bool;
    fn parse(&self, content: &[u8], file_type: FileType) -> ParseResult<Vec<ParsedTransaction>>;
}

/// Supported Colombian banks
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Bank {
    Bancolombia,
    Davivienda,
    BancoDeBogota,
    BbvaColombia,
    BancoDeOccidente,
    ScotiabankColpatria,
    ItauColombia,
    Nequi,
    Daviplata,
    NuColombia,
    LuloBank,
    Movii,
    RappiPay,
}

impl Bank {
    pub fn as_str(&self) -> &str {
        match self {
            Bank::Bancolombia => "Bancolombia",
            Bank::Davivienda => "Davivienda",
            Bank::BancoDeBogota => "Banco de Bogotá",
            Bank::BbvaColombia => "BBVA Colombia",
            Bank::BancoDeOccidente => "Banco de Occidente",
            Bank::ScotiabankColpatria => "Scotiabank Colpatria",
            Bank::ItauColombia => "Itaú Colombia",
            Bank::Nequi => "Nequi",
            Bank::Daviplata => "Daviplata",
            Bank::NuColombia => "Nu Colombia",
            Bank::LuloBank => "Lulo Bank",
            Bank::Movii => "MOVii",
            Bank::RappiPay => "RappiPay",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bank_names() {
        assert_eq!(Bank::Bancolombia.as_str(), "Bancolombia");
        assert_eq!(Bank::Nequi.as_str(), "Nequi");
        assert_eq!(Bank::NuColombia.as_str(), "Nu Colombia");
    }

    #[test]
    fn test_parsed_transaction_serialization() {
        let tx = ParsedTransaction {
            date: NaiveDate::from_ymd_opt(2025, 1, 15).unwrap(),
            description: "COMPRA EN D1".to_string(),
            amount: Decimal::new(-25000, 0),
            balance: Some(Decimal::new(1500000, 0)),
            reference: Some("REF123".to_string()),
        };

        let json = serde_json::to_string(&tx).unwrap();
        assert!(json.contains("COMPRA EN D1"));
    }
}
