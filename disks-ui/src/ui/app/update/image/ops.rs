//! Image backup/restore via storage-service (Phase 1: start operation, return operation_id).
//! Progress and completion are handled by subscription in the app.

use crate::client::{FilesystemsClient, ImageClient};
use crate::models::UiDrive;
use crate::ui::dialogs::state::ImageOperationKind;
use storage_models::VolumeInfo;

/// Start a backup or restore operation via the storage-service.
/// Returns the operation_id for progress tracking and cancel.
/// Caller is responsible for unmounting before restore (this function does it).
pub(super) async fn start_image_operation(
    kind: ImageOperationKind,
    drive: UiDrive,
    partition: Option<VolumeInfo>,
    image_path: String,
) -> anyhow::Result<String> {
    let image_client = ImageClient::new()
        .await
        .map_err(|e| anyhow::anyhow!("Failed to create image client: {}", e))?;

    match kind {
        ImageOperationKind::CreateFromDrive => {
            let device = &drive.disk.device;
            let operation_id = image_client
                .backup_drive(device, &image_path)
                .await
                .map_err(|e| anyhow::anyhow!("Failed to start backup: {}", e))?;
            Ok(operation_id)
        }
        ImageOperationKind::CreateFromPartition => {
            let Some(ref p) = partition else {
                anyhow::bail!("No partition selected");
            };
            let device = p
                .device_path
                .as_deref()
                .ok_or_else(|| anyhow::anyhow!("Partition has no device path"))?;
            let operation_id = image_client
                .backup_partition(device, &image_path)
                .await
                .map_err(|e| anyhow::anyhow!("Failed to start backup: {}", e))?;
            Ok(operation_id)
        }
        ImageOperationKind::RestoreToDrive => {
            let fs_client = FilesystemsClient::new()
                .await
                .map_err(|e| anyhow::anyhow!("Failed to create filesystems client: {}", e))?;
            for p in &drive.volumes_flat {
                if p.is_mounted() {
                    let device = p
                        .volume
                        .device_path
                        .as_ref()
                        .ok_or_else(|| anyhow::anyhow!("Partition has no device path"))?;
                    fs_client
                        .unmount(device, false, false)
                        .await
                        .map_err(|e| anyhow::anyhow!("Failed to unmount {}: {}", device, e))?;
                }
            }
            let device = &drive.disk.device;
            let operation_id = image_client
                .restore_drive(device, &image_path)
                .await
                .map_err(|e| anyhow::anyhow!("Failed to start restore: {}", e))?;
            Ok(operation_id)
        }
        ImageOperationKind::RestoreToPartition => {
            let Some(ref p) = partition else {
                anyhow::bail!("No partition selected");
            };
            if p.is_mounted() {
                let fs_client = FilesystemsClient::new()
                    .await
                    .map_err(|e| anyhow::anyhow!("Failed to create filesystems client: {}", e))?;
                let device = p
                    .device_path
                    .as_ref()
                    .ok_or_else(|| anyhow::anyhow!("Partition has no device path"))?;
                fs_client
                    .unmount(device, false, false)
                    .await
                    .map_err(|e| anyhow::anyhow!("Failed to unmount: {}", e))?;
            }
            let device = p
                .device_path
                .as_deref()
                .ok_or_else(|| anyhow::anyhow!("Partition has no device path"))?;
            let operation_id = image_client
                .restore_partition(device, &image_path)
                .await
                .map_err(|e| anyhow::anyhow!("Failed to start restore: {}", e))?;
            Ok(operation_id)
        }
    }
}
