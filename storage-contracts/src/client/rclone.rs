// SPDX-License-Identifier: GPL-3.0-only

use crate::client::connection::shared_connection;
use crate::client::error::ClientError;
use storage_types::rclone::{MountStatusResult, RemoteConfig, RemoteConfigList, TestResult};
use zbus::proxy;

/// D-Bus proxy interface for RClone operations
#[proxy(
    interface = "org.cosmic.ext.Storage.Service.Rclone",
    default_service = "org.cosmic.ext.Storage.Service",
    default_path = "/org/cosmic/ext/Storage/Service/rclone"
)]
trait RcloneInterface {
    /// List all configured RClone remotes
    async fn list_remotes(&self) -> zbus::Result<String>;

    /// Get detailed configuration for a specific remote
    async fn get_remote(&self, name: &str, scope: &str) -> zbus::Result<String>;

    /// Test connectivity and authentication for a remote
    async fn test_remote(&self, name: &str, scope: &str) -> zbus::Result<String>;

    /// Mount a remote
    async fn mount(&self, name: &str, scope: &str) -> zbus::Result<()>;

    /// Unmount a remote
    async fn unmount(&self, name: &str, scope: &str) -> zbus::Result<()>;

    /// Get current mount status for a remote
    async fn get_mount_status(&self, name: &str, scope: &str) -> zbus::Result<String>;

    /// Check if a remote is set to mount on boot
    async fn get_mount_on_boot(&self, name: &str, scope: &str) -> zbus::Result<bool>;

    /// Enable or disable mount on boot
    async fn set_mount_on_boot(&self, name: &str, scope: &str, enabled: bool) -> zbus::Result<()>;

    /// Create a new remote configuration
    async fn create_remote(&self, config: &str, scope: &str) -> zbus::Result<()>;

    /// Update an existing remote configuration
    async fn update_remote(&self, name: &str, config: &str, scope: &str) -> zbus::Result<()>;

    /// Delete a remote configuration
    async fn delete_remote(&self, name: &str, scope: &str) -> zbus::Result<()>;

    /// List of supported remote types
    async fn supported_remote_types(&self) -> zbus::Result<Vec<String>>;
}

/// Client for RClone operations via D-Bus
pub struct RcloneClient {
    proxy: RcloneInterfaceProxy<'static>,
}

impl std::fmt::Debug for RcloneClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RcloneClient").finish_non_exhaustive()
    }
}

impl RcloneClient {
    /// Create a new RClone client connected to the storage service
    pub async fn new() -> Result<Self, ClientError> {
        let conn = shared_connection().await?;

        let proxy = RcloneInterfaceProxy::new(conn)
            .await
            .map_err(|e| ClientError::Connection(format!("Failed to create proxy: {}", e)))?;

        Ok(Self { proxy })
    }

    /// List all configured RClone remotes
    pub async fn list_remotes(&self) -> Result<RemoteConfigList, ClientError> {
        let json = self.proxy.list_remotes().await?;
        let list: RemoteConfigList = serde_json::from_str(&json)?;
        Ok(list)
    }

    /// Test connectivity and authentication for a remote
    pub async fn test_remote(&self, name: &str, scope: &str) -> Result<TestResult, ClientError> {
        let json = self.proxy.test_remote(name, scope).await?;
        let result: TestResult = serde_json::from_str(&json)?;
        Ok(result)
    }

    /// Mount a remote
    pub async fn mount(&self, name: &str, scope: &str) -> Result<(), ClientError> {
        Ok(self.proxy.mount(name, scope).await?)
    }

    /// Unmount a remote
    pub async fn unmount(&self, name: &str, scope: &str) -> Result<(), ClientError> {
        Ok(self.proxy.unmount(name, scope).await?)
    }

    /// Get current mount status for a remote
    pub async fn get_mount_status(
        &self,
        name: &str,
        scope: &str,
    ) -> Result<MountStatusResult, ClientError> {
        let json = self.proxy.get_mount_status(name, scope).await?;
        let status: MountStatusResult = serde_json::from_str(&json)?;
        Ok(status)
    }

    /// Check if a remote is set to mount on boot
    pub async fn get_mount_on_boot(&self, name: &str, scope: &str) -> Result<bool, ClientError> {
        Ok(self.proxy.get_mount_on_boot(name, scope).await?)
    }

    /// Enable or disable mount on boot
    pub async fn set_mount_on_boot(
        &self,
        name: &str,
        scope: &str,
        enabled: bool,
    ) -> Result<(), ClientError> {
        Ok(self.proxy.set_mount_on_boot(name, scope, enabled).await?)
    }

    /// Create a new remote configuration
    pub async fn create_remote(&self, config: &RemoteConfig) -> Result<(), ClientError> {
        let json = serde_json::to_string(config)?;
        let scope = config.scope.to_string();
        Ok(self.proxy.create_remote(&json, &scope).await?)
    }

    /// Update an existing remote configuration
    pub async fn update_remote(
        &self,
        name: &str,
        config: &RemoteConfig,
    ) -> Result<(), ClientError> {
        let json = serde_json::to_string(config)?;
        let scope = config.scope.to_string();
        Ok(self.proxy.update_remote(name, &json, &scope).await?)
    }

    /// Delete a remote configuration
    pub async fn delete_remote(&self, name: &str, scope: &str) -> Result<(), ClientError> {
        Ok(self.proxy.delete_remote(name, scope).await?)
    }
}
