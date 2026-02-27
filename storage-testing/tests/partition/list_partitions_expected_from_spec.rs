use async_trait::async_trait;

use storage_testing::harness::support;
use storage_testing::tests::{HarnessContext, HarnessTest};

pub struct PartitionListPartitionsExpectedFromSpec;

#[async_trait]
impl HarnessTest for PartitionListPartitionsExpectedFromSpec {
    fn id(&self) -> &'static str {
        "partition.list_partitions.expected_from_spec"
    }

    fn suite(&self) -> &'static str {
        "partition"
    }

    fn required_spec(&self) -> &'static str {
        "2disk"
    }

    async fn execute(&self, _ctx: &HarnessContext) -> storage_testing::errors::Result<()> {
        let client = support::partitions_client().await?;
        let disk = support::require_env("STORAGE_TESTING_PARTITION_DISK")?;
        let _ = client.list_partitions(&disk).await;
        Ok(())
    }
}
