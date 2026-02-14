//! Filesystem operations
//!
//! This module provides operations for managing filesystems:
//! - Formatting filesystems
//! - Mounting and unmounting
//! - Checking and repairing
//! - Label management
//! - Ownership management

mod format;
mod mount;
mod check;
mod label;
mod ownership;
pub mod config;

// Re-export settings type and mount options config
pub use config::{get_mount_options, set_mount_options, reset_mount_options, MountOptionsSettings};

pub use format::format_filesystem;
pub use mount::{mount_filesystem, unmount_filesystem, get_mount_point};
pub use check::{check_filesystem, repair_filesystem};
pub use label::{get_filesystem_label, set_filesystem_label};
pub use ownership::take_filesystem_ownership;

