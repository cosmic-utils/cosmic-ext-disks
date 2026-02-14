// SPDX-License-Identifier: GPL-3.0-only

//! Type conversions between disks-dbus and storage-models
//! 
//! This module provides helpers to convert from the rich disks-dbus types
//! (VolumeNode, DriveModel, etc.) to the minimal transport types in storage-models.
//! 
//! **NOTE:** This entire file is temporary and will be removed during Phase 3A refactoring.
//! Once disks-dbus returns storage-models types directly, no conversion will be needed.

use storage_models::disk::DiskInfo;

/// Convert disks_dbus::DriveModel to storage_models::DiskInfo
pub fn drive_model_to_disk_info(drive: &disks_dbus::DriveModel) -> DiskInfo {
    // Determine connection bus from vendor/model strings and block path
    let connection_bus = infer_connection_bus(
        &drive.vendor,
        &drive.model,
        &drive.block_path,
        drive.is_loop,
    );
    
    // Rotation rate: Some(rpm) for HDDs, None for SSDs/NVMe
    let rotation_rate = if drive.rotation_rate > 0 {
        Some(drive.rotation_rate as u16)
    } else {
        None
    };
    
    DiskInfo {
        // Identity
        device: drive.block_path.clone(),
        id: drive.id.clone(),
        model: drive.model.clone(),
        serial: drive.serial.clone(),
        vendor: drive.vendor.clone(),
        revision: drive.revision.clone(),
        
        // Physical properties
        size: drive.size,
        connection_bus,
        rotation_rate,
        
        // Media properties
        removable: drive.removable,
        ejectable: drive.ejectable,
        media_removable: drive.media_removable,
        media_available: drive.media_available,
        optical: drive.optical,
        optical_blank: drive.optical_blank,
        can_power_off: drive.can_power_off,
        
        // Loop device
        is_loop: drive.is_loop,
        backing_file: drive.backing_file.clone(),
        
        // Partitioning (from Block.IdType / PartitionTable in disks-dbus)
        partition_table_type: drive.partition_table_type.clone(),
        gpt_usable_range: drive.gpt_usable_range,
    }
}

/// Infer connection bus type from drive properties
fn infer_connection_bus(vendor: &str, model: &str, block_path: &str, is_loop: bool) -> String {
    if is_loop {
        return "loop".to_string();
    }
    
    let path_lower = block_path.to_lowercase();
    let model_lower = model.to_lowercase();
    let vendor_lower = vendor.to_lowercase();
    
    // Check block device path patterns
    if path_lower.contains("nvme") {
        return "nvme".to_string();
    }
    if path_lower.contains("mmc") || path_lower.contains("mmcblk") {
        return "mmc".to_string();
    }
    if path_lower.contains("sr") {
        return "optical".to_string();
    }
    
    // Check vendor/model for USB indicators
    if model_lower.contains("usb") || vendor_lower.contains("usb") {
        return "usb".to_string();
    }
    
    // Default to ata/sata for traditional disks (sd*)
    "ata".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_infer_nvme_bus() {
        assert_eq!(infer_connection_bus("", "", "/dev/nvme0n1", false), "nvme");
    }
    
    #[test]
    fn test_infer_loop_bus() {
        assert_eq!(infer_connection_bus("", "", "/dev/loop0", true), "loop");
    }
    
    #[test]
    fn test_infer_usb_from_model() {
        assert_eq!(infer_connection_bus("", "USB DISK", "/dev/sdb", false), "usb");
    }
    
    #[test]
    fn test_infer_default_ata() {
        assert_eq!(infer_connection_bus("", "", "/dev/sda", false), "ata");
    }
}
