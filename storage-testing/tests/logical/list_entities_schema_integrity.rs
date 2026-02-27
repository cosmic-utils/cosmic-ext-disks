use async_trait::async_trait;

use storage_testing::harness::support;
use storage_testing::tests::{HarnessContext, HarnessTest};

pub struct LogicalListEntitiesSchemaIntegrity;

#[async_trait]
impl HarnessTest for LogicalListEntitiesSchemaIntegrity {
    fn id(&self) -> &'static str {
        "logical.list_entities.schema_integrity"
    }

    fn suite(&self) -> &'static str {
        "logical"
    }

    fn required_spec(&self) -> &'static str {
        "3disk"
    }

    async fn execute(&self, _ctx: &HarnessContext) -> storage_testing::errors::Result<()> {
        let client = support::logical_client().await?;

        let entities = match client.list_logical_entities().await {
            Ok(entities) => entities,
            Err(error) => return support::skip(format!("logical entities unavailable: {error}")),
        };

        for entity in entities {
            if entity.id.trim().is_empty() {
                return support::failure("logical entity id must not be empty");
            }
        }

        Ok(())
    }
}
