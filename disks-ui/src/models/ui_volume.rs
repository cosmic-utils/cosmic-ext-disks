// SPDX-License-Identifier: GPL-3.0-only

//! UI model for a volume with hierarchical children and owned client

use storage_models::VolumeInfo;
use crate::client::{FilesystemsClient, error::ClientError};
use std::ops::Deref;

/// UI model wrapping VolumeInfo with owned client and hierarchical children
/// 
/// This model:
/// - Owns a FilesystemsClient for filesystem operations
/// - Maintains hierarchical children relationships
/// - Provides helper methods for recursive searches
/// - Supports atomic updates
#[derive(Debug)]
pub struct UiVolume {
    /// Volume information from storage-service
    pub volume: VolumeInfo,
    
    /// Child volumes (e.g., unlocked LUKS containers, nested partitions)
    pub children: Vec<UiVolume>,
    
    /// Owned client for filesystem operations
    client: FilesystemsClient,
}

impl UiVolume {
    /// Create a new UiVolume from VolumeInfo
    /// 
    /// Note: children is empty initially - call build_volume_tree() to populate.
    pub async fn new(volume: VolumeInfo) -> Result<Self, ClientError> {
        let client = FilesystemsClient::new().await?;
        Ok(Self {
            volume,
            children: Vec::new(),
            client,
        })
    }
    
    /// Create a UiVolume with children (used by tree builder)
    /// 
    /// Note: This uses a blocking client creation internally.
    pub fn with_children(volume: VolumeInfo, children: Vec<UiVolume>) -> Result<Self, ClientError> {
        // Create client using tokio block_on
        let rt = tokio::runtime::Runtime::new()
            .map_err(|e| ClientError::Connection(format!("Failed to create runtime: {}", e)))?;
        let client = rt.block_on(FilesystemsClient::new())?;
        
        Ok(Self {
            volume,
            children,
            client,
        })
    }
    
    /// Find a volume by device path (recursive search)
    /// 
    /// # Example
    /// ```no_run
    /// if let Some(volume) = drive.volumes[0].find_by_device("/dev/sda1") {
    ///     println!("Found: {:?}", volume.volume.label);
    /// }
    /// ```
    pub fn find_by_device(&self, device: &str) -> Option<&UiVolume> {
        // Check self
        if self.volume.device_path.as_ref().map_or(false, |d| d == device) {
            return Some(self);
        }
        
        // Search children recursively
        for child in &self.children {
            if let Some(found) = child.find_by_device(device) {
                return Some(found);
            }
        }
        
        None
    }
    
    /// Find a volume by device path (mutable, recursive search)
    pub fn find_by_device_mut(&mut self, device: &str) -> Option<&mut UiVolume> {
        // Check self
        if self.volume.device_path.as_ref().map_or(false, |d| d == device) {
            return Some(self);
        }
        
        // Search children recursively
        for child in &mut self.children {
            if let Some(found) = child.find_by_device_mut(device) {
                return Some(found);
            }
        }
        
        None
    }
    
    /// Collect all mounted descendants (recursive)
    /// 
    /// Used for operations that need to unmount a volume tree
    /// (e.g., before deleting a partition).
    /// 
    /// # Example
    /// ```no_run
    /// let mounted = volume.collect_mounted_descendants();
    /// for descendant in mounted {
    ///     fs_client.unmount(&descendant, false, false).await?;
    /// }
    /// ```
    pub fn collect_mounted_descendants(&self) -> Vec<String> {
        let mut result = Vec::new();
        
        // Check self
        if !self.volume.mount_points.is_empty() {
            if let Some(device) = &self.volume.device_path {
                result.push(device.clone());
            }
        }
        
        // Collect from children recursively
        for child in &self.children {
            result.extend(child.collect_mounted_descendants());
        }
        
        result
    }
    
    /// Update this volume or a child with new VolumeInfo (atomic update)
    /// 
    /// Returns true if the volume was found and updated.
    /// 
    /// # Example
    /// ```no_run
    /// // After mounting /dev/sda1
    /// if !root_volume.update_volume("/dev/sda1", &updated_info) {
    ///     // Volume not found in this subtree
    /// }
    /// ```
    pub fn update_volume(&mut self, device: &str, updated_info: &VolumeInfo) -> bool {
        // Check if this is the target volume
        if self.volume.device_path.as_ref().map_or(false, |d| d == device) {
            // Update volume info, preserving children
            let old_children = std::mem::take(&mut self.volume.children);
            self.volume = updated_info.clone();
            self.volume.children = old_children;
            return true;
        }
        
        // Search children
        for child in &mut self.children {
            if child.update_volume(device, updated_info) {
                return true;
            }
        }
        
        false
    }
    
    /// Add a child volume (used for atomic tree mutations)
    /// 
    /// # Example
    /// ```no_run
    /// // After unlocking a LUKS container
    /// locked_partition.add_child(unlocked_volume);
    /// ```
    pub fn add_child(&mut self, child: UiVolume) {
        self.children.push(child);
    }
    
    /// Remove a child volume by device path
    /// 
    /// Returns true if the child was found and removed.
    pub fn remove_child(&mut self, device: &str) -> bool {
        // Try to remove direct child
        if let Some(idx) = self.children.iter().position(|c| {
            c.volume.device_path.as_ref().map_or(false, |d| d == device)
        }) {
            self.children.remove(idx);
            return true;
        }
        
        // Try to remove from nested children
        for child in &mut self.children {
            if child.remove_child(device) {
                return true;
            }
        }
        
        false
    }
    
    /// Get device path if available
    pub fn device(&self) -> Option<&str> {
        self.volume.device_path.as_deref()
    }
    
    /// Get filesystem client for operations
    pub fn filesystem_client(&self) -> &FilesystemsClient {
        &self.client
    }
    
    /// Check if this volume can be mounted
    pub fn can_mount(&self) -> bool {
        self.volume.can_mount()
    }
    
    /// Check if this volume is mounted
    pub fn is_mounted(&self) -> bool {
        self.volume.is_mounted()
    }
    
    /// Get object_path (device_path) for compatibility during migration
    pub fn object_path(&self) -> Option<String> {
        self.volume.device_path.clone()
    }
}

/// Deref to VolumeInfo to expose all volume fields directly
impl Deref for UiVolume {
    type Target = VolumeInfo;
    
    fn deref(&self) -> &Self::Target {
        &self.volume
    }
}

/// Clone creates a shallow clone of data but with new client instance.
/// This is acceptable since clients are lightweight D-Bus proxies.
impl Clone for UiVolume {
    fn clone(&self) -> Self {
        // Create new client instance (blocking runtime for sync context)
        let rt = tokio::runtime::Runtime::new().expect("Failed to create runtime");
        let client = rt.block_on(FilesystemsClient::new()).expect("Failed to create FilesystemsClient");
        
        Self {
            volume: self.volume.clone(),
            children: self.children.clone(),
            client,
        }
    }
}

// Note: Clone creates a shallow clone that shares the same client connection state.
// This is acceptable for UI rendering purposes.
