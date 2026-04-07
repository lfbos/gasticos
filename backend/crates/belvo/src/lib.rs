//! Belvo Open Banking API client for Gasticos.
//!
//! This crate provides a Rust client for interacting with the Belvo API
//! to fetch bank account data and transactions from Colombian banks.

pub mod accounts;
pub mod client;
pub mod error;
pub mod institutions;
pub mod links;
pub mod models;
pub mod transactions;
pub mod webhook;
pub mod widget;

pub use client::BelvoClient;
pub use error::{BelvoError, Result};
pub use models::*;
