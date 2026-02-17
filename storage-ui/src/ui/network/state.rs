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

    /// Active editor state
    pub editor: Option<NetworkEditorState>,
}

impl NetworkState {
    /// Create new network state
    pub fn new() -> Self {
        Self::default()
    }

    /// Start editing a remote configuration
    pub fn start_edit(&mut self, config: RemoteConfig) {
        self.editor = Some(NetworkEditorState::from_config(config));
    }

    /// Start creating a new remote
    pub fn start_create(&mut self, default_type: String) {
        self.editor = Some(NetworkEditorState::new(default_type));
    }

    /// Close the editor
    pub fn clear_editor(&mut self) {
        self.editor = None;
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

#[derive(Debug, Clone)]
pub struct NetworkEditorState {
    pub name: String,
    pub remote_type: String,
    pub scope: ConfigScope,
    pub options: HashMap<String, String>,
    pub original_name: Option<String>,
    pub original_scope: Option<ConfigScope>,
    pub is_new: bool,
    pub running: bool,
    pub error: Option<String>,
    pub new_option_key: String,
    pub new_option_value: String,
    pub show_advanced: bool,
    pub show_hidden: bool,
    pub mount_on_boot: Option<bool>,
}

impl NetworkEditorState {
    pub fn new(default_type: String) -> Self {
        Self {
            name: String::new(),
            remote_type: default_type,
            scope: ConfigScope::User,
            options: HashMap::new(),
            original_name: None,
            original_scope: None,
            is_new: true,
            running: false,
            error: None,
            new_option_key: String::new(),
            new_option_value: String::new(),
            show_advanced: false,
            show_hidden: false,
            mount_on_boot: None,
        }
    }

    pub fn from_config(config: RemoteConfig) -> Self {
        Self {
            name: config.name.clone(),
            remote_type: config.remote_type.clone(),
            scope: config.scope,
            options: config.options.clone(),
            original_name: Some(config.name),
            original_scope: Some(config.scope),
            is_new: false,
            running: false,
            error: None,
            new_option_key: String::new(),
            new_option_value: String::new(),
            show_advanced: false,
            show_hidden: false,
            mount_on_boot: None,
        }
    }
}
