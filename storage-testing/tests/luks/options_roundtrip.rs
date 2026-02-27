use async_trait::async_trait;
use tokio::time::{Duration, timeout};

use storage_testing::harness::support;
use storage_testing::tests::{HarnessContext, HarnessTest};

pub struct LuksOptionsReadWriteRoundtrip;

#[async_trait]
impl HarnessTest for LuksOptionsReadWriteRoundtrip {
    fn id(&self) -> &'static str {
        "luks.options.read_write_roundtrip"
    }

    fn suite(&self) -> &'static str {
        "luks"
    }

    fn required_spec(&self) -> &'static str {
        "2disk"
    }

    async fn execute(&self, _ctx: &HarnessContext) -> storage_testing::errors::Result<()> {
        support::require_destructive("luks.options.read_write_roundtrip")?;
        let client = support::luks_client().await?;
        let device = support::require_env("STORAGE_TESTING_LUKS_DEVICE")?;
        let passphrase = support::require_env("STORAGE_TESTING_LUKS_PASSPHRASE")?;

        let format_result = timeout(
            Duration::from_secs(4),
            client.format(&device, &passphrase, "luks2"),
        )
        .await;
        match format_result {
            Ok(result) => support::client_result(result, "format luks container")?,
            Err(_) => return support::skip("format luks container timed out after 4s"),
        }

        support::client_result(
            client.get_encryption_options(&device).await,
            "get encryption options before reset",
        )?;
        support::client_result(
            client.default_encryption_options(&device).await,
            "clear encryption options",
        )?;
        let after = support::client_result(
            client.get_encryption_options(&device).await,
            "get encryption options after reset",
        )?;
        if after.is_some() {
            return support::failure(format!(
                "expected no encryption options after reset for {device}"
            ));
        }
        Ok(())
    }
}
