use async_trait::async_trait;
use std::process::Command;

use storage_testing::harness::support;
use storage_testing::tests::{HarnessContext, HarnessTest};

pub struct LogicalBtrfsAddRemoveMember;

#[async_trait]
impl HarnessTest for LogicalBtrfsAddRemoveMember {
    fn id(&self) -> &'static str {
        "logical.btrfs.add_remove_member"
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
        support::require_destructive("logical.btrfs.add_remove_member")?;
        let client = support::logical_client().await?;
        let filesystems = support::filesystems_client().await?;
        let source_device = support::require_env("STORAGE_TESTING_BTRFS_SOURCE_DEVICE")?;
        let member = support::require_env("STORAGE_TESTING_BTRFS_MEMBER_DEVICE")?;
        let mountpoint = support::require_env("STORAGE_TESTING_BTRFS_MOUNT")?;

        let _ = support::client_result(
            filesystems.unmount(&mountpoint, true, true).await,
            "unmount existing filesystem for btrfs setup",
        );
        support::client_result(
            filesystems
                .format(&source_device, "btrfs", "storage-testing-btrfs", None)
                .await,
            "format btrfs source device",
        )?;
        let mounted_at = support::client_result(
            filesystems.mount(&source_device, &mountpoint, None).await,
            "mount btrfs source device",
        )?;

        support::client_result(
            client
                .btrfs_add_device(member.clone(), mounted_at.clone())
                .await,
            "add btrfs member",
        )?;
        let entities_after_add =
            support::client_result(client.list_logical_entities().await, "list logical entities")?;
        let member_present = entities_after_add.iter().any(|entity| {
            entity
                .members
                .iter()
                .any(|logical_member| logical_member.device_path.as_deref() == Some(member.as_str()))
        });
        if !member_present {
            return support::failure(format!("btrfs member not found after add: {member}"));
        }

        support::client_result(
            client
                .btrfs_remove_device(member.clone(), mounted_at.clone())
                .await,
            "remove btrfs member",
        )?;
        let entities_after_remove =
            support::client_result(client.list_logical_entities().await, "list logical entities")?;
        let member_still_present = entities_after_remove.iter().any(|entity| {
            entity
                .members
                .iter()
                .any(|logical_member| logical_member.device_path.as_deref() == Some(member.as_str()))
        });
        if member_still_present {
            return support::failure(format!("btrfs member still present after remove: {member}"));
        }

        support::client_result(
            filesystems.unmount(&mounted_at, false, false).await,
            "unmount btrfs source device",
        )?;

        for device in [source_device, member] {
            let _ = Command::new("wipefs").args(["-a", &device]).status();
        }
        Ok(())
    }
}
