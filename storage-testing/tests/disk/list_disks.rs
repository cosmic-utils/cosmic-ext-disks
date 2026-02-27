use async_trait::async_trait;

use storage_testing::harness::support;
use storage_testing::tests::{HarnessContext, HarnessTest};

pub struct DiskListDisks;

#[async_trait]
impl HarnessTest for DiskListDisks {
    fn id(&self) -> &'static str {
        "disk.list_disks.non_empty_or_empty_ok"
    }

    fn suite(&self) -> &'static str {
        "disk"
    }

    fn required_spec(&self) -> &'static str {
        "2disk"
    }

    async fn execute(&self, _ctx: &HarnessContext) -> storage_testing::errors::Result<()> {
        let client = support::disks_client().await?;
        match client.list_disks().await {
            Ok(_) => Ok(()),
            Err(error) if support::should_skip(&error) => support::skip(format!("disk list: {error}")),
            Err(error) => support::failure(format!("disk list failed: {error}")),
        }
    }
}
