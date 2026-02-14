// SPDX-License-Identifier: GPL-3.0-only

//! Disk discovery and management D-Bus interface
//!
//! This module provides D-Bus methods for listing disks, getting disk information,
//! and monitoring disk hotplug events.

use anyhow::Result;
use disks_dbus::DiskManager;
use serde_json;
use zbus::{interface, Connection};

use crate::auth::check_polkit_auth;

/// D-Bus interface for disk discovery and SMART operations
pub struct DisksHandler {
    manager: DiskManager,
}

impl DisksHandler {
    /// Create a new DisksHandler
    pub async fn new() -> Result<Self> {
        let manager = DiskManager::new().await?;
        Ok(Self { manager })
    }
    
    /// Get the underlying DiskManager for internal use
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
    async fn list_disks(&self, #[zbus(connection)] connection: &Connection) -> zbus::fdo::Result<String> {
        // Check Polkit authorization
        check_polkit_auth(
            connection,
            "org.cosmic.ext.storage-service.disk-read",
        )
        .await
        .map_err(|e| zbus::fdo::Error::from(e))?;
        
        tracing::debug!("ListDisks called");
        
        // Get disks from disks-dbus using new storage-models API
        let disks = disks_dbus::DriveModel::get_disks()
            .await
            .map_err(|e| {
                tracing::error!("Failed to get disks: {e}");
                zbus::fdo::Error::Failed(format!("Failed to enumerate disks: {e}"))
            })?;
        
        tracing::debug!("Found {} disks", disks.len());
        
        // Serialize to JSON
        let json = serde_json::to_string(&disks)
            .map_err(|e| {
                tracing::error!("Failed to serialize disks: {e}");
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
    async fn get_disk_info(
        &self,
        device: String,
        #[zbus(connection)] connection: &Connection,
    ) -> zbus::fdo::Result<String> {
        // Check Polkit authorization
        check_polkit_auth(
            connection,
            "org.cosmic.ext.storage-service.disk-read",
        )
        .await
        .map_err(|e| zbus::fdo::Error::from(e))?;
        
        tracing::debug!("GetDiskInfo called for device: {device}");
        
        // Get all disks and find the requested one
        let disks = disks_dbus::DriveModel::get_disks()
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
        let device_name = device.strip_prefix("/dev/").unwrap_or(&device);
        
        // Try to find the disk by matching device name
        // The device field contains UDisks2 paths like "/org/freedesktop/UDisks2/block_devices/sda"
        let disk = disks
            .into_iter()
            .find(|d| {
                // Exact match on full device field (handles UDisks2 paths)
                if d.device == device {
                    return true;
                }
                
                // Extract the device name from the UDisks2 path (last component)
                // e.g., "/org/freedesktop/UDisks2/block_devices/sda" -> "sda"
                if let Some(disk_name) = d.device.rsplit('/').next() {
                    if disk_name == device_name {
                        return true;
                    }
                }
                
                // Also check if id matches (for serial/model lookups)
                if d.id == device || d.id == device_name {
                    return true;
                }
                
                false
            })
            .ok_or_else(|| {
                tracing::warn!("Device not found: {device}");
                zbus::fdo::Error::Failed(format!("Device not found: {device}"))
            })?;
        
        tracing::debug!("Found disk: device={}, id={}", disk.device, disk.id);
        
        // Serialize to JSON
        let json = serde_json::to_string(&disk)
            .map_err(|e| {
                tracing::error!("Failed to serialize disk info: {e}");
                zbus::fdo::Error::Failed(format!("Serialization error: {e}"))
            })?;
        
        Ok(json)
    }
    
    /// Get SMART status for a specific disk
    /// 
    /// Args:
    /// - device: Device identifier (e.g., "/dev/sda", "sda", or UDisks2 path)
    /// 
    /// Returns: JSON-serialized SmartStatus
    /// 
    /// Authorization: org.cosmic.ext.storage-service.smart-read (allow_active)
    async fn get_smart_status(
        &self,
        #[zbus(connection)] connection: &zbus::Connection,
        device: String,
    ) -> zbus::fdo::Result<String> {
        // Check authorization
        crate::auth::check_polkit_auth(connection, "org.cosmic.ext.storage-service.smart-read")
            .await
            .map_err(|e| zbus::fdo::Error::Failed(format!("Authorization failed: {e}")))?;
        
        tracing::debug!("Getting SMART status for device: {device}");
        
        // Get all drives (DriveModel instances, not DiskInfo)
        let drives = disks_dbus::DriveModel::get_drives()
            .await
            .map_err(|e| {
                tracing::error!("Failed to get drives: {e}");
                zbus::fdo::Error::Failed(format!("Failed to enumerate drives: {e}"))
            })?;
        
        // Extract device name from input (strip "/dev/" prefix if present)
        let device_name = device.strip_prefix("/dev/").unwrap_or(&device);
        
        // Find the matching drive
        let drive_model = drives
            .into_iter()
            .find(|d| {
                let disk_info: storage_models::DiskInfo = d.clone().into();
                // Match on device field
                if disk_info.device == device {
                    return true;
                }
                // Extract the device name from the UDisks2 path
                if let Some(disk_name) = disk_info.device.rsplit('/').next() {
                    if disk_name == device_name {
                        return true;
                    }
                }
                // Also check if id matches
                if disk_info.id == device || disk_info.id == device_name {
                    return true;
                }
                false
            })
            .ok_or_else(|| {
                tracing::warn!("Device not found: {device}");
                zbus::fdo::Error::Failed(format!("Device not found: {device}"))
            })?;
        
        // Get SMART info from the drive
        let smart_info = drive_model
            .smart_info()
            .await
            .map_err(|e| {
                let err_str = e.to_string().to_lowercase();
                if err_str.contains("not supported") {
                    tracing::debug!("SMART not supported for {device}");
                    zbus::fdo::Error::NotSupported(format!("SMART not supported for this device"))
                } else {
                    tracing::error!("Failed to get SMART info: {e}");
                    zbus::fdo::Error::Failed(format!("Failed to get SMART info: {e}"))
                }
            })?;
        
        // Convert to storage_models::SmartStatus
        let smart_status = storage_models::SmartStatus {
            device: device.clone(),
            healthy: !smart_info.selftest_status.as_ref()
                .map(|s| s.to_lowercase().contains("fail"))
                .unwrap_or(false),
            temperature_celsius: smart_info.temperature_c.map(|t| t as i16),
            power_on_hours: smart_info.power_on_hours,
            power_cycle_count: smart_info.attributes.get("Power_Cycle_Count")
                .and_then(|v| v.parse().ok()),
            test_running: smart_info.selftest_status.as_ref()
                .map(|s| s.to_lowercase().contains("progress") || s.to_lowercase().contains("running"))
                .unwrap_or(false),
            test_percent_remaining: None,
        };
        
        // Serialize to JSON
        let json = serde_json::to_string(&smart_status)
            .map_err(|e| {
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
    async fn get_smart_attributes(
        &self,
        #[zbus(connection)] connection: &zbus::Connection,
        device: String,
    ) -> zbus::fdo::Result<String> {
        // Check authorization
        crate::auth::check_polkit_auth(connection, "org.cosmic.ext.storage-service.smart-read")
            .await
            .map_err(|e| zbus::fdo::Error::Failed(format!("Authorization failed: {e}")))?;
        
        tracing::debug!("Getting SMART attributes for device: {device}");
        
        // Get all drives (DriveModel instances, not DiskInfo)
        let drives = disks_dbus::DriveModel::get_drives()
            .await
            .map_err(|e| {
                tracing::error!("Failed to get drives: {e}");
                zbus::fdo::Error::Failed(format!("Failed to enumerate drives: {e}"))
            })?;
        
        // Extract device name from input (strip "/dev/" prefix if present)
        let device_name = device.strip_prefix("/dev/").unwrap_or(&device);
        
        // Find the matching drive
        let drive_model = drives
            .into_iter()
            .find(|d| {
                let disk_info: storage_models::DiskInfo = d.clone().into();
                if disk_info.device == device {
                    return true;
                }
                if let Some(disk_name) = disk_info.device.rsplit('/').next() {
                    if disk_name == device_name {
                        return true;
                    }
                }
                if disk_info.id == device || disk_info.id == device_name {
                    return true;
                }
                false
            })
            .ok_or_else(|| {
                tracing::warn!("Device not found: {device}");
                zbus::fdo::Error::Failed(format!("Device not found: {device}"))
            })?;
        
        // Get SMART info from the drive
        let smart_info = drive_model
            .smart_info()
            .await
            .map_err(|e| {
                let err_str = e.to_string().to_lowercase();
                if err_str.contains("not supported") {
                   tracing::debug!("SMART not supported for {device}");
                    zbus::fdo::Error::NotSupported(format!("SMART not supported for this device"))
                } else {
                    tracing::error!("Failed to get SMART info: {e}");
                    zbus::fdo::Error::Failed(format!("Failed to get SMART info: {e}"))
                }
            })?;
        
        // Convert BTreeMap<String, String> to Vec<SmartAttribute>
        let mut attributes = Vec::new();
        
        for (key, value) in smart_info.attributes.iter() {
            if let Ok(raw_value) = value.parse::<u64>() {
                attributes.push(storage_models::SmartAttribute {
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
        let json = serde_json::to_string(&attributes)
            .map_err(|e| {
                tracing::error!("Failed to serialize SMART attributes: {e}");
                zbus::fdo::Error::Failed(format!("Failed to serialize SMART attributes: {e}"))
            })?;
        
        Ok(json)
    }
    
    /// Start a SMART self-test
    /// 
    /// Args:
    /// - device: Device identifier (e.g., "/dev/sda", "sda", or UDisks2 path)
    /// - test_type: Test type ("short" or "extended")
    /// 
    /// Authorization: org.cosmic.ext.storage-service.smart-test (auth_admin_keep)
    async fn start_smart_test(
        &self,
        #[zbus(connection)] connection: &zbus::Connection,
        device: String,
        test_type: String,
    ) -> zbus::fdo::Result<()> {
        // Check authorization (requires admin password)
        crate::auth::check_polkit_auth(connection, "org.cosmic.ext.storage-service.smart-test")
            .await
            .map_err(|e| zbus::fdo::Error::Failed(format!("Authorization failed: {e}")))?;
        
        tracing::info!("Starting SMART {} test for device: {}", test_type, device);
        
        // Validate test type
        let test_kind = match test_type.to_lowercase().as_str() {
            "short" => disks_dbus::SmartSelfTestKind::Short,
            "extended" | "long" => disks_dbus::SmartSelfTestKind::Extended,
            _ => {
                tracing::warn!("Invalid test type: {test_type}");
                return Err(zbus::fdo::Error::InvalidArgs(
                    format!("Invalid test type: {test_type}. Must be 'short' or 'extended'")
                ));
            }
        };
        
        // Get all drives (DriveModel instances, not DiskInfo)
        let drives = disks_dbus::DriveModel::get_drives()
            .await
            .map_err(|e| {
                tracing::error!("Failed to get drives: {e}");
                zbus::fdo::Error::Failed(format!("Failed to enumerate drives: {e}"))
            })?;
        
        // Extract device name from input (strip "/dev/" prefix if present)
        let device_name = device.strip_prefix("/dev/").unwrap_or(&device);
        
        // Find the matching drive
        let drive_model = drives
            .into_iter()
            .find(|d| {
                let disk_info: storage_models::DiskInfo = d.clone().into();
                if disk_info.device == device {
                    return true;
                }
                if let Some(disk_name) = disk_info.device.rsplit('/').next() {
                    if disk_name == device_name {
                        return true;
                    }
                }
                if disk_info.id == device || disk_info.id == device_name {
                    return true;
                }
                false
            })
            .ok_or_else(|| {
                tracing::warn!("Device not found: {device}");
                zbus::fdo::Error::Failed(format!("Device not found: {device}"))
            })?;
        
        // Start the self-test
        drive_model
            .smart_selftest_start(test_kind)
            .await
            .map_err(|e| {
                let err_str = e.to_string().to_lowercase();
                if err_str.contains("not supported") {
                    tracing::debug!("SMART self-test not supported for {device}");
                    zbus::fdo::Error::NotSupported(format!("SMART self-test not supported for this device"))
                } else {
                    tracing::error!("Failed to start SMART self-test: {e}");
                    zbus::fdo::Error::Failed(format!("Failed to start SMART self-test: {e}"))
                }
            })?;
        
        tracing::info!("SMART {} test started successfully for {}", test_type, device);
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
) -> Result<()> {
    use zbus::zvariant::{OwnedObjectPath, OwnedValue};
    use std::collections::HashMap;
    
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
    let mut added_stream = obj_manager
        .receive_signal("InterfacesAdded")
        .await?;
    
    // Subscribe to InterfacesRemoved signal  
    let mut removed_stream = obj_manager
        .receive_signal("InterfacesRemoved")
        .await?;
    
    // Spawn task to handle added signals
    let connection_clone = connection.clone();
    let iface_ref_clone = iface_ref.clone();
    tokio::spawn(async move {
        use futures_util::StreamExt;
        
        loop {
            match added_stream.next().await {
                Some(signal) => {
                    match signal.body().deserialize::<(OwnedObjectPath, HashMap<String, HashMap<String, OwnedValue>>)>() {
                        Ok((object_path, interfaces)) => {
                            // Check if this is a Drive interface being added
                            if interfaces.contains_key("org.freedesktop.UDisks2.Drive") {
                                tracing::debug!("Drive added: {}", object_path);
                                
                                // Get the drive info
                                match get_disk_info_for_path(&connection_clone, &object_path.as_ref()).await {
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
                                                ).await {
                                                    tracing::error!("Failed to emit disk_added signal: {}", e);
                                                }
                                            }
                                            Err(e) => {
                                                tracing::error!("Failed to serialize disk info: {}", e);
                                            }
                                        }
                                    }
                                    Err(e) => {
                                        tracing::error!("Failed to get disk info for {}: {}", object_path, e);
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            tracing::error!("Failed to parse InterfacesAdded signal: {}", e);
                        }
                    }
                }
                None => break,
            }
        }
    });
    
    // Spawn task to handle removed signals
    let iface_ref_clone = iface_ref.clone();
    tokio::spawn(async move {
        use futures_util::StreamExt;
        
        loop {
            match removed_stream.next().await {
                Some(signal) => {
                    match signal.body().deserialize::<(OwnedObjectPath, Vec<String>)>() {
                        Ok((object_path, interfaces)) => {
                            // Check if Drive interface is being removed
                            if interfaces.contains(&"org.freedesktop.UDisks2.Drive".to_string()) {
                                // Extract device name from object path
                                // e.g., /org/freedesktop/UDisks2/drives/Samsung_SSD_970_EVO_S1234 -> device path
                                let device = format!("/dev/{}", object_path.as_str()
                                    .rsplit('/')
                                    .next()
                                    .unwrap_or("unknown"));
                                
                                tracing::info!("Disk removed: {} ({})", device, object_path);
                                
                                // Emit signal
                                if let Err(e) = DisksHandler::disk_removed(
                                    iface_ref_clone.signal_emitter(),
                                    &device,
                                ).await {
                                    tracing::error!("Failed to emit disk_removed signal: {}", e);
                                }
                            }
                        }
                        Err(e) => {
                            tracing::error!("Failed to parse InterfacesRemoved signal: {}", e);
                        }
                    }
                }
                None => break,
            }
        }
    });
    
    tracing::info!("Disk hotplug monitoring started");
    Ok(())
}

/// Helper function to get DiskInfo for a specific UDisks2 object path
async fn get_disk_info_for_path(
    _connection: &zbus::Connection,
    object_path: &zbus::zvariant::ObjectPath<'_>,
) -> Result<storage_models::DiskInfo> {
    // Get all drives and find the one matching this object path
    let drives = disks_dbus::DriveModel::get_drives().await?;
    
    for drive in drives {
        // Check if this drive's path matches
        if drive.path.as_str() == object_path.as_str() {
            let disk_info: storage_models::DiskInfo = drive.into();
            return Ok(disk_info);
        }
    }
    
    Err(anyhow::anyhow!("Drive not found for path: {}", object_path))
}
