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
    /// List all disks on the system
    /// 
    /// Returns a JSON-serialized array of DiskInfo objects.
    /// 
    /// **Authorization:** Requires `disks-read` (allow_active)
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
            "org.cosmic.ext.storage-service.disks-read",
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
    /// **Authorization:** Requires `disks-read` (allow_active)
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
            "org.cosmic.ext.storage-service.disks-read",
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
}
