// SPDX-License-Identifier: GPL-3.0-only

//! LUKS unlocking operations

use std::collections::HashMap;
use udisks2::encrypted::EncryptedProxy;
use zbus::{Connection, zvariant::{OwnedObjectPath, Value}};
use crate::error::DiskError;

/// Unlock a LUKS container
///
/// Returns the cleartext device path (e.g., "/dev/mapper/luks-...")
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
