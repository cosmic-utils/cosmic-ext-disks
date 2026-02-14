//! Image and backup operations
//!
//! This module provides operations for working with disk images:
//! - Loop device setup
//! - Image mounting
//! - Backup and restore operations

pub mod loop_setup;
pub mod backup;
pub(crate) mod udisks_call;

pub use loop_setup::*;
pub use backup::*;
