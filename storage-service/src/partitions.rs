// SPDX-License-Identifier: GPL-3.0-only

//! Partition management D-Bus interface
//!
//! This module provides D-Bus methods for managing disk partitions,
//! including creating/deleting partitions and partition tables.

use zbus::zvariant::OwnedObjectPath;
use zbus::{Connection, interface};

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

        let disk_volumes = storage_dbus::disk::get_disks_with_partitions()
            .await
            .map_err(|e| {
                tracing::error!("Failed to get drives: {e}");
                zbus::fdo::Error::Failed(format!("Failed to enumerate drives: {e}"))
            })?;

        let device_name = disk.strip_prefix("/dev/").unwrap_or(&disk);
        let (_disk_info, partitions) = disk_volumes
            .into_iter()
            .find(|(d, _)| {
                d.device == disk
                    || d.device.rsplit('/').next() == Some(device_name)
                    || d.id == disk
                    || d.id == device_name
            })
            .ok_or_else(|| {
                tracing::warn!("Device not found: {disk}");
                zbus::fdo::Error::Failed(format!("Device not found: {disk}"))
            })?;

        tracing::debug!("Found {} partitions for {}", partitions.len(), disk);

        // Serialize to JSON
        let json = serde_json::to_string(&partitions).map_err(|e| {
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
        check_polkit_auth(
            connection,
            "org.cosmic.ext.storage-service.partition-modify",
        )
        .await
        .map_err(|e| zbus::fdo::Error::Failed(format!("Authorization failed: {e}")))?;

        tracing::info!("Creating {} partition table on disk: {}", table_type, disk);

        // Validate and normalize table type
        let normalized_type = match table_type.to_lowercase().as_str() {
            "gpt" => "gpt",
            "dos" | "mbr" | "msdos" => "dos",
            _ => {
                tracing::warn!("Invalid partition table type: {table_type}");
                return Err(zbus::fdo::Error::InvalidArgs(format!(
                    "Invalid table type: {table_type}. Must be 'gpt' or 'dos'/'mbr'"
                )));
            }
        };

        let disk_device = if disk.starts_with("/dev/") {
            disk.clone()
        } else {
            format!("/dev/{}", disk)
        };

        let block_path = storage_dbus::block_object_path_for_device(&disk_device)
            .await
            .map_err(|e| {
                tracing::error!("Failed to resolve device: {e}");
                zbus::fdo::Error::Failed(format!("Device not found: {e}"))
            })?;

        storage_dbus::create_partition_table(block_path.as_str(), normalized_type)
            .await
            .map_err(|e| {
                tracing::error!("Failed to create partition table: {e}");
                zbus::fdo::Error::Failed(format!("Failed to create partition table: {e}"))
            })?;

        tracing::info!(
            "Successfully created {} partition table on {}",
            normalized_type,
            disk
        );
        let _ = Self::partition_table_created(&signal_ctx, &disk_device, normalized_type).await;
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
        check_polkit_auth(
            connection,
            "org.cosmic.ext.storage-service.partition-modify",
        )
        .await
        .map_err(|e| zbus::fdo::Error::Failed(format!("Authorization failed: {e}")))?;

        tracing::info!(
            "Creating partition on {}: offset={}, size={}, type={}",
            disk,
            offset,
            size,
            type_id
        );

        let disk_device = if disk.starts_with("/dev/") {
            disk.clone()
        } else {
            format!("/dev/{}", disk)
        };

        let block_path = storage_dbus::block_object_path_for_device(&disk_device)
            .await
            .map_err(|e| {
                tracing::error!("Failed to resolve device: {e}");
                zbus::fdo::Error::Failed(format!("Device not found: {e}"))
            })?;

        let device_path =
            storage_dbus::create_partition(block_path.as_str(), offset, size, &type_id)
                .await
                .map_err(|e| {
                    tracing::error!("Failed to create partition: {e}");
                    zbus::fdo::Error::Failed(format!("Failed to create partition: {e}"))
                })?;

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
        })
        .unwrap_or_default();
        let _ =
            Self::partition_created(&signal_ctx, &disk, &device_path, &partition_info_json).await;
        Ok(device_path)
    }

    /// Create a new partition with filesystem formatting (all-in-one)
    ///
    /// Args:
    /// - disk: Device identifier (e.g., "/dev/sda", "sda")
    /// - info_json: JSON-serialized CreatePartitionInfo
    ///
    /// Returns: Device path of created partition (e.g., "/dev/sda1")
    ///
    /// Authorization: org.cosmic.ext.storage-service.partition-modify (auth_admin_keep)
    async fn create_partition_with_filesystem(
        &self,
        #[zbus(connection)] connection: &Connection,
        #[zbus(signal_context)] signal_ctx: zbus::object_server::SignalEmitter<'_>,
        disk: String,
        info_json: String,
    ) -> zbus::fdo::Result<String> {
        // Check authorization
        check_polkit_auth(
            connection,
            "org.cosmic.ext.storage-service.partition-modify",
        )
        .await
        .map_err(|e| zbus::fdo::Error::Failed(format!("Authorization failed: {e}")))?;

        // Parse the CreatePartitionInfo
        let info: storage_models::CreatePartitionInfo =
            serde_json::from_str(&info_json).map_err(|e| {
                tracing::error!("Failed to parse CreatePartitionInfo: {e}");
                zbus::fdo::Error::InvalidArgs(format!("Invalid partition info JSON: {e}"))
            })?;

        tracing::info!(
            "Creating partition with filesystem on {}: offset={}, size={}, fs={}, luks={}",
            disk,
            info.offset,
            info.size,
            info.filesystem_type,
            info.password_protected
        );

        let disk_device = if disk.starts_with("/dev/") {
            disk.clone()
        } else {
            format!("/dev/{}", disk)
        };

        let block_path = storage_dbus::block_object_path_for_device(&disk_device)
            .await
            .map_err(|e| {
                tracing::error!("Failed to resolve device: {e}");
                zbus::fdo::Error::Failed(format!("Device not found: {e}"))
            })?;

        let device_path =
            storage_dbus::create_partition_with_filesystem(block_path.as_str(), &info)
                .await
                .map_err(|e| {
                    tracing::error!("Failed to create partition with filesystem: {e}");
                    zbus::fdo::Error::Failed(format!("Failed to create partition: {e}"))
                })?;

        tracing::info!(
            "Successfully created partition with filesystem: {}",
            device_path
        );

        let partition_info_json = serde_json::to_string(&storage_models::PartitionInfo {
            device: device_path.clone(),
            number: 0,
            parent_path: disk.clone(),
            size: info.size,
            offset: info.offset,
            type_id: info.selected_type.clone(),
            type_name: String::new(),
            flags: 0,
            name: info.name.clone(),
            uuid: String::new(),
            table_type: String::new(),
            has_filesystem: !info.filesystem_type.is_empty(),
            filesystem_type: if info.filesystem_type.is_empty() {
                None
            } else {
                Some(info.filesystem_type.clone())
            },
            mount_points: vec![],
            usage: None,
        })
        .unwrap_or_default();
        let _ =
            Self::partition_created(&signal_ctx, &disk, &device_path, &partition_info_json).await;
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
        check_polkit_auth(
            connection,
            "org.cosmic.ext.storage-service.partition-modify",
        )
        .await
        .map_err(|e| zbus::fdo::Error::Failed(format!("Authorization failed: {e}")))?;

        tracing::info!("Deleting partition: {}", partition);

        // Find partition path from device
        let partition_path = self.find_partition_path(&partition).await?;

        // Delegate to storage-dbus operation
        storage_dbus::delete_partition(&partition_path.to_string())
            .await
            .map_err(|e| {
                tracing::error!("Failed to delete partition: {e}");
                zbus::fdo::Error::Failed(format!("Failed to delete partition: {e}"))
            })?;

        tracing::info!("Successfully deleted partition: {}", partition);
        let disk = partition
            .trim_end_matches(|c: char| c.is_ascii_digit())
            .to_string();
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
        check_polkit_auth(
            connection,
            "org.cosmic.ext.storage-service.partition-modify",
        )
        .await
        .map_err(|e| zbus::fdo::Error::Failed(format!("Authorization failed: {e}")))?;

        tracing::info!("Resizing partition {} to {} bytes", partition, new_size);

        // Find partition path
        let partition_path = self.find_partition_path(&partition).await?;

        // Delegate to storage-dbus operation
        storage_dbus::resize_partition(&partition_path.to_string(), new_size)
            .await
            .map_err(|e| {
                tracing::error!("Failed to resize partition: {e}");
                zbus::fdo::Error::Failed(format!("Failed to resize partition: {e}"))
            })?;

        tracing::info!("Successfully resized partition: {}", partition);
        let disk = partition
            .trim_end_matches(|c: char| c.is_ascii_digit())
            .to_string();
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
        check_polkit_auth(
            connection,
            "org.cosmic.ext.storage-service.partition-modify",
        )
        .await
        .map_err(|e| zbus::fdo::Error::Failed(format!("Authorization failed: {e}")))?;

        tracing::info!("Setting partition {} type to {}", partition, type_id);

        // Find partition path
        let partition_path = self.find_partition_path(&partition).await?;

        // Delegate to storage-dbus operation
        storage_dbus::set_partition_type(&partition_path.to_string(), &type_id)
            .await
            .map_err(|e| {
                tracing::error!("Failed to set partition type: {e}");
                zbus::fdo::Error::Failed(format!("Failed to set partition type: {e}"))
            })?;

        tracing::info!("Successfully set partition type: {}", partition);
        let disk = partition
            .trim_end_matches(|c: char| c.is_ascii_digit())
            .to_string();
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
        check_polkit_auth(
            connection,
            "org.cosmic.ext.storage-service.partition-modify",
        )
        .await
        .map_err(|e| zbus::fdo::Error::Failed(format!("Authorization failed: {e}")))?;

        tracing::info!("Setting partition {} flags to 0x{:x}", partition, flags);

        // Find partition path
        let partition_path = self.find_partition_path(&partition).await?;

        // Delegate to storage-dbus operation
        storage_dbus::set_partition_flags(&partition_path.to_string(), flags)
            .await
            .map_err(|e| {
                tracing::error!("Failed to set partition flags: {e}");
                zbus::fdo::Error::Failed(format!("Failed to set partition flags: {e}"))
            })?;

        tracing::info!("Successfully set partition flags: {}", partition);
        let disk = partition
            .trim_end_matches(|c: char| c.is_ascii_digit())
            .to_string();
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
        check_polkit_auth(
            connection,
            "org.cosmic.ext.storage-service.partition-modify",
        )
        .await
        .map_err(|e| zbus::fdo::Error::Failed(format!("Authorization failed: {e}")))?;

        tracing::info!("Setting partition {} name to '{}'", partition, name);

        // Validate name length (GPT allows 36 characters)
        if name.len() > 36 {
            tracing::warn!("Partition name too long: {} characters", name.len());
            return Err(zbus::fdo::Error::InvalidArgs(format!(
                "Partition name must be 36 characters or less (got {})",
                name.len()
            )));
        }

        // Find partition path
        let partition_path = self.find_partition_path(&partition).await?;

        // Delegate to storage-dbus operation
        storage_dbus::set_partition_name(&partition_path.to_string(), &name)
            .await
            .map_err(|e| {
                tracing::error!("Failed to set partition name: {e}");
                zbus::fdo::Error::Failed(format!("Failed to set partition name: {e}"))
            })?;

        tracing::info!("Successfully set partition name: {}", partition);
        let disk = partition
            .trim_end_matches(|c: char| c.is_ascii_digit())
            .to_string();
        let _ = Self::partition_modified(&signal_ctx, &disk, &partition, "").await;
        Ok(())
    }
}

/// Helper methods
impl PartitionsHandler {
    /// Find UDisks2 partition object path from device path
    async fn find_partition_path(&self, partition: &str) -> zbus::fdo::Result<OwnedObjectPath> {
        let device = if partition.starts_with("/dev/") {
            partition.to_string()
        } else {
            format!("/dev/{}", partition)
        };
        storage_dbus::block_object_path_for_device(&device)
            .await
            .map_err(|e| {
                tracing::warn!("Partition not found: {} - {}", partition, e);
                zbus::fdo::Error::Failed(format!("Partition not found: {}", partition))
            })
    }
}
