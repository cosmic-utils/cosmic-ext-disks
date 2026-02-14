// SPDX-License-Identifier: GPL-3.0-only

//! Filesystem ownership management

use std::collections::HashMap;
use udisks2::filesystem::FilesystemProxy;
use zbus::{Connection, zvariant::{OwnedObjectPath, Value}};
use crate::error::DiskError;

/// Helper function to find UDisks2 block object path for a device
async fn find_block_object_path(device_path: &str) -> Result<OwnedObjectPath, DiskError> {
    let drives = crate::disk::model::DriveModel::get_drives().await
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

/// Take ownership of a mounted filesystem
///
/// # Arguments
/// * `device` - Device path (e.g., "/dev/sda1")
/// * `recursive` - Take ownership of child mounts
pub async fn take_filesystem_ownership(device: &str, recursive: bool) -> Result<(), DiskError> {
    let connection = Connection::system()
        .await
        .map_err(|e| DiskError::ConnectionFailed(format!("Failed to connect to system bus: {}", e)))?;
    
    let block_path = find_block_object_path(device).await?;
    
    let fs_proxy = FilesystemProxy::builder(&connection)
        .path(&block_path)
        .map_err(|e| DiskError::InvalidPath(format!("Invalid filesystem path: {}", e)))?
        .build()
        .await
        .map_err(|e| DiskError::DBusError(e.to_string()))?;
    
    let mut options: HashMap<&str, Value<'_>> = HashMap::new();
    options.insert("recursive", Value::from(recursive));
    
    fs_proxy
        .take_ownership(options)
        .await
        .map_err(|e| DiskError::OperationFailed(format!("Take ownership failed: {}", e)))?;
    
    Ok(())
}
