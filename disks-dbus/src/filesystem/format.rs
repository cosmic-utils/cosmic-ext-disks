// SPDX-License-Identifier: GPL-3.0-only

//! Filesystem formatting operations

use std::collections::HashMap;
use udisks2::block::BlockProxy;
use zbus::{Connection, zvariant::{OwnedObjectPath, Value}};
use crate::error::DiskError;
use storage_models::FormatOptions;

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

/// Format a filesystem
pub async fn format_filesystem(
    device_path: &str,
    fs_type: &str,
    label: &str,
    _options: FormatOptions,
) -> Result<(), DiskError> {
    let connection = Connection::system().await
        .map_err(|e| DiskError::ConnectionFailed(e.to_string()))?;
    
    // Find block device object path
    let block_path = find_block_object_path(device_path).await?;
    
    // Format using Block.Format
    let block_proxy = BlockProxy::builder(&connection)
        .path(&block_path)
        .map_err(|e| DiskError::DBusError(e.to_string()))?
        .build()
        .await
        .map_err(|e| DiskError::DBusError(e.to_string()))?;
    
    let mut format_opts: HashMap<&str, Value<'_>> = HashMap::new();
    if !label.is_empty() {
        format_opts.insert("label", Value::from(label));
    }
    
    block_proxy.format(fs_type, format_opts).await
        .map_err(|e| DiskError::OperationFailed(format!("Format failed: {}", e)))?;
    
    Ok(())
}
