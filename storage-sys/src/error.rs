// SPDX-License-Identifier: GPL-3.0-only

use thiserror::Error;

/// Error types for system-level operations
#[derive(Error, Debug)]
pub enum SysError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    #[error("Device not found: {0}")]
    DeviceNotFound(String),

    #[error("Operation failed: {0}")]
    OperationFailed(String),

    // RClone-specific errors
    #[error("RClone binary not found")]
    RCloneNotFound,

    #[error("RClone configuration not found")]
    RCloneConfigNotFound,

    #[error("RClone configuration parse error: {0}")]
    RCloneConfigParse(String),

    #[error("RClone remote not found: {0}")]
    RCloneRemoteNotFound(String),

    #[error("RClone mount failed: {0}")]
    RCloneMountFailed(String),

    #[error("RClone unmount failed: {0}")]
    RCloneUnmountFailed(String),

    #[error("RClone test failed: {0}")]
    RCloneTestFailed(String),

    #[error("RClone remote already mounted: {0}")]
    RCloneAlreadyMounted(String),

    #[error("RClone remote not mounted: {0}")]
    RCloneNotMounted(String),

    #[error("Mount point already exists: {0}")]
    MountPointExists(String),

    #[error("Mount point does not exist: {0}")]
    MountPointNotFound(String),
}

/// Result type alias for system operations
pub type Result<T> = std::result::Result<T, SysError>;
