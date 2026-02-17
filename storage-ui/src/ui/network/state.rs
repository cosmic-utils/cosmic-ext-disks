// SPDX-License-Identifier: GPL-3.0-only

//! State for network mount management

use std::collections::HashMap;
use storage_common::rclone::{ConfigScope, MountStatus, RemoteConfig};

/// Runtime state of a network mount
#[derive(Debug, Clone)]
pub struct NetworkMountState {
    /// The remote configuration
    pub config: RemoteConfig,
    /// Current mount status
    pub status: MountStatus,
    /// Whether an operation is in progress
    pub loading: bool,
    /// Last error message if any
    pub error: Option<String>,
}

impl NetworkMountState {
    pub fn new(config: RemoteConfig) -> Self {
        Self {
            config,
            status: MountStatus::Unmounted,
            loading: false,
            error: None,
        }
    }

    /// Check if this mount is currently mounted
    pub fn is_mounted(&self) -> bool {
        self.status.is_mounted()
    }
}

/// State for the network section of the sidebar
#[derive(Debug, Default)]
pub struct NetworkState {
    /// All configured remotes with their state
    pub mounts: HashMap<(String, ConfigScope), NetworkMountState>,

    /// Currently selected remote (name, scope)
    pub selected: Option<(String, ConfigScope)>,

    /// Whether remotes are being loaded
    pub loading: bool,

    /// Whether RClone is available
    pub rclone_available: bool,

    /// Last error message
    pub error: Option<String>,
}

impl NetworkState {
    /// Create new network state
    pub fn new() -> Self {
        Self::default()
    }

    /// Set remotes from loaded configuration
    pub fn set_remotes(&mut self, remotes: Vec<RemoteConfig>) {
        // Preserve existing mount status where possible
        let old_mounts = std::mem::take(&mut self.mounts);

        for config in remotes {
            let key = (config.name.clone(), config.scope);
            let state = if let Some(existing) = old_mounts.get(&key) {
                // Preserve status and loading state
                NetworkMountState {
                    config,
                    status: existing.status.clone(),
                    loading: existing.loading,
                    error: None,
                }
            } else {
                NetworkMountState::new(config)
            };
            self.mounts.insert(key, state);
        }
    }

    /// Get a mount by name and scope
    pub fn get_mount(&self, name: &str, scope: ConfigScope) -> Option<&NetworkMountState> {
        self.mounts.get(&(name.to_string(), scope))
    }

    /// Get a mutable mount by name and scope
    pub fn get_mount_mut(
        &mut self,
        name: &str,
        scope: ConfigScope,
    ) -> Option<&mut NetworkMountState> {
        self.mounts.get_mut(&(name.to_string(), scope))
    }

    /// Update mount status
    pub fn set_mount_status(&mut self, name: &str, scope: ConfigScope, status: MountStatus) {
        if let Some(mount) = self.mounts.get_mut(&(name.to_string(), scope)) {
            mount.status = status;
            mount.loading = false;
        }
    }

    /// Set loading state for a mount
    pub fn set_loading(&mut self, name: &str, scope: ConfigScope, loading: bool) {
        if let Some(mount) = self.mounts.get_mut(&(name.to_string(), scope)) {
            mount.loading = loading;
        }
    }

    /// Set error for a mount
    pub fn set_error(&mut self, name: &str, scope: ConfigScope, error: Option<String>) {
        if let Some(mount) = self.mounts.get_mut(&(name.to_string(), scope)) {
            mount.error = error;
        }
    }

    /// Check if a remote is selected
    pub fn is_selected(&self, name: &str, scope: ConfigScope) -> bool {
        self.selected == Some((name.to_string(), scope))
    }

    /// Select a remote
    pub fn select(&mut self, name: Option<String>, scope: Option<ConfigScope>) {
        self.selected = name.zip(scope);
    }

    /// Get list of remotes sorted by name
    pub fn sorted_mounts(&self) -> Vec<&NetworkMountState> {
        let mut mounts: Vec<_> = self.mounts.values().collect();
        mounts.sort_by(|a, b| {
            // Sort by scope first (User before System), then by name
            match (a.config.scope, b.config.scope) {
                (ConfigScope::User, ConfigScope::System) => std::cmp::Ordering::Less,
                (ConfigScope::System, ConfigScope::User) => std::cmp::Ordering::Greater,
                _ => a.config.name.cmp(&b.config.name),
            }
        });
        mounts
    }

    /// Get user-scope remotes only
    pub fn user_mounts(&self) -> Vec<&NetworkMountState> {
        let mut mounts: Vec<_> = self
            .mounts
            .values()
            .filter(|m| m.config.scope == ConfigScope::User)
            .collect();
        mounts.sort_by(|a, b| a.config.name.cmp(&b.config.name));
        mounts
    }

    /// Get system-scope remotes only
    pub fn system_mounts(&self) -> Vec<&NetworkMountState> {
        let mut mounts: Vec<_> = self
            .mounts
            .values()
            .filter(|m| m.config.scope == ConfigScope::System)
            .collect();
        mounts.sort_by(|a, b| a.config.name.cmp(&b.config.name));
        mounts
    }
}
