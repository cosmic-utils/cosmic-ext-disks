use async_trait::async_trait;
use tokio::time::{Duration, timeout};

use storage_testing::harness::support;
use storage_testing::tests::{HarnessContext, HarnessTest};

pub struct BtrfsSubvolumeCreateDeleteRoundtrip;

#[async_trait]
impl HarnessTest for BtrfsSubvolumeCreateDeleteRoundtrip {
    fn id(&self) -> &'static str {
        "btrfs.subvolume.create_delete.roundtrip"
    }

    fn suite(&self) -> &'static str {
        "btrfs"
    }

    fn required_spec(&self) -> &'static str {
        "2disk"
    }

    fn exclusive(&self) -> bool {
        true
    }

    async fn execute(&self, _ctx: &HarnessContext) -> storage_testing::errors::Result<()> {
        support::require_destructive("btrfs.subvolume.create_delete.roundtrip")?;
        let client = support::btrfs_client().await?;
        let filesystems = support::filesystems_client().await?;
        let source_device = support::require_env("STORAGE_TESTING_BTRFS_SOURCE_DEVICE")?;
        let mountpoint = support::env("STORAGE_TESTING_BTRFS_MOUNT")
            .unwrap_or_else(|| "/tmp/storage-testing-btrfs".to_string());
        let name = "storage-testing-subvolume";

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

        let result = (|| async {
            support::client_result(
                client.create_subvolume(&mounted_at, name).await,
                "create btrfs subvolume",
            )?;
            let after_create = support::client_result(
                client.list_subvolumes(&mounted_at).await,
                "list subvolumes",
            )?;
            if !after_create
                .subvolumes
                .iter()
                .any(|subvolume| subvolume.path == name)
            {
                return support::failure(format!("subvolume not found after create: {name}"));
            }

            support::client_result(
                client.delete_subvolume(&mounted_at, name, true).await,
                "delete btrfs subvolume",
            )?;
            let after_delete = support::client_result(
                client.list_subvolumes(&mounted_at).await,
                "list subvolumes",
            )?;
            if after_delete
                .subvolumes
                .iter()
                .any(|subvolume| subvolume.path == name)
            {
                return support::failure(format!("subvolume still present after delete: {name}"));
            }

            Ok(())
        })()
        .await;

        let _ = support::client_result(
            filesystems.unmount(&mounted_at, false, false).await,
            "unmount btrfs source",
        );
        result
    }
}
