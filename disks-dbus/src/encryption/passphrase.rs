// SPDX-License-Identifier: GPL-3.0-only

//! LUKS passphrase management

use std::collections::HashMap;
use udisks2::encrypted::EncryptedProxy;
use zbus::{Connection, zvariant::{OwnedObjectPath, Value}};
use crate::disks::DiskError;

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
