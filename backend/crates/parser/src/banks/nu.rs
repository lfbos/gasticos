//! Nu Colombia bank statement parser.

use chrono::NaiveDate;
use regex::Regex;
use rust_decimal::Decimal;
use std::str::FromStr;
use tracing::debug;

use crate::{BankParser, ParsedTransaction, ParserError};

/// Parser for Nu Colombia bank statements.
pub struct NuParser;

impl BankParser for NuParser {
    fn bank_name(&self) -> &str {
        "Nu"
    }

    fn can_parse(&self, text: &str) -> bool {
        text.contains("Nu Colombia") || text.contains("Nu Financiera") || text.contains("Nu Placa")
    }

    fn parse(&self, text: &str) -> Result<Vec<ParsedTransaction>, ParserError> {
        parse_nu(text)
    }
}

/// Parse Nu statement text into transactions.
fn parse_nu(text: &str) -> Result<Vec<ParsedTransaction>, ParserError> {
    // Extract year from the period header
    // Format: "Período\n01 - 31 MAR 2026" or "01 - 31 MAR 2026"
    let year_regex = Regex::new(r"(\d{2})\s*-\s*\d{2}\s+[A-Z]{3}\s+(\d{4})")
        .map_err(|e| ParserError::ParseTransaction(e.to_string()))?;

    let year: i32 = year_regex
        .captures(text)
        .and_then(|cap| cap.get(2))
        .and_then(|m| m.as_str().parse().ok())
        .ok_or_else(|| ParserError::ParseDate("Could not extract year from header".to_string()))?;

    debug!("Extracted year from Nu statement: {}", year);

    // Transaction line regex
    // Format: DD mmm Description [-]$AMOUNT
    // Examples:
    // 02 mar Enviaste a Luis Felipe Junior Boscan -$250.000,00
    // 09 mar Recibiste de Banco Citibank +$4.000.000,00
    let tx_regex = Regex::new(
        r"(?m)(\d{2})\s+(ene|feb|mar|abr|may|jun|jul|ago|sep|oct|nov|dic)\s+(.+?)\s+([+-]?)\$([\d.]+,\d{2})",
    )
    .map_err(|e| ParserError::ParseTransaction(e.to_string()))?;

    let mut transactions = Vec::new();

    for cap in tx_regex.captures_iter(text) {
        let day: u32 = cap[1]
            .parse()
            .map_err(|e| ParserError::ParseDate(format!("Invalid day: {}", e)))?;
        let month_str = &cap[2];
        let description = cap[3].trim().to_string();
        let sign = &cap[4];
        // Colombian format: periods as thousands separator, comma as decimal
        let amount_str = cap[5].replace('.', "").replace(',', ".");

        // Skip 4x1000 tax lines
        if description.contains("Impuesto del 4x1000") {
            continue;
        }

        let month = parse_spanish_month(month_str)?;

        let date = NaiveDate::from_ymd_opt(year, month, day).ok_or_else(|| {
            ParserError::ParseDate(format!("Invalid date: {}/{}/{}", year, month, day))
        })?;

        let mut amount = Decimal::from_str(&amount_str).map_err(|e| {
            ParserError::ParseAmount(format!("Invalid amount '{}': {}", amount_str, e))
        })?;

        // Apply sign
        let is_income = sign == "+" || sign.is_empty() && description.starts_with("Recibiste");
        if !is_income && amount > Decimal::ZERO {
            amount = -amount;
        }

        transactions.push(ParsedTransaction {
            date,
            description,
            amount,
            balance: None, // Nu doesn't show running balance in statement
            reference: None,
            is_income,
            sequence: 0,
        });
    }

    // Assign sequence numbers
    for (i, tx) in transactions.iter_mut().enumerate() {
        tx.sequence = i as i32;
    }

    debug!(
        "Parsed {} transactions from Nu statement",
        transactions.len()
    );

    Ok(transactions)
}

/// Parse Spanish month abbreviation to month number.
fn parse_spanish_month(month_str: &str) -> Result<u32, ParserError> {
    match month_str.to_lowercase().as_str() {
        "ene" => Ok(1),
        "feb" => Ok(2),
        "mar" => Ok(3),
        "abr" => Ok(4),
        "may" => Ok(5),
        "jun" => Ok(6),
        "jul" => Ok(7),
        "ago" => Ok(8),
        "sep" => Ok(9),
        "oct" => Ok(10),
        "nov" => Ok(11),
        "dic" => Ok(12),
        _ => Err(ParserError::ParseDate(format!(
            "Unknown month: {}",
            month_str
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_can_parse_nu() {
        let parser = NuParser;
        assert!(parser.can_parse("Nu Colombia Compañía de Financiamiento S.A."));
        assert!(parser.can_parse("Nu Financiera\nBogotá D.C."));
        assert!(parser.can_parse("Nu Placa\nLBN709"));
        assert!(!parser.can_parse("Bancolombia statement"));
    }

    #[test]
    fn test_parse_nu_transactions() {
        let text = r#"
Hola, Luis
Llegó tu extracto de Marzo
Período
01 - 31 MAR 2026

Movimientos

02 mar Enviaste a Luis Felipe Junior Boscan -$250.000,00
Impuesto del 4x1000 -$1.000,00
03 mar Pagaste tu tarjeta -$100.000,00
09 mar Recibiste de Banco Citibank +$4.000.000,00
06 mar Compra en DOLLARCITY VALLE con tarjeta débito -$202.000,00
"#;
        let parser = NuParser;
        let transactions = parser.parse(text).unwrap();

        // Should have 4 transactions (skipping 4x1000 tax)
        assert_eq!(transactions.len(), 4);

        // First transaction - expense
        assert_eq!(
            transactions[0].date,
            NaiveDate::from_ymd_opt(2026, 3, 2).unwrap()
        );
        assert_eq!(
            transactions[0].description,
            "Enviaste a Luis Felipe Junior Boscan"
        );
        assert_eq!(
            transactions[0].amount,
            Decimal::from_str("-250000.00").unwrap()
        );
        assert!(!transactions[0].is_income);

        // Third transaction - income (Recibiste)
        assert_eq!(
            transactions[2].date,
            NaiveDate::from_ymd_opt(2026, 3, 9).unwrap()
        );
        assert_eq!(transactions[2].description, "Recibiste de Banco Citibank");
        assert_eq!(
            transactions[2].amount,
            Decimal::from_str("4000000.00").unwrap()
        );
        assert!(transactions[2].is_income);
    }

    #[test]
    fn test_parse_spanish_month() {
        assert_eq!(parse_spanish_month("ene").unwrap(), 1);
        assert_eq!(parse_spanish_month("mar").unwrap(), 3);
        assert_eq!(parse_spanish_month("dic").unwrap(), 12);
        assert!(parse_spanish_month("invalid").is_err());
    }
}
