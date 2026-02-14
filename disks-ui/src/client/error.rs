// SPDX-License-Identifier: GPL-3.0-only

use thiserror::Error;

/// Errors that can occur when calling the storage service via D-Bus
#[derive(Error, Debug, Clone)]
pub enum ClientError {
    #[error("D-Bus connection error: {0}")]
    Connection(String),
    
    #[error("D-Bus method call error: {0}")]
    MethodCall(String),
    
    #[error("Service not available (is storage-service running?)")]
    ServiceNotAvailable,
    
    #[error("Permission denied: {0}")]
    PermissionDenied(String),
    
    #[error("Operation failed: {0}")]
    OperationFailed(String),
    
    #[error("Invalid argument: {0}")]
    InvalidArgument(String),
    
    #[error("Parse error: {0}")]
    ParseError(String),
}

impl From<zbus::Error> for ClientError {
    fn from(err: zbus::Error) -> Self {
        match &err {
            zbus::Error::FDO(fdo_err) => {
                // Use Display trait to convert error to string
                let error_str = fdo_err.to_string();
                
                // Check error string for specific patterns
                if error_str.contains("AccessDenied") || error_str.contains("Access denied") {
                    ClientError::PermissionDenied(error_str)
                } else if error_str.contains("ServiceUnknown") || error_str.contains("Service not known") {
                    ClientError::ServiceNotAvailable
                } else if error_str.contains("InvalidArgs") || error_str.contains("Invalid arguments") {
                    ClientError::InvalidArgument(error_str)
                } else if error_str.contains("Failed") || error_str.contains("Operation failed") {
                    ClientError::OperationFailed(error_str)
                } else {
                    ClientError::MethodCall(error_str)
                }
            },
            _ => ClientError::Connection(err.to_string()),
        }
    }
}

impl From<serde_json::Error> for ClientError {
    fn from(err: serde_json::Error) -> Self {
        ClientError::ParseError(err.to_string())
    }
}
