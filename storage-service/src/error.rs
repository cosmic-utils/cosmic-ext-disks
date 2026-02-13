// SPDX-License-Identifier: GPL-3.0-only

use thiserror::Error;
use zbus::fdo;

/// Service-specific errors
#[derive(Debug, Error)]
pub enum ServiceError {
    #[error("BTRFS error: {0}")]
    Btrfs(#[from] disks_btrfs::BtrfsError),
    
    #[error("Authorization failed: {0}")]
    AuthorizationFailed(String),
    
    #[error("D-Bus error: {0}")]
    DBus(String),
    
    #[error("Invalid argument: {0}")]
    InvalidArgument(String),
    
    #[error("Operation failed: {0}")]
    OperationFailed(String),
}

impl From<ServiceError> for fdo::Error {
    fn from(err: ServiceError) -> Self {
        match err {
            ServiceError::AuthorizationFailed(msg) => {
                fdo::Error::AccessDenied(msg)
            }
            ServiceError::InvalidArgument(msg) => {
                fdo::Error::InvalidArgs(msg)
            }
            _ => fdo::Error::Failed(err.to_string()),
        }
    }
}

impl From<zbus::Error> for ServiceError {
    fn from(err: zbus::Error) -> Self {
        ServiceError::DBus(err.to_string())
    }
}

pub type Result<T> = std::result::Result<T, ServiceError>;
