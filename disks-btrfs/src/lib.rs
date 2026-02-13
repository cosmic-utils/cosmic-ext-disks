// SPDX-License-Identifier: GPL-3.0-only

//! BTRFS operations library for COSMIC Disks
//! 
//! This library provides a safe Rust interface for BTRFS subvolume management
//! operations including creation, deletion, snapshots, and metadata queries.

pub mod error;
pub mod types;
pub mod subvolume;
pub mod usage;

// Re-export commonly used types
pub use error::{BtrfsError, Result};
pub use types::{BtrfsSubvolume, FilesystemUsage, SubvolumeList};
pub use subvolume::SubvolumeManager;
pub use usage::get_filesystem_usage;

// Re-export btrfsutil types for convenience
pub use btrfsutil;
