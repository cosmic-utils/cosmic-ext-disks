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
}

/// Result type alias for system operations
pub type Result<T> = std::result::Result<T, SysError>;
