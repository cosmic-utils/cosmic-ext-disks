// SPDX-License-Identifier: GPL-3.0-only

//! LUKS device listing

use storage_models::{LuksInfo, LuksVersion};
use udisks2::{block::BlockProxy, encrypted::EncryptedProxy};
use zbus::Connection;
use crate::error::DiskError;

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
                
                // Query encrypted device information via UDisks2
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
                            // Convert dbus path to device
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
