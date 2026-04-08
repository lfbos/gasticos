//! Nequi bank statement parser.

use chrono::NaiveDate;
use regex::Regex;
use rust_decimal::Decimal;
use std::str::FromStr;
use tracing::debug;

use crate::{BankParser, ParsedTransaction, ParserError};

/// Parser for Nequi bank statements.
pub struct NequiParser;

impl BankParser for NequiParser {
    fn bank_name(&self) -> &str {
        "Nequi"
    }

    fn can_parse(&self, text: &str) -> bool {
        text.contains("Nequi") || text.contains("depósito de bajo monto")
    }

    fn parse(&self, text: &str) -> Result<Vec<ParsedTransaction>, ParserError> {
        parse_nequi(text)
    }
}

/// Parse Nequi statement text into transactions.
fn parse_nequi(text: &str) -> Result<Vec<ParsedTransaction>, ParserError> {
    // Transaction line regex
    // Format: DD/MM/YYYY Description $[-]VALUE $BALANCE
    // Examples:
    // 27/03/2026 Para SAININA ANDREA $-360,000.00 $40,976.17
    // 27/03/2026 Recarga desde Bancolombia $100,000.00 $400,976.17
    let tx_regex =
        Regex::new(r"(?m)(\d{2}/\d{2}/\d{4})\s+(.+?)\s+\$(-?[\d,]+\.\d{2})\s+\$([\d,]+\.\d{2})")
            .map_err(|e| ParserError::ParseTransaction(e.to_string()))?;

    let mut transactions = Vec::new();

    for cap in tx_regex.captures_iter(text) {
        let date_str = &cap[1];
        let description = cap[2].trim().to_string();
        let amount_str = cap[3].replace(',', "");
        let balance_str = cap[4].replace(',', "");

        // Parse date DD/MM/YYYY
        let date = parse_date_dmy(date_str)?;

        let amount = Decimal::from_str(&amount_str).map_err(|e| {
            ParserError::ParseAmount(format!("Invalid amount '{}': {}", amount_str, e))
        })?;

        let balance = Decimal::from_str(&balance_str).map_err(|e| {
            ParserError::ParseAmount(format!("Invalid balance '{}': {}", balance_str, e))
        })?;

        let is_income = amount > Decimal::ZERO;

        transactions.push(ParsedTransaction {
            date,
            description,
            amount,
            balance: Some(balance),
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
        "Parsed {} transactions from Nequi statement",
        transactions.len()
    );

    Ok(transactions)
}

/// Parse a date in DD/MM/YYYY format.
fn parse_date_dmy(date_str: &str) -> Result<NaiveDate, ParserError> {
    let parts: Vec<&str> = date_str.split('/').collect();
    if parts.len() != 3 {
        return Err(ParserError::ParseDate(format!(
            "Invalid date format: {}",
            date_str
        )));
    }

    let day: u32 = parts[0]
        .parse()
        .map_err(|e| ParserError::ParseDate(format!("Invalid day: {}", e)))?;
    let month: u32 = parts[1]
        .parse()
        .map_err(|e| ParserError::ParseDate(format!("Invalid month: {}", e)))?;
    let year: i32 = parts[2]
        .parse()
        .map_err(|e| ParserError::ParseDate(format!("Invalid year: {}", e)))?;

    NaiveDate::from_ymd_opt(year, month, day)
        .ok_or_else(|| ParserError::ParseDate(format!("Invalid date: {}", date_str)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_can_parse_nequi() {
        let parser = NequiParser;
        assert!(parser.can_parse("Extracto de depósito de bajo monto de:\nNequi"));
        assert!(parser.can_parse("depósito de bajo monto"));
        assert!(!parser.can_parse("Bancolombia statement"));
    }

    #[test]
    fn test_parse_nequi_transactions() {
        let text = r#"
Extracto de depósito de bajo monto de:
LUIS BOSCAN
Número de depósito de bajo monto:  3205326843
Estado de depósito de bajo monto para el período de: 2026/03/01 a 2026/03/31

Fecha del movimiento Descripción Valor Saldo
27/03/2026 Para SAININA ANDREA $-360,000.00 $40,976.17
27/03/2026 Recarga desde Bancolombia $100,000.00 $400,976.17
20/03/2026 COMPRA EN APPLE COM BILL $-27,000.00 $976.17
19/03/2026 Pago de Intereses $3.28 $27,976.17
"#;
        let parser = NequiParser;
        let transactions = parser.parse(text).unwrap();

        assert_eq!(transactions.len(), 4);

        // First transaction - expense
        assert_eq!(
            transactions[0].date,
            NaiveDate::from_ymd_opt(2026, 3, 27).unwrap()
        );
        assert_eq!(transactions[0].description, "Para SAININA ANDREA");
        assert_eq!(
            transactions[0].amount,
            Decimal::from_str("-360000.00").unwrap()
        );
        assert!(!transactions[0].is_income);

        // Second transaction - income
        assert_eq!(
            transactions[1].date,
            NaiveDate::from_ymd_opt(2026, 3, 27).unwrap()
        );
        assert_eq!(transactions[1].description, "Recarga desde Bancolombia");
        assert_eq!(
            transactions[1].amount,
            Decimal::from_str("100000.00").unwrap()
        );
        assert!(transactions[1].is_income);

        // Apple purchase
        assert_eq!(transactions[2].description, "COMPRA EN APPLE COM BILL");
        assert!(!transactions[2].is_income);

        // Interest payment
        assert_eq!(transactions[3].description, "Pago de Intereses");
        assert!(transactions[3].is_income);
    }

    #[test]
    fn test_parse_date_dmy() {
        let date = parse_date_dmy("27/03/2026").unwrap();
        assert_eq!(date, NaiveDate::from_ymd_opt(2026, 3, 27).unwrap());
    }

    #[test]
    fn test_parse_date_dmy_invalid() {
        assert!(parse_date_dmy("invalid").is_err());
        assert!(parse_date_dmy("32/13/2026").is_err());
    }
}
