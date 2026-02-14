//! Disk-level operations
//! 
//! This module provides operations that work at the disk/drive level:
//! - Discovery and enumeration
//! - Formatting (creating partition tables)
//! - Power management (eject, standby, etc.)

pub mod discovery;
pub mod power;
pub mod format;
pub mod image;
pub(crate) mod model;
pub(crate) mod volume_tree;

// Re-export key functions and types
// DriveModel is now internal only
pub use discovery::{get_disks, get_disks_with_volumes, get_disks_with_partitions};
pub use power::{eject_drive, power_off_drive, standby_drive, wakeup_drive, remove_drive};
pub use format::format_disk;
pub use image::{open_for_backup, open_for_restore};
