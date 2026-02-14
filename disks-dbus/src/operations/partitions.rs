// SPDX-License-Identifier: GPL-3.0-only

//! Partition management operations via UDisks2

use std::collections::HashMap;
use udisks2::{
    block::BlockProxy,
    partition::PartitionProxy,
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

/// Delete a partition
pub async fn delete_partition(partition_path: &str) -> Result<(), DiskError> {
    let connection = Connection::system().await
        .map_err(|e| DiskError::ConnectionFailed(e.to_string()))?;
    
    let obj_path: OwnedObjectPath = partition_path.try_into()
        .map_err(|e| DiskError::InvalidPath(format!("Invalid partition path: {}", e)))?;
    
    let partition_proxy = PartitionProxy::builder(&connection)
        .path(&obj_path)
        .map_err(|e| DiskError::DBusError(e.to_string()))?
        .build()
        .await
        .map_err(|e| DiskError::DBusError(e.to_string()))?;
    
    let options: HashMap<&str, Value<'_>> = HashMap::new();
    partition_proxy.delete(options).await
        .map_err(|e| DiskError::OperationFailed(format!("Delete partition failed: {}", e)))?;
    
    Ok(())
}

/// Resize a partition
pub async fn resize_partition(
    partition_path: &str,
    new_size: u64,
) -> Result<(), DiskError> {
    let connection = Connection::system().await
        .map_err(|e| DiskError::ConnectionFailed(e.to_string()))?;
    
    let obj_path: OwnedObjectPath = partition_path.try_into()
        .map_err(|e| DiskError::InvalidPath(format!("Invalid partition path: {}", e)))?;
    
    let partition_proxy = PartitionProxy::builder(&connection)
        .path(&obj_path)
        .map_err(|e| DiskError::DBusError(e.to_string()))?
        .build()
        .await
        .map_err(|e| DiskError::DBusError(e.to_string()))?;
    
    let options: HashMap<&str, Value<'_>> = HashMap::new();
    partition_proxy.resize(new_size, options).await
        .map_err(|e| DiskError::OperationFailed(format!("Resize partition failed: {}", e)))?;
    
    Ok(())
}

/// Set partition type
pub async fn set_partition_type(
    partition_path: &str,
    type_id: &str,
) -> Result<(), DiskError> {
    let connection = Connection::system().await
        .map_err(|e| DiskError::ConnectionFailed(e.to_string()))?;
    
    let obj_path: OwnedObjectPath = partition_path.try_into()
        .map_err(|e| DiskError::InvalidPath(format!("Invalid partition path: {}", e)))?;
    
    let partition_proxy = PartitionProxy::builder(&connection)
        .path(&obj_path)
        .map_err(|e| DiskError::DBusError(e.to_string()))?
        .build()
        .await
        .map_err(|e| DiskError::DBusError(e.to_string()))?;
    
    let options: HashMap<&str, Value<'_>> = HashMap::new();
    partition_proxy.set_type(type_id, options).await
        .map_err(|e| DiskError::OperationFailed(format!("Set partition type failed: {}", e)))?;
    
    Ok(())
}

/// Set partition flags
pub async fn set_partition_flags(
    partition_path: &str,
    flags: u64,
) -> Result<(), DiskError> {
    let connection = Connection::system().await
        .map_err(|e| DiskError::ConnectionFailed(e.to_string()))?;
    
    let obj_path: OwnedObjectPath = partition_path.try_into()
        .map_err(|e| DiskError::InvalidPath(format!("Invalid partition path: {}", e)))?;
    
    let partition_proxy = PartitionProxy::builder(&connection)
        .path(&obj_path)
        .map_err(|e| DiskError::DBusError(e.to_string()))?
        .build()
        .await
        .map_err(|e| DiskError::DBusError(e.to_string()))?;
    
    let options: HashMap<&str, Value<'_>> = HashMap::new();
    let flags_bitfield = enumflags2::BitFlags::from_bits_truncate(flags);
    partition_proxy.set_flags(flags_bitfield, options).await
        .map_err(|e| DiskError::OperationFailed(format!("Set partition flags failed: {}", e)))?;
    
    Ok(())
}

/// Set partition name (GPT only)
pub async fn set_partition_name(
    partition_path: &str,
    name: &str,
) -> Result<(), DiskError> {
    let connection = Connection::system().await
        .map_err(|e| DiskError::ConnectionFailed(e.to_string()))?;
    
    let obj_path: OwnedObjectPath = partition_path.try_into()
        .map_err(|e| DiskError::InvalidPath(format!("Invalid partition path: {}", e)))?;
    
    let partition_proxy = PartitionProxy::builder(&connection)
        .path(&obj_path)
        .map_err(|e| DiskError::DBusError(e.to_string()))?
        .build()
        .await
        .map_err(|e| DiskError::DBusError(e.to_string()))?;
    
    let options: HashMap<&str, Value<'_>> = HashMap::new();
    partition_proxy.set_name(name, options).await
        .map_err(|e| DiskError::OperationFailed(format!("Set partition name failed: {}", e)))?;
    
    Ok(())
}
