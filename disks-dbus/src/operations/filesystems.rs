// SPDX-License-Identifier: GPL-3.0-only

//! Filesystem operations via UDisks2

use std::collections::HashMap;
use udisks2::{block::BlockProxy, filesystem::FilesystemProxy};
use zbus::{Connection, zvariant::{OwnedObjectPath, Value}};
use crate::disks::DiskError;
use storage_models::{FormatOptions, MountOptions};

/// Helper function to find UDisks2 block object path for a device
async fn find_block_object_path(device_path: &str) -> Result<OwnedObjectPath, DiskError> {
    let drives = crate::DriveModel::get_drives().await
        .map_err(|e| DiskError::OperationFailed(format!("Failed to get drives: {}", e)))?;
    
    for drive in drives {
        for volume in &drive.volumes {
            if let Some(ref dev_path) = volume.device_path {
                if dev_path == device_path {
                    return Ok(volume.object_path.clone());
                }
            }
        }
    }
    
    Err(DiskError::DeviceNotFound(device_path.to_string()))
}

/// Format a filesystem
pub async fn format_filesystem(
    device_path: &str,
    fs_type: &str,
    label: &str,
    _options: FormatOptions,
) -> Result<(), DiskError> {
    let connection = Connection::system().await
        .map_err(|e| DiskError::ConnectionFailed(e.to_string()))?;
    
    // Find block device object path
    let block_path = find_block_object_path(device_path).await?;
    
    // Format using Block.Format
    let block_proxy = BlockProxy::builder(&connection)
        .path(&block_path)
        .map_err(|e| DiskError::DBusError(e.to_string()))?
        .build()
        .await
        .map_err(|e| DiskError::DBusError(e.to_string()))?;
    
    let mut format_opts: HashMap<&str, Value<'_>> = HashMap::new();
    if !label.is_empty() {
        format_opts.insert("label", Value::from(label));
    }
    
    block_proxy.format(fs_type, format_opts).await
        .map_err(|e| DiskError::OperationFailed(format!("Format failed: {}", e)))?;
    
    Ok(())
}

/// Mount a filesystem
pub async fn mount_filesystem(
    device_path: &str,
    mount_point: &str,
    options: MountOptions,
) -> Result<String, DiskError> {
    let connection = Connection::system().await
        .map_err(|e| DiskError::ConnectionFailed(e.to_string()))?;
    
    // Find filesystem object path
    let drives = crate::DriveModel::get_drives().await
        .map_err(|e| DiskError::OperationFailed(format!("Failed to get drives: {}", e)))?;
    let mut fs_path: Option<OwnedObjectPath> = None;
    
    for drive in drives {
        for volume in &drive.volumes {
            if let Some(ref dev_path) = volume.device_path {
                if dev_path == device_path {
                    fs_path = Some(volume.object_path.clone());
                    break;
                }
            }
        }
    }
    
    let fs_path = fs_path.ok_or_else(|| 
        DiskError::DeviceNotFound(device_path.to_string()))?;
    
    let fs_proxy = FilesystemProxy::builder(&connection)
        .path(&fs_path)
        .map_err(|e| DiskError::DBusError(e.to_string()))?
        .build()
        .await
        .map_err(|e| DiskError::DBusError(e.to_string()))?;
    
    // Build mount options
    let mut opts: HashMap<&str, Value<'_>> = HashMap::new();
    let mut options_vec = Vec::new();
    
    if options.read_only {
        options_vec.push("ro");
    }
    if options.no_exec {
        options_vec.push("noexec");
    }
    if options.no_suid {
        options_vec.push("nosuid");
    }
    for opt in &options.other {
        options_vec.push(opt.as_str());
    }
    
    if !options_vec.is_empty() {
        opts.insert("options", Value::from(options_vec.join(",")));
    }
    
    // Mount
    let mount_point_bytes = fs_proxy.mount(opts).await
        .map_err(|e| DiskError::OperationFailed(format!("Mount failed: {}", e)))?;
    
    // mount_point_bytes is a String returned by UDisks2
    Ok(mount_point_bytes)
}

/// Unmount a filesystem
pub async fn unmount_filesystem(device_or_mount: &str, force: bool) -> Result<(), DiskError> {
    let connection = Connection::system().await
        .map_err(|e| DiskError::ConnectionFailed(e.to_string()))?;
    
    // Find filesystem object path
    let drives = crate::DriveModel::get_drives().await
        .map_err(|e| DiskError::OperationFailed(format!("Failed to get drives: {}", e)))?;
    let mut fs_path: Option<OwnedObjectPath> = None;
    
    for drive in drives {
        for volume in &drive.volumes {
            if let Some(ref dev_path) = volume.device_path {
                if dev_path == device_or_mount {
                    fs_path = Some(volume.object_path.clone());
                    break;
                }
            }
            // Also check mount points
            for mp in &volume.mount_points {
                if mp == device_or_mount {
                    fs_path = Some(volume.object_path.clone());
                    break;
                }
            }
        }
    }
    
    let fs_path = fs_path.ok_or_else(|| 
        DiskError::DeviceNotFound(device_or_mount.to_string()))?;
    
    let fs_proxy = FilesystemProxy::builder(&connection)
        .path(&fs_path)
        .map_err(|e| DiskError::DBusError(e.to_string()))?
        .build()
        .await
        .map_err(|e| DiskError::DBusError(e.to_string()))?;
    
    let mut opts: HashMap<&str, Value<'_>> = HashMap::new();
    if force {
        opts.insert("force", Value::from(true));
    }
    
    fs_proxy.unmount(opts).await
        .map_err(|e| DiskError::OperationFailed(format!("Unmount failed: {}", e)))?;
    
    Ok(())
}

/// Check and repair a filesystem
pub async fn check_filesystem(device_path: &str, repair: bool) -> Result<bool, DiskError> {
    let connection = Connection::system().await
        .map_err(|e| DiskError::ConnectionFailed(e.to_string()))?;
    
    // Find filesystem object path
    let drives = crate::DriveModel::get_drives().await
        .map_err(|e| DiskError::OperationFailed(format!("Failed to get drives: {}", e)))?;
    let mut fs_path: Option<OwnedObjectPath> = None;
    
    for drive in drives {
        for volume in &drive.volumes {
            if let Some(ref dev_path) = volume.device_path {
                if dev_path == device_path {
                    fs_path = Some(volume.object_path.clone());
                    break;
                }
            }
        }
    }
    
    let fs_path = fs_path.ok_or_else(|| 
        DiskError::DeviceNotFound(device_path.to_string()))?;
    
    let fs_proxy = FilesystemProxy::builder(&connection)
        .path(&fs_path)
        .map_err(|e| DiskError::DBusError(e.to_string()))?
        .build()
        .await
        .map_err(|e| DiskError::DBusError(e.to_string()))?;
    
    let mut opts: HashMap<&str, Value<'_>> = HashMap::new();
    opts.insert("repair", Value::from(repair));
    
    let result = fs_proxy.check(opts).await
        .map_err(|e| DiskError::OperationFailed(format!("Filesystem check failed: {}", e)))?;
    
    Ok(result)
}

/// Set filesystem label
pub async fn set_filesystem_label(device_path: &str, label: &str) -> Result<(), DiskError> {
    let connection = Connection::system().await
        .map_err(|e| DiskError::ConnectionFailed(e.to_string()))?;
    
    // Find filesystem object path
    let drives = crate::DriveModel::get_drives().await
        .map_err(|e| DiskError::OperationFailed(format!("Failed to get drives: {}", e)))?;
    let mut fs_path: Option<OwnedObjectPath> = None;
    
    for drive in drives {
        for volume in &drive.volumes {
            if let Some(ref dev_path) = volume.device_path {
                if dev_path == device_path {
                    fs_path = Some(volume.object_path.clone());
                    break;
                }
            }
        }
    }
    
    let fs_path = fs_path.ok_or_else(|| 
        DiskError::DeviceNotFound(device_path.to_string()))?;
    
    let fs_proxy = FilesystemProxy::builder(&connection)
        .path(&fs_path)
        .map_err(|e| DiskError::DBusError(e.to_string()))?
        .build()
        .await
        .map_err(|e| DiskError::DBusError(e.to_string()))?;
    
    let opts: HashMap<&str, Value<'_>> = HashMap::new();
    fs_proxy.set_label(label, opts).await
        .map_err(|e| DiskError::OperationFailed(format!("Set label failed: {}", e)))?;
    
    Ok(())
}

/// Get filesystem label for a device
///
/// # Arguments
/// * `device` - Device path (e.g., "/dev/sda1")
///
/// # Returns
/// The filesystem label (may be empty string if no label set)
pub async fn get_filesystem_label(device: &str) -> Result<String, DiskError> {
    let connection = Connection::system()
        .await
        .map_err(|e| DiskError::ConnectionFailed(format!("Failed to connect to system bus: {}", e)))?;
    
    let block_path = find_block_object_path(device).await?;
    
    let block_proxy = BlockProxy::builder(&connection)
        .path(&block_path)
        .map_err(|e| DiskError::InvalidPath(format!("Invalid block path: {}", e)))?
        .build()
        .await
        .map_err(|e| DiskError::DBusError(e.to_string()))?;
    
    let label = block_proxy.id_label().await
        .unwrap_or_default();
    
    Ok(label)
}

/// Take ownership of a mounted filesystem
///
/// # Arguments
/// * `device` - Device path (e.g., "/dev/sda1")
/// * `recursive` - Take ownership of child mounts
pub async fn take_filesystem_ownership(device: &str, recursive: bool) -> Result<(), DiskError> {
    let connection = Connection::system()
        .await
        .map_err(|e| DiskError::ConnectionFailed(format!("Failed to connect to system bus: {}", e)))?;
    
    let block_path = find_block_object_path(device).await?;
    
    let fs_proxy = FilesystemProxy::builder(&connection)
        .path(&block_path)
        .map_err(|e| DiskError::InvalidPath(format!("Invalid filesystem path: {}", e)))?
        .build()
        .await
        .map_err(|e| DiskError::DBusError(e.to_string()))?;
    
    let mut options: HashMap<&str, Value<'_>> = HashMap::new();
    options.insert("recursive", Value::from(recursive));
    
    fs_proxy
        .take_ownership(options)
        .await
        .map_err(|e| DiskError::OperationFailed(format!("Take ownership failed: {}", e)))?;
    
    Ok(())
}

/// Get the mount point for a mounted device
///
/// # Arguments
/// * `device` - Device path (e.g., "/dev/sda1")
///
/// # Returns
/// The mount point path, or error if not mounted
pub async fn get_mount_point(device: &str) -> Result<String, DiskError> {
    let connection = Connection::system()
        .await
        .map_err(|e| DiskError::ConnectionFailed(format!("Failed to connect to system bus: {}", e)))?;
    
    let block_path = find_block_object_path(device).await?;
    
    let fs_proxy = FilesystemProxy::builder(&connection)
        .path(&block_path)
        .map_err(|e| DiskError::InvalidPath(format!("Invalid filesystem path: {}", e)))?
        .build()
        .await
        .map_err(|e| DiskError::DBusError(e.to_string()))?;
    
    let mount_points = fs_proxy.mount_points().await
        .map_err(|e| DiskError::DBusError(e.to_string()))?;
    
    if mount_points.is_empty() {
        return Err(DiskError::OperationFailed("Device is not mounted".to_string()));
    }
    
    // mount_points returns Vec<Vec<u8>> with each mount point as null-terminated string
    let mount_str = String::from_utf8(
        mount_points[0].clone().into_iter().filter(|&b| b != 0).collect()
    ).map_err(|e| DiskError::OperationFailed(format!("Invalid mount point encoding: {}", e)))?;
    
    Ok(mount_str)
}
