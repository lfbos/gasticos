//! Background job handlers for Gasticos.
//!
//! This module contains job processors for various background tasks:
//! - `belvo_sync`: Sync transactions from Belvo Open Banking

pub mod belvo_sync;

pub use belvo_sync::{process_belvo_sync_job, BelvoSyncJobPayload};
