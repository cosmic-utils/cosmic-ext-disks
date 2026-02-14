// Legacy disks module - provides types for backwards compatibility
// The implementation has been moved to new domain modules (disk/, partition/, filesystem/, etc.)

pub mod volume;

use thiserror::Error;

// Configuration settings types (still used by storage-models and consumers)
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct MountOptionsSettings {
    pub identify_as: String,
    pub mount_point: String,
    pub filesystem_type: String,
    pub mount_at_startup: bool,
    pub require_auth: bool,
    pub show_in_ui: bool,
    pub other_options: String,
    pub display_name: String,
    pub icon_name: String,
    pub symbolic_icon_name: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct EncryptionOptionsSettings {
    pub name: String,
    pub unlock_at_startup: bool,
    pub require_auth: bool,
    pub other_options: String,
}

#[derive(Error, Debug)]
pub enum DiskError {
    #[error("The model {0} is not connected")]
    NotConnected(String),

    #[error("Device is busy: {device} at {mount_point}")]
    ResourceBusy { device: String, mount_point: String },

    #[error("Connection failed: {0}")]
    ConnectionFailed(String),

    #[error("Device not found: {0}")]
    DeviceNotFound(String),

    #[error("Invalid path: {0}")]
    InvalidPath(String),

    #[error("D-Bus error: {0}")]
    DBusError(String),

    #[error("Operation failed: {0}")]
    OperationFailed(String),

    #[error("Zbus Error")]
    ZbusError(#[from] zbus::Error),
}
