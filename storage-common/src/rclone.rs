// SPDX-License-Identifier: GPL-3.0-only

//! Data models for RClone mount management
//!
//! This module defines the types used for RClone configuration and mount state
//! across the storage-service, storage-sys, and storage-ui crates.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Defines whether a configuration is per-user or system-wide
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ConfigScope {
    /// Per-user configuration (~/.config/rclone/rclone.conf)
    User,
    /// System-wide configuration (/etc/rclone.conf)
    System,
}

impl ConfigScope {
    /// Get the config file path for this scope
    pub fn config_path(&self) -> PathBuf {
        match self {
            ConfigScope::User => {
                if let Some(home) = std::env::var_os("HOME") {
                    PathBuf::from(home).join(".config/rclone/rclone.conf")
                } else {
                    PathBuf::from("~/.config/rclone/rclone.conf")
                }
            }
            ConfigScope::System => PathBuf::from("/etc/rclone.conf"),
        }
    }

    /// Get the mount point prefix for this scope
    pub fn mount_prefix(&self) -> PathBuf {
        match self {
            ConfigScope::User => {
                if let Some(home) = std::env::var_os("HOME") {
                    PathBuf::from(home).join("mnt")
                } else {
                    PathBuf::from("~/mnt")
                }
            }
            ConfigScope::System => PathBuf::from("/mnt/rclone"),
        }
    }

    /// Get the full mount point path for a remote
    pub fn mount_point(&self, remote_name: &str) -> PathBuf {
        self.mount_prefix().join(remote_name)
    }
}

impl std::fmt::Display for ConfigScope {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigScope::User => write!(f, "user"),
            ConfigScope::System => write!(f, "system"),
        }
    }
}

impl std::str::FromStr for ConfigScope {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "user" => Ok(ConfigScope::User),
            "system" => Ok(ConfigScope::System),
            _ => Err(format!("Invalid scope: {}", s)),
        }
    }
}

/// Runtime state of a network mount
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "lowercase")]
pub enum MountStatus {
    /// Not currently mounted
    Unmounted,
    /// Mount operation in progress
    Mounting,
    /// Successfully mounted and accessible
    Mounted,
    /// Unmount operation in progress
    Unmounting,
    /// Error state with message
    Error(String),
}

impl Default for MountStatus {
    fn default() -> Self {
        MountStatus::Unmounted
    }
}

impl MountStatus {
    /// Check if the mount is in a terminal state (not transitioning)
    pub fn is_terminal(&self) -> bool {
        matches!(self, MountStatus::Unmounted | MountStatus::Mounted | MountStatus::Error(_))
    }

    /// Check if the mount is currently mounted
    pub fn is_mounted(&self) -> bool {
        matches!(self, MountStatus::Mounted)
    }
}

/// Type of network mount backend. Designed for future extensibility.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MountType {
    /// RClone mount
    RClone,
    // Future:
    // Samba,
    // Ftp,
}

impl Default for MountType {
    fn default() -> Self {
        MountType::RClone
    }
}

/// Configuration for a single RClone remote
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoteConfig {
    /// Unique name for this remote (e.g., "my-drive")
    pub name: String,

    /// Backend type (e.g., "drive", "s3", "ftp")
    pub remote_type: String,

    /// Configuration scope (user or system)
    pub scope: ConfigScope,

    /// Raw configuration key-value pairs from rclone.conf
    #[serde(default)]
    pub options: HashMap<String, String>,

    /// Whether sensitive fields (tokens, secrets) are present
    #[serde(default)]
    pub has_secrets: bool,
}

impl RemoteConfig {
    /// Create a new remote configuration
    pub fn new(name: String, remote_type: String, scope: ConfigScope) -> Self {
        Self {
            name,
            remote_type,
            scope,
            options: HashMap::new(),
            has_secrets: false,
        }
    }

    /// Get the mount point for this remote
    pub fn mount_point(&self) -> PathBuf {
        self.scope.mount_point(&self.name)
    }

    /// Validate the remote name
    pub fn validate_name(&self) -> Result<(), String> {
        if self.name.is_empty() {
            return Err("Remote name cannot be empty".to_string());
        }
        if !self
            .name
            .chars()
            .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
        {
            return Err(
                "Remote name must contain only alphanumeric characters, dashes, or underscores"
                    .to_string(),
            );
        }
        Ok(())
    }
}

/// Represents a mountable network storage resource with runtime state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkMount {
    /// Reference to the remote configuration name
    pub remote_name: String,

    /// Configuration scope
    pub scope: ConfigScope,

    /// Current mount status
    pub status: MountStatus,

    /// Mount point path (resolved from scope)
    pub mount_point: PathBuf,

    /// Mount type for extensibility
    pub mount_type: MountType,
}

impl NetworkMount {
    /// Create a new network mount from a remote config
    pub fn from_config(config: &RemoteConfig) -> Self {
        Self {
            remote_name: config.name.clone(),
            scope: config.scope,
            status: MountStatus::Unmounted,
            mount_point: config.mount_point(),
            mount_type: MountType::RClone,
        }
    }
}

/// Container for listing remotes with scope information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoteConfigList {
    /// List of remote configurations
    pub remotes: Vec<RemoteConfig>,

    /// Path to user config file if it exists
    pub user_config_path: Option<PathBuf>,

    /// Path to system config file if it exists
    pub system_config_path: Option<PathBuf>,
}

impl RemoteConfigList {
    /// Create an empty list
    pub fn empty() -> Self {
        Self {
            remotes: Vec::new(),
            user_config_path: None,
            system_config_path: None,
        }
    }
}

/// Result of a remote configuration test
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestResult {
    /// Whether the test succeeded
    pub success: bool,

    /// Human-readable message describing the result
    pub message: String,

    /// Latency in milliseconds (if applicable)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub latency_ms: Option<u64>,
}

impl TestResult {
    /// Create a success result
    pub fn success(message: impl Into<String>) -> Self {
        Self {
            success: true,
            message: message.into(),
            latency_ms: None,
        }
    }

    /// Create a failure result
    pub fn failure(message: impl Into<String>) -> Self {
        Self {
            success: false,
            message: message.into(),
            latency_ms: None,
        }
    }

    /// Add latency information
    pub fn with_latency(mut self, latency_ms: u64) -> Self {
        self.latency_ms = Some(latency_ms);
        self
    }
}

/// Result of getting mount status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MountStatusResult {
    /// Current mount status
    pub status: MountStatus,

    /// Mount point path
    pub mount_point: PathBuf,
}

impl MountStatusResult {
    /// Create a new mount status result
    pub fn new(status: MountStatus, mount_point: PathBuf) -> Self {
        Self { status, mount_point }
    }
}

/// List of supported RClone remote types
pub const SUPPORTED_REMOTE_TYPES: &[&str] = &[
    "drive",      // Google Drive
    "s3",         // Amazon S3
    "dropbox",    // Dropbox
    "onedrive",   // Microsoft OneDrive
    "ftp",        // FTP
    "sftp",       // SFTP
    "webdav",     // WebDAV
    "googlecloudstorage", // Google Cloud Storage
    "azureblob",  // Azure Blob Storage
    "b2",         // Backblaze B2
    "box",        // Box
    "pcloud",     // pCloud
    "yandex",     // Yandex Disk
    "alias",      // Alias for testing
];
