use async_trait::async_trait;

use storage_testing::harness::support;
use storage_testing::tests::{HarnessContext, HarnessTest};

pub struct RcloneMountStatusQuery;

#[async_trait]
impl HarnessTest for RcloneMountStatusQuery {
    fn id(&self) -> &'static str {
        "rclone.mount_status.query"
    }

    fn suite(&self) -> &'static str {
        "rclone"
    }

    fn required_spec(&self) -> &'static str {
        "2disk"
    }

    async fn execute(&self, _ctx: &HarnessContext) -> storage_testing::errors::Result<()> {
        let client = support::rclone_client().await?;
        let name = support::env("STORAGE_TESTING_RCLONE_NAME").unwrap_or_else(|| "test".to_string());
        let scope = support::env("STORAGE_TESTING_RCLONE_SCOPE").unwrap_or_else(|| "user".to_string());
        let _ = client.get_mount_status(&name, &scope).await;
        Ok(())
    }
}
