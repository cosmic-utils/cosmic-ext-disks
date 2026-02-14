// SPDX-License-Identifier: GPL-3.0-only

//! Partition management D-Bus interface
//!
//! This module provides D-Bus methods for managing disk partitions,
//! including creating/deleting partitions and partition tables.

use zbus::{interface, Connection};

use crate::auth::check_polkit_auth;

/// D-Bus interface for partition management operations
pub struct PartitionsHandler;

impl PartitionsHandler {
    /// Create a new PartitionsHandler
    pub fn new() -> Self {
        Self
    }
}

#[interface(name = "org.cosmic.ext.StorageService.Partitions")]
impl PartitionsHandler {
    /// Signal emitted when a partition is created
    /// 
    /// Args:
    /// - disk: Device path of the disk (e.g., "/dev/sda")
    /// - partition: Device path of the new partition (e.g., "/dev/sda1")
    /// - partition_info: JSON-serialized PartitionInfo
    #[zbus(signal)]
    async fn partition_created(
        signal_ctxt: &zbus::object_server::SignalEmitter<'_>,
        disk: &str,
        partition: &str,
        partition_info: &str,
    ) -> zbus::Result<()>;
    
    /// Signal emitted when a partition is deleted
    /// 
    /// Args:
    /// - disk: Device path of the disk (e.g., "/dev/sda")
    /// - partition: Device path of the deleted partition (e.g., "/dev/sda1")
    #[zbus(signal)]
    async fn partition_deleted(
        signal_ctxt: &zbus::object_server::SignalEmitter<'_>,
        disk: &str,
        partition: &str,
    ) -> zbus::Result<()>;
    
    /// Signal emitted when a partition is modified
    /// 
    /// Args:
    /// - disk: Device path of the disk (e.g., "/dev/sda")
    /// - partition: Device path of the modified partition (e.g., "/dev/sda1")
    /// - partition_info: JSON-serialized PartitionInfo
    #[zbus(signal)]
    async fn partition_modified(
        signal_ctxt: &zbus::object_server::SignalEmitter<'_>,
        disk: &str,
        partition: &str,
        partition_info: &str,
    ) -> zbus::Result<()>;
    
    /// Signal emitted when a partition table is created
    /// 
    /// Args:
    /// - disk: Device path of the disk (e.g., "/dev/sda")
    /// - table_type: Type of partition table ("gpt" or "dos")
    #[zbus(signal)]
    async fn partition_table_created(
        signal_ctxt: &zbus::object_server::SignalEmitter<'_>,
        disk: &str,
        table_type: &str,
    ) -> zbus::Result<()>;
    
    /// List all partitions on a disk
    /// 
    /// Args:
    /// - disk: Device identifier (e.g., "/dev/sda", "sda", or UDisks2 path)
    /// 
    /// Returns: JSON-serialized Vec<PartitionInfo>
    /// 
    /// Authorization: org.cosmic.ext.storage-service.partition-read (allow_active)
    async fn list_partitions(
        &self,
        #[zbus(connection)] connection: &Connection,
        disk: String,
    ) -> zbus::fdo::Result<String> {
        // Check authorization
        check_polkit_auth(connection, "org.cosmic.ext.storage-service.partition-read")
            .await
            .map_err(|e| zbus::fdo::Error::Failed(format!("Authorization failed: {e}")))?;
        
        tracing::debug!("Listing partitions for disk: {disk}");
        
        // Get all drives and find the requested one
        let drives = disks_dbus::DriveModel::get_drives()
            .await
            .map_err(|e| {
                tracing::error!("Failed to get drives: {e}");
                zbus::fdo::Error::Failed(format!("Failed to enumerate drives: {e}"))
            })?;
        
        // Extract device name from input (strip "/dev/" prefix if present)
        let device_name = disk.strip_prefix("/dev/").unwrap_or(&disk);
        
        // Find the matching drive
        let drive_model = drives
            .into_iter()
            .find(|d| {
                let disk_info: storage_models::DiskInfo = d.clone().into();
                if disk_info.device == disk {
                    return true;
                }
                if let Some(disk_name) = disk_info.device.rsplit('/').next() {
                    if disk_name == device_name {
                        return true;
                    }
                }
                if disk_info.id == disk || disk_info.id == device_name {
                    return true;
                }
                false
            })
            .ok_or_else(|| {
                tracing::warn!("Device not found: {disk}");
                zbus::fdo::Error::Failed(format!("Device not found: {disk}"))
            })?;
        
        // Get partitions for this drive
        let partitions = drive_model.get_partitions();
        
        tracing::debug!("Found {} partitions for {}", partitions.len(), disk);
        
        // Serialize to JSON
        let json = serde_json::to_string(&partitions)
            .map_err(|e| {
                tracing::error!("Failed to serialize partitions: {e}");
                zbus::fdo::Error::Failed(format!("Failed to serialize partitions: {e}"))
            })?;
        
        Ok(json)
    }
    
    /// Create a new partition table (destroys all existing partitions!)
    /// 
    /// Args:
    /// - disk: Device identifier (e.g., "/dev/sda", "sda")
    /// - table_type: Type of partition table ("gpt" or "dos"/"mbr")
    /// 
    /// Authorization: org.cosmic.ext.storage-service.partition-modify (auth_admin_keep)
    async fn create_partition_table(
        &self,
        #[zbus(connection)] connection: &Connection,
        disk: String,
        table_type: String,
    ) -> zbus::fdo::Result<()> {
        // Check authorization (requires admin password)
        check_polkit_auth(connection, "org.cosmic.ext.storage-service.partition-modify")
            .await
            .map_err(|e| zbus::fdo::Error::Failed(format!("Authorization failed: {e}")))?;
        
        tracing::info!("Creating {} partition table on disk: {}", table_type, disk);
        
        // Validate and normalize table type
        let normalized_type = match table_type.to_lowercase().as_str() {
            "gpt" => "gpt",
            "dos" | "mbr" | "msdos" => "dos",
            _ => {
                tracing::warn!("Invalid partition table type: {table_type}");
                return Err(zbus::fdo::Error::InvalidArgs(
                    format!("Invalid table type: {table_type}. Must be 'gpt' or 'dos'/'mbr'")
                ));
            }
        };
        
        // Get all drives and find the requested one
        let drives = disks_dbus::DriveModel::get_drives()
            .await
            .map_err(|e| {
                tracing::error!("Failed to get drives: {e}");
                zbus::fdo::Error::Failed(format!("Failed to enumerate drives: {e}"))
            })?;
        
        // Extract device name from input (strip "/dev/" prefix if present)
        let device_name = disk.strip_prefix("/dev/").unwrap_or(&disk);
        
        // Find the matching drive
        let drive_model = drives
            .into_iter()
            .find(|d| {
                let disk_info: storage_models::DiskInfo = d.clone().into();
                if disk_info.device == disk {
                    return true;
                }
                if let Some(disk_name) = disk_info.device.rsplit('/').next() {
                    if disk_name == device_name {
                        return true;
                    }
                }
                if disk_info.id == disk || disk_info.id == device_name {
                    return true;
                }
                false
            })
            .ok_or_else(|| {
                tracing::warn!("Device not found: {disk}");
                zbus::fdo::Error::Failed(format!("Device not found: {disk}"))
            })?;
        
        // Create partition table on drive (format_disk creates GPT/DOS partition table)
        drive_model
            .format_disk(normalized_type, false)
            .await
            .map_err(|e| {
                tracing::error!("Failed to create partition table: {e}");
                zbus::fdo::Error::Failed(format!("Failed to create partition table: {e}"))
            })?;
        
        tracing::info!("Successfully created {} partition table on {}", normalized_type, disk);
        
        // TODO: Emit PartitionTableCreated signal
        
        Ok(())
    }
}
