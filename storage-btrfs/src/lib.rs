// SPDX-License-Identifier: GPL-3.0-only

//! BTRFS operations library for COSMIC Disks
//!
//! This library provides a safe Rust interface for BTRFS subvolume management
//! operations including creation, deletion, snapshots, and metadata queries.

pub mod error;
pub mod subvolume;
pub mod usage;

// Re-export commonly used types
pub use error::{BtrfsError, Result};
pub use subvolume::SubvolumeManager;
pub use usage::get_filesystem_usage;

// Re-export shared models
pub use storage_common::btrfs::*;

// Re-export btrfsutil types for convenience
pub use btrfsutil;
