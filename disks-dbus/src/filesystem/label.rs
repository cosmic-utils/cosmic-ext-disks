// SPDX-License-Identifier: GPL-3.0-only

//! Filesystem label management

use std::collections::HashMap;
use udisks2::{block::BlockProxy, filesystem::FilesystemProxy};
use zbus::{Connection, zvariant::{OwnedObjectPath, Value}};
use crate::disks::DiskError;

/// Helper function to find UDisks2 block object path for a device
async fn find_block_object_path(device_path: &str) -> Result<OwnedObjectPath, DiskError> {
    let drives = crate::DriveModel::get_drives().await
        .map_err(|e| DiskError::OperationFailed(format!("Failed to get drives: {}", e)))?;
    
    for drive in drives {
        for volume in &drive.volumes {
            if let Some(ref dev_path) = volume.device_path {
                if dev_path == device_path {
                    return Ok(volume.object_path.clone());
                }
            }
        }
    }
    
    Err(DiskError::DeviceNotFound(device_path.to_string()))
}

/// Get filesystem label for a device
///
/// # Arguments
/// * `device` - Device path (e.g., "/dev/sda1")
///
/// # Returns
/// The filesystem label (may be empty string if no label set)
pub async fn get_filesystem_label(device: &str) -> Result<String, DiskError> {
    let connection = Connection::system()
        .await
        .map_err(|e| DiskError::ConnectionFailed(format!("Failed to connect to system bus: {}", e)))?;
    
    let block_path = find_block_object_path(device).await?;
    
    let block_proxy = BlockProxy::builder(&connection)
        .path(&block_path)
        .map_err(|e| DiskError::InvalidPath(format!("Invalid block path: {}", e)))?
        .build()
        .await
        .map_err(|e| DiskError::DBusError(e.to_string()))?;
    
    let label = block_proxy.id_label().await
        .unwrap_or_default();
    
    Ok(label)
}

/// Set filesystem label
pub async fn set_filesystem_label(device_path: &str, label: &str) -> Result<(), DiskError> {
    let connection = Connection::system().await
        .map_err(|e| DiskError::ConnectionFailed(e.to_string()))?;
    
    // Find filesystem object path
    let fs_path = find_block_object_path(device_path).await?;
    
    let fs_proxy = FilesystemProxy::builder(&connection)
        .path(&fs_path)
        .map_err(|e| DiskError::DBusError(e.to_string()))?
        .build()
        .await
        .map_err(|e| DiskError::DBusError(e.to_string()))?;
    
    let opts: HashMap<&str, Value<'_>> = HashMap::new();
    fs_proxy.set_label(label, opts).await
        .map_err(|e| DiskError::OperationFailed(format!("Set label failed: {}", e)))?;
    
    Ok(())
}
