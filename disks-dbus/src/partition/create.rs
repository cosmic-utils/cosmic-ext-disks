// SPDX-License-Identifier: GPL-3.0-only

//! Partition creation operations

use std::collections::HashMap;
use udisks2::{
    block::BlockProxy,
    partitiontable::PartitionTableProxy,
};
use zbus::{Connection, zvariant::{OwnedObjectPath, Value}};
use crate::disks::DiskError;

/// Create a partition table on a disk
pub async fn create_partition_table(
    disk_path: &str,
    table_type: &str,
) -> Result<(), DiskError> {
    let connection = Connection::system().await
        .map_err(|e| DiskError::ConnectionFailed(e.to_string()))?;
    
    // Find the drive by device path
    let drives = crate::DriveModel::get_drives().await
        .map_err(|e| DiskError::OperationFailed(format!("Failed to get drives: {}", e)))?;
    let drive = drives.into_iter()
        .find(|d| d.block_path == disk_path)
        .ok_or_else(|| DiskError::DeviceNotFound(disk_path.to_string()))?;
    
    // Use the drive's format_disk method
    drive.format_disk(table_type, false).await
        .map_err(|e| DiskError::OperationFailed(format!("Format disk failed: {}", e)))?;
    
    Ok(())
}

/// Create a partition
pub async fn create_partition(
    disk_path: &str,
    offset: u64,
    size: u64,
    type_id: &str,
) -> Result<String, DiskError> {
    let connection = Connection::system().await
        .map_err(|e| DiskError::ConnectionFailed(e.to_string()))?;
    
    let block_path: OwnedObjectPath = disk_path.try_into()
        .map_err(|e| DiskError::InvalidPath(format!("Invalid device path: {}", e)))?;
    
    // Create partition table proxy
    let table_proxy = PartitionTableProxy::builder(&connection)
        .path(&block_path)
        .map_err(|e| DiskError::DBusError(e.to_string()))?
        .build()
        .await
        .map_err(|e| DiskError::DBusError(e.to_string()))?;
    
    // Create partition
    let options: HashMap<&str, Value<'_>> = HashMap::new();
    let partition_path = table_proxy
        .create_partition(offset, size, type_id, "", options)
        .await
        .map_err(|e| DiskError::OperationFailed(format!("Create partition failed: {}", e)))?;
    
    // Get device path
    let block_proxy = BlockProxy::builder(&connection)
        .path(&partition_path)
        .map_err(|e| DiskError::DBusError(e.to_string()))?
        .build()
        .await
        .map_err(|e| DiskError::DBusError(e.to_string()))?;
    
    let device_bytes = block_proxy.preferred_device().await
        .map_err(|e| DiskError::DBusError(e.to_string()))?;
    
    let device_path = String::from_utf8(device_bytes.into_iter().filter(|&b| b != 0).collect())
        .unwrap_or_else(|_| format!("/dev/{}", partition_path.as_str().rsplit('/').next().unwrap_or("unknown")));
    
    Ok(device_path)
}
