use async_trait::async_trait;

use storage_testing::harness::support;
use storage_testing::tests::{HarnessContext, HarnessTest};

pub struct DiskListVolumesSchemaIntegrity;

#[async_trait]
impl HarnessTest for DiskListVolumesSchemaIntegrity {
    fn id(&self) -> &'static str {
        "disk.list_volumes.schema_integrity"
    }

    fn suite(&self) -> &'static str {
        "disk"
    }

    fn required_spec(&self) -> &'static str {
        "2disk"
    }

    async fn execute(&self, _ctx: &HarnessContext) -> storage_testing::errors::Result<()> {
        let client = support::disks_client().await?;
        let volumes = match client.list_volumes().await {
            Ok(volumes) => volumes,
            Err(error) if support::should_skip(&error) => {
                return support::skip(format!("list volumes: {error}"));
            }
            Err(error) => return support::failure(format!("list volumes failed: {error}")),
        };

        for volume in volumes {
            if let Some(parent) = &volume.parent_path
                && parent.trim().is_empty()
            {
                return support::failure("volume parent_path must not be empty when present");
            }
        }
        Ok(())
    }
}
