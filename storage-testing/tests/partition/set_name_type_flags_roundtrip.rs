use async_trait::async_trait;

use storage_testing::harness::support;
use storage_testing::tests::{HarnessContext, HarnessTest};

pub struct PartitionSetNameTypeFlagsRoundtrip;

#[async_trait]
impl HarnessTest for PartitionSetNameTypeFlagsRoundtrip {
    fn id(&self) -> &'static str {
        "partition.set_name_type_flags.roundtrip"
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
        support::require_destructive("partition.set_name_type_flags.roundtrip")?;
        let client = support::partitions_client().await?;
        let disk = support::require_env("STORAGE_TESTING_PARTITION_DISK")?;
        let partition = support::require_env("STORAGE_TESTING_PARTITION_DEVICE")?;

        let before = support::client_result(client.list_partitions(&disk).await, "list partitions")?;
        let current = before
            .iter()
            .find(|entry| entry.device == partition)
            .ok_or_else(|| storage_testing::errors::TestingError::TestFailed {
                reason: format!("partition not found: {partition}"),
            })?;

        support::client_result(
            client
                .set_partition_type(&partition, &current.type_id)
                .await,
            "set partition type",
        )?;
        support::client_result(
            client.set_partition_name(&partition, "storage-test").await,
            "set partition name",
        )?;
        support::client_result(
            client.set_partition_flags(&partition, 0).await,
            "set partition flags",
        )?;

        let after = support::client_result(client.list_partitions(&disk).await, "list partitions")?;
        let updated = after
            .iter()
            .find(|entry| entry.device == partition)
            .ok_or_else(|| storage_testing::errors::TestingError::TestFailed {
                reason: format!("partition missing after mutation: {partition}"),
            })?;

        if updated.name != "storage-test" {
            return support::failure(format!(
                "partition name mismatch: expected storage-test, got {}",
                updated.name
            ));
        }
        if updated.flags != 0 {
            return support::failure(format!(
                "partition flags mismatch: expected 0, got {}",
                updated.flags
            ));
        }
        Ok(())
    }
}
