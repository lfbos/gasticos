//! Background job handlers for Gasticos.
//!
//! This module contains job processors for various background tasks:
//! - `belvo_sync`: Sync transactions from Belvo Open Banking
//! - `categorize`: Auto-categorize transactions using Colombian merchant dictionary

pub mod belvo_sync;
pub mod categorize;

pub use belvo_sync::{process_belvo_sync_job, BelvoSyncJobPayload};
pub use categorize::{process_categorize_job, CategorizeJobPayload};
