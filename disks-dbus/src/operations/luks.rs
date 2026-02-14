// SPDX-License-Identifier: GPL-3.0-only

//! LUKS encryption operations via UDisks2

use std::collections::HashMap;
use udisks2::{block::BlockProxy, encrypted::EncryptedProxy};
use zbus::{Connection, zvariant::{OwnedObjectPath, Value}};
use crate::disks::DiskError;
use storage_models::{LuksInfo, LuksVersion};

/// Unlock a LUKS container
pub async fn unlock_luks(
    device_path: &str,
    passphrase: &str,
) -> Result<String, DiskError> {
    let connection = Connection::system().await
        .map_err(|e| DiskError::ConnectionFailed(e.to_string()))?;
    
    // Find encrypted device object path
    let drives = crate::DriveModel::get_drives().await
        .map_err(|e| DiskError::OperationFailed(format!("Failed to get drives: {}", e)))?;
    let mut encrypted_path: Option<OwnedObjectPath> = None;
    
    for drive in drives {
        for volume in &drive.volumes {
            if let Some(ref dev_path) = volume.device_path {
                if dev_path == device_path {
                    encrypted_path = Some(volume.object_path.clone());
                    break;
                }
            }
        }
    }
    
    let encrypted_path = encrypted_path.ok_or_else(|| 
        DiskError::DeviceNotFound(device_path.to_string()))?;
    
    let encrypted_proxy = EncryptedProxy::builder(&connection)
        .path(&encrypted_path)
        .map_err(|e| DiskError::DBusError(e.to_string()))?
        .build()
        .await
        .map_err(|e| DiskError::DBusError(e.to_string()))?;
    
    let opts: HashMap<&str, Value<'_>> = HashMap::new();
    let cleartext_path = encrypted_proxy.unlock(passphrase, opts).await
        .map_err(|e| DiskError::OperationFailed(format!("Unlock failed: {}", e)))?;
    
    Ok(cleartext_path.to_string())
}

/// Lock a LUKS container
pub async fn lock_luks(device_path: &str) -> Result<(), DiskError> {
    let connection = Connection::system().await
        .map_err(|e| DiskError::ConnectionFailed(e.to_string()))?;
    
    // Find encrypted device object path
    let drives = crate::DriveModel::get_drives().await
        .map_err(|e| DiskError::OperationFailed(format!("Failed to get drives: {}", e)))?;
    let mut encrypted_path: Option<OwnedObjectPath> = None;
    
    for drive in drives {
        for volume in &drive.volumes {
            if let Some(ref dev_path) = volume.device_path {
                if dev_path == device_path {
                    encrypted_path = Some(volume.object_path.clone());
                    break;
                }
            }
        }
    }
    
    let encrypted_path = encrypted_path.ok_or_else(|| 
        DiskError::DeviceNotFound(device_path.to_string()))?;
    
    let encrypted_proxy = EncryptedProxy::builder(&connection)
        .path(&encrypted_path)
        .map_err(|e| DiskError::DBusError(e.to_string()))?
        .build()
        .await
        .map_err(|e| DiskError::DBusError(e.to_string()))?;
    
    let opts: HashMap<&str, Value<'_>> = HashMap::new();
    encrypted_proxy.lock(opts).await
        .map_err(|e| DiskError::OperationFailed(format!("Lock failed: {}", e)))?;
    
    Ok(())
}

/// Change LUKS passphrase
pub async fn change_luks_passphrase(
    device_path: &str,
    old_passphrase: &str,
    new_passphrase: &str,
) -> Result<(), DiskError> {
    let connection = Connection::system().await
        .map_err(|e| DiskError::ConnectionFailed(e.to_string()))?;
    
    // Find encrypted device object path
    let drives = crate::DriveModel::get_drives().await
        .map_err(|e| DiskError::OperationFailed(format!("Failed to get drives: {}", e)))?;
    let mut encrypted_path: Option<OwnedObjectPath> = None;
    
    for drive in drives {
        for volume in &drive.volumes {
            if let Some(ref dev_path) = volume.device_path {
                if dev_path == device_path {
                    encrypted_path = Some(volume.object_path.clone());
                    break;
                }
            }
        }
    }
    
    let encrypted_path = encrypted_path.ok_or_else(|| 
        DiskError::DeviceNotFound(device_path.to_string()))?;
    
    let encrypted_proxy = EncryptedProxy::builder(&connection)
        .path(&encrypted_path)
        .map_err(|e| DiskError::DBusError(e.to_string()))?
        .build()
        .await
        .map_err(|e| DiskError::DBusError(e.to_string()))?;
    
    let opts: HashMap<&str, Value<'_>> = HashMap::new();
    encrypted_proxy.change_passphrase(old_passphrase, new_passphrase, opts).await
        .map_err(|e| DiskError::OperationFailed(format!("Change passphrase failed: {}", e)))?;
    
    Ok(())
}

/// Format a device as LUKS encrypted container
pub async fn format_luks(
    device_path: &str,
    passphrase: &str,
    version: &str,
) -> Result<(), DiskError> {
    let connection = Connection::system().await
        .map_err(|e| DiskError::ConnectionFailed(e.to_string()))?;
    
    // Find block device object path  
    let drives = crate::DriveModel::get_drives().await
        .map_err(|e| DiskError::OperationFailed(format!("Failed to get drives: {}", e)))?;
    let mut block_path: Option<OwnedObjectPath> = None;
    
    for drive in drives {
        for volume in &drive.volumes {
            if let Some(ref dev_path) = volume.device_path {
                if dev_path == device_path {
                    block_path = Some(volume.object_path.clone());
                    break;
                }
            }
        }
    }
    
    let block_path = block_path.ok_or_else(|| 
        DiskError::DeviceNotFound(device_path.to_string()))?;
    
    let block_proxy = BlockProxy::builder(&connection)
        .path(&block_path)
        .map_err(|e| DiskError::DBusError(e.to_string()))?
        .build()
        .await
        .map_err(|e| DiskError::DBusError(e.to_string()))?;
    
    // Validate and use LUKS version
    let luks_type = if version == "luks1" {
        "luks1"
    } else {
        "luks2" // Default to luks2
    };
    
    let mut options: HashMap<&str, Value<'_>> = HashMap::new();
    options.insert("encrypt.passphrase", Value::from(passphrase));
    
    block_proxy.format(luks_type, options).await
        .map_err(|e| DiskError::OperationFailed(format!("Format failed: {}", e)))?;
    
    Ok(())
}

/// List all LUKS encrypted devices
pub async fn list_luks_devices() -> Result<Vec<LuksInfo>, DiskError> {
    let connection = Connection::system().await
        .map_err(|e| DiskError::ConnectionFailed(e.to_string()))?;
    
    let drives = crate::DriveModel::get_drives().await
        .map_err(|e| DiskError::OperationFailed(format!("Failed to get drives: {}", e)))?;
    
    let mut luks_devices = Vec::new();
    
    for drive in drives {
        for volume in &drive.volumes_flat {
            // Check if this is a LUKS volume
            if volume.id_type == "crypto_LUKS" {
                let device = volume.device_path.clone().unwrap_or_default();
                
                // Try to get encryption details
                if let Ok(encrypted_proxy) = EncryptedProxy::builder(&connection)
                    .path(&volume.path)
                    .map_err(|e| DiskError::DBusError(e.to_string()))?
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
                    
                    // Get LUKS version from block proxy
                    if let Ok(block_proxy) = BlockProxy::builder(&connection)
                        .path(&volume.path)
                        .map_err(|e| DiskError::DBusError(e.to_string()))?
                        .build()
                        .await
                    {
                        let id_version = block_proxy.id_version().await.unwrap_or_default();
                        let version = if id_version.contains('2') {
                            LuksVersion::Luks2
                        } else {
                            LuksVersion::Luks1
                        };
                        
                        // Get crypto properties (defaults, UDisks2 doesn't expose these easily)
                        let cipher = String::from("aes-xts-plain64");
                        let key_size = 256;
                        let keyslot_count = 8;
                        
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
    
    Ok(luks_devices)
}
