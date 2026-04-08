//! Bank statement parser library for Gasticos.
//!
//! This crate provides PDF parsing capabilities for Colombian bank statements.
//! Supported banks: Bancolombia, Nequi, Nu Colombia.

pub mod banks;

use chrono::NaiveDate;
use rust_decimal::Decimal;
use thiserror::Error;

/// Errors that can occur during parsing.
#[derive(Debug, Error)]
pub enum ParserError {
    #[error("Failed to read PDF: {0}")]
    PdfRead(String),

    #[error("Invalid password for PDF")]
    InvalidPassword,

    #[error("Unsupported bank or format")]
    UnsupportedFormat,

    #[error("Failed to parse transaction: {0}")]
    ParseTransaction(String),

    #[error("Failed to parse date: {0}")]
    ParseDate(String),

    #[error("Failed to parse amount: {0}")]
    ParseAmount(String),

    #[error("No transactions found in statement")]
    NoTransactionsFound,
}

/// A parsed transaction from a bank statement.
#[derive(Debug, Clone, PartialEq)]
pub struct ParsedTransaction {
    /// Transaction date
    pub date: NaiveDate,

    /// Transaction description (merchant, reference, etc.)
    pub description: String,

    /// Transaction amount (positive for income, negative for expenses)
    pub amount: Decimal,

    /// Balance after transaction (if available)
    pub balance: Option<Decimal>,

    /// Reference number (if available)
    pub reference: Option<String>,

    /// Whether this is an income transaction
    pub is_income: bool,

    /// Sequence number within the statement (0-indexed)
    /// Used to distinguish identical transactions in the same file
    pub sequence: i32,
}

/// File type for parsing.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FileType {
    Pdf,
    Csv,
    Excel,
}

/// Trait for bank-specific parsers.
pub trait BankParser: Send + Sync {
    /// Returns the bank name.
    fn bank_name(&self) -> &str;

    /// Check if this parser can handle the given content.
    fn can_parse(&self, text: &str) -> bool;

    /// Parse the extracted text into transactions.
    fn parse(&self, text: &str) -> Result<Vec<ParsedTransaction>, ParserError>;
}

/// Extract text from a password-protected PDF.
///
/// Uses qpdf for decryption (more reliable with various encryption types),
/// then pdf-extract for text extraction.
pub fn extract_pdf_text(content: &[u8], password: &str) -> Result<String, ParserError> {
    use std::io::Write;
    use std::process::Command;
    use std::time::{SystemTime, UNIX_EPOCH};

    // Create unique temp files for qpdf (using PID + timestamp for uniqueness)
    let temp_dir = std::env::temp_dir();
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let unique_id = format!("{}_{}", std::process::id(), timestamp);
    let input_path = temp_dir.join(format!("gasticos_input_{}.pdf", unique_id));
    let output_path = temp_dir.join(format!("gasticos_output_{}.pdf", unique_id));

    // Write input PDF to temp file
    let mut input_file = std::fs::File::create(&input_path)
        .map_err(|e| ParserError::PdfRead(format!("Failed to create temp file: {}", e)))?;
    input_file
        .write_all(content)
        .map_err(|e| ParserError::PdfRead(format!("Failed to write temp file: {}", e)))?;
    drop(input_file);

    tracing::debug!(
        "Attempting to decrypt PDF with qpdf, password length: {}",
        password.len()
    );

    // Use qpdf to decrypt the PDF
    let qpdf_result = Command::new("qpdf")
        .args([
            &format!("--password={}", password),
            "--decrypt",
            input_path.to_str().unwrap(),
            output_path.to_str().unwrap(),
        ])
        .output();

    // Clean up input file
    let _ = std::fs::remove_file(&input_path);

    let output = match qpdf_result {
        Ok(output) => output,
        Err(e) => {
            // qpdf not found, try fallback with lopdf
            tracing::warn!("qpdf not found ({}), trying lopdf fallback", e);
            return extract_pdf_text_lopdf(content, password);
        }
    };

    tracing::debug!("qpdf exit code: {:?}", output.status.code());
    tracing::debug!("qpdf stderr: {}", String::from_utf8_lossy(&output.stderr));

    // qpdf exit codes:
    // 0 = success
    // 2 = errors (file not found, invalid password, etc.)
    // 3 = warnings (file is damaged but recoverable)
    let exit_code = output.status.code().unwrap_or(1);
    let stderr = String::from_utf8_lossy(&output.stderr);

    if exit_code == 2 {
        let _ = std::fs::remove_file(&output_path);

        if stderr.contains("invalid password") {
            return Err(ParserError::InvalidPassword);
        }
        return Err(ParserError::PdfRead(format!("qpdf failed: {}", stderr)));
    }

    // Exit code 0 or 3 (warnings) - proceed with output file
    if !output_path.exists() {
        return Err(ParserError::PdfRead(
            "qpdf did not produce output file".to_string(),
        ));
    }

    // Read decrypted PDF
    let decrypted = std::fs::read(&output_path)
        .map_err(|e| ParserError::PdfRead(format!("Failed to read decrypted PDF: {}", e)))?;

    // Clean up output file
    let _ = std::fs::remove_file(&output_path);

    // Extract text using pdf-extract
    pdf_extract::extract_text_from_mem(&decrypted)
        .map_err(|e| ParserError::PdfRead(format!("Failed to extract text: {}", e)))
}

/// Fallback text extraction using lopdf (for when qpdf is not available).
fn extract_pdf_text_lopdf(content: &[u8], password: &str) -> Result<String, ParserError> {
    use lopdf::Document;
    use std::io::Cursor;

    let cursor = Cursor::new(content);
    let mut doc = Document::load_from(cursor)
        .map_err(|e| ParserError::PdfRead(format!("Failed to load PDF: {}", e)))?;

    if doc.is_encrypted() {
        doc.decrypt(password)
            .map_err(|_| ParserError::InvalidPassword)?;
    }

    let mut decrypted = Vec::new();
    doc.save_to(&mut decrypted)
        .map_err(|e| ParserError::PdfRead(format!("Failed to save decrypted PDF: {}", e)))?;

    pdf_extract::extract_text_from_mem(&decrypted)
        .map_err(|e| ParserError::PdfRead(format!("Failed to extract text: {}", e)))
}

/// Parse a bank statement PDF.
///
/// This function extracts text from the PDF and tries to identify the bank,
/// then uses the appropriate parser to extract transactions.
pub fn parse_statement(
    content: &[u8],
    password: &str,
) -> Result<(String, Vec<ParsedTransaction>), ParserError> {
    let text = extract_pdf_text(content, password)?;

    // Try each bank parser
    let parsers: Vec<Box<dyn BankParser>> = vec![
        Box::new(banks::BancolombiaParser),
        Box::new(banks::NequiParser),
        Box::new(banks::NuParser),
    ];

    for parser in parsers {
        if parser.can_parse(&text) {
            let transactions = parser.parse(&text)?;
            if transactions.is_empty() {
                return Err(ParserError::NoTransactionsFound);
            }
            return Ok((parser.bank_name().to_string(), transactions));
        }
    }

    Err(ParserError::UnsupportedFormat)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parser_error_display() {
        let err = ParserError::InvalidPassword;
        assert_eq!(err.to_string(), "Invalid password for PDF");
    }

    #[test]
    fn test_file_type_enum() {
        assert_eq!(FileType::Pdf, FileType::Pdf);
        assert_ne!(FileType::Pdf, FileType::Csv);
    }
}
