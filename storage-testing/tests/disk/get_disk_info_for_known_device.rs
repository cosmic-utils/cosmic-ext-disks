use async_trait::async_trait;

use storage_testing::harness::support;
use storage_testing::tests::{HarnessContext, HarnessTest};

pub struct DiskGetDiskInfoForKnownDevice;

#[async_trait]
impl HarnessTest for DiskGetDiskInfoForKnownDevice {
    fn id(&self) -> &'static str {
        "disk.get_disk_info.for_known_device"
    }

    fn suite(&self) -> &'static str {
        "disk"
    }

    fn required_spec(&self) -> &'static str {
        "2disk"
    }

    async fn execute(&self, _ctx: &HarnessContext) -> storage_testing::errors::Result<()> {
        let client = support::disks_client().await?;
        let configured_device = support::env("STORAGE_TESTING_DISK_DEVICE");

        let disks = match client.list_disks().await {
            Ok(disks) => disks,
            Err(error) if support::should_skip(&error) => {
                return support::skip(format!("disk info prefetch: {error}"));
            }
            Err(error) => return support::failure(format!("disk prefetch failed: {error}")),
        };

        let target_device = if let Some(configured) = configured_device {
            if disks.iter().any(|disk| disk.device == configured) {
                configured
            } else if let Some(first_disk) = disks.first() {
                first_disk.device.clone()
            } else {
                return support::skip("no disks available");
            }
        } else if let Some(first_disk) = disks.first() {
            first_disk.device.clone()
        } else {
            return support::skip("no disks available");
        };

        let info = client
            .get_disk_info(&target_device)
            .await
            .map_err(|error| storage_testing::errors::TestingError::TestFailed {
                reason: format!("disk info failed: {error}"),
            })?;

        if info.device != target_device {
            return support::failure("disk info device mismatch");
        }

        Ok(())
    }
}
