//! Bancolombia bank statement parser.

use chrono::NaiveDate;
use regex::Regex;
use rust_decimal::Decimal;
use std::str::FromStr;
use tracing::debug;

use crate::{BankParser, ParsedTransaction, ParserError};

/// Parser for Bancolombia bank statements.
pub struct BancolombiaParser;

impl BankParser for BancolombiaParser {
    fn bank_name(&self) -> &str {
        "Bancolombia"
    }

    fn can_parse(&self, text: &str) -> bool {
        (text.contains("CUENTA DE AHORROS") && text.contains("BANCOLOMBIA"))
            || (text.contains("ESTADO DE CUENTA") && text.contains("SUCURSAL"))
    }

    fn parse(&self, text: &str) -> Result<Vec<ParsedTransaction>, ParserError> {
        parse_bancolombia(text)
    }
}

/// Parse Bancolombia statement text into transactions.
///
/// Handles two formats:
/// 1. Single-line format: "DD/MM DESCRIPTION VALUE BALANCE"
/// 2. Columnar format: dates, descriptions, values, balances in separate columns
fn parse_bancolombia(text: &str) -> Result<Vec<ParsedTransaction>, ParserError> {
    // Extract both start and end dates from the header
    let date_range_regex =
        Regex::new(r"DESDE:\s*(\d{4})/(\d{2})/\d{2}\s+HASTA:\s*(\d{4})/(\d{2})/\d{2}")
            .map_err(|e| ParserError::ParseTransaction(e.to_string()))?;

    let (start_year, start_month, end_year, _end_month) = date_range_regex
        .captures(text)
        .map(|cap| {
            (
                cap[1].parse::<i32>().unwrap_or(2025),
                cap[2].parse::<u32>().unwrap_or(1),
                cap[3].parse::<i32>().unwrap_or(2025),
                cap[4].parse::<u32>().unwrap_or(12),
            )
        })
        .ok_or_else(|| {
            ParserError::ParseDate("Could not extract date range from header".to_string())
        })?;

    debug!(
        "Extracted date range from Bancolombia statement: {}/{} to {}/{}",
        start_year, start_month, end_year, _end_month
    );

    // First try single-line format (page 1 style)
    let mut transactions = parse_single_line_format(text, start_year, start_month, end_year)?;

    // Then parse columnar format (page 2+ style)
    let columnar_transactions = parse_columnar_format(text, start_year, start_month, end_year)?;
    transactions.extend(columnar_transactions);

    // Sort by date
    transactions.sort_by(|a, b| a.date.cmp(&b.date));

    // Assign sequence numbers
    for (i, tx) in transactions.iter_mut().enumerate() {
        tx.sequence = i as i32;
    }

    debug!(
        "Parsed {} transactions from Bancolombia statement",
        transactions.len()
    );

    Ok(transactions)
}

/// Parse single-line format transactions.
fn parse_single_line_format(
    text: &str,
    start_year: i32,
    start_month: u32,
    end_year: i32,
) -> Result<Vec<ParsedTransaction>, ParserError> {
    let tx_regex =
        Regex::new(r"(?m)^(\d{1,2})/(\d{1,2})\s+(.+?)\s+(-?[\d,]+\.\d{2})\s+([\d,]+\.\d{2})$")
            .map_err(|e| ParserError::ParseTransaction(e.to_string()))?;

    let mut transactions = Vec::new();

    for cap in tx_regex.captures_iter(text) {
        let day: u32 = cap[1].parse().unwrap_or(1);
        let month: u32 = cap[2].parse().unwrap_or(1);
        let description = cap[3].trim().to_string();
        let amount_str = cap[4].replace(',', "");
        let balance_str = cap[5].replace(',', "");

        let year = determine_year(month, start_year, start_month, end_year);

        let amount = Decimal::from_str(&amount_str).unwrap_or(Decimal::ZERO);
        let balance = Decimal::from_str(&balance_str).unwrap_or(Decimal::ZERO);

        if let Some(date) = NaiveDate::from_ymd_opt(year, month, day) {
            transactions.push(ParsedTransaction {
                date,
                description,
                amount,
                balance: Some(balance),
                reference: None,
                is_income: amount > Decimal::ZERO,
                sequence: 0,
            });
        }
    }

    Ok(transactions)
}

/// Parse columnar format transactions.
/// The PDF extracts as columns: dates, then descriptions, then values, then balances.
fn parse_columnar_format(
    text: &str,
    start_year: i32,
    start_month: u32,
    end_year: i32,
) -> Result<Vec<ParsedTransaction>, ParserError> {
    // Split text into pages (by PÁGINA: marker)
    let pages: Vec<&str> = text.split("PÁGINA:").collect();

    let mut all_transactions = Vec::new();

    for page_text in pages.iter().skip(1) {
        // Skip page 1 which uses single-line format
        if page_text.contains("PÁGINA: 1") || page_text.starts_with(" 1\n") {
            continue;
        }

        let transactions = parse_columnar_page(page_text, start_year, start_month, end_year)?;
        all_transactions.extend(transactions);
    }

    Ok(all_transactions)
}

/// Parse a single page in columnar format.
fn parse_columnar_page(
    page_text: &str,
    start_year: i32,
    start_month: u32,
    end_year: i32,
) -> Result<Vec<ParsedTransaction>, ParserError> {
    let lines: Vec<&str> = page_text.lines().collect();

    // Find the header line index
    let header_idx = lines
        .iter()
        .position(|l| l.contains("FECHA") && l.contains("DESCRIPCIÓN"));
    let start_idx = header_idx.map(|i| i + 1).unwrap_or(0);

    // Collect dates, descriptions, amounts, and balances
    let mut dates: Vec<(u32, u32)> = Vec::new(); // (day, month)
    let mut descriptions: Vec<String> = Vec::new();
    let mut amounts: Vec<Decimal> = Vec::new();
    let mut balances: Vec<Decimal> = Vec::new();

    let date_regex = Regex::new(r"^(\d{1,2})/(\d{1,2})$").unwrap();
    let amount_regex = Regex::new(r"^-?[\d,]*\.?\d+$").unwrap();

    let mut in_dates = true;
    let mut in_descriptions = false;
    let mut in_amounts = false;

    for line in lines.iter().skip(start_idx) {
        let trimmed = line.trim();

        if trimmed.is_empty() {
            continue;
        }

        // Detect date lines (DD/MM format)
        if let Some(caps) = date_regex.captures(trimmed) {
            if in_dates {
                let day: u32 = caps[1].parse().unwrap_or(1);
                let month: u32 = caps[2].parse().unwrap_or(1);
                dates.push((day, month));
                continue;
            }
        }

        // After dates come descriptions (text that's not a number)
        if in_dates && !date_regex.is_match(trimmed) && !amount_regex.is_match(trimmed) {
            in_dates = false;
            in_descriptions = true;
        }

        if in_descriptions {
            // Check if this looks like an amount (starts transition to amounts)
            let clean = trimmed.replace([',', '.'], "");
            if amount_regex.is_match(trimmed)
                || clean.chars().all(|c| c.is_ascii_digit() || c == '-')
            {
                // This might be an amount - check if it has decimal
                if trimmed.contains('.') || trimmed.parse::<i64>().is_ok() {
                    in_descriptions = false;
                    in_amounts = true;
                } else {
                    descriptions.push(trimmed.to_string());
                    continue;
                }
            } else {
                descriptions.push(trimmed.to_string());
                continue;
            }
        }

        if in_amounts {
            let clean = trimmed.replace(',', "");
            if let Ok(amount) = Decimal::from_str(&clean) {
                if amounts.len() < dates.len() {
                    amounts.push(amount);
                } else {
                    balances.push(amount);
                }
            }
        }
    }

    // Match them up
    let count = dates.len().min(descriptions.len()).min(amounts.len());
    let mut transactions = Vec::new();

    for i in 0..count {
        let (day, month) = dates[i];
        let year = determine_year(month, start_year, start_month, end_year);

        if let Some(date) = NaiveDate::from_ymd_opt(year, month, day) {
            let amount = amounts[i];
            let balance = balances.get(i).copied();

            transactions.push(ParsedTransaction {
                date,
                description: descriptions[i].clone(),
                amount,
                balance,
                reference: None,
                is_income: amount > Decimal::ZERO,
                sequence: 0,
            });
        }
    }

    Ok(transactions)
}

/// Determine the year based on month and statement date range.
fn determine_year(month: u32, start_year: i32, start_month: u32, end_year: i32) -> i32 {
    if start_year == end_year || month >= start_month {
        start_year
    } else {
        end_year
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_can_parse_bancolombia() {
        let parser = BancolombiaParser;
        assert!(parser.can_parse("ESTADO DE CUENTA\nCUENTA DE AHORROS\nBANCOLOMBIA"));
        assert!(parser.can_parse("ESTADO DE CUENTA\nSUCURSAL UNICENTRO"));
        assert!(!parser.can_parse("Nequi statement"));
    }

    #[test]
    fn test_parse_bancolombia_single_line() {
        let text = r#"
ESTADO DE CUENTA
DESDE: 2025/09/30 HASTA: 2025/12/31
CUENTA DE AHORROS

FECHA DESCRIPCIÓN SUCURSAL DCTO. VALOR SALDO
1/10 PAGO INTERBANC DLOCAL COLOMBIA 6,871,029.80 7,525,710.84
1/10 TRANSFERENCIA DESDE NEQUI 100,000.00 7,625,710.84
1/10 TRANSFERENCIAS A NEQUI -28,000.00 7,597,710.84
3/10 COMPRA EN NETFLIX -44,900.00 4,265,688.31
"#;
        let parser = BancolombiaParser;
        let transactions = parser.parse(text).unwrap();

        assert_eq!(transactions.len(), 4);

        assert_eq!(
            transactions[0].date,
            NaiveDate::from_ymd_opt(2025, 10, 1).unwrap()
        );
        assert_eq!(
            transactions[0].description,
            "PAGO INTERBANC DLOCAL COLOMBIA"
        );
        assert_eq!(
            transactions[0].amount,
            Decimal::from_str("6871029.80").unwrap()
        );
        assert!(transactions[0].is_income);
    }

    #[test]
    fn test_parse_year_rollover() {
        let text = r#"
ESTADO DE CUENTA
DESDE: 2025/11/30 HASTA: 2026/01/31

FECHA DESCRIPCIÓN VALOR SALDO
15/12 TRANSACTION IN DECEMBER 100,000.00 500,000.00
5/1 TRANSACTION IN JANUARY 50,000.00 550,000.00
"#;
        let parser = BancolombiaParser;
        let transactions = parser.parse(text).unwrap();

        assert_eq!(transactions.len(), 2);
        assert_eq!(
            transactions[0].date,
            NaiveDate::from_ymd_opt(2025, 12, 15).unwrap()
        );
        assert_eq!(
            transactions[1].date,
            NaiveDate::from_ymd_opt(2026, 1, 5).unwrap()
        );
    }
}
