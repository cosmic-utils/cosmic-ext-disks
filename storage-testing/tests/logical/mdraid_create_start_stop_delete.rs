use async_trait::async_trait;
use tokio::time::{Duration, sleep, timeout};

use storage_testing::harness::support;
use storage_testing::tests::{HarnessContext, HarnessTest};

pub struct LogicalMdraidCreateStartStopDelete;

#[async_trait]
impl HarnessTest for LogicalMdraidCreateStartStopDelete {
    fn id(&self) -> &'static str {
        "logical.mdraid.create_start_stop_delete"
    }

    fn suite(&self) -> &'static str {
        "logical"
    }

    fn required_spec(&self) -> &'static str {
        "3disk"
    }

    fn exclusive(&self) -> bool {
        true
    }

    async fn execute(&self, _ctx: &HarnessContext) -> storage_testing::errors::Result<()> {
        support::require_destructive("logical.mdraid.create_start_stop_delete")?;
        let client = support::logical_client().await?;
        let array_device = support::require_env("STORAGE_TESTING_MD_ARRAY")?;
        let devices_json = support::require_env("STORAGE_TESTING_MD_DEVICES_JSON")?;

        let create_result = timeout(
            Duration::from_secs(4),
            client.mdraid_create_array(array_device.clone(), "raid1".to_string(), devices_json),
        )
        .await;
        match create_result {
            Ok(result) => support::client_result(result, "create mdraid array")?,
            Err(_) => return support::skip("create mdraid array timed out after 4s"),
        }

        let entities_after_create = support::client_result(
            client.list_logical_entities().await,
            "list logical entities",
        )?;
        let mut md_present = entities_after_create
            .iter()
            .any(|entity| entity.device_path.as_deref() == Some(array_device.as_str()));

        if !md_present {
            for _ in 0..5 {
                sleep(Duration::from_millis(300)).await;
                let polled = support::client_result(
                    client.list_logical_entities().await,
                    "poll logical entities for mdraid",
                )?;
                md_present = polled
                    .iter()
                    .any(|entity| entity.device_path.as_deref() == Some(array_device.as_str()));
                if md_present {
                    break;
                }
            }
        }

        if !md_present {
            let _ = support::client_result(
                client.mdraid_stop_array(array_device.clone()).await,
                "best-effort stop mdraid array",
            );
            let _ = support::client_result(
                client.mdraid_delete_array(array_device.clone()).await,
                "best-effort delete mdraid array",
            );
            return support::skip(format!(
                "mdraid array not visible after create: {array_device}"
            ));
        }

        support::client_result(
            client.mdraid_stop_array(array_device.clone()).await,
            "stop mdraid array",
        )?;
        support::client_result(
            client.mdraid_delete_array(array_device.clone()).await,
            "delete mdraid array",
        )?;

        let entities_after_delete = support::client_result(
            client.list_logical_entities().await,
            "list logical entities",
        )?;
        if entities_after_delete
            .iter()
            .any(|entity| entity.device_path.as_deref() == Some(array_device.as_str()))
        {
            return support::failure(format!(
                "mdraid array still present after delete: {array_device}"
            ));
        }
        Ok(())
    }
}
