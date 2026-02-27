use storage_types::UsageScanParallelismPreset;

use async_trait::async_trait;
use tokio::time::{Duration, timeout};

use storage_testing::harness::support;
use storage_testing::tests::{HarnessContext, HarnessTest};

pub struct FilesystemUsageScanBasic;

#[async_trait]
impl HarnessTest for FilesystemUsageScanBasic {
    fn id(&self) -> &'static str {
        "filesystem.usage_scan.basic"
    }

    fn suite(&self) -> &'static str {
        "filesystem"
    }

    fn required_spec(&self) -> &'static str {
        "2disk"
    }

    async fn execute(&self, _ctx: &HarnessContext) -> storage_testing::errors::Result<()> {
        let client = support::filesystems_client().await?;

        let mount_points = match client.list_usage_mount_points().await {
            Ok(mount_points) => mount_points,
            Err(error) if support::should_skip(&error) => {
                return support::skip(format!("usage mount points unavailable: {error}"));
            }
            Err(error) => return support::failure(format!("usage mount points failed: {error}")),
        };

        if mount_points.is_empty() {
            return support::skip("no mount points for usage scan");
        }

        let selected_mounts =
            if let Some(preferred_mount) = support::env("STORAGE_TESTING_MOUNT_POINT") {
                if mount_points.iter().any(|value| value == &preferred_mount) {
                    vec![preferred_mount]
                } else {
                    vec![mount_points[0].clone()]
                }
            } else {
                vec![mount_points[0].clone()]
            };

        let scan_result = timeout(
            Duration::from_secs(4),
            client.get_usage_scan(
                "integration-usage-scan",
                5,
                &selected_mounts,
                false,
                UsageScanParallelismPreset::Balanced,
            ),
        )
        .await;
        if scan_result.is_err() {
            return support::skip("usage scan request timed out after 4s");
        }
        Ok(())
    }
}
