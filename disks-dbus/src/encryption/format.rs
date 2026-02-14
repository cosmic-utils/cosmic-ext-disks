// SPDX-License-Identifier: GPL-3.0-only

//! LUKS encryption / formatting operations

use std::collections::HashMap;
use udisks2::block::BlockProxy;
use zbus::{Connection, zvariant::{OwnedObjectPath, Value}};
use crate::error::DiskError;

/// Format a device as LUKS encrypted container
pub async fn format_luks(
    device_path: &str,
    passphrase: &str,
    version: &str,
) -> Result<(), DiskError> {
    let connection = Connection::system().await
        .map_err(|e| DiskError::ConnectionFailed(e.to_string()))?;
    
    // Find block device object path  
    let drives = crate::disk::model::DriveModel:: get_drives().await
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
