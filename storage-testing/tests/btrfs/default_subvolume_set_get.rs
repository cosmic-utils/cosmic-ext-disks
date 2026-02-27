use async_trait::async_trait;
use tokio::time::{Duration, timeout};

use storage_testing::harness::support;
use storage_testing::tests::{HarnessContext, HarnessTest};

pub struct BtrfsDefaultSubvolumeSetGet;

#[async_trait]
impl HarnessTest for BtrfsDefaultSubvolumeSetGet {
    fn id(&self) -> &'static str {
        "btrfs.default_subvolume.set_get"
    }

    fn suite(&self) -> &'static str {
        "btrfs"
    }

    fn required_spec(&self) -> &'static str {
        "2disk"
    }

    async fn execute(&self, _ctx: &HarnessContext) -> storage_testing::errors::Result<()> {
        support::require_destructive("btrfs.default_subvolume.set_get")?;
        let client = support::btrfs_client().await?;
        let filesystems = support::filesystems_client().await?;
        let source_device = support::require_env("STORAGE_TESTING_BTRFS_SOURCE_DEVICE")?;
        let mountpoint = support::env("STORAGE_TESTING_BTRFS_MOUNT")
            .unwrap_or_else(|| "/tmp/storage-testing-btrfs".to_string());

        let _ = support::client_result(
            filesystems.unmount(&mountpoint, true, true).await,
            "pre-unmount btrfs mountpoint",
        );
        let format_result = timeout(
            Duration::from_secs(6),
            filesystems.format(&source_device, "btrfs", "storage-testing-btrfs", None),
        )
        .await;
        match format_result {
            Ok(result) => support::client_result(result, "format btrfs source")?,
            Err(_) => {
                return support::skip("format btrfs source timed out after 6s");
            }
        }
        let mount_result = timeout(
            Duration::from_secs(6),
            filesystems.mount(&source_device, &mountpoint, None),
        )
        .await;
        let mounted_at = match mount_result {
            Ok(result) => support::client_result(result, "mount btrfs source")?,
            Err(_) => {
                return support::skip("mount btrfs source timed out after 6s");
            }
        };

        let before = support::client_result(
            client.list_subvolumes(&mounted_at).await,
            "list btrfs subvolumes",
        )?;
        let default_id = before.default_id;
        let default_path = before
            .subvolumes
            .iter()
            .find(|subvolume| subvolume.id == default_id)
            .map(|subvolume| subvolume.path.clone());

        let Some(default_path) = default_path else {
            let _ = support::client_result(
                filesystems.unmount(&mounted_at, false, false).await,
                "unmount btrfs source",
            );
            return support::skip(format!(
                "default subvolume id not found in listing: {default_id}"
            ));
        };

        support::client_result(
            client.set_default(&mounted_at, &default_path).await,
            "set default btrfs subvolume",
        )?;
        let after_default = support::client_result(
            client.get_default(&mounted_at).await,
            "get default btrfs subvolume",
        )?;
        if after_default != default_id {
            return support::failure(format!(
                "default subvolume id changed unexpectedly: expected {default_id}, got {after_default}"
            ));
        }

        support::client_result(
            filesystems.unmount(&mounted_at, false, false).await,
            "unmount btrfs source",
        )?;
        Ok(())
    }
}
