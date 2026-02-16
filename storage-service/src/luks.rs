// SPDX-License-Identifier: GPL-3.0-only

//! LUKS encryption D-Bus interface
//!
//! This module provides D-Bus methods for managing LUKS encrypted volumes.

use storage_common::EncryptionOptionsSettings;
use storage_service_macros::authorized_interface;
use zbus::message::Header as MessageHeader;
use zbus::{interface, Connection};

/// D-Bus interface for LUKS encryption operations
pub struct LuksHandler;

impl LuksHandler {
    /// Create a new LuksHandler
    pub fn new() -> Self {
        Self
    }
}

#[interface(name = "org.cosmic.ext.StorageService.Luks")]
impl LuksHandler {
    /// Signal emitted when a LUKS container is formatted
    #[zbus(signal)]
    async fn container_created(
        signal_ctxt: &zbus::object_server::SignalEmitter<'_>,
        device: &str,
    ) -> zbus::Result<()>;

    /// Signal emitted when a LUKS container is unlocked
    #[zbus(signal)]
    async fn container_unlocked(
        signal_ctxt: &zbus::object_server::SignalEmitter<'_>,
        device: &str,
        cleartext_device: &str,
    ) -> zbus::Result<()>;

    /// Signal emitted when a LUKS container is locked
    #[zbus(signal)]
    async fn container_locked(
        signal_ctxt: &zbus::object_server::SignalEmitter<'_>,
        device: &str,
    ) -> zbus::Result<()>;

    /// List all LUKS encrypted devices
    ///
    /// Returns: JSON-serialized Vec<LuksInfo>
    ///
    /// Authorization: org.cosmic.ext.storage-service.luks-read (allow_active)
    #[authorized_interface(action = "org.cosmic.ext.storage-service.luks-read")]
    async fn list_encrypted_devices(
        &self,
        #[zbus(connection)] _connection: &Connection,
        #[zbus(header)] _header: MessageHeader<'_>,
    ) -> zbus::fdo::Result<String> {
        tracing::debug!("Listing encrypted devices (UID {})", caller.uid);

        // Delegate to storage-dbus operation
        let luks_devices = storage_dbus::list_luks_devices().await.map_err(|e| {
            tracing::error!("Failed to list encrypted devices: {e}");
            zbus::fdo::Error::Failed(format!("Failed to list encrypted devices: {e}"))
        })?;

        tracing::debug!("Found {} encrypted devices", luks_devices.len());

        let json = serde_json::to_string(&luks_devices)
            .map_err(|e| zbus::fdo::Error::Failed(format!("Serialize error: {e}")))?;

        Ok(json)
    }

    /// Format a device as a LUKS encrypted container
    ///
    /// Args:
    /// - device: Device path (e.g., "/dev/sda1")
    /// - passphrase: Encryption passphrase
    /// - version: LUKS version ("luks1" or "luks2", defaults to "luks2")
    ///
    /// Authorization: org.cosmic.ext.storage-service.luks-format (auth_admin - always prompt)
    #[authorized_interface(action = "org.cosmic.ext.storage-service.luks-format")]
    async fn format(
        &self,
        #[zbus(connection)] _connection: &Connection,
        #[zbus(header)] _header: MessageHeader<'_>,
        #[zbus(signal_context)] signal_ctx: zbus::object_server::SignalEmitter<'_>,
        device: String,
        passphrase: String,
        version: String,
    ) -> zbus::fdo::Result<()> {
        tracing::info!("Formatting device '{}' as LUKS (UID {})", device, caller.uid);

        // Validate version
        let luks_version = if version.is_empty() || version == "luks2" {
            "luks2"
        } else if version == "luks1" {
            "luks1"
        } else {
            return Err(zbus::fdo::Error::InvalidArgs(format!(
                "Invalid LUKS version: {}. Use 'luks1' or 'luks2'",
                version
            )));
        };

        // Delegate to storage-dbus operation
        storage_dbus::format_luks(&device, &passphrase, luks_version)
            .await
            .map_err(|e| {
                tracing::error!("LUKS format failed: {e}");
                zbus::fdo::Error::Failed(format!("Format failed: {e}"))
            })?;

        tracing::info!("Device '{}' formatted as LUKS successfully", device);
        let _ = Self::container_created(&signal_ctx, &device).await;
        Ok(())
    }

    /// Unlock (open) a LUKS encrypted device
    ///
    /// Args:
    /// - device: Device path (e.g., "/dev/sda1")
    /// - passphrase: Decryption passphrase
    ///
    /// Returns: Cleartext device path (e.g., "/dev/mapper/luks-xxx")
    ///
    /// Authorization: org.cosmic.ext.storage-service.luks-unlock (allow_active)
    #[authorized_interface(action = "org.cosmic.ext.storage-service.luks-unlock")]
    async fn unlock(
        &self,
        #[zbus(connection)] _connection: &Connection,
        #[zbus(header)] _header: MessageHeader<'_>,
        #[zbus(signal_context)] signal_ctx: zbus::object_server::SignalEmitter<'_>,
        device: String,
        passphrase: String,
    ) -> zbus::fdo::Result<String> {
        tracing::info!("Unlocking LUKS device '{}' (UID {})", device, caller.uid);

        // Delegate to storage-dbus operation
        let cleartext_device = storage_dbus::unlock_luks(&device, &passphrase)
            .await
            .map_err(|e| {
                tracing::error!("Unlock failed: {e}");
                zbus::fdo::Error::Failed(format!("Unlock failed: {e}"))
            })?;

        tracing::info!(
            "LUKS device '{}' unlocked to '{}'",
            device,
            cleartext_device
        );
        let _ = Self::container_unlocked(&signal_ctx, &device, &cleartext_device).await;
        Ok(cleartext_device)
    }

    /// Lock (close) a LUKS encrypted device
    ///
    /// Args:
    /// - device: Device path (e.g., "/dev/sda1")
    ///
    /// Authorization: org.cosmic.ext.storage-service.luks-lock (allow_active)
    #[authorized_interface(action = "org.cosmic.ext.storage-service.luks-lock")]
    async fn lock(
        &self,
        #[zbus(connection)] _connection: &Connection,
        #[zbus(header)] _header: MessageHeader<'_>,
        #[zbus(signal_context)] signal_ctx: zbus::object_server::SignalEmitter<'_>,
        device: String,
    ) -> zbus::fdo::Result<()> {
        tracing::info!("Locking LUKS device '{}' (UID {})", device, caller.uid);

        // Delegate to storage-dbus operation
        storage_dbus::lock_luks(&device).await.map_err(|e| {
            tracing::error!("Lock failed: {e}");
            zbus::fdo::Error::Failed(format!("Lock failed: {e}"))
        })?;

        tracing::info!("LUKS device '{}' locked successfully", device);
        let _ = Self::container_locked(&signal_ctx, &device).await;
        Ok(())
    }

    /// Change the passphrase of a LUKS device
    ///
    /// Args:
    /// - device: Device path (e.g., "/dev/sda1")
    /// - current_passphrase: Current passphrase
    /// - new_passphrase: New passphrase
    ///
    /// Authorization: org.cosmic.ext.storage-service.luks-modify (auth_admin_keep)
    #[authorized_interface(action = "org.cosmic.ext.storage-service.luks-modify")]
    async fn change_passphrase(
        &self,
        #[zbus(connection)] _connection: &Connection,
        #[zbus(header)] _header: MessageHeader<'_>,
        device: String,
        current_passphrase: String,
        new_passphrase: String,
    ) -> zbus::fdo::Result<()> {
        tracing::info!("Changing passphrase for LUKS device '{}' (UID {})", device, caller.uid);

        // Delegate to storage-dbus operation
        storage_dbus::change_luks_passphrase(&device, &current_passphrase, &new_passphrase)
            .await
            .map_err(|e| {
                tracing::error!("Change passphrase failed: {e}");
                zbus::fdo::Error::Failed(format!("Change passphrase failed: {e}"))
            })?;

        tracing::info!(
            "Passphrase changed successfully for LUKS device '{}'",
            device
        );

        Ok(())
    }

    /// Get encryption options (crypttab settings) for a LUKS device
    ///
    /// Returns: JSON-serialized Option<EncryptionOptionsSettings> ("null" if none)
    ///
    /// Authorization: org.cosmic.ext.storage-service.luks-read (allow_active)
    #[authorized_interface(action = "org.cosmic.ext.storage-service.luks-read")]
    async fn get_encryption_options(
        &self,
        #[zbus(connection)] _connection: &Connection,
        #[zbus(header)] _header: MessageHeader<'_>,
        device: String,
    ) -> zbus::fdo::Result<String> {
        tracing::debug!("Getting encryption options for '{}' (UID {})", device, caller.uid);

        // Delegate to storage-dbus operation
        let settings = storage_dbus::get_encryption_options(&device)
            .await
            .map_err(|e| {
                tracing::error!("Failed to get encryption options: {e}");
                zbus::fdo::Error::Failed(format!("Failed to get encryption options: {e}"))
            })?;

        let json = serde_json::to_string(&settings)
            .map_err(|e| zbus::fdo::Error::Failed(format!("Serialize error: {e}")))?;

        Ok(json)
    }

    /// Set encryption options (crypttab) for a LUKS device
    ///
    /// Args:
    /// - device: Device path (e.g. "/dev/sda1")
    /// - options_json: JSON-serialized EncryptionOptionsSettings (name, unlock_at_startup, require_auth, other_options, optional passphrase)
    ///
    /// Authorization: org.cosmic.ext.storage-service.luks-set-options
    #[authorized_interface(action = "org.cosmic.ext.storage-service.luks-set-options")]
    async fn set_encryption_options(
        &self,
        #[zbus(connection)] _connection: &Connection,
        #[zbus(header)] _header: MessageHeader<'_>,
        device: String,
        options_json: String,
    ) -> zbus::fdo::Result<()> {
        tracing::debug!("Setting encryption options for '{}' (UID {})", device, caller.uid);

        let settings: EncryptionOptionsSettings = serde_json::from_str(&options_json)
            .map_err(|e| zbus::fdo::Error::InvalidArgs(format!("Invalid options JSON: {e}")))?;

        // Delegate to storage-dbus operation
        storage_dbus::set_encryption_options(&device, &settings)
            .await
            .map_err(|e| {
                tracing::error!("Failed to set encryption options: {e}");
                zbus::fdo::Error::Failed(format!("Failed to set encryption options: {e}"))
            })?;

        tracing::info!("Set encryption options for LUKS device '{}'", device);
        Ok(())
    }

    /// Clear encryption options (remove crypttab entry) for a LUKS device
    ///
    /// Authorization: org.cosmic.ext.storage-service.luks-set-options
    #[authorized_interface(action = "org.cosmic.ext.storage-service.luks-set-options")]
    async fn default_encryption_options(
        &self,
        #[zbus(connection)] _connection: &Connection,
        #[zbus(header)] _header: MessageHeader<'_>,
        device: String,
    ) -> zbus::fdo::Result<()> {
        tracing::debug!("Clearing encryption options for '{}' (UID {})", device, caller.uid);

        // Delegate to storage-dbus operation
        storage_dbus::clear_encryption_options(&device)
            .await
            .map_err(|e| {
                tracing::error!("Failed to clear encryption options: {e}");
                zbus::fdo::Error::Failed(format!("Failed to clear encryption options: {e}"))
            })?;

        tracing::info!("Cleared encryption options for LUKS device '{}'", device);
        Ok(())
    }

    // NOTE: add_key and remove_key are not available in UDisks2 EncryptedProxy
    // These would need to be implemented via direct cryptsetup luksAddKey/luksRemoveKey commands
    // or via raw D-Bus method calls if UDisks2 exposes them under different names.
    // For now, users can use change_passphrase to update their passphrase.
}
