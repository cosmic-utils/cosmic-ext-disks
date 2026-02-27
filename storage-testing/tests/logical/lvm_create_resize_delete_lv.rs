use async_trait::async_trait;
use std::process::Command;

use storage_testing::harness::support;
use storage_testing::tests::{HarnessContext, HarnessTest};

pub struct LogicalLvmCreateResizeDeleteLv;

#[async_trait]
impl HarnessTest for LogicalLvmCreateResizeDeleteLv {
    fn id(&self) -> &'static str {
        "logical.lvm.create_resize_delete_lv"
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
        support::require_destructive("logical.lvm.create_resize_delete_lv")?;
        let client = support::logical_client().await?;
        let vg_name = support::require_env("STORAGE_TESTING_LVM_VG")?;
        let pvs_json = support::require_env("STORAGE_TESTING_LVM_PVS_JSON")?;
        let pvs: Vec<String> = serde_json::from_str(&pvs_json).map_err(|error| {
            storage_testing::errors::TestingError::TestFailed {
                reason: format!("invalid LVM PV JSON: {error}"),
            }
        })?;
        let lv_path = format!("/dev/{vg_name}/storage_test_lv");

        let _ = support::client_result(
            client.lvm_delete_logical_volume(lv_path.clone()).await,
            "cleanup stale lvm logical volume",
        );
        let _ = support::client_result(
            client.lvm_delete_volume_group(vg_name.clone()).await,
            "cleanup stale lvm volume group",
        );

        for pv in &pvs {
            let _ = Command::new("pvremove").args(["-ff", "-y", pv]).status();
            let _ = Command::new("wipefs").args(["-a", pv]).status();
        }

        support::client_result(
            client
                .lvm_create_volume_group(vg_name.clone(), pvs_json)
                .await,
            "create lvm volume group",
        )?;

        support::client_result(
            client
                .lvm_create_logical_volume(vg_name.clone(), "storage_test_lv".to_string(), 16 * 1024 * 1024)
                .await,
            "create lvm logical volume",
        )?;

        let entities_after_create =
            support::client_result(client.list_logical_entities().await, "list logical entities")?;
        if !entities_after_create
            .iter()
            .any(|entity| entity.device_path.as_deref() == Some(lv_path.as_str()))
        {
            return support::failure(format!("logical volume not found after create: {lv_path}"));
        }

        support::client_result(
            client
                .lvm_resize_logical_volume(lv_path.clone(), 24 * 1024 * 1024)
                .await,
            "resize lvm logical volume",
        )?;
        support::client_result(
            client.lvm_delete_logical_volume(lv_path.clone()).await,
            "delete lvm logical volume",
        )?;
        support::client_result(
            client.lvm_delete_volume_group(vg_name.clone()).await,
            "delete lvm volume group",
        )?;

        let entities_after_delete =
            support::client_result(client.list_logical_entities().await, "list logical entities")?;
        if entities_after_delete
            .iter()
            .any(|entity| entity.device_path.as_deref() == Some(lv_path.as_str()))
        {
            return support::failure(format!(
                "logical volume still present after delete: {lv_path}"
            ));
        }

        if entities_after_delete.iter().any(|entity| entity.name == vg_name) {
            return support::failure(format!("volume group still present after delete: {vg_name}"));
        }
        Ok(())
    }
}
