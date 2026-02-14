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
pub mod model;
pub mod volume_tree;

// Re-export key functions
pub use power::{eject_drive, power_off_drive, standby_drive, wakeup_drive, remove_drive};
pub use format::format_disk;
pub use image::{open_for_backup, open_for_restore};
