// SPDX-License-Identifier: GPL-3.0-only

//! Partition management D-Bus interface
//!
//! This module provides D-Bus methods for managing disk partitions,
//! including creating/deleting partitions and partition tables.

use std::collections::HashMap;
use udisks2::{partition::PartitionProxy, partitiontable::PartitionTableProxy, block::BlockProxy};
use zbus::{interface, Connection};
use zbus::zvariant::{OwnedObjectPath, Value};

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
        #[zbus(signal_context)] signal_ctx: zbus::object_server::SignalEmitter<'_>,
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
        let _ = Self::partition_table_created(&signal_ctx, &disk, normalized_type).await;
        Ok(())
    }
    
    /// Create a new partition in available space
    /// 
    /// Args:
    /// - disk: Device identifier (e.g., "/dev/sda", "sda")
    /// - offset: Start offset in bytes
    /// - size: Size in bytes
    /// - type_id: Partition type (GPT GUID or MBR type code)
    /// 
    /// Returns: Device path of created partition (e.g., "/dev/sda1")
    /// 
    /// Authorization: org.cosmic.ext.storage-service.partition-modify (auth_admin_keep)
    async fn create_partition(
        &self,
        #[zbus(connection)] connection: &Connection,
        #[zbus(signal_context)] signal_ctx: zbus::object_server::SignalEmitter<'_>,
        disk: String,
        offset: u64,
        size: u64,
        type_id: String,
    ) -> zbus::fdo::Result<String> {
        // Check authorization
        check_polkit_auth(connection, "org.cosmic.ext.storage-service.partition-modify")
            .await
            .map_err(|e| zbus::fdo::Error::Failed(format!("Authorization failed: {e}")))?;
        
        tracing::info!("Creating partition on {}: offset={}, size={}, type={}", disk, offset, size, type_id);
        
        // Find drive and get its block path
        let drives = disks_dbus::DriveModel::get_drives()
            .await
            .map_err(|e| {
                tracing::error!("Failed to get drives: {e}");
                zbus::fdo::Error::Failed(format!("Failed to enumerate drives: {e}"))
            })?;
        
        let device_name = disk.strip_prefix("/dev/").unwrap_or(&disk);
        
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
        
        // Get block path from drive
        let block_path: OwnedObjectPath = drive_model.block_path.as_str().try_into()
            .map_err(|e| {
                tracing::error!("Invalid block path: {e}");
                zbus::fdo::Error::Failed(format!("Invalid block path: {e}"))
            })?;
        
        // Create partition using UDisks2 PartitionTable.CreatePartition
        let table_proxy = PartitionTableProxy::builder(connection)
            .path(&block_path)
            .map_err(|e| {
                tracing::error!("Failed to create proxy: {e}");
                zbus::fdo::Error::Failed(format!("Failed to create proxy: {e}"))
            })?
            .build()
            .await
            .map_err(|e| {
                tracing::error!("Failed to build partition table proxy: {e}");
                zbus::fdo::Error::Failed(format!("Failed to access partition table: {e}"))
            })?;
        
        // Call CreatePartition
        let options: HashMap<&str, Value<'_>> = HashMap::new();
        let partition_path = table_proxy
            .create_partition(offset, size, &type_id, "", options)
            .await
            .map_err(|e| {
                tracing::error!("Failed to create partition: {e}");
                zbus::fdo::Error::Failed(format!("Failed to create partition: {e}"))
            })?;
        
        // Get device path of created partition
        let block_proxy = BlockProxy::builder(connection)
            .path(&partition_path)
            .map_err(|e| {
                tracing::error!("Failed to create block proxy: {e}");
                zbus::fdo::Error::Failed(format!("Failed to create block proxy: {e}"))
            })?
            .build()
            .await
            .map_err(|e| {
                tracing::error!("Failed to build block proxy: {e}");
                zbus::fdo::Error::Failed(format!("Failed to access created partition: {e}"))
            })?;
        
        let device_bytes = block_proxy.preferred_device().await
            .map_err(|e| {
                tracing::error!("Failed to get device path: {e}");
                zbus::fdo::Error::Failed(format!("Failed to get device path: {e}"))
            })?;
        
        // Decode device path from bytestring
        let device_path = String::from_utf8(device_bytes.into_iter().filter(|&b| b != 0).collect())
            .unwrap_or_else(|_| format!("/dev/{}", partition_path.as_str().rsplit('/').next().unwrap_or("unknown")));
        
        tracing::info!("Successfully created partition: {}", device_path);
        let partition_info_json = serde_json::to_string(&storage_models::PartitionInfo {
            device: device_path.clone(),
            number: 0,
            parent_path: disk.clone(),
            size,
            offset,
            type_id: type_id.clone(),
            type_name: String::new(),
            flags: 0,
            name: String::new(),
            uuid: String::new(),
            table_type: String::new(),
            has_filesystem: false,
            filesystem_type: None,
            mount_points: vec![],
            usage: None,
        }).unwrap_or_default();
        let _ = Self::partition_created(&signal_ctx, &disk, &device_path, &partition_info_json).await;
        Ok(device_path)
    }
    
    /// Delete an existing partition
    /// 
    /// Args:
    /// - partition: Partition device path (e.g., "/dev/sda1")
    /// 
    /// Authorization: org.cosmic.ext.storage-service.partition-modify (auth_admin_keep)
    async fn delete_partition(
        &self,
        #[zbus(connection)] connection: &Connection,
        #[zbus(signal_context)] signal_ctx: zbus::object_server::SignalEmitter<'_>,
        partition: String,
    ) -> zbus::fdo::Result<()> {
        // Check authorization
        check_polkit_auth(connection, "org.cosmic.ext.storage-service.partition-modify")
            .await
            .map_err(|e| zbus::fdo::Error::Failed(format!("Authorization failed: {e}")))?;
        
        tracing::info!("Deleting partition: {}", partition);
        
        // Find partition path from device
        let partition_path = self.find_partition_path(&partition).await?;
        
        // Check mount points
        if let Ok(fs_proxy_builder) = udisks2::filesystem::FilesystemProxy::builder(connection)
            .path(&partition_path)
        {
            if let Ok(fs_proxy) = fs_proxy_builder.build().await {
                let mount_points = fs_proxy.mount_points().await.unwrap_or_default();
                if !mount_points.is_empty() {
                    tracing::warn!("Partition {} is mounted", partition);
                    return Err(zbus::fdo::Error::Failed(
                        format!("Partition is mounted. Unmount it first.")
                    ));
                }
            }
        }
        
        // Delete partition
        let partition_proxy = PartitionProxy::builder(connection)
            .path(&partition_path)
            .map_err(|e| {
                tracing::error!("Failed to create partition proxy: {e}");
                zbus::fdo::Error::Failed(format!("Failed to create partition proxy: {e}"))
            })?
            .build()
            .await
            .map_err(|e| {
                tracing::error!("Failed to build partition proxy: {e}");
                zbus::fdo::Error::Failed(format!("Failed to access partition: {e}"))
            })?;
        
        let options: HashMap<&str, Value<'_>> = HashMap::new();
        partition_proxy.delete(options).await
            .map_err(|e| {
                tracing::error!("Failed to delete partition: {e}");
                zbus::fdo::Error::Failed(format!("Failed to delete partition: {e}"))
            })?;
        
        tracing::info!("Successfully deleted partition: {}", partition);
        let disk = partition.trim_end_matches(|c: char| c.is_ascii_digit()).to_string();
        let _ = Self::partition_deleted(&signal_ctx, &disk, &partition).await;
        Ok(())
    }
    
    /// Resize an existing partition
    /// 
    /// Args:
    /// - partition: Partition device path (e.g., "/dev/sda1")
    /// - new_size: New size in bytes
    /// 
    /// Authorization: org.cosmic.ext.storage-service.partition-modify (auth_admin_keep)
    async fn resize_partition(
        &self,
        #[zbus(connection)] connection: &Connection,
        #[zbus(signal_context)] signal_ctx: zbus::object_server::SignalEmitter<'_>,
        partition: String,
        new_size: u64,
    ) -> zbus::fdo::Result<()> {
        // Check authorization
        check_polkit_auth(connection, "org.cosmic.ext.storage-service.partition-modify")
            .await
            .map_err(|e| zbus::fdo::Error::Failed(format!("Authorization failed: {e}")))?;
        
        tracing::info!("Resizing partition {} to {} bytes", partition, new_size);
        
        // Find partition path
        let partition_path = self.find_partition_path(&partition).await?;
        
        // Resize partition
        let partition_proxy = PartitionProxy::builder(connection)
            .path(&partition_path)
            .map_err(|e| {
                tracing::error!("Failed to create partition proxy: {e}");
                zbus::fdo::Error::Failed(format!("Failed to create partition proxy: {e}"))
            })?
            .build()
            .await
            .map_err(|e| {
                tracing::error!("Failed to build partition proxy: {e}");
                zbus::fdo::Error::Failed(format!("Failed to access partition: {e}"))
            })?;
        
        let options: HashMap<&str, Value<'_>> = HashMap::new();
        partition_proxy.resize(new_size, options).await
            .map_err(|e| {
                tracing::error!("Failed to resize partition: {e}");
                zbus::fdo::Error::Failed(format!("Failed to resize partition: {e}"))
            })?;
        
        tracing::info!("Successfully resized partition: {}", partition);
        let disk = partition.trim_end_matches(|c: char| c.is_ascii_digit()).to_string();
        let _ = Self::partition_modified(&signal_ctx, &disk, &partition, "").await;
        Ok(())
    }
    
    /// Set partition type (GPT GUID or MBR type code)
    /// 
    /// Args:
    /// - partition: Partition device path (e.g., "/dev/sda1")
    /// - type_id: Partition type identifier
    /// 
    /// Authorization: org.cosmic.ext.storage-service.partition-modify (auth_admin_keep)
    async fn set_partition_type(
        &self,
        #[zbus(connection)] connection: &Connection,
        #[zbus(signal_context)] signal_ctx: zbus::object_server::SignalEmitter<'_>,
        partition: String,
        type_id: String,
    ) -> zbus::fdo::Result<()> {
        // Check authorization
        check_polkit_auth(connection, "org.cosmic.ext.storage-service.partition-modify")
            .await
            .map_err(|e| zbus::fdo::Error::Failed(format!("Authorization failed: {e}")))?;
        
        tracing::info!("Setting partition {} type to {}", partition, type_id);
        
        // Find partition path
        let partition_path = self.find_partition_path(&partition).await?;
        
        // Set partition type
        let partition_proxy = PartitionProxy::builder(connection)
            .path(&partition_path)
            .map_err(|e| {
                tracing::error!("Failed to create partition proxy: {e}");
                zbus::fdo::Error::Failed(format!("Failed to create partition proxy: {e}"))
            })?
            .build()
            .await
            .map_err(|e| {
                tracing::error!("Failed to build partition proxy: {e}");
                zbus::fdo::Error::Failed(format!("Failed to access partition: {e}"))
            })?;
        
        let options: HashMap<&str, Value<'_>> = HashMap::new();
        partition_proxy.set_type(&type_id, options).await
            .map_err(|e| {
                tracing::error!("Failed to set partition type: {e}");
                zbus::fdo::Error::Failed(format!("Failed to set partition type: {e}"))
            })?;
        
        tracing::info!("Successfully set partition type: {}", partition);
        let disk = partition.trim_end_matches(|c: char| c.is_ascii_digit()).to_string();
        let _ = Self::partition_modified(&signal_ctx, &disk, &partition, "").await;
        Ok(())
    }
    
    /// Set partition flags (bootable, hidden, etc.)
    /// 
    /// Args:
    /// - partition: Partition device path (e.g., "/dev/sda1")
    /// - flags: Flags as u64 bitfield (0x01 = bootable, 0x02 = system, 0x04 = hidden)
    /// 
    /// Authorization: org.cosmic.ext.storage-service.partition-modify (auth_admin_keep)
    async fn set_partition_flags(
        &self,
        #[zbus(connection)] connection: &Connection,
        #[zbus(signal_context)] signal_ctx: zbus::object_server::SignalEmitter<'_>,
        partition: String,
        flags: u64,
    ) -> zbus::fdo::Result<()> {
        // Check authorization
        check_polkit_auth(connection, "org.cosmic.ext.storage-service.partition-modify")
            .await
            .map_err(|e| zbus::fdo::Error::Failed(format!("Authorization failed: {e}")))?;
        
        tracing::info!("Setting partition {} flags to 0x{:x}", partition, flags);
        
        // Find partition path
        let partition_path = self.find_partition_path(&partition).await?;
        
        // Set partition flags
        let partition_proxy = PartitionProxy::builder(connection)
            .path(&partition_path)
            .map_err(|e| {
                tracing::error!("Failed to create partition proxy: {e}");
                zbus::fdo::Error::Failed(format!("Failed to create partition proxy: {e}"))
            })?
            .build()
            .await
            .map_err(|e| {
                tracing::error!("Failed to build partition proxy: {e}");
                zbus::fdo::Error::Failed(format!("Failed to access partition: {e}"))
            })?;
        
        let options: HashMap<&str, Value<'_>> = HashMap::new();
        let flags_bitfield = enumflags2::BitFlags::from_bits_truncate(flags);
        partition_proxy.set_flags(flags_bitfield, options).await
            .map_err(|e| {
                tracing::error!("Failed to set partition flags: {e}");
                zbus::fdo::Error::Failed(format!("Failed to set partition flags: {e}"))
            })?;
        
        tracing::info!("Successfully set partition flags: {}", partition);
        let disk = partition.trim_end_matches(|c: char| c.is_ascii_digit()).to_string();
        let _ = Self::partition_modified(&signal_ctx, &disk, &partition, "").await;
        Ok(())
    }
    
    /// Set partition name (GPT only)
    /// 
    /// Args:
    /// - partition: Partition device path (e.g., "/dev/sda1")
    /// - name: Partition name (max 36 characters for GPT)
    /// 
    /// Authorization: org.cosmic.ext.storage-service.partition-modify (auth_admin_keep)
    async fn set_partition_name(
        &self,
        #[zbus(connection)] connection: &Connection,
        #[zbus(signal_context)] signal_ctx: zbus::object_server::SignalEmitter<'_>,
        partition: String,
        name: String,
    ) -> zbus::fdo::Result<()> {
        // Check authorization
        check_polkit_auth(connection, "org.cosmic.ext.storage-service.partition-modify")
            .await
            .map_err(|e| zbus::fdo::Error::Failed(format!("Authorization failed: {e}")))?;
        
        tracing::info!("Setting partition {} name to '{}'", partition, name);
        
        // Validate name length (GPT allows 36 characters)
        if name.len() > 36 {
            tracing::warn!("Partition name too long: {} characters", name.len());
            return Err(zbus::fdo::Error::InvalidArgs(
                format!("Partition name must be 36 characters or less (got {})", name.len())
            ));
        }
        
        // Find partition path
        let partition_path = self.find_partition_path(&partition).await?;
        
        // Set partition name
        let partition_proxy = PartitionProxy::builder(connection)
            .path(&partition_path)
            .map_err(|e| {
                tracing::error!("Failed to create partition proxy: {e}");
                zbus::fdo::Error::Failed(format!("Failed to create partition proxy: {e}"))
            })?
            .build()
            .await
            .map_err(|e| {
                tracing::error!("Failed to build partition proxy: {e}");
                zbus::fdo::Error::Failed(format!("Failed to access partition: {e}"))
            })?;
        
        let options: HashMap<&str, Value<'_>> = HashMap::new();
        partition_proxy.set_name(&name, options).await
            .map_err(|e| {
                tracing::error!("Failed to set partition name: {e}");
                zbus::fdo::Error::Failed(format!("Failed to set partition name: {e}"))
            })?;
        
        tracing::info!("Successfully set partition name: {}", partition);
        let disk = partition.trim_end_matches(|c: char| c.is_ascii_digit()).to_string();
        let _ = Self::partition_modified(&signal_ctx, &disk, &partition, "").await;
        Ok(())
    }
}

/// Helper methods
impl PartitionsHandler {
    /// Find UDisks2 partition object path from device path
    async fn find_partition_path(
        &self,
        partition: &str,
    ) -> zbus::fdo::Result<OwnedObjectPath> {
        // Get all drives and search their partitions
        let drives = disks_dbus::DriveModel::get_drives()
            .await
            .map_err(|e| {
                tracing::error!("Failed to get drives: {e}");
                zbus::fdo::Error::Failed(format!("Failed to enumerate drives: {e}"))
            })?;
        
        let partition_clean = partition.strip_prefix("/dev/").unwrap_or(partition);
        
        for drive in drives {
            let partitions = drive.get_partitions();
            for part_info in partitions {
                // Check if device matches
                let part_device = part_info.device.strip_prefix("/dev/").unwrap_or(&part_info.device);
                if part_device == partition_clean || part_info.device == partition {
                    // Found it - need to get UDisks2 path
                    // The path is typically /org/freedesktop/UDisks2/block_devices/sdXN
                    let path = format!("/org/freedesktop/UDisks2/block_devices/{}", part_device);
                    return path.try_into()
                        .map_err(|e| {
                            tracing::error!("Invalid partition path: {e}");
                            zbus::fdo::Error::Failed(format!("Invalid partition path: {e}"))
                        });
                }
            }
        }
        
        tracing::warn!("Partition not found: {}", partition);
        Err(zbus::fdo::Error::Failed(format!("Partition not found: {}", partition)))
    }
}
