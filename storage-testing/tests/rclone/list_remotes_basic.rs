use async_trait::async_trait;

use storage_testing::harness::support;
use storage_testing::tests::{HarnessContext, HarnessTest};

pub struct RcloneListRemotesBasic;

#[async_trait]
impl HarnessTest for RcloneListRemotesBasic {
    fn id(&self) -> &'static str {
        "rclone.list_remotes.basic"
    }

    fn suite(&self) -> &'static str {
        "rclone"
    }

    fn required_spec(&self) -> &'static str {
        "2disk"
    }

    async fn execute(&self, _ctx: &HarnessContext) -> storage_testing::errors::Result<()> {
        let client = support::rclone_client().await?;
        let _ = client.list_remotes().await;
        Ok(())
    }
}
