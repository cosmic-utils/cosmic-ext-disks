//! LUKS encryption operations
//!
//! This module provides operations for managing LUKS encrypted devices:
//! - Formatting LUKS devices
//! - Unlocking and locking
//! - Passphrase management
//! - Listing encrypted devices

mod format;
mod unlock;
mod lock;
mod passphrase;
mod list;
pub mod config;

pub use format::format_luks;
pub use unlock::unlock_luks;
pub use lock::lock_luks;
pub use passphrase::change_luks_passphrase;
pub use list::list_luks_devices;

