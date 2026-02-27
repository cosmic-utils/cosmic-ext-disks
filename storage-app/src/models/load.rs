// SPDX-License-Identifier: GPL-3.0-only

//! Helper functions for loading UiDrive instances from storage-service

use super::UiDrive;
use std::time::Instant;
use storage_contracts::client::{DisksClient, error::ClientError};
use storage_types::DiskInfo;

fn drive_section_label(disk: &DiskInfo) -> &'static str {
    if disk.is_loop || disk.backing_file.is_some() {
        "images"
    } else if disk.removable {
        "external"
    } else {
        "internal"
    }
}

pub async fn load_drive_candidates() -> Result<Vec<DiskInfo>, ClientError> {
    let client = DisksClient::new().await?;
    client.list_disks().await
}

pub async fn build_drive_timed(disk: DiskInfo) -> (Result<UiDrive, String>, u128) {
    let start = Instant::now();
    let device = disk.device.clone();
    let section = drive_section_label(&disk);

    let result = UiDrive::new(disk).await.map_err(|e| e.to_string());
    let elapsed_ms = start.elapsed().as_millis();

    match &result {
        Ok(_) => tracing::info!(%device, section, elapsed_ms, "drive build complete"),
        Err(error) => tracing::info!(%device, section, elapsed_ms, %error, "drive build failed"),
    }

    (result, elapsed_ms)
}

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
    let disks = load_drive_candidates().await?;

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
