use crate::client::FilesystemsClient;
use crate::models::UiDrive;
use crate::ui::dialogs::state::ImageOperationKind;
use storage_models::VolumeInfo;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use tokio::fs::OpenOptions;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

/// Open a block device for reading (for backup).
fn open_device_for_read(path: &str) -> anyhow::Result<std::fs::File> {
    Ok(std::fs::OpenOptions::new().read(true).open(path)?)
}

/// Open a block device for writing (for restore).
fn open_device_for_write(path: &str) -> anyhow::Result<std::fs::File> {
    Ok(std::fs::OpenOptions::new().write(true).open(path)?)
}

async fn copy_with_cancel<R, W>(
    mut reader: R,
    mut writer: W,
    cancel: Arc<AtomicBool>,
) -> anyhow::Result<u64>
where
    R: tokio::io::AsyncRead + Unpin,
    W: tokio::io::AsyncWrite + Unpin,
{
    let mut buf = vec![0u8; 4 * 1024 * 1024];
    let mut total: u64 = 0;

    loop {
        if cancel.load(Ordering::Relaxed) {
            anyhow::bail!("Cancelled");
        }

        let n = reader.read(&mut buf).await?;
        if n == 0 {
            break;
        }

        writer.write_all(&buf[..n]).await?;
        total = total.saturating_add(n as u64);
    }

    writer.flush().await?;
    Ok(total)
}

pub(super) async fn run_image_operation(
    kind: ImageOperationKind,
    drive: UiDrive,
    partition: Option<VolumeInfo>,
    image_path: String,
    cancel: Arc<AtomicBool>,
) -> anyhow::Result<()> {
    match kind {
        ImageOperationKind::CreateFromDrive => {
            let path = &drive.disk.device;
            let file = open_device_for_read(path)?;
            let reader = tokio::fs::File::from_std(file);
            let writer = OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&image_path)
                .await?;

            let _bytes = copy_with_cancel(reader, writer, cancel).await?;
            Ok(())
        }
        ImageOperationKind::CreateFromPartition => {
            let Some(partition) = partition else {
                anyhow::bail!("No partition selected");
            };

            let path = partition
                .device_path
                .as_deref()
                .ok_or_else(|| anyhow::anyhow!("Partition has no device path"))?;
            let file = open_device_for_read(path)?;
            let reader = tokio::fs::File::from_std(file);
            let writer = OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&image_path)
                .await?;

            let _bytes = copy_with_cancel(reader, writer, cancel).await?;
            Ok(())
        }
        ImageOperationKind::RestoreToDrive => {
            // Preflight: attempt to unmount all mounted partitions.
            let fs_client = FilesystemsClient::new().await
                .map_err(|e| anyhow::anyhow!("Failed to create filesystems client: {}", e))?;
            
            for p in &drive.volumes_flat {
                if p.is_mounted() {
                    let device = p.volume.device_path.as_ref()
                        .ok_or_else(|| anyhow::anyhow!("Partition has no device path"))?;
                    fs_client.unmount(device, false, false).await
                        .map_err(|e| anyhow::anyhow!("Failed to unmount {}: {}", device, e))?;
                }
            }

            let src_meta = tokio::fs::metadata(&image_path).await?;
            if src_meta.len() > drive.disk.size {
                anyhow::bail!(
                    "Image is larger than the selected drive (image={} bytes, drive={} bytes)",
                    src_meta.len(),
                    drive.disk.size
                );
            }

            let src = tokio::fs::File::open(&image_path).await?;
            let file = open_device_for_write(&drive.disk.device)?;
            let dest = tokio::fs::File::from_std(file);

            let _bytes = copy_with_cancel(src, dest, cancel).await?;
            Ok(())
        }
        ImageOperationKind::RestoreToPartition => {
            let Some(partition) = partition else {
                anyhow::bail!("No partition selected");
            };

            if partition.is_mounted() {
                let fs_client = FilesystemsClient::new().await
                    .map_err(|e| anyhow::anyhow!("Failed to create filesystems client: {}", e))?;
                let device = partition.device_path.as_ref()
                    .ok_or_else(|| anyhow::anyhow!("Partition has no device path"))?;
                fs_client.unmount(device, false, false).await
                    .map_err(|e| anyhow::anyhow!("Failed to unmount: {}", e))?;
            }

            let src_meta = tokio::fs::metadata(&image_path).await?;
            if src_meta.len() > partition.size {
                anyhow::bail!(
                    "Image is larger than the selected partition (image={} bytes, partition={} bytes)",
                    src_meta.len(),
                    partition.size
                );
            }

            let path = partition
                .device_path
                .as_deref()
                .ok_or_else(|| anyhow::anyhow!("Partition has no device path"))?;
            let src = tokio::fs::File::open(&image_path).await?;
            let file = open_device_for_write(path)?;
            let dest = tokio::fs::File::from_std(file);

            let _bytes = copy_with_cancel(src, dest, cancel).await?;
            Ok(())
        }
    }
}
