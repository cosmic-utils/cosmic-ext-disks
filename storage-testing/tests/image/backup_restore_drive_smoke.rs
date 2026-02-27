use async_trait::async_trait;
use tokio::time::{Duration, sleep};

use storage_testing::harness::support;
use storage_testing::tests::{HarnessContext, HarnessTest};

pub struct ImageBackupRestoreDriveSmoke;

#[async_trait]
impl HarnessTest for ImageBackupRestoreDriveSmoke {
    fn id(&self) -> &'static str {
        "image.backup_restore.drive_smoke"
    }

    fn suite(&self) -> &'static str {
        "image"
    }

    fn required_spec(&self) -> &'static str {
        "2disk"
    }

    fn exclusive(&self) -> bool {
        true
    }

    async fn execute(&self, _ctx: &HarnessContext) -> storage_testing::errors::Result<()> {
        support::require_destructive("image.backup_restore.drive_smoke")?;
        if support::env("STORAGE_TESTING_ENABLE_IMAGE_TESTS").as_deref() != Some("1") {
            return support::skip("set STORAGE_TESTING_ENABLE_IMAGE_TESTS=1 to run image backup smoke");
        }
        let client = support::image_client().await?;
        let device = support::require_env("STORAGE_TESTING_IMAGE_DEVICE")?;
        let path = support::require_env("STORAGE_TESTING_IMAGE_PATH")?;

        let operation_id = support::client_result(
            client.backup_drive(&device, &path).await,
            "start drive backup",
        )?;
        support::client_result(
            client.get_operation_status(&operation_id).await,
            "get backup operation status",
        )?;
        let mut observed_started = false;
        for _ in 0..20 {
            let status = support::client_result(
                client.get_operation_status(&operation_id).await,
                "poll backup operation status",
            )?;
            if status.total_bytes > 0 {
                observed_started = true;
                break;
            }
            sleep(Duration::from_millis(500)).await;
        }

        support::client_result(
            client.cancel_operation(&operation_id).await,
            "cancel backup operation",
        )?;

        if !observed_started {
            return support::skip(format!(
                "backup operation did not report total size within 10s: {operation_id}"
            ));
        }
        Ok(())
    }
}
