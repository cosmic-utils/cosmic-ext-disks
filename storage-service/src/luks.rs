// SPDX-License-Identifier: GPL-3.0-only

//! LUKS encryption D-Bus interface
//!
//! This module provides D-Bus methods for managing LUKS encrypted volumes.

use std::collections::HashMap;
use udisks2::{block::BlockProxy, encrypted::EncryptedProxy};
use zbus::{interface, Connection};
use zbus::zvariant::{OwnedObjectPath, OwnedValue, Value};
use storage_models::{EncryptionOptionsSettings, LuksInfo, LuksVersion};
use disks_dbus::{
    bytestring_owned_value, join_options, set_token_present, split_options, stable_dedup,
    owned_value_to_bytestring, ConfigurationItem, UDisks2BlockConfigurationProxy,
};

use crate::auth::check_polkit_auth;

/// D-Bus interface for LUKS encryption operations
pub struct LuksHandler;

impl LuksHandler {
    /// Create a new LuksHandler
    pub fn new() -> Self {
        Self
    }
    
    /// Convert UDisks2 path to device path
    async fn path_to_device(connection: &Connection, path: &OwnedObjectPath) -> Result<String, zbus::fdo::Error> {
        let block_proxy = BlockProxy::builder(connection)
            .path(path)?
            .build()
            .await
            .map_err(|e| zbus::fdo::Error::Failed(format!("Failed to get block device: {e}")))?;
        
        let device_bytes = block_proxy.device().await
            .map_err(|e| zbus::fdo::Error::Failed(format!("Failed to get device path: {e}")))?;
        
        // Convert bytestring to String
        let device = String::from_utf8(device_bytes.into_iter().filter(|&b| b != 0).collect())
            .unwrap_or_default();
        
        Ok(device)
    }
    
    /// Convert device path to UDisks2 path
    fn device_to_path(device: &str) -> OwnedObjectPath {
        let name = device.trim_start_matches("/dev/");
        let encoded = name.replace('/', "_").replace('-', "_");
        OwnedObjectPath::try_from(format!("/org/freedesktop/UDisks2/block_devices/{}", encoded))
            .unwrap_or_else(|_| OwnedObjectPath::try_from("/org/freedesktop/UDisks2/block_devices/sda1").unwrap())
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
    async fn list_encrypted_devices(
        &self,
        #[zbus(connection)] connection: &Connection,
    ) -> zbus::fdo::Result<String> {
        check_polkit_auth(connection, "org.cosmic.ext.storage-service.luks-read")
            .await
            .map_err(|e| zbus::fdo::Error::Failed(format!("Authorization failed: {e}")))?;
        
        tracing::debug!("Listing encrypted devices");
        
        // Get all drives and look for LUKS partitions
        let drives = disks_dbus::DriveModel::get_drives()
            .await
            .map_err(|e| {
                tracing::error!("Failed to get drives: {e}");
                zbus::fdo::Error::Failed(format!("Failed to enumerate drives: {e}"))
            })?;
        
        let mut luks_devices = Vec::new();
        
        for drive in drives {
            for volume in &drive.volumes_flat {
                // Check if this is a LUKS volume
                if volume.id_type == "crypto_LUKS" {
                    let device = volume.device_path.clone().unwrap_or_default();
                    
                    // Try to get encryption details
                    if let Ok(encrypted_proxy) = EncryptedProxy::builder(connection)
                        .path(&volume.path)?
                        .build()
                        .await
                    {
                        // Check if unlocked
                        let cleartext = encrypted_proxy.cleartext_device().await.ok();
                        let unlocked = cleartext.is_some() && !cleartext.as_ref().unwrap().as_str().is_empty();
                        
                        let cleartext_device = if unlocked {
                            cleartext.and_then(|p| {
                                // Convert path to device
                                let name = p.as_str().trim_start_matches("/org/freedesktop/UDisks2/block_devices/");
                                Some(format!("/dev/{}", name.replace('_', "/")))
                            })
                        } else {
                            None
                        };
                        
                        // Get LUKS version and cipher info from block proxy
                        if let Ok(block_proxy) = BlockProxy::builder(connection)
                            .path(&volume.path)?
                            .build()
                            .await
                        {
                            let id_version = block_proxy.id_version().await.unwrap_or_default();
                            let version = if id_version.contains('2') {
                                LuksVersion::Luks2
                            } else {
                                LuksVersion::Luks1
                            };
                            
                            // Get crypto properties
                            let cipher = String::from("aes-xts-plain64"); // Default, UDisks2 doesn't expose this easily
                            let key_size = 256; // Default
                            let keyslot_count = 8; // LUKS default
                            
                            luks_devices.push(LuksInfo {
                                device,
                                version,
                                cipher,
                                key_size,
                                unlocked,
                                cleartext_device,
                                keyslot_count,
                            });
                        }
                    }
                }
            }
        }
        
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
    async fn format(
        &self,
        #[zbus(connection)] connection: &Connection,
        #[zbus(signal_context)] signal_ctx: zbus::object_server::SignalEmitter<'_>,
        device: String,
        passphrase: String,
        version: String,
    ) -> zbus::fdo::Result<()> {
        check_polkit_auth(connection, "org.cosmic.ext.storage-service.luks-format")
            .await
            .map_err(|e| zbus::fdo::Error::Failed(format!("Authorization failed: {e}")))?;
        
        tracing::info!("Formatting device '{}' as LUKS", device);
        
        // Validate version
        let luks_version = if version.is_empty() || version == "luks2" {
            "luks2"
        } else if version == "luks1" {
            "luks1"
        } else {
            return Err(zbus::fdo::Error::InvalidArgs(
                format!("Invalid LUKS version: {}. Use 'luks1' or 'luks2'", version)
            ));
        };
        
        let path = Self::device_to_path(&device);
        
        // Get block proxy
        let block_proxy = BlockProxy::builder(connection)
            .path(&path)?
            .build()
            .await
            .map_err(|e| {
                tracing::error!("Failed to get block device: {e}");
                zbus::fdo::Error::Failed(format!("Failed to get block device: {e}"))
            })?;
        
        // Format as LUKS
        let mut options: HashMap<&str, Value<'_>> = HashMap::new();
        options.insert("encrypt.passphrase", Value::new(passphrase));
        
        block_proxy.format(luks_version, options)
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
    async fn unlock(
        &self,
        #[zbus(connection)] connection: &Connection,
        #[zbus(signal_context)] signal_ctx: zbus::object_server::SignalEmitter<'_>,
        device: String,
        passphrase: String,
    ) -> zbus::fdo::Result<String> {
        check_polkit_auth(connection, "org.cosmic.ext.storage-service.luks-unlock")
            .await
            .map_err(|e| zbus::fdo::Error::Failed(format!("Authorization failed: {e}")))?;
        
        tracing::info!("Unlocking LUKS device '{}'", device);
        
        let path = Self::device_to_path(&device);
        
        // Get encrypted proxy
        let encrypted_proxy = EncryptedProxy::builder(connection)
            .path(&path)?
            .build()
            .await
            .map_err(|e| {
                tracing::error!("Failed to get encrypted device: {e}");
                zbus::fdo::Error::Failed(format!("Failed to get encrypted device: {e}"))
            })?;
        
        // Unlock
        let options: HashMap<&str, Value<'_>> = HashMap::new();
        let cleartext_path = encrypted_proxy.unlock(&passphrase, options)
            .await
            .map_err(|e| {
                tracing::error!("Unlock failed: {e}");
                zbus::fdo::Error::Failed(format!("Unlock failed: {e}"))
            })?;
        
        // Convert cleartext path to device path
        let cleartext_device = Self::path_to_device(connection, &cleartext_path).await?;
        
        tracing::info!("LUKS device '{}' unlocked to '{}'", device, cleartext_device);
        let _ = Self::container_unlocked(&signal_ctx, &device, &cleartext_device).await;
        Ok(cleartext_device)
    }
    
    /// Lock (close) a LUKS encrypted device
    /// 
    /// Args:
    /// - device: Device path (e.g., "/dev/sda1")
    /// 
    /// Authorization: org.cosmic.ext.storage-service.luks-lock (allow_active)
    async fn lock(
        &self,
        #[zbus(connection)] connection: &Connection,
        #[zbus(signal_context)] signal_ctx: zbus::object_server::SignalEmitter<'_>,
        device: String,
    ) -> zbus::fdo::Result<()> {
        check_polkit_auth(connection, "org.cosmic.ext.storage-service.luks-lock")
            .await
            .map_err(|e| zbus::fdo::Error::Failed(format!("Authorization failed: {e}")))?;
        
        tracing::info!("Locking LUKS device '{}'", device);
        
        let path = Self::device_to_path(&device);
        
        // Get encrypted proxy
        let encrypted_proxy = EncryptedProxy::builder(connection)
            .path(&path)?
            .build()
            .await
            .map_err(|e| {
                tracing::error!("Failed to get encrypted device: {e}");
                zbus::fdo::Error::Failed(format!("Failed to get encrypted device: {e}"))
            })?;
        
        // Lock
        let options: HashMap<&str, Value<'_>> = HashMap::new();
        encrypted_proxy.lock(options)
            .await
            .map_err(|e| {
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
    async fn change_passphrase(
        &self,
        #[zbus(connection)] connection: &Connection,
        device: String,
        current_passphrase: String,
        new_passphrase: String,
    ) -> zbus::fdo::Result<()> {
        check_polkit_auth(connection, "org.cosmic.ext.storage-service.luks-modify")
            .await
            .map_err(|e| zbus::fdo::Error::Failed(format!("Authorization failed: {e}")))?;
        
        tracing::info!("Changing passphrase for LUKS device '{}'", device);
        
        let path = Self::device_to_path(&device);
        
        // Get encrypted proxy
        let encrypted_proxy = EncryptedProxy::builder(connection)
            .path(&path)?
            .build()
            .await
            .map_err(|e| {
                tracing::error!("Failed to get encrypted device: {e}");
                zbus::fdo::Error::Failed(format!("Failed to get encrypted device: {e}"))
            })?;
        
        // Change passphrase
        let options: HashMap<&str, Value<'_>> = HashMap::new();
        encrypted_proxy.change_passphrase(&current_passphrase, &new_passphrase, options)
            .await
            .map_err(|e| {
                tracing::error!("Change passphrase failed: {e}");
                zbus::fdo::Error::Failed(format!("Change passphrase failed: {e}"))
            })?;
        
        tracing::info!("Passphrase changed successfully for LUKS device '{}'", device);
        
        Ok(())
    }
    
    /// Get encryption options (crypttab settings) for a LUKS device
    ///
    /// Returns: JSON-serialized Option<EncryptionOptionsSettings> ("null" if none)
    ///
    /// Authorization: org.cosmic.ext.storage-service.luks-read (allow_active)
    async fn get_encryption_options(
        &self,
        #[zbus(connection)] connection: &Connection,
        device: String,
    ) -> zbus::fdo::Result<String> {
        check_polkit_auth(connection, "org.cosmic.ext.storage-service.luks-read")
            .await
            .map_err(|e| zbus::fdo::Error::Failed(format!("Authorization failed: {e}")))?;

        let drives = disks_dbus::DriveModel::get_drives()
            .await
            .map_err(|e| {
                tracing::error!("Failed to get drives: {e}");
                zbus::fdo::Error::Failed(format!("Failed to enumerate drives: {e}"))
            })?;

        for drive in drives {
            for vol in &drive.volumes_flat {
                if vol.device_path.as_deref() == Some(device.as_str()) {
                    match vol.get_encryption_options_settings().await {
                        Ok(Some(s)) => {
                            let out = EncryptionOptionsSettings {
                                name: s.name,
                                unlock_at_startup: s.unlock_at_startup,
                                require_auth: s.require_auth,
                                other_options: s.other_options,
                                passphrase: None,
                            };
                            return serde_json::to_string(&Some(out))
                                .map_err(|e| zbus::fdo::Error::Failed(format!("Serialize: {e}")));
                        }
                        Ok(None) => return Ok("null".to_string()),
                        Err(e) => {
                            tracing::warn!("get_encryption_options_settings failed: {e}");
                            return Ok("null".to_string());
                        }
                    }
                }
            }
        }
        Ok("null".to_string())
    }

    /// Set encryption options (crypttab) for a LUKS device
    ///
    /// Args:
    /// - device: Device path (e.g. "/dev/sda1")
    /// - options_json: JSON-serialized EncryptionOptionsSettings (name, unlock_at_startup, require_auth, other_options, optional passphrase)
    ///
    /// Authorization: org.cosmic.ext.storage-service.luks-set-options
    async fn set_encryption_options(
        &self,
        #[zbus(connection)] connection: &Connection,
        device: String,
        options_json: String,
    ) -> zbus::fdo::Result<()> {
        check_polkit_auth(connection, "org.cosmic.ext.storage-service.luks-set-options")
            .await
            .map_err(|e| zbus::fdo::Error::Failed(format!("Authorization failed: {e}")))?;

        let settings: EncryptionOptionsSettings = serde_json::from_str(&options_json)
            .map_err(|e| zbus::fdo::Error::InvalidArgs(format!("Invalid options JSON: {e}")))?;

        if settings.name.trim().is_empty() {
            return Err(zbus::fdo::Error::InvalidArgs("Name must not be empty".to_string()));
        }

        let block_path = Self::device_to_path(&device);
        let block_proxy = BlockProxy::builder(connection)
            .path(&block_path)
            .map_err(|e| zbus::fdo::Error::Failed(format!("Failed to create block proxy: {e}")))?
            .build()
            .await
            .map_err(|e| zbus::fdo::Error::Failed(format!("Failed to access block device: {e}")))?;

        let id_uuid = block_proxy.id_uuid().await
            .map_err(|e| zbus::fdo::Error::Failed(format!("Failed to get block UUID: {e}")))?
            .trim()
            .to_string();
        if id_uuid.is_empty() {
            return Err(zbus::fdo::Error::Failed(
                "Missing block UUID; cannot write crypttab entry".to_string(),
            ));
        }
        let device_uuid = format!("UUID={id_uuid}");

        let mut tokens = split_options(&settings.other_options);
        tokens = set_token_present(tokens, "noauto", !settings.unlock_at_startup);
        tokens = set_token_present(tokens, "x-udisks-auth", settings.require_auth);
        let options = join_options(&stable_dedup(tokens));

        let config_proxy = UDisks2BlockConfigurationProxy::builder(connection)
            .path(&block_path)
            .map_err(|e| zbus::fdo::Error::Failed(format!("Failed to create config proxy: {e}")))?
            .build()
            .await
            .map_err(|e| zbus::fdo::Error::Failed(format!("Failed to access block configuration: {e}")))?;

        let items = config_proxy.configuration().await
            .map_err(|e| zbus::fdo::Error::Failed(format!("Failed to get configuration: {e}")))?;
        let old_item = items.iter().find(|(t, _)| t == "crypttab").cloned();

        let existing_passphrase_path = old_item
            .as_ref()
            .and_then(|(_, d)| d.get("passphrase-path"))
            .and_then(owned_value_to_bytestring)
            .unwrap_or_default();

        let passphrase = settings.passphrase.as_deref().unwrap_or("");
        let (passphrase_path, passphrase_contents) = if passphrase.is_empty() {
            (String::new(), String::new())
        } else if !existing_passphrase_path.is_empty() && !existing_passphrase_path.starts_with("/dev") {
            (existing_passphrase_path, passphrase.to_string())
        } else {
            (format!("/etc/luks-keys/{}", settings.name.trim()), passphrase.to_string())
        };

        let mut dict: HashMap<String, OwnedValue> = HashMap::new();
        dict.insert("device".to_string(), bytestring_owned_value(&device_uuid));
        dict.insert("name".to_string(), bytestring_owned_value(settings.name.trim()));
        dict.insert("options".to_string(), bytestring_owned_value(options.trim()));
        dict.insert("passphrase-path".to_string(), bytestring_owned_value(&passphrase_path));
        dict.insert("passphrase-contents".to_string(), bytestring_owned_value(&passphrase_contents));

        let new_item: ConfigurationItem = ("crypttab".to_string(), dict);
        let empty = HashMap::new();

        if let Some(old_item) = old_item {
            config_proxy
                .update_configuration_item(old_item, new_item, empty)
                .await
                .map_err(|e| zbus::fdo::Error::Failed(format!("Failed to update crypttab: {e}")))?;
        } else {
            config_proxy
                .add_configuration_item(new_item, empty)
                .await
                .map_err(|e| zbus::fdo::Error::Failed(format!("Failed to add crypttab entry: {e}")))?;
        }

        tracing::info!("Set encryption options for LUKS device '{}'", device);
        Ok(())
    }

    /// Clear encryption options (remove crypttab entry) for a LUKS device
    ///
    /// Authorization: org.cosmic.ext.storage-service.luks-set-options
    async fn default_encryption_options(
        &self,
        #[zbus(connection)] connection: &Connection,
        device: String,
    ) -> zbus::fdo::Result<()> {
        check_polkit_auth(connection, "org.cosmic.ext.storage-service.luks-set-options")
            .await
            .map_err(|e| zbus::fdo::Error::Failed(format!("Authorization failed: {e}")))?;

        let block_path = Self::device_to_path(&device);
        let config_proxy = UDisks2BlockConfigurationProxy::builder(connection)
            .path(&block_path)
            .map_err(|e| zbus::fdo::Error::Failed(format!("Failed to create config proxy: {e}")))?
            .build()
            .await
            .map_err(|e| zbus::fdo::Error::Failed(format!("Failed to access block configuration: {e}")))?;

        let items = config_proxy.configuration().await
            .map_err(|e| zbus::fdo::Error::Failed(format!("Failed to get configuration: {e}")))?;
        if let Some(old_item) = items.iter().find(|(t, _)| t == "crypttab").cloned() {
            config_proxy
                .remove_configuration_item(old_item, HashMap::new())
                .await
                .map_err(|e| zbus::fdo::Error::Failed(format!("Failed to remove crypttab entry: {e}")))?;
        }
        tracing::info!("Cleared encryption options for LUKS device '{}'", device);
        Ok(())
    }

    // NOTE: add_key and remove_key are not available in UDisks2 EncryptedProxy
    // These would need to be implemented via direct cryptsetup luksAddKey/luksRemoveKey commands
    // or via raw D-Bus method calls if UDisks2 exposes them under different names.
    // For now, users can use change_passphrase to update their passphrase.
}
