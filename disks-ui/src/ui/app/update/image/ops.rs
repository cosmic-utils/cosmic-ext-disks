use crate::ui::dialogs::state::ImageOperationKind;
use disks_dbus::DriveModel;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use tokio::fs::OpenOptions;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

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
    drive: DriveModel,
    partition: Option<disks_dbus::VolumeModel>,
    image_path: String,
    cancel: Arc<AtomicBool>,
) -> anyhow::Result<()> {
    match kind {
        ImageOperationKind::CreateFromDrive => {
            let fd = drive.open_for_backup().await?;
            let reader = tokio::fs::File::from_std(std::fs::File::from(fd));
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

            let fd = partition.open_for_backup().await?;
            let reader = tokio::fs::File::from_std(std::fs::File::from(fd));
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
            for p in &drive.volumes_flat {
                if p.is_mounted() {
                    p.unmount().await?;
                }
            }

            let src_meta = tokio::fs::metadata(&image_path).await?;
            if src_meta.len() > drive.size {
                anyhow::bail!(
                    "Image is larger than the selected drive (image={} bytes, drive={} bytes)",
                    src_meta.len(),
                    drive.size
                );
            }

            let src = tokio::fs::File::open(&image_path).await?;
            let fd = drive.open_for_restore().await?;
            let dest = tokio::fs::File::from_std(std::fs::File::from(fd));

            let _bytes = copy_with_cancel(src, dest, cancel).await?;
            Ok(())
        }
        ImageOperationKind::RestoreToPartition => {
            let Some(partition) = partition else {
                anyhow::bail!("No partition selected");
            };

            if partition.is_mounted() {
                partition.unmount().await?;
            }

            let src_meta = tokio::fs::metadata(&image_path).await?;
            if src_meta.len() > partition.size {
                anyhow::bail!(
                    "Image is larger than the selected partition (image={} bytes, partition={} bytes)",
                    src_meta.len(),
                    partition.size
                );
            }

            let src = tokio::fs::File::open(&image_path).await?;
            let fd = partition.open_for_restore().await?;
            let dest = tokio::fs::File::from_std(std::fs::File::from(fd));

            let _bytes = copy_with_cancel(src, dest, cancel).await?;
            Ok(())
        }
    }
}
