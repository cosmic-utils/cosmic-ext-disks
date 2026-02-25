// SPDX-License-Identifier: GPL-3.0-only

//! Disk discovery and management D-Bus interface
//!
//! This module provides D-Bus methods for listing disks, getting disk information,
//! and monitoring disk hotplug events.

use anyhow::Result;
use std::sync::Arc;
use storage_dbus::DiskManager;
use storage_service_macros::authorized_interface;
use zbus::message::Header as MessageHeader;
use zbus::{Connection, interface};

use crate::service::domain::disks::{DefaultDisksDomain, DisksDomain};

/// D-Bus interface for disk discovery and SMART operations
pub struct DisksHandler {
    #[allow(dead_code)]
    manager: DiskManager,
    domain: Arc<dyn DisksDomain>,
}

impl DisksHandler {
    /// Create a new DisksHandler
    pub async fn new() -> Result<Self> {
        let manager = DiskManager::new().await?;
        Ok(Self {
            manager,
            domain: Arc::new(DefaultDisksDomain),
        })
    }

    /// Get the underlying DiskManager for internal use
    #[allow(dead_code)]
    pub fn manager(&self) -> &DiskManager {
        &self.manager
    }
}

#[interface(name = "org.cosmic.ext.StorageService.Disks")]
impl DisksHandler {
    /// Signal emitted when a disk is added to the system
    ///
    /// Args:
    /// - device: Device path (e.g., "/dev/sda")
    /// - disk_info: JSON-serialized DiskInfo
    #[zbus(signal)]
    async fn disk_added(
        signal_ctxt: &zbus::object_server::SignalEmitter<'_>,
        device: &str,
        disk_info: &str,
    ) -> zbus::Result<()>;

    /// Signal emitted when a disk is removed from the system
    ///
    /// Args:
    /// - device: Device path (e.g., "/dev/sda")
    #[zbus(signal)]
    async fn disk_removed(
        signal_ctxt: &zbus::object_server::SignalEmitter<'_>,
        device: &str,
    ) -> zbus::Result<()>;
    /// List all disks on the system
    ///
    /// Returns a JSON-serialized array of DiskInfo objects.
    ///
    /// **Authorization:** Requires `disk-read` (allow_active)
    ///
    /// **Example:**
    /// ```bash
    /// busctl call org.cosmic.ext.StorageService \
    ///   /org/cosmic/ext/StorageService/disks \
    ///   org.cosmic.ext.StorageService.Disks \
    ///   ListDisks
    /// ```
    #[authorized_interface(action = "org.cosmic.ext.storage-service.disk-read")]
    async fn list_disks(
        &self,
        #[zbus(connection)] _connection: &Connection,
        #[zbus(header)] _header: MessageHeader<'_>,
    ) -> zbus::fdo::Result<String> {
        tracing::debug!("ListDisks called (UID {})", caller.uid);

        // Get disks from storage-dbus using new storage-common API
        let disks = storage_dbus::disk::get_disks(&self.manager)
            .await
            .map_err(|e| {
                tracing::error!("Failed to get disks: {e}");
                zbus::fdo::Error::Failed(format!("Failed to enumerate disks: {e}"))
            })?;

        tracing::debug!("Found {} disks", disks.len());

        // Serialize to JSON
        let json = serde_json::to_string(&disks).map_err(|e| {
            tracing::error!("Failed to serialize disks: {e}");
            zbus::fdo::Error::Failed(format!("Serialization error: {e}"))
        })?;

        Ok(json)
    }

    /// List all volumes across all disks
    ///
    /// Returns a flat list of all volumes (partitions, filesystems, LUKS containers, etc.)
    /// with parent_path populated for building hierarchies in the UI.
    ///
    /// Returns a JSON-serialized array of VolumeInfo objects.
    ///
    /// **Authorization:** Requires `disk-read` (allow_active)
    ///
    /// **Example:**
    /// ```bash
    /// busctl call org.cosmic.ext.StorageService \
    ///   /org/cosmic/ext/StorageService/disks \
    ///   org.cosmic.ext.StorageService.Disks \
    ///   ListVolumes
    /// ```
    #[authorized_interface(action = "org.cosmic.ext.storage-service.disk-read")]
    async fn list_volumes(
        &self,
        #[zbus(connection)] _connection: &Connection,
        #[zbus(header)] _header: MessageHeader<'_>,
    ) -> zbus::fdo::Result<String> {
        tracing::debug!("ListVolumes called (UID {})", caller.uid);

        // Get all drives using storage-dbus
        let disk_volumes = storage_dbus::disk::get_disks_with_volumes(&self.manager)
            .await
            .map_err(|e| {
                tracing::error!("Failed to get drives: {e}");
                zbus::fdo::Error::Failed(format!("Failed to enumerate drives: {e}"))
            })?;

        // Flatten volumes from all drives and populate parent_path
        let mut all_volumes = Vec::new();

        for (disk_info, volumes) in disk_volumes {
            let disk_device = disk_info.device.clone(); // e.g., "/dev/sda"

            // Recursively flatten volume tree
            fn flatten_volumes(
                vol_info: &storage_common::VolumeInfo,
                parent_device: Option<String>,
                output: &mut Vec<storage_common::VolumeInfo>,
            ) {
                // Clone and update parent_path
                let mut vol = vol_info.clone();
                vol.parent_path = parent_device.clone();

                // Process children recursively
                let current_device = vol.device_path.clone();
                for child in &vol_info.children {
                    flatten_volumes(child, current_device.clone(), output);
                }

                // Clear children in the flat output (not hierarchical)
                vol.children.clear();

                output.push(vol);
            }

            // Process each root volume
            for volume_info in &volumes {
                flatten_volumes(volume_info, Some(disk_device.clone()), &mut all_volumes);
            }
        }

        tracing::debug!("Found {} total volumes", all_volumes.len());

        // Serialize to JSON
        let json = serde_json::to_string(&all_volumes).map_err(|e| {
            tracing::error!("Failed to serialize volumes: {e}");
            zbus::fdo::Error::Failed(format!("Serialization error: {e}"))
        })?;

        Ok(json)
    }

    /// Get detailed information for a specific disk
    ///
    /// **Arguments:**
    /// - `device`: Device path (e.g., "/dev/sda")
    ///
    /// Returns a JSON-serialized DiskInfo object.
    ///
    /// **Authorization:** Requires `disk-read` (allow_active)
    ///
    /// **Example:**
    /// ```bash
    /// busctl call org.cosmic.ext.StorageService \
    ///   /org/cosmic/ext/StorageService/disks \
    ///   org.cosmic.ext.StorageService.Disks \
    ///   GetDiskInfo s "/dev/sda"
    /// ```
    #[authorized_interface(action = "org.cosmic.ext.storage-service.disk-read")]
    async fn get_disk_info(
        &self,
        #[zbus(connection)] _connection: &Connection,
        #[zbus(header)] _header: MessageHeader<'_>,
        device: String,
    ) -> zbus::fdo::Result<String> {
        tracing::debug!(
            "GetDiskInfo called for device: {device} (UID {})",
            caller.uid
        );

        // Get all disks and find the requested one
        let disks = storage_dbus::disk::get_disks(&self.manager)
            .await
            .map_err(|e| {
                tracing::error!("Failed to get disks: {e}");
                zbus::fdo::Error::Failed(format!("Failed to enumerate disks: {e}"))
            })?;

        // Log available disks for debugging
        tracing::debug!("Found {} disks total", disks.len());
        for d in &disks {
            tracing::debug!("Available disk: device={}, id={}", d.device, d.id);
        }

        // Extract device name from input (strip "/dev/" prefix if present)
        let disk = disks
            .into_iter()
            .find(|d| self.domain.disk_matches(d, &device))
            .ok_or_else(|| {
                tracing::warn!("Device not found: {device}");
                zbus::fdo::Error::Failed(format!("Device not found: {device}"))
            })?;

        tracing::debug!("Found disk: device={}, id={}", disk.device, disk.id);

        // Serialize to JSON
        let json = serde_json::to_string(&disk).map_err(|e| {
            tracing::error!("Failed to serialize disk info: {e}");
            zbus::fdo::Error::Failed(format!("Serialization error: {e}"))
        })?;

        Ok(json)
    }

    /// Get detailed information for a specific volume
    ///
    /// This method supports atomic updates - clients can query a single volume
    /// instead of refreshing the entire volume list.
    ///
    /// **Arguments:**
    /// - `device`: Device path (e.g., "/dev/sda1", "/dev/mapper/luks-...")
    ///
    /// Returns a JSON-serialized VolumeInfo object with parent_path populated.
    ///
    /// **Authorization:** Requires `disk-read` (allow_active)
    ///
    /// **Example:**
    /// ```bash
    /// busctl call org.cosmic.ext.StorageService \
    ///   /org/cosmic/ext/StorageService/disks \
    ///   org.cosmic.ext.StorageService.Disks \
    ///   GetVolumeInfo s "/dev/sda1"
    /// ```
    #[authorized_interface(action = "org.cosmic.ext.storage-service.disk-read")]
    async fn get_volume_info(
        &self,
        #[zbus(connection)] _connection: &Connection,
        #[zbus(header)] _header: MessageHeader<'_>,
        device: String,
    ) -> zbus::fdo::Result<String> {
        tracing::debug!(
            "GetVolumeInfo called for device: {device} (UID {})",
            caller.uid
        );

        // Get all drives and search for the volume
        let disk_volumes = storage_dbus::disk::get_disks_with_volumes(&self.manager)
            .await
            .map_err(|e| {
                tracing::error!("Failed to get drives: {e}");
                zbus::fdo::Error::Failed(format!("Failed to enumerate drives: {e}"))
            })?;

        // Search for the volume
        fn find_volume(
            vol_info: &storage_common::VolumeInfo,
            target_device: &str,
            parent_device: Option<String>,
        ) -> Option<storage_common::VolumeInfo> {
            // Check if this is the target volume
            if vol_info.device_path.as_deref() == Some(target_device) {
                let mut vol = vol_info.clone();
                vol.parent_path = parent_device;
                vol.children.clear(); // Flatten
                return Some(vol);
            }

            // Search children
            for child in &vol_info.children {
                if let Some(found) = find_volume(child, target_device, vol_info.device_path.clone())
                {
                    return Some(found);
                }
            }

            None
        }

        // Search all drives
        for (disk_info, volumes) in disk_volumes {
            let disk_device = disk_info.device.clone();

            for volume_info in &volumes {
                if let Some(vol_info) = find_volume(volume_info, &device, Some(disk_device.clone()))
                {
                    let json = serde_json::to_string(&vol_info).map_err(|e| {
                        tracing::error!("Failed to serialize volume info: {e}");
                        zbus::fdo::Error::Failed(format!("Serialization error: {e}"))
                    })?;

                    tracing::debug!("Found volume: device={}", device);
                    return Ok(json);
                }
            }
        }

        tracing::warn!("Volume not found: {device}");
        Err(zbus::fdo::Error::Failed(format!(
            "Volume not found: {device}"
        )))
    }

    /// Get SMART status for a specific disk
    ///
    /// Args:
    /// - device: Device identifier (e.g., "/dev/sda", "sda", or UDisks2 path)
    ///
    /// Returns: JSON-serialized SmartStatus
    ///
    /// Authorization: org.cosmic.ext.storage-service.smart-read (allow_active)
    #[authorized_interface(action = "org.cosmic.ext.storage-service.smart-read")]
    async fn get_smart_status(
        &self,
        #[zbus(connection)] _connection: &Connection,
        #[zbus(header)] _header: MessageHeader<'_>,
        device: String,
    ) -> zbus::fdo::Result<String> {
        tracing::debug!(
            "Getting SMART status for device: {device} (UID {})",
            caller.uid
        );

        // Normalize device path (add /dev/ if missing)
        let device_path = if device.starts_with("/dev/") {
            device.clone()
        } else {
            format!("/dev/{}", device)
        };

        // Get SMART info using the device path
        let smart_info = storage_dbus::get_smart_info_by_device(&device_path)
            .await
            .map_err(|e| {
                let err_str = e.to_string().to_lowercase();
                if err_str.contains("not supported") || err_str.contains("device not found") {
                    tracing::debug!("SMART not supported or device not found for {device}");
                    zbus::fdo::Error::NotSupported(
                        "SMART not supported for this device".to_string(),
                    )
                } else {
                    tracing::error!("Failed to get SMART info: {e}");
                    zbus::fdo::Error::Failed(format!("Failed to get SMART info: {e}"))
                }
            })?;

        // Convert to storage_common::SmartStatus
        let smart_status = storage_common::SmartStatus {
            device: device.clone(),
            healthy: !smart_info
                .selftest_status
                .as_ref()
                .map(|s| s.to_lowercase().contains("fail"))
                .unwrap_or(false),
            temperature_celsius: smart_info.temperature_c.map(|t| t as i16),
            power_on_hours: smart_info.power_on_hours,
            power_cycle_count: smart_info
                .attributes
                .get("Power_Cycle_Count")
                .and_then(|v| v.parse().ok()),
            test_running: smart_info
                .selftest_status
                .as_ref()
                .map(|s| {
                    s.to_lowercase().contains("progress") || s.to_lowercase().contains("running")
                })
                .unwrap_or(false),
            test_percent_remaining: None,
        };

        // Serialize to JSON
        let json = serde_json::to_string(&smart_status).map_err(|e| {
            tracing::error!("Failed to serialize SMART status: {e}");
            zbus::fdo::Error::Failed(format!("Failed to serialize SMART status: {e}"))
        })?;

        Ok(json)
    }

    /// Get detailed SMART attributes for a specific disk
    ///
    /// Args:
    /// - device: Device identifier (e.g., "/dev/sda", "sda", or UDisks2 path)
    ///
    /// Returns: JSON-serialized Vec<SmartAttribute>
    ///
    /// Authorization: org.cosmic.ext.storage-service.smart-read (allow_active)
    #[authorized_interface(action = "org.cosmic.ext.storage-service.smart-read")]
    async fn get_smart_attributes(
        &self,
        #[zbus(connection)] _connection: &Connection,
        #[zbus(header)] _header: MessageHeader<'_>,
        device: String,
    ) -> zbus::fdo::Result<String> {
        tracing::debug!(
            "Getting SMART attributes for device: {device} (UID {})",
            caller.uid
        );

        // Normalize device path
        let device_path = if device.starts_with("/dev/") {
            device.clone()
        } else {
            format!("/dev/{}", device)
        };

        // Get SMART info by device
        let smart_info = storage_dbus::get_smart_info_by_device(&device_path)
            .await
            .map_err(|e| {
                let err_str = e.to_string().to_lowercase();
                if err_str.contains("not supported") {
                    tracing::debug!("SMART not supported for {device}");
                    zbus::fdo::Error::NotSupported(
                        "SMART not supported for this device".to_string(),
                    )
                } else {
                    tracing::error!("Failed to get SMART info: {e}");
                    zbus::fdo::Error::Failed(format!("Failed to get SMART info: {e}"))
                }
            })?;

        // Convert BTreeMap<String, String> to Vec<SmartAttribute>
        let mut attributes = Vec::new();

        for (key, value) in smart_info.attributes.iter() {
            if let Ok(raw_value) = value.parse::<u64>() {
                attributes.push(storage_common::SmartAttribute {
                    id: 0,
                    name: key.clone(),
                    current: 100,
                    worst: 100,
                    threshold: 0,
                    raw_value,
                    failing: false,
                });
            }
        }

        // Serialize to JSON
        let json = serde_json::to_string(&attributes).map_err(|e| {
            tracing::error!("Failed to serialize SMART attributes: {e}");
            zbus::fdo::Error::Failed(format!("Failed to serialize SMART attributes: {e}"))
        })?;

        Ok(json)
    }

    /// Eject removable media (optical drives, USB sticks)
    ///
    /// Args:
    /// - device: Device identifier (e.g., "/dev/sda", "sda", or UDisks2 path)
    ///
    /// Authorization: org.cosmic.ext.storage-service.disk-eject (allow_active)
    #[authorized_interface(action = "org.cosmic.ext.storage-service.disk-eject")]
    async fn eject(
        &self,
        #[zbus(connection)] _connection: &Connection,
        #[zbus(header)] _header: MessageHeader<'_>,
        device: String,
    ) -> zbus::fdo::Result<()> {
        tracing::debug!("Ejecting device: {device} (UID {})", caller.uid);

        let device_path = if device.starts_with("/dev/") {
            device.clone()
        } else {
            format!("/dev/{}", device)
        };

        let disk_volumes = storage_dbus::disk::get_disks_with_volumes(&self.manager)
            .await
            .map_err(|e| {
                tracing::error!("Failed to get drives: {e}");
                zbus::fdo::Error::Failed(format!("Failed to enumerate drives: {e}"))
            })?;

        let device_name = device_path.strip_prefix("/dev/").unwrap_or(&device_path);
        let (disk_info, _) = disk_volumes
            .into_iter()
            .find(|(d, _)| {
                d.device == device_path
                    || d.device.rsplit('/').next() == Some(device_name)
                    || d.id == device_path
                    || d.id == device_name
            })
            .ok_or_else(|| {
                tracing::warn!("Device not found: {device}");
                zbus::fdo::Error::Failed(format!("Device not found: {device}"))
            })?;

        storage_dbus::eject_drive_by_device(&device_path, disk_info.ejectable)
            .await
            .map_err(|e| {
                tracing::error!("Failed to eject device: {e}");
                zbus::fdo::Error::Failed(format!("Eject failed: {e}"))
            })?;

        tracing::info!("Successfully ejected device: {device}");
        Ok(())
    }

    /// Power off a drive (external USB drives)
    ///
    /// Args:
    /// - device: Device identifier (e.g., "/dev/sda", "sda", or UDisks2 path)
    ///
    /// Authorization: org.cosmic.ext.storage-service.disk-power-off (auth_admin_keep)
    #[authorized_interface(action = "org.cosmic.ext.storage-service.disk-power-off")]
    async fn power_off(
        &self,
        #[zbus(connection)] _connection: &Connection,
        #[zbus(header)] _header: MessageHeader<'_>,
        device: String,
    ) -> zbus::fdo::Result<()> {
        tracing::debug!("Powering off device: {device} (UID {})", caller.uid);

        let device_path = if device.starts_with("/dev/") {
            device.clone()
        } else {
            format!("/dev/{}", device)
        };

        let disk_volumes = storage_dbus::disk::get_disks_with_volumes(&self.manager)
            .await
            .map_err(|e| zbus::fdo::Error::Failed(format!("Failed to enumerate drives: {e}")))?;

        let device_name = device_path.strip_prefix("/dev/").unwrap_or(&device_path);
        let (disk_info, _) = disk_volumes
            .into_iter()
            .find(|(d, _)| {
                d.device == device_path
                    || d.device.rsplit('/').next() == Some(device_name)
                    || d.id == device_path
                    || d.id == device_name
            })
            .ok_or_else(|| zbus::fdo::Error::Failed(format!("Device not found: {device}")))?;

        storage_dbus::power_off_drive_by_device(&device_path, disk_info.can_power_off)
            .await
            .map_err(|e| {
                tracing::error!("Failed to power off device: {e}");
                zbus::fdo::Error::Failed(format!("Power off failed: {e}"))
            })?;

        tracing::info!("Successfully powered off device: {device}");
        Ok(())
    }

    /// Put drive in standby mode (low power, ATA drives)
    ///
    /// Args:
    /// - device: Device identifier (e.g., "/dev/sda", "sda", or UDisks2 path)
    ///
    /// Authorization: org.cosmic.ext.storage-service.disk-standby (allow_active)
    #[authorized_interface(action = "org.cosmic.ext.storage-service.disk-standby")]
    async fn standby_now(
        &self,
        #[zbus(connection)] _connection: &Connection,
        #[zbus(header)] _header: MessageHeader<'_>,
        device: String,
    ) -> zbus::fdo::Result<()> {
        tracing::debug!("Putting device in standby: {device} (UID {})", caller.uid);

        let device_path = if device.starts_with("/dev/") {
            device.clone()
        } else {
            format!("/dev/{}", device)
        };

        let disk_volumes = storage_dbus::disk::get_disks_with_volumes(&self.manager)
            .await
            .map_err(|e| zbus::fdo::Error::Failed(format!("Failed to enumerate drives: {e}")))?;

        let device_name = device_path.strip_prefix("/dev/").unwrap_or(&device_path);
        let _disk = disk_volumes
            .into_iter()
            .find(|(d, _)| {
                d.device == device_path
                    || d.device.rsplit('/').next() == Some(device_name)
                    || d.id == device_path
                    || d.id == device_name
            })
            .ok_or_else(|| zbus::fdo::Error::Failed(format!("Device not found: {device}")))?;

        storage_dbus::standby_drive_by_device(&device_path)
            .await
            .map_err(|e| {
                tracing::error!("Failed to put device in standby: {e}");
                zbus::fdo::Error::Failed(format!("Standby failed: {e}"))
            })?;

        tracing::info!("Successfully put device in standby: {device}");
        Ok(())
    }

    /// Wake up drive from standby mode (ATA drives)
    ///
    /// Args:
    /// - device: Device identifier (e.g., "/dev/sda", "sda", or UDisks2 path)
    ///
    /// Authorization: org.cosmic.ext.storage-service.disk-standby (allow_active)
    #[authorized_interface(action = "org.cosmic.ext.storage-service.disk-standby")]
    async fn wakeup(
        &self,
        #[zbus(connection)] _connection: &Connection,
        #[zbus(header)] _header: MessageHeader<'_>,
        device: String,
    ) -> zbus::fdo::Result<()> {
        tracing::debug!("Waking up device: {device} (UID {})", caller.uid);

        let device_path = if device.starts_with("/dev/") {
            device.clone()
        } else {
            format!("/dev/{}", device)
        };

        let disk_volumes = storage_dbus::disk::get_disks_with_volumes(&self.manager)
            .await
            .map_err(|e| zbus::fdo::Error::Failed(format!("Failed to enumerate drives: {e}")))?;

        let device_name = device_path.strip_prefix("/dev/").unwrap_or(&device_path);
        let _disk = disk_volumes
            .into_iter()
            .find(|(d, _)| {
                d.device == device_path
                    || d.device.rsplit('/').next() == Some(device_name)
                    || d.id == device_path
                    || d.id == device_name
            })
            .ok_or_else(|| zbus::fdo::Error::Failed(format!("Device not found: {device}")))?;

        storage_dbus::wakeup_drive_by_device(&device_path)
            .await
            .map_err(|e| {
                tracing::error!("Failed to wake up device: {e}");
                zbus::fdo::Error::Failed(format!("Wakeup failed: {e}"))
            })?;

        tracing::info!("Successfully woke up device: {device}");
        Ok(())
    }

    /// Safely remove a drive (unmount all volumes, lock LUKS, then eject/power off)
    ///
    /// Args:
    /// - device: Device identifier (e.g., "/dev/sda", "sda", or UDisks2 path)
    ///
    /// Authorization: org.cosmic.ext.storage-service.disk-remove (auth_admin_keep)
    #[authorized_interface(action = "org.cosmic.ext.storage-service.disk-remove")]
    async fn remove(
        &self,
        #[zbus(connection)] _connection: &Connection,
        #[zbus(header)] _header: MessageHeader<'_>,
        device: String,
    ) -> zbus::fdo::Result<()> {
        tracing::debug!("Safely removing device: {device} (UID {})", caller.uid);

        let device_path = if device.starts_with("/dev/") {
            device.clone()
        } else {
            format!("/dev/{}", device)
        };

        let disk_volumes = storage_dbus::disk::get_disks_with_volumes(&self.manager)
            .await
            .map_err(|e| zbus::fdo::Error::Failed(format!("Failed to enumerate drives: {e}")))?;

        let device_name = device_path.strip_prefix("/dev/").unwrap_or(&device_path);
        let (disk_info, _) = disk_volumes
            .into_iter()
            .find(|(d, _)| {
                d.device == device_path
                    || d.device.rsplit('/').next() == Some(device_name)
                    || d.id == device_path
                    || d.id == device_name
            })
            .ok_or_else(|| zbus::fdo::Error::Failed(format!("Device not found: {device}")))?;

        storage_dbus::remove_drive_by_device(
            &device_path,
            disk_info.is_loop,
            disk_info.removable,
            disk_info.can_power_off,
        )
        .await
        .map_err(|e| {
            tracing::error!("Failed to safely remove device: {e}");
            zbus::fdo::Error::Failed(format!("Remove failed: {e}"))
        })?;

        tracing::info!("Successfully removed device: {device}");
        Ok(())
    }

    /// Start a SMART self-test
    ///
    /// Args:
    /// - device: Device identifier (e.g., "/dev/sda", "sda", or UDisks2 path)
    /// - test_type: Type of test ("short", "long", "conveyance")
    ///
    /// Authorization: org.cosmic.ext.storage-service.smart-test (auth_admin_keep)
    #[authorized_interface(action = "org.cosmic.ext.storage-service.smart-test")]
    async fn start_smart_test(
        &self,
        #[zbus(connection)] _connection: &Connection,
        #[zbus(header)] _header: MessageHeader<'_>,
        device: String,
        test_type: String,
    ) -> zbus::fdo::Result<()> {
        tracing::info!(
            "Starting SMART {} test for device: {} (UID {})",
            test_type,
            device,
            caller.uid
        );

        // Validate test type
        let test_kind = match test_type.to_lowercase().as_str() {
            "short" => storage_dbus::SmartSelfTestKind::Short,
            "extended" | "long" => storage_dbus::SmartSelfTestKind::Extended,
            _ => {
                tracing::warn!("Invalid test type: {test_type}");
                return Err(zbus::fdo::Error::InvalidArgs(format!(
                    "Invalid test type: {test_type}. Must be 'short' or 'extended'"
                )));
            }
        };

        let device_path = if device.starts_with("/dev/") {
            device.clone()
        } else {
            format!("/dev/{}", device)
        };

        let disk_volumes = storage_dbus::disk::get_disks_with_volumes(&self.manager)
            .await
            .map_err(|e| {
                tracing::error!("Failed to get drives: {e}");
                zbus::fdo::Error::Failed(format!("Failed to enumerate drives: {e}"))
            })?;

        let device_name = device_path.strip_prefix("/dev/").unwrap_or(&device_path);
        let _disk = disk_volumes
            .into_iter()
            .find(|(d, _)| {
                d.device == device_path
                    || d.device.rsplit('/').next() == Some(device_name)
                    || d.id == device_path
                    || d.id == device_name
            })
            .ok_or_else(|| {
                tracing::warn!("Device not found: {device}");
                zbus::fdo::Error::Failed(format!("Device not found: {device}"))
            })?;

        storage_dbus::start_drive_smart_selftest_by_device(&device_path, test_kind)
            .await
            .map_err(|e| {
                let err_str = e.to_string().to_lowercase();
                if err_str.contains("not supported") {
                    tracing::debug!("SMART self-test not supported for {device}");
                    zbus::fdo::Error::NotSupported(
                        "SMART self-test not supported for this device".to_string(),
                    )
                } else {
                    tracing::error!("Failed to start SMART self-test: {e}");
                    zbus::fdo::Error::Failed(format!("Failed to start SMART self-test: {e}"))
                }
            })?;

        tracing::info!(
            "SMART {} test started successfully for {}",
            test_type,
            device
        );
        Ok(())
    }
}

/// Monitor UDisks2 for disk hotplug events and emit D-Bus signals
///
/// This function subscribes to UDisks2's InterfacesAdded and InterfacesRemoved signals
/// and emits DiskAdded/DiskRemoved signals when drives are hotplugged.
pub async fn monitor_hotplug_events(
    connection: zbus::Connection,
    object_path: &str,
    manager: storage_dbus::DiskManager,
) -> Result<()> {
    use std::collections::HashMap;
    use zbus::zvariant::{OwnedObjectPath, OwnedValue};

    tracing::info!("Starting disk hotplug monitoring");

    // Create proxy to UDisks2 ObjectManager
    let obj_manager = zbus::Proxy::new(
        &connection,
        "org.freedesktop.UDisks2",
        "/org/freedesktop/UDisks2",
        "org.freedesktop.DBus.ObjectManager",
    )
    .await?;

    // Get signal emitter for our DisksHandler
    let object_server = connection.object_server();
    let iface_ref = object_server
        .interface::<_, DisksHandler>(object_path)
        .await?;

    // Subscribe to InterfacesAdded signal
    let mut added_stream = obj_manager.receive_signal("InterfacesAdded").await?;

    // Subscribe to InterfacesRemoved signal
    let mut removed_stream = obj_manager.receive_signal("InterfacesRemoved").await?;

    // Spawn task to handle added signals
    let iface_ref_clone = iface_ref.clone();
    let manager_clone = manager;
    tokio::spawn(async move {
        use futures_util::StreamExt;

        while let Some(signal) = added_stream.next().await {
            match signal.body().deserialize::<(
                OwnedObjectPath,
                HashMap<String, HashMap<String, OwnedValue>>,
            )>() {
                Ok((object_path, interfaces)) => {
                    // Check if this is a Drive interface being added
                    if interfaces.contains_key("org.freedesktop.UDisks2.Drive") {
                        tracing::debug!("Drive added: {}", object_path);

                        // Get the drive info
                        match get_disk_info_for_path(&manager_clone, &object_path.as_ref()).await {
                            Ok(disk_info) => {
                                let device = disk_info.device.clone();
                                match serde_json::to_string(&disk_info) {
                                    Ok(json) => {
                                        tracing::info!("Disk added: {}", device);

                                        // Emit signal
                                        if let Err(e) = DisksHandler::disk_added(
                                            iface_ref_clone.signal_emitter(),
                                            &device,
                                            &json,
                                        )
                                        .await
                                        {
                                            tracing::error!(
                                                "Failed to emit disk_added signal: {}",
                                                e
                                            );
                                        }
                                    }
                                    Err(e) => {
                                        tracing::error!("Failed to serialize disk info: {}", e);
                                    }
                                }
                            }
                            Err(e) => {
                                tracing::error!(
                                    "Failed to get disk info for {}: {}",
                                    object_path,
                                    e
                                );
                            }
                        }
                    }
                }
                Err(e) => {
                    tracing::error!("Failed to parse InterfacesAdded signal: {}", e);
                }
            }
        }
    });

    // Spawn task to handle removed signals
    let iface_ref_clone = iface_ref.clone();
    tokio::spawn(async move {
        use futures_util::StreamExt;

        while let Some(signal) = removed_stream.next().await {
            match signal
                .body()
                .deserialize::<(OwnedObjectPath, Vec<String>)>()
            {
                Ok((object_path, interfaces)) => {
                    // Check if Drive interface is being removed
                    if interfaces.contains(&"org.freedesktop.UDisks2.Drive".to_string()) {
                        // Extract device name from object path
                        // e.g., /org/freedesktop/UDisks2/drives/Samsung_SSD_970_EVO_S1234 -> device path
                        let device = format!(
                            "/dev/{}",
                            object_path.as_str().rsplit('/').next().unwrap_or("unknown")
                        );

                        tracing::info!("Disk removed: {} ({})", device, object_path);

                        // Emit signal
                        if let Err(e) =
                            DisksHandler::disk_removed(iface_ref_clone.signal_emitter(), &device)
                                .await
                        {
                            tracing::error!("Failed to emit disk_removed signal: {}", e);
                        }
                    }
                }
                Err(e) => {
                    tracing::error!("Failed to parse InterfacesRemoved signal: {}", e);
                }
            }
        }
    });

    tracing::info!("Disk hotplug monitoring started");
    Ok(())
}

/// Helper function to get DiskInfo for a specific UDisks2 drive object path
async fn get_disk_info_for_path(
    manager: &storage_dbus::DiskManager,
    object_path: &zbus::zvariant::ObjectPath<'_>,
) -> Result<storage_common::DiskInfo> {
    storage_dbus::get_disk_info_for_drive_path(manager, object_path.as_str())
        .await
        .map_err(|e| anyhow::anyhow!("{}", e))
}
