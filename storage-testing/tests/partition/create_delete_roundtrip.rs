use async_trait::async_trait;

use storage_testing::harness::support;
use storage_testing::tests::{HarnessContext, HarnessTest};

pub struct PartitionCreateDeleteRoundtrip;

#[async_trait]
impl HarnessTest for PartitionCreateDeleteRoundtrip {
    fn id(&self) -> &'static str {
        "partition.create_delete.roundtrip"
    }

    fn suite(&self) -> &'static str {
        "partition"
    }

    fn required_spec(&self) -> &'static str {
        "2disk"
    }

    fn exclusive(&self) -> bool {
        true
    }

    async fn execute(&self, _ctx: &HarnessContext) -> storage_testing::errors::Result<()> {
        support::require_destructive("partition.create_delete.roundtrip")?;
        let client = support::partitions_client().await?;
        let disk = support::require_env("STORAGE_TESTING_PARTITION_DISK")?;
        support::client_result(
            client.create_partition_table(&disk, "gpt").await,
            "create GPT partition table",
        )?;
        let partitions =
            support::client_result(client.list_partitions(&disk).await, "list partitions")?;
        if !partitions.is_empty() {
            return support::failure("expected empty partition table after recreation");
        }
        Ok(())
    }
}
