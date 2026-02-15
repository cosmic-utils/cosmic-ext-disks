// SPDX-License-Identifier: GPL-3.0-only

//! SMART types - re-exported from storage-common
//!
//! Domain types are defined in storage-common; this module re-exports them
//! and provides internal helpers for UDisks2-specific conversions.

// Re-export canonical domain types from storage-common
pub use storage_common::{SmartInfo, SmartSelfTestKind};
