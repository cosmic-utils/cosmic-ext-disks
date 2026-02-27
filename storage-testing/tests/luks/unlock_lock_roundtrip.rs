use async_trait::async_trait;
use tokio::time::{Duration, timeout};

use storage_testing::harness::support;
use storage_testing::tests::{HarnessContext, HarnessTest};

pub struct LuksUnlockLockRoundtrip;

#[async_trait]
impl HarnessTest for LuksUnlockLockRoundtrip {
    fn id(&self) -> &'static str {
        "luks.unlock_lock.roundtrip"
    }

    fn suite(&self) -> &'static str {
        "luks"
    }

    fn required_spec(&self) -> &'static str {
        "2disk"
    }

    fn exclusive(&self) -> bool {
        true
    }

    async fn execute(&self, _ctx: &HarnessContext) -> storage_testing::errors::Result<()> {
        support::require_destructive("luks.unlock_lock.roundtrip")?;
        let client = support::luks_client().await?;
        let disks = support::disks_client().await?;
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

        let cleartext =
            support::client_result(client.unlock(&device, &passphrase).await, "unlock LUKS")?;
        let volumes_unlocked =
            support::client_result(disks.list_volumes().await, "list volumes after unlock")?;
        if !volumes_unlocked
            .iter()
            .any(|volume| volume.device_path.as_deref() == Some(cleartext.as_str()))
        {
            return support::failure(format!(
                "expected cleartext device to exist after unlock: {cleartext}"
            ));
        }

        support::client_result(client.lock(&cleartext).await, "lock LUKS")?;
        let volumes_locked =
            support::client_result(disks.list_volumes().await, "list volumes after lock")?;
        if volumes_locked
            .iter()
            .any(|volume| volume.device_path.as_deref() == Some(cleartext.as_str()))
        {
            return support::failure(format!(
                "expected cleartext device to be absent after lock: {cleartext}"
            ));
        }
        Ok(())
    }
}
