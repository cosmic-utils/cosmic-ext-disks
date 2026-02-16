// SPDX-License-Identifier: GPL-3.0-only

//! Canonical domain models for COSMIC storage management
//!
//! This crate defines the single source of truth for all storage domain types.
//! These models are used throughout the stack:
//!
//! - **storage-dbus**: Returns these types directly from its public API
//! - **storage-service**: Serializes/deserializes these types for D-Bus transport
//! - **storage-ui**: Consumes these types, optionally wrapping them for UI state
//!
//! ## Architecture
//!
//! The type system supports two hierarchies:
//!
//! ### Flat Hierarchy (for operations)
//! - `DiskInfo` → physical disk metadata
//! - `PartitionInfo` → partition metadata
//! - `FilesystemInfo` → filesystem details
//!
//! ### Tree Hierarchy (for UI display)
//! - `VolumeInfo` → recursive tree structure containing any `VolumeKind`
//!
//! This eliminates circular conversions and ensures data consistency across all components.

pub mod btrfs;
pub mod caller;
pub mod common;
pub mod disk;
pub mod encryption;
pub mod filesystem;
pub mod lvm;
pub mod partition;
pub mod partition_types;
pub mod smart;
pub mod volume;

// Re-export all public types
pub use btrfs::*;
pub use caller::*;
pub use common::*;
pub use disk::*;
pub use encryption::*;
pub use filesystem::*;
pub use lvm::*;
pub use partition::*;
pub use partition_types::*;
pub use smart::*;
pub use volume::*;
