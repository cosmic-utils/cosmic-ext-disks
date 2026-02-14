// SPDX-License-Identifier: GPL-3.0-only

//! Helper functions for loading UiDrive instances from storage-service

use crate::client::{DisksClient, error::ClientError};
use super::UiDrive;

/// Load all drives from storage-service as UiDrive instances
/// 
/// Each UiDrive is created with its own client and initial data load.
/// 
/// # Example
/// ```no_run
/// let drives = load_all_drives().await?;
/// for drive in drives {
///     println!("Drive: {} ({} volumes)", drive.device(), drive.volumes.len());
/// }
/// ```
pub async fn load_all_drives() -> Result<Vec<UiDrive>, ClientError> {
    let client = DisksClient::new().await?;
    let disks = client.list_disks().await?;
    
    let mut drives = Vec::new();
    for disk in disks {
        match UiDrive::new(disk).await {
            Ok(drive) => drives.push(drive),
            Err(e) => {
                tracing::warn!("Failed to load drive data: {}", e);
                // Continue with other drives even if one fails
            }
        }
    }
    
    Ok(drives)
}
