// SPDX-License-Identifier: GPL-3.0-only

//! SMART types - re-exported from storage-models
//!
//! Domain types are defined in storage-models; this module re-exports them
//! and provides internal helpers for UDisks2-specific conversions.

// Re-export canonical domain types from storage-models
pub use storage_models::{SmartInfo, SmartSelfTestKind};
