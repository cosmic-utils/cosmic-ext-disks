use async_trait::async_trait;

use storage_testing::harness::support;
use storage_testing::tests::{HarnessContext, HarnessTest};

pub struct FilesystemCheckReadonlyPath;

#[async_trait]
impl HarnessTest for FilesystemCheckReadonlyPath {
    fn id(&self) -> &'static str {
        "filesystem.check.readonly_path"
    }

    fn suite(&self) -> &'static str {
        "filesystem"
    }

    fn required_spec(&self) -> &'static str {
        "2disk"
    }

    async fn execute(&self, _ctx: &HarnessContext) -> storage_testing::errors::Result<()> {
        let client = support::filesystems_client().await?;
        let device = support::require_env("STORAGE_TESTING_CHECK_DEVICE")?;
        let _ = client.check(&device, false).await;
        Ok(())
    }
}
