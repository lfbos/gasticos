//! Integration tests for PDF parsing.
//!
//! These tests use actual PDF files from the statements folder.

use parser::{extract_pdf_text, parse_statement};
use std::fs;

const BANCOLOMBIA_PASSWORD: &str = "307899709";
const NEQUI_PASSWORD: &str = "7899709";
const NU_PASSWORD: &str = "7899709";

fn get_statements_path() -> std::path::PathBuf {
    // Navigate from backend/crates/parser/tests to project root/statements
    let manifest_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest_dir
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("statements")
}

#[test]
fn test_parse_bancolombia_pdf() {
    let statements_path = get_statements_path();
    let pdf_path = statements_path.join("ExtractoBancolombia.pdf");

    if !pdf_path.exists() {
        eprintln!("Skipping test: PDF file not found at {:?}", pdf_path);
        return;
    }

    let content = fs::read(&pdf_path).expect("Failed to read PDF file");
    let (bank, transactions) =
        parse_statement(&content, BANCOLOMBIA_PASSWORD).expect("Failed to parse Bancolombia PDF");

    assert_eq!(bank, "Bancolombia");
    assert!(!transactions.is_empty(), "Should have parsed transactions");

    // Print first few transactions for verification
    println!("Parsed {} Bancolombia transactions:", transactions.len());
    for tx in transactions.iter().take(5) {
        println!(
            "  {} | {} | {} | {:?}",
            tx.date, tx.description, tx.amount, tx.is_income
        );
    }
}

#[test]
fn test_parse_nequi_pdf() {
    let statements_path = get_statements_path();
    let pdf_path = statements_path.join("ExtractoNequi.pdf");

    if !pdf_path.exists() {
        eprintln!("Skipping test: PDF file not found at {:?}", pdf_path);
        return;
    }

    let content = fs::read(&pdf_path).expect("Failed to read PDF file");
    let (bank, transactions) =
        parse_statement(&content, NEQUI_PASSWORD).expect("Failed to parse Nequi PDF");

    assert_eq!(bank, "Nequi");
    assert!(!transactions.is_empty(), "Should have parsed transactions");

    // Print first few transactions for verification
    println!("Parsed {} Nequi transactions:", transactions.len());
    for tx in transactions.iter().take(5) {
        println!(
            "  {} | {} | {} | {:?}",
            tx.date, tx.description, tx.amount, tx.is_income
        );
    }
}

#[test]
fn test_parse_nu_pdf() {
    let statements_path = get_statements_path();
    let pdf_path = statements_path.join("ExtractoNu.pdf");

    if !pdf_path.exists() {
        eprintln!("Skipping test: PDF file not found at {:?}", pdf_path);
        return;
    }

    let content = fs::read(&pdf_path).expect("Failed to read PDF file");
    let (bank, transactions) =
        parse_statement(&content, NU_PASSWORD).expect("Failed to parse Nu PDF");

    assert_eq!(bank, "Nu");
    assert!(!transactions.is_empty(), "Should have parsed transactions");

    // Print first few transactions for verification
    println!("Parsed {} Nu transactions:", transactions.len());
    for tx in transactions.iter().take(5) {
        println!(
            "  {} | {} | {} | {:?}",
            tx.date, tx.description, tx.amount, tx.is_income
        );
    }
}

#[test]
fn test_invalid_password() {
    let statements_path = get_statements_path();
    let pdf_path = statements_path.join("ExtractoBancolombia.pdf");

    if !pdf_path.exists() {
        eprintln!("Skipping test: PDF file not found at {:?}", pdf_path);
        return;
    }

    let content = fs::read(&pdf_path).expect("Failed to read PDF file");
    let result = parse_statement(&content, "wrong_password");

    assert!(result.is_err(), "Should fail with wrong password");
}

#[test]
fn test_extract_text_bancolombia() {
    let statements_path = get_statements_path();
    let pdf_path = statements_path.join("ExtractoBancolombia.pdf");

    if !pdf_path.exists() {
        eprintln!("Skipping test: PDF file not found at {:?}", pdf_path);
        return;
    }

    let content = fs::read(&pdf_path).expect("Failed to read PDF file");
    let text =
        extract_pdf_text(&content, BANCOLOMBIA_PASSWORD).expect("Failed to extract text from PDF");

    assert!(
        text.contains("BANCOLOMBIA")
            || text.contains("CUENTA DE AHORROS")
            || text.contains("ESTADO DE CUENTA")
    );
}
