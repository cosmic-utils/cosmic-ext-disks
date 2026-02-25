// SPDX-License-Identifier: GPL-3.0-only

//! RClone mount management D-Bus interface
//!
//! This module provides D-Bus methods for managing RClone remotes and mounts,
//! including listing remotes, mounting/unmounting, and configuration management.

use std::sync::Arc;
use storage_macros::authorized_interface;
use storage_sys::{RCloneCli, is_mount_on_boot_enabled, set_mount_on_boot};
use storage_types::rclone::{
    ConfigScope, MountStatus, MountStatusResult, RemoteConfig, RemoteConfigList, TestResult,
    rclone_provider, supported_remote_types,
};
use zbus::message::Header as MessageHeader;
use zbus::object_server::SignalEmitter;
use zbus::{Connection, interface};

use crate::policies::rclone::{RcloneDomain, RclonePolicy};

/// D-Bus interface for RClone mount management operations
pub struct RcloneHandler {
    cli: RCloneCli,
    domain: Arc<dyn RcloneDomain>,
}

impl RcloneHandler {
    /// Create a new RcloneHandler
    pub fn new() -> Result<Self, storage_sys::SysError> {
        let cli = RCloneCli::new()?;
        tracing::info!("RcloneHandler initialized successfully");
        Ok(Self {
            cli,
            domain: Arc::new(RclonePolicy),
        })
    }

    /// Get the home directory for a specific UID
    fn get_home_for_uid(uid: u32) -> Option<std::path::PathBuf> {
        unsafe {
            let pw = libc::getpwuid(uid);
            if pw.is_null() {
                tracing::warn!("Failed to get passwd entry for UID {}", uid);
                return None;
            }
            let dir = std::ffi::CStr::from_ptr((*pw).pw_dir);
            dir.to_str().ok().map(std::path::PathBuf::from)
        }
    }

    /// Get the config path for a scope, checking if it exists
    /// For User scope, uses the provided UID to find the correct home directory
    fn get_existing_config_path(
        scope: ConfigScope,
        uid: Option<u32>,
    ) -> Option<std::path::PathBuf> {
        let path = Self::get_config_path_for_uid(scope, uid);
        if path.exists() { Some(path) } else { None }
    }

    /// Get the config path for a scope with an optional UID
    /// For User scope, uses the provided UID to find the correct home directory
    fn get_config_path_for_uid(scope: ConfigScope, uid: Option<u32>) -> std::path::PathBuf {
        match scope {
            ConfigScope::User => {
                if let Some(uid) = uid {
                    Self::get_home_for_uid(uid)
                        .map(|home| home.join(".config/rclone/rclone.conf"))
                        .unwrap_or_else(|| scope.config_path())
                } else {
                    scope.config_path()
                }
            }
            ConfigScope::System => scope.config_path(),
        }
    }

    /// Get the mount point for a remote with an optional UID
    fn get_mount_point_for_uid(
        remote_name: &str,
        scope: ConfigScope,
        uid: Option<u32>,
    ) -> std::path::PathBuf {
        match scope {
            ConfigScope::User => {
                if let Some(uid) = uid {
                    Self::get_home_for_uid(uid)
                        .map(|home| home.join("mnt").join(remote_name))
                        .unwrap_or_else(|| scope.mount_point(remote_name))
                } else {
                    scope.mount_point(remote_name)
                }
            }
            ConfigScope::System => scope.mount_point(remote_name),
        }
    }

    /// Get the caller UID from a message header
    async fn get_caller_uid(
        connection: &Connection,
        header: &MessageHeader<'_>,
    ) -> zbus::fdo::Result<u32> {
        let sender = header
            .sender()
            .ok_or_else(|| zbus::fdo::Error::Failed("No sender in message header".to_string()))?
            .as_str()
            .to_string();

        let dbus_proxy = zbus::fdo::DBusProxy::new(connection)
            .await
            .map_err(|e| zbus::fdo::Error::Failed(format!("D-Bus connection error: {}", e)))?;

        let bus_name: zbus::names::BusName = sender
            .clone()
            .try_into()
            .map_err(|e| zbus::fdo::Error::Failed(format!("Invalid bus name: {}", e)))?;

        dbus_proxy
            .get_connection_unix_user(bus_name)
            .await
            .map_err(|e| zbus::fdo::Error::Failed(format!("Failed to get caller UID: {}", e)))
    }

    /// Convert scope string to ConfigScope enum
    fn parse_scope(&self, scope: &str) -> Result<ConfigScope, zbus::fdo::Error> {
        self.domain.parse_scope(scope)
    }

    fn validate_remote_config(&self, config: &RemoteConfig) -> Result<(), zbus::fdo::Error> {
        self.domain.validate_remote_config(config)
    }
}

#[interface(name = "org.cosmic.ext.Storage.Service.Rclone")]
impl RcloneHandler {
    /// Signal emitted when mount status changes
    #[zbus(signal)]
    async fn mount_changed(
        signal_ctxt: &SignalEmitter<'_>,
        name: &str,
        scope: &str,
        status: &str,
    ) -> zbus::Result<()>;

    /// List all configured RClone remotes from both user and system config files
    #[authorized_interface(action = "org.cosmic.ext.storage.service.rclone-read")]
    async fn list_remotes(
        &self,
        #[zbus(connection)] _connection: &Connection,
        #[zbus(header)] _header: MessageHeader<'_>,
    ) -> zbus::fdo::Result<String> {
        self.domain.require_available()?;
        tracing::info!("Listing RClone remotes (UID {})", caller.uid);

        let mut remotes = Vec::new();
        let user_config_path = Self::get_existing_config_path(ConfigScope::User, Some(caller.uid));
        let system_config_path = Self::get_existing_config_path(ConfigScope::System, None);

        // Read user remotes
        if let Some(ref path) = user_config_path {
            match self.cli.list_remotes(path) {
                Ok(names) => {
                    let config = self.cli.read_config(path);
                    for name in names {
                        let options = config
                            .as_ref()
                            .ok()
                            .and_then(|c| c.get(&name).cloned())
                            .unwrap_or_default();
                        let remote_type = options
                            .get("type")
                            .and_then(|v| v.clone())
                            .unwrap_or_else(|| "unknown".to_string());
                        let has_secrets = rclone_provider(&remote_type).is_some_and(|provider| {
                            provider.options.iter().any(|option| {
                                option.is_secure()
                                    && options
                                        .get(&option.name)
                                        .and_then(|v| v.as_ref())
                                        .is_some_and(|v| !v.trim().is_empty())
                            })
                        });

                        remotes.push(RemoteConfig {
                            name,
                            remote_type,
                            scope: ConfigScope::User,
                            options: options
                                .into_iter()
                                .filter(|(k, _)| k != "type")
                                .filter_map(|(k, v)| v.map(|v| (k, v)))
                                .collect(),
                            has_secrets,
                        });
                    }
                }
                Err(e) => {
                    tracing::warn!("Failed to list user remotes: {}", e);
                }
            }
        }

        // Read system remotes
        if let Some(ref path) = system_config_path {
            match self.cli.list_remotes(path) {
                Ok(names) => {
                    let config = self.cli.read_config(path);
                    for name in names {
                        let options = config
                            .as_ref()
                            .ok()
                            .and_then(|c| c.get(&name).cloned())
                            .unwrap_or_default();
                        let remote_type = options
                            .get("type")
                            .and_then(|v| v.clone())
                            .unwrap_or_else(|| "unknown".to_string());
                        let has_secrets = rclone_provider(&remote_type).is_some_and(|provider| {
                            provider.options.iter().any(|option| {
                                option.is_secure()
                                    && options
                                        .get(&option.name)
                                        .and_then(|v| v.as_ref())
                                        .is_some_and(|v| !v.trim().is_empty())
                            })
                        });

                        remotes.push(RemoteConfig {
                            name,
                            remote_type,
                            scope: ConfigScope::System,
                            options: options
                                .into_iter()
                                .filter(|(k, _)| k != "type")
                                .filter_map(|(k, v)| v.map(|v| (k, v)))
                                .collect(),
                            has_secrets,
                        });
                    }
                }
                Err(e) => {
                    tracing::warn!("Failed to list system remotes: {}", e);
                }
            }
        }

        let list = RemoteConfigList {
            remotes,
            user_config_path,
            system_config_path,
        };

        serde_json::to_string(&list)
            .map_err(|e| zbus::fdo::Error::Failed(format!("Serialization error: {}", e)))
    }

    /// Get detailed configuration for a specific remote
    #[authorized_interface(action = "org.cosmic.ext.storage.service.rclone-read")]
    async fn get_remote(
        &self,
        #[zbus(connection)] _connection: &Connection,
        #[zbus(header)] _header: MessageHeader<'_>,
        name: &str,
        scope: &str,
    ) -> zbus::fdo::Result<String> {
        tracing::info!(
            "Getting remote {} (scope: {}, UID {})",
            name,
            scope,
            caller.uid
        );

        let scope = self.parse_scope(scope)?;
        let config_path = Self::get_existing_config_path(scope, Some(caller.uid))
            .ok_or_else(|| zbus::fdo::Error::Failed("Config file not found".to_string()))?;

        let config = self
            .cli
            .read_config(&config_path)
            .map_err(|e| zbus::fdo::Error::Failed(format!("Failed to read config: {}", e)))?;

        let options = config
            .get(name)
            .cloned()
            .ok_or_else(|| zbus::fdo::Error::Failed(format!("Remote {} not found", name)))?;

        let remote_type = options
            .get("type")
            .and_then(|v| v.clone())
            .unwrap_or_else(|| "unknown".to_string());
        let has_secrets = rclone_provider(&remote_type).is_some_and(|provider| {
            provider.options.iter().any(|option| {
                option.is_secure()
                    && options
                        .get(&option.name)
                        .and_then(|v| v.as_ref())
                        .is_some_and(|v| !v.trim().is_empty())
            })
        });

        let remote = RemoteConfig {
            name: name.to_string(),
            remote_type,
            scope,
            options: options
                .into_iter()
                .filter_map(|(k, v)| v.map(|v| (k, v)))
                .collect(),
            has_secrets,
        };

        serde_json::to_string(&remote)
            .map_err(|e| zbus::fdo::Error::Failed(format!("Serialization error: {}", e)))
    }

    /// Test connectivity and authentication for a remote
    #[authorized_interface(action = "org.cosmic.ext.storage.service.rclone-test")]
    async fn test_remote(
        &self,
        #[zbus(connection)] _connection: &Connection,
        #[zbus(header)] _header: MessageHeader<'_>,
        name: &str,
        scope: &str,
    ) -> zbus::fdo::Result<String> {
        tracing::info!(
            "Testing remote {} (scope: {}, UID {})",
            name,
            scope,
            caller.uid
        );

        let scope = self.parse_scope(scope)?;
        let config_path = Self::get_existing_config_path(scope, Some(caller.uid))
            .ok_or_else(|| zbus::fdo::Error::Failed("Config file not found".to_string()))?;

        let (success, message, latency_ms) = self
            .cli
            .test_remote(name, &config_path)
            .map_err(|e| zbus::fdo::Error::Failed(format!("Test failed: {}", e)))?;

        let result = TestResult {
            success,
            message,
            latency_ms: Some(latency_ms),
        };

        serde_json::to_string(&result)
            .map_err(|e| zbus::fdo::Error::Failed(format!("Serialization error: {}", e)))
    }

    /// Mount a remote
    async fn mount(
        &self,
        #[zbus(connection)] connection: &Connection,
        #[zbus(header)] header: MessageHeader<'_>,
        #[zbus(signal_context)] signal_ctxt: SignalEmitter<'_>,
        name: &str,
        scope: &str,
    ) -> zbus::fdo::Result<()> {
        tracing::info!("Mounting remote {} (scope: {})", name, scope);

        let scope_enum = self.parse_scope(scope)?;
        let caller_uid = Self::get_caller_uid(connection, &header).await?;

        // For system scope, check polkit authorization
        if scope_enum == ConfigScope::System {
            let sender = header
                .sender()
                .ok_or_else(|| zbus::fdo::Error::Failed("No sender in message header".to_string()))?
                .as_str()
                .to_string();

            let authorized = crate::auth::check_authorization(
                connection,
                &sender,
                "org.cosmic.ext.storage.service.rclone-mount",
            )
            .await
            .map_err(|e| zbus::fdo::Error::Failed(format!("Authorization check failed: {}", e)))?;

            if !authorized {
                return Err(zbus::fdo::Error::AccessDenied(
                    "Not authorized for system-wide mount operations".to_string(),
                ));
            }
        }

        let config_path = Self::get_config_path_for_uid(scope_enum, Some(caller_uid));
        let mount_point = Self::get_mount_point_for_uid(name, scope_enum, Some(caller_uid));

        self.cli
            .mount(
                name,
                &mount_point,
                &config_path,
                scope_enum,
                Some(caller_uid),
            )
            .map_err(|e| zbus::fdo::Error::Failed(format!("Mount failed: {}", e)))?;

        // Emit signal
        Self::mount_changed(&signal_ctxt, name, scope, "Mounted")
            .await
            .ok();

        Ok(())
    }

    /// Unmount a remote
    async fn unmount(
        &self,
        #[zbus(connection)] connection: &Connection,
        #[zbus(header)] header: MessageHeader<'_>,
        #[zbus(signal_context)] signal_ctxt: SignalEmitter<'_>,
        name: &str,
        scope: &str,
    ) -> zbus::fdo::Result<()> {
        tracing::info!("Unmounting remote {} (scope: {})", name, scope);

        let scope_enum = self.parse_scope(scope)?;
        let caller_uid = Self::get_caller_uid(connection, &header).await?;

        // For system scope, check polkit authorization
        if scope_enum == ConfigScope::System {
            let sender = header
                .sender()
                .ok_or_else(|| zbus::fdo::Error::Failed("No sender in message header".to_string()))?
                .as_str()
                .to_string();

            let authorized = crate::auth::check_authorization(
                connection,
                &sender,
                "org.cosmic.ext.storage.service.rclone-mount",
            )
            .await
            .map_err(|e| zbus::fdo::Error::Failed(format!("Authorization check failed: {}", e)))?;

            if !authorized {
                return Err(zbus::fdo::Error::AccessDenied(
                    "Not authorized for system-wide mount operations".to_string(),
                ));
            }
        }

        let mount_point = Self::get_mount_point_for_uid(name, scope_enum, Some(caller_uid));

        self.cli
            .unmount(&mount_point)
            .map_err(|e| zbus::fdo::Error::Failed(format!("Unmount failed: {}", e)))?;

        // Emit signal
        Self::mount_changed(&signal_ctxt, name, scope, "Unmounted")
            .await
            .ok();

        Ok(())
    }

    /// Get current mount status for a remote
    #[authorized_interface(action = "org.cosmic.ext.storage.service.rclone-read")]
    async fn get_mount_status(
        &self,
        #[zbus(connection)] _connection: &Connection,
        #[zbus(header)] _header: MessageHeader<'_>,
        name: &str,
        scope: &str,
    ) -> zbus::fdo::Result<String> {
        tracing::info!(
            "Getting mount status for {} (scope: {}, UID {})",
            name,
            scope,
            caller.uid
        );

        let scope = self.parse_scope(scope)?;
        let mount_point = Self::get_mount_point_for_uid(name, scope, Some(caller.uid));

        let status = if RCloneCli::is_mounted(&mount_point)
            .map_err(|e| zbus::fdo::Error::Failed(format!("Failed to check mount status: {}", e)))?
        {
            MountStatus::Mounted
        } else {
            MountStatus::Unmounted
        };

        let result = MountStatusResult::new(status, mount_point);

        serde_json::to_string(&result)
            .map_err(|e| zbus::fdo::Error::Failed(format!("Serialization error: {}", e)))
    }

    /// Check if a remote is enabled to mount on boot
    #[authorized_interface(action = "org.cosmic.ext.storage.service.rclone-read")]
    async fn get_mount_on_boot(
        &self,
        #[zbus(connection)] connection: &Connection,
        #[zbus(header)] header: MessageHeader<'_>,
        name: &str,
        scope: &str,
    ) -> zbus::fdo::Result<bool> {
        let scope_enum = self.parse_scope(scope)?;
        let caller_uid = Self::get_caller_uid(connection, &header).await?;
        let home = if scope_enum == ConfigScope::User {
            Self::get_home_for_uid(caller_uid)
        } else {
            None
        };

        is_mount_on_boot_enabled(
            scope_enum,
            name,
            if scope_enum == ConfigScope::User {
                Some(caller_uid)
            } else {
                None
            },
            home.as_deref(),
        )
        .map_err(|e| zbus::fdo::Error::Failed(format!("Failed to read mount on boot: {}", e)))
    }

    /// Enable or disable mount on boot
    async fn set_mount_on_boot(
        &self,
        #[zbus(connection)] connection: &Connection,
        #[zbus(header)] header: MessageHeader<'_>,
        name: &str,
        scope: &str,
        enabled: bool,
    ) -> zbus::fdo::Result<()> {
        let scope_enum = self.parse_scope(scope)?;
        let caller_uid = Self::get_caller_uid(connection, &header).await?;

        if scope_enum == ConfigScope::System {
            let sender = header
                .sender()
                .ok_or_else(|| zbus::fdo::Error::Failed("No sender in message header".to_string()))?
                .as_str()
                .to_string();

            let authorized = crate::auth::check_authorization(
                connection,
                &sender,
                "org.cosmic.ext.storage.service.rclone-mount",
            )
            .await
            .map_err(|e| zbus::fdo::Error::Failed(format!("Authorization check failed: {}", e)))?;

            if !authorized {
                return Err(zbus::fdo::Error::AccessDenied(
                    "Not authorized for system-wide mount operations".to_string(),
                ));
            }
        }

        let home = if scope_enum == ConfigScope::User {
            Self::get_home_for_uid(caller_uid)
        } else {
            None
        };

        let result = set_mount_on_boot(
            scope_enum,
            name,
            enabled,
            if scope_enum == ConfigScope::User {
                Some(caller_uid)
            } else {
                None
            },
            home.as_deref(),
        );

        if let Err(ref err) = result {
            tracing::error!(
                ?err,
                scope = ?scope_enum,
                remote = name,
                enabled,
                caller_uid,
                home = ?home,
                "Failed to update mount on boot"
            );
        }

        result
            .map_err(|e| zbus::fdo::Error::Failed(format!("Failed to update mount on boot: {}", e)))
    }

    /// Create a new remote configuration
    async fn create_remote(
        &self,
        #[zbus(connection)] connection: &Connection,
        #[zbus(header)] header: MessageHeader<'_>,
        config: &str,
        scope: &str,
    ) -> zbus::fdo::Result<()> {
        tracing::info!("Creating remote (scope: {})", scope);

        let scope_enum = self.parse_scope(scope)?;
        let caller_uid = Self::get_caller_uid(connection, &header).await?;

        // For system scope, check polkit authorization
        if scope_enum == ConfigScope::System {
            let sender = header
                .sender()
                .ok_or_else(|| zbus::fdo::Error::Failed("No sender in message header".to_string()))?
                .as_str()
                .to_string();

            let authorized = crate::auth::check_authorization(
                connection,
                &sender,
                "org.cosmic.ext.storage.service.rclone-config",
            )
            .await
            .map_err(|e| zbus::fdo::Error::Failed(format!("Authorization check failed: {}", e)))?;

            if !authorized {
                return Err(zbus::fdo::Error::AccessDenied(
                    "Not authorized for system-wide config operations".to_string(),
                ));
            }
        }

        let remote_config: RemoteConfig = serde_json::from_str(config)
            .map_err(|e| zbus::fdo::Error::Failed(format!("Invalid config JSON: {}", e)))?;

        self.validate_remote_config(&remote_config)?;

        let config_path = Self::get_config_path_for_uid(scope_enum, Some(caller_uid));

        // Read existing config
        let mut existing = if config_path.exists() {
            self.cli
                .read_config(&config_path)
                .map_err(|e| zbus::fdo::Error::Failed(format!("Failed to read config: {}", e)))?
        } else {
            std::collections::HashMap::new()
        };

        // Check if remote already exists
        if existing.contains_key(&remote_config.name) {
            return Err(zbus::fdo::Error::Failed(format!(
                "Remote {} already exists",
                remote_config.name
            )));
        }

        // Add new remote
        let mut options = std::collections::HashMap::new();
        options.insert("type".to_string(), Some(remote_config.remote_type.clone()));
        for (k, v) in &remote_config.options {
            if k.eq_ignore_ascii_case("type") {
                continue;
            }
            options.insert(k.clone(), Some(v.clone()));
        }
        existing.insert(remote_config.name, options);

        // Write config
        self.cli
            .write_config(&config_path, &existing)
            .map_err(|e| zbus::fdo::Error::Failed(format!("Failed to write config: {}", e)))?;

        Ok(())
    }

    /// Update an existing remote configuration
    async fn update_remote(
        &self,
        #[zbus(connection)] connection: &Connection,
        #[zbus(header)] header: MessageHeader<'_>,
        name: &str,
        config: &str,
        scope: &str,
    ) -> zbus::fdo::Result<()> {
        tracing::info!("Updating remote {} (scope: {})", name, scope);

        let scope_enum = self.parse_scope(scope)?;
        let caller_uid = Self::get_caller_uid(connection, &header).await?;

        // For system scope, check polkit authorization
        if scope_enum == ConfigScope::System {
            let sender = header
                .sender()
                .ok_or_else(|| zbus::fdo::Error::Failed("No sender in message header".to_string()))?
                .as_str()
                .to_string();

            let authorized = crate::auth::check_authorization(
                connection,
                &sender,
                "org.cosmic.ext.storage.service.rclone-config",
            )
            .await
            .map_err(|e| zbus::fdo::Error::Failed(format!("Authorization check failed: {}", e)))?;

            if !authorized {
                return Err(zbus::fdo::Error::AccessDenied(
                    "Not authorized for system-wide config operations".to_string(),
                ));
            }
        }

        let remote_config: RemoteConfig = serde_json::from_str(config)
            .map_err(|e| zbus::fdo::Error::Failed(format!("Invalid config JSON: {}", e)))?;

        self.validate_remote_config(&remote_config)?;

        let config_path = Self::get_config_path_for_uid(scope_enum, Some(caller_uid));

        // Read existing config
        let mut existing = self
            .cli
            .read_config(&config_path)
            .map_err(|e| zbus::fdo::Error::Failed(format!("Failed to read config: {}", e)))?;

        // Check if remote exists
        if !existing.contains_key(name) {
            return Err(zbus::fdo::Error::Failed(format!(
                "Remote {} not found",
                name
            )));
        }

        // Update remote
        let mut options = std::collections::HashMap::new();
        options.insert("type".to_string(), Some(remote_config.remote_type.clone()));
        for (k, v) in &remote_config.options {
            if k.eq_ignore_ascii_case("type") {
                continue;
            }
            options.insert(k.clone(), Some(v.clone()));
        }
        existing.insert(name.to_string(), options);

        // Write config
        self.cli
            .write_config(&config_path, &existing)
            .map_err(|e| zbus::fdo::Error::Failed(format!("Failed to write config: {}", e)))?;

        Ok(())
    }

    /// Delete a remote configuration
    async fn delete_remote(
        &self,
        #[zbus(connection)] connection: &Connection,
        #[zbus(header)] header: MessageHeader<'_>,
        name: &str,
        scope: &str,
    ) -> zbus::fdo::Result<()> {
        tracing::info!("Deleting remote {} (scope: {})", name, scope);

        let scope_enum = self.parse_scope(scope)?;
        let caller_uid = Self::get_caller_uid(connection, &header).await?;

        // For system scope, check polkit authorization
        if scope_enum == ConfigScope::System {
            let sender = header
                .sender()
                .ok_or_else(|| zbus::fdo::Error::Failed("No sender in message header".to_string()))?
                .as_str()
                .to_string();

            let authorized = crate::auth::check_authorization(
                connection,
                &sender,
                "org.cosmic.ext.storage.service.rclone-config",
            )
            .await
            .map_err(|e| zbus::fdo::Error::Failed(format!("Authorization check failed: {}", e)))?;

            if !authorized {
                return Err(zbus::fdo::Error::AccessDenied(
                    "Not authorized for system-wide config operations".to_string(),
                ));
            }
        }

        let config_path = Self::get_config_path_for_uid(scope_enum, Some(caller_uid));

        // Read existing config
        let mut existing = self
            .cli
            .read_config(&config_path)
            .map_err(|e| zbus::fdo::Error::Failed(format!("Failed to read config: {}", e)))?;

        // Remove remote
        if existing.remove(name).is_none() {
            return Err(zbus::fdo::Error::Failed(format!(
                "Remote {} not found",
                name
            )));
        }

        // Write config
        self.cli
            .write_config(&config_path, &existing)
            .map_err(|e| zbus::fdo::Error::Failed(format!("Failed to write config: {}", e)))?;

        Ok(())
    }

    /// Get list of supported remote types
    async fn supported_remote_types(&self) -> zbus::fdo::Result<Vec<String>> {
        self.domain.require_available()?;
        Ok(supported_remote_types().to_vec())
    }
}
