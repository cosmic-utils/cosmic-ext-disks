use std::path::PathBuf;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum TestingError {
    #[error("runtime missing: install podman or docker")]
    RuntimeMissing,
    #[error("privilege required: run with sudo/root for host storage actions")]
    PrivilegeRequired,
    #[error("spec not found for '{spec_name}' in resources/lab-specs")]
    SpecNotFound { spec_name: String },
    #[error("invalid spec '{spec_name}': {reason}")]
    SpecInvalid { spec_name: String, reason: String },
    #[error("command failed: {command}; stderr: {stderr}")]
    CommandFailed { command: String, stderr: String },
    #[error("ledger io error for {path:?}: {reason}")]
    LedgerIo { path: PathBuf, reason: String },
    #[error("container runtime failure: {reason}")]
    ContainerRuntimeFailed { reason: String },
    #[error("service startup failed: {reason}")]
    ServiceStartupFailed { reason: String },
}

pub type Result<T> = std::result::Result<T, TestingError>;
