// SPDX-License-Identifier: GPL-3.0-only

//! Data models for RClone mount management
//!
//! This module defines the types used for RClone configuration and mount state
//! across the storage-service, storage-sys, and storage-app crates.

mod mount;
mod provider_catalog;
mod remote;
mod results;
mod scope;

pub use mount::{MountStatus, MountType};
pub use provider_catalog::{
    RcloneProvider, RcloneProviderOption, RcloneProviderOptionExample, rclone_provider,
    rclone_providers, supported_remote_types,
};
pub use remote::{NetworkMount, RemoteConfig, RemoteConfigList};
pub use results::{MountStatusResult, TestResult};
pub use scope::ConfigScope;
