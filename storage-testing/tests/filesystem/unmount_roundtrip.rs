use async_trait::async_trait;

use storage_testing::harness::support;
use storage_testing::tests::{HarnessContext, HarnessTest};

pub struct FilesystemMountUnmountRoundtrip;

#[async_trait]
impl HarnessTest for FilesystemMountUnmountRoundtrip {
    fn id(&self) -> &'static str {
        "filesystem.mount.unmount.roundtrip"
    }

    fn suite(&self) -> &'static str {
        "filesystem"
    }

    fn required_spec(&self) -> &'static str {
        "2disk"
    }

    fn exclusive(&self) -> bool {
        true
    }

    async fn execute(&self, _ctx: &HarnessContext) -> storage_testing::errors::Result<()> {
        support::require_destructive("filesystem.mount.unmount.roundtrip")?;
        let client = support::filesystems_client().await?;
        let disks = support::disks_client().await?;
        let device = support::require_env("STORAGE_TESTING_MOUNT_DEVICE")?;
        let mount_point = support::env("STORAGE_TESTING_MOUNT_POINT")
            .unwrap_or_else(|| "/tmp/storage-testing-mount".to_string());

        let _ = support::client_result(
            client.unmount(&device, true, true).await,
            "pre-unmount filesystem by device",
        );
        let _ = support::client_result(
            client.unmount(&mount_point, true, true).await,
            "pre-unmount filesystem by mount point",
        );

        let mounted_at = support::client_result(
            client.mount(&device, &mount_point, None).await,
            "mount filesystem",
        )?;
        let volumes_after_mount =
            support::client_result(disks.list_volumes().await, "list volumes after mount")?;
        let mounted_volume = volumes_after_mount
            .iter()
            .find(|volume| volume.device_path.as_deref() == Some(device.as_str()))
            .ok_or_else(|| storage_testing::errors::TestingError::TestFailed {
                reason: format!("mounted device not visible in volume list: {device}"),
            })?;
        if !mounted_volume
            .mount_points
            .iter()
            .any(|entry| entry == &mounted_at)
        {
            return support::failure(format!(
                "expected mount point {mounted_at} to be present for {device}"
            ));
        }

        support::client_result(
            client.unmount(&mounted_at, false, false).await,
            "unmount filesystem",
        )?;
        let volumes_after_unmount =
            support::client_result(disks.list_volumes().await, "list volumes after unmount")?;
        let unmounted_volume = volumes_after_unmount
            .iter()
            .find(|volume| volume.device_path.as_deref() == Some(device.as_str()))
            .ok_or_else(|| storage_testing::errors::TestingError::TestFailed {
                reason: format!("device missing after unmount: {device}"),
            })?;
        if !unmounted_volume.mount_points.is_empty() {
            return support::failure(format!(
                "expected no mount points after unmount, found {:?}",
                unmounted_volume.mount_points
            ));
        }
        Ok(())
    }
}
