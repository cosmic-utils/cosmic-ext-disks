// SPDX-License-Identifier: GPL-3.0-only

//! Filesystem check and repair operations

use std::collections::HashMap;
use udisks2::filesystem::FilesystemProxy;
use zbus::{Connection, zvariant::{OwnedObjectPath, Value}};
use crate::error::DiskError;

/// Check and repair a filesystem
///
/// Returns true if filesystem is clean, false if errors were found (and repaired if repair=true)
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

/// Repair a filesystem
///
/// This is a convenience wrapper around check_filesystem with repair=true
pub async fn repair_filesystem(device_path: &str) -> Result<(), DiskError> {
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
    let ok = fs_proxy.repair(opts).await
        .map_err(|e| DiskError::OperationFailed(format!("Filesystem repair failed: {}", e)))?;
    
    if ok {
        Ok(())
    } else {
        Err(DiskError::OperationFailed(
            "Filesystem repair completed but reported failure".to_string()
        ))
    }
}
