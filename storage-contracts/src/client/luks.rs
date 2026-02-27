// SPDX-License-Identifier: GPL-3.0-only

use crate::client::connection::shared_connection;
use crate::client::error::ClientError;
use storage_types::EncryptionOptionsSettings;
use zbus::proxy;

/// D-Bus proxy interface for LUKS encryption operations
#[proxy(
    interface = "org.cosmic.ext.Storage.Service.Luks",
    default_service = "org.cosmic.ext.Storage.Service",
    default_path = "/org/cosmic/ext/Storage/Service/luks"
)]
pub trait LuksInterface {
    /// List all encrypted devices
    async fn list_encrypted_devices(&self) -> zbus::Result<String>;

    /// Format a device with LUKS encryption
    async fn format(&self, device: &str, passphrase: &str, version: &str) -> zbus::Result<()>;

    /// Unlock a LUKS volume
    async fn unlock(&self, device: &str, passphrase: &str) -> zbus::Result<String>;

    /// Lock a LUKS volume
    async fn lock(&self, cleartext_device: &str) -> zbus::Result<()>;

    /// Change LUKS passphrase
    async fn change_passphrase(
        &self,
        device: &str,
        old_passphrase: &str,
        new_passphrase: &str,
    ) -> zbus::Result<()>;

    /// Get encryption options (crypttab settings) for a LUKS device
    async fn get_encryption_options(&self, device: &str) -> zbus::Result<String>;

    /// Set encryption options (crypttab) for a LUKS device
    async fn set_encryption_options(&self, device: &str, options_json: &str) -> zbus::Result<()>;

    /// Clear encryption options (remove crypttab entry) for a LUKS device
    async fn default_encryption_options(&self, device: &str) -> zbus::Result<()>;

    /// Signal emitted when a LUKS container is formatted (matches storage-service container_created)
    #[zbus(signal)]
    async fn container_created(&self, device: &str) -> zbus::Result<()>;

    /// Signal emitted when a LUKS container is unlocked
    #[zbus(signal)]
    async fn container_unlocked(&self, device: &str, cleartext_device: &str) -> zbus::Result<()>;

    /// Signal emitted when a LUKS container is locked
    #[zbus(signal)]
    async fn container_locked(&self, device: &str) -> zbus::Result<()>;
}

/// Client for LUKS encryption operations
pub struct LuksClient {
    proxy: LuksInterfaceProxy<'static>,
}

impl LuksClient {
    /// Create a new LUKS client connected to the storage service
    pub async fn new() -> Result<Self, ClientError> {
        let conn = shared_connection().await?;

        let proxy = LuksInterfaceProxy::new(conn)
            .await
            .map_err(|e| ClientError::Connection(format!("Failed to create LUKS proxy: {}", e)))?;

        Ok(Self { proxy })
    }

    /// Unlock a LUKS volume, returns cleartext device path (e.g., /dev/mapper/luks-...)
    pub async fn unlock(&self, device: &str, passphrase: &str) -> Result<String, ClientError> {
        Ok(self.proxy.unlock(device, passphrase).await?)
    }

    /// Format a device with LUKS encryption
    pub async fn format(
        &self,
        device: &str,
        passphrase: &str,
        version: &str,
    ) -> Result<(), ClientError> {
        Ok(self.proxy.format(device, passphrase, version).await?)
    }

    /// Lock a LUKS volume
    pub async fn lock(&self, cleartext_device: &str) -> Result<(), ClientError> {
        Ok(self.proxy.lock(cleartext_device).await?)
    }

    /// Change LUKS passphrase
    pub async fn change_passphrase(
        &self,
        device: &str,
        old_passphrase: &str,
        new_passphrase: &str,
    ) -> Result<(), ClientError> {
        Ok(self
            .proxy
            .change_passphrase(device, old_passphrase, new_passphrase)
            .await?)
    }

    /// Get encryption options (crypttab settings) for a LUKS device
    pub async fn get_encryption_options(
        &self,
        device: &str,
    ) -> Result<Option<EncryptionOptionsSettings>, ClientError> {
        let json = self.proxy.get_encryption_options(device).await?;
        let opt: Option<EncryptionOptionsSettings> = serde_json::from_str(&json).map_err(|e| {
            ClientError::ParseError(format!("Failed to parse encryption options: {}", e))
        })?;
        Ok(opt)
    }

    /// Set encryption options (crypttab) for a LUKS device
    pub async fn set_encryption_options(
        &self,
        device: &str,
        options: &EncryptionOptionsSettings,
    ) -> Result<(), ClientError> {
        let json = serde_json::to_string(options)
            .map_err(|e| ClientError::ParseError(format!("Failed to serialize options: {}", e)))?;
        Ok(self.proxy.set_encryption_options(device, &json).await?)
    }

    /// Clear encryption options (remove crypttab entry) for a LUKS device
    pub async fn default_encryption_options(&self, device: &str) -> Result<(), ClientError> {
        Ok(self.proxy.default_encryption_options(device).await?)
    }

    /// Get the underlying proxy for signal subscriptions
    pub fn proxy(&self) -> &LuksInterfaceProxy<'static> {
        &self.proxy
    }
}
