use async_trait::async_trait;

use storage_testing::harness::support;
use storage_testing::tests::{HarnessContext, HarnessTest};

pub struct FilesystemMountOptionsReadWriteRoundtrip;

#[async_trait]
impl HarnessTest for FilesystemMountOptionsReadWriteRoundtrip {
    fn id(&self) -> &'static str {
        "filesystem.mount_options.read_write_roundtrip"
    }

    fn suite(&self) -> &'static str {
        "filesystem"
    }

    fn required_spec(&self) -> &'static str {
        "2disk"
    }

    async fn execute(&self, _ctx: &HarnessContext) -> storage_testing::errors::Result<()> {
        let client = support::filesystems_client().await?;
        let device = support::require_env("STORAGE_TESTING_MOUNT_OPTIONS_DEVICE")?;

        support::client_result(
            client.get_mount_options(&device).await,
            "get mount options before reset",
        )?;
        support::client_result(
            client.default_mount_options(&device).await,
            "clear mount options",
        )?;
        let after = support::client_result(
            client.get_mount_options(&device).await,
            "get mount options after reset",
        )?;
        if after.is_some() {
            return support::failure(format!(
                "expected no persistent mount options after reset for {device}"
            ));
        }
        Ok(())
    }
}
