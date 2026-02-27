// SPDX-License-Identifier: GPL-3.0-only

use std::collections::BTreeMap;
use std::process::Command;
use std::sync::Arc;

use storage_macros::authorized_interface;
use storage_types::{LogicalEntity, LogicalMember};
use zbus::message::Header as MessageHeader;
use zbus::object_server::SignalEmitter;
use zbus::{Connection, interface};

use crate::policies::logical::{LogicalDomain, LogicalPolicy};

pub struct LogicalHandler {
    domain: Arc<dyn LogicalDomain>,
}

impl LogicalHandler {
    pub fn new() -> Self {
        Self {
            domain: Arc::new(LogicalPolicy),
        }
    }

    async fn discover_entities(&self) -> zbus::fdo::Result<Vec<LogicalEntity>> {
        self.domain.require_read()?;

        let manager = storage_udisks::DiskManager::new().await.map_err(|error| {
            zbus::fdo::Error::Failed(format!("Failed to initialize disk manager: {error}"))
        })?;

        let mut udisks_entities = storage_udisks::discover_logical_entities(&manager)
            .await
            .map_err(|error| {
                zbus::fdo::Error::Failed(format!("UDisks discovery failed: {error}"))
            })?;

        let fallback_entities =
            storage_sys::discover_logical_entities_fallback().map_err(|error| {
                zbus::fdo::Error::Failed(format!("Fallback discovery failed: {error}"))
            })?;

        let mut by_id = BTreeMap::<String, LogicalEntity>::new();
        for entity in fallback_entities {
            by_id.insert(entity.id.clone(), entity);
        }
        for entity in udisks_entities.drain(..) {
            by_id.insert(entity.id.clone(), entity);
        }

        Ok(by_id.into_values().collect())
    }

    fn run_cmd(command: &str, args: &[String]) -> zbus::fdo::Result<String> {
        let output = Command::new(command).args(args).output().map_err(|error| {
            zbus::fdo::Error::Failed(format!("Failed to run {command}: {error}"))
        })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            return Err(zbus::fdo::Error::Failed(format!(
                "{command} failed: {stderr}"
            )));
        }

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    async fn emit_topology_changed(connection: &Connection, reason: &str) -> zbus::fdo::Result<()> {
        let emitter = SignalEmitter::new(connection, "/org/cosmic/ext/Storage/Service/logical")
            .map_err(|error| zbus::fdo::Error::Failed(format!("Signal context error: {error}")))?;
        Self::logical_topology_changed(&emitter, reason)
            .await
            .map_err(|error| zbus::fdo::Error::Failed(format!("Signal emit error: {error}")))
    }
}

#[interface(name = "org.cosmic.ext.Storage.Service.Logical")]
impl LogicalHandler {
    #[zbus(signal)]
    async fn logical_topology_changed(ctxt: &SignalEmitter<'_>, reason: &str) -> zbus::Result<()>;

    #[authorized_interface(action = "org.cosmic.ext.storage.service.logical-read")]
    async fn list_logical_entities(
        &self,
        #[zbus(connection)] _connection: &Connection,
        #[zbus(header)] _header: MessageHeader<'_>,
    ) -> zbus::fdo::Result<String> {
        let entities = self.discover_entities().await?;
        tracing::debug!(
            "Listing {} logical entities (UID {})",
            entities.len(),
            caller.uid
        );

        serde_json::to_string(&entities)
            .map_err(|error| zbus::fdo::Error::Failed(format!("Serialize error: {error}")))
    }

    #[authorized_interface(action = "org.cosmic.ext.storage.service.logical-read")]
    async fn get_logical_entity(
        &self,
        #[zbus(connection)] _connection: &Connection,
        #[zbus(header)] _header: MessageHeader<'_>,
        entity_id: String,
    ) -> zbus::fdo::Result<String> {
        let entities = self.discover_entities().await?;
        let entity = entities
            .into_iter()
            .find(|candidate| candidate.id == entity_id)
            .ok_or_else(|| {
                zbus::fdo::Error::Failed(format!("Logical entity not found: {entity_id}"))
            })?;

        serde_json::to_string(&entity)
            .map_err(|error| zbus::fdo::Error::Failed(format!("Serialize error: {error}")))
    }

    #[authorized_interface(action = "org.cosmic.ext.storage.service.logical-read")]
    async fn list_logical_members(
        &self,
        #[zbus(connection)] _connection: &Connection,
        #[zbus(header)] _header: MessageHeader<'_>,
        entity_id: String,
    ) -> zbus::fdo::Result<String> {
        let entities = self.discover_entities().await?;
        let members: Vec<LogicalMember> = entities
            .into_iter()
            .find(|candidate| candidate.id == entity_id)
            .map(|entity| entity.members)
            .unwrap_or_default();

        serde_json::to_string(&members)
            .map_err(|error| zbus::fdo::Error::Failed(format!("Serialize error: {error}")))
    }

    #[authorized_interface(action = "org.cosmic.ext.storage.service.logical-modify")]
    async fn lvm_create_volume_group(
        &self,
        #[zbus(connection)] connection: &Connection,
        #[zbus(header)] _header: MessageHeader<'_>,
        vg_name: String,
        devices_json: String,
    ) -> zbus::fdo::Result<()> {
        self.domain.require_modify()?;
        let devices: Vec<String> = serde_json::from_str(&devices_json).map_err(|error| {
            zbus::fdo::Error::InvalidArgs(format!("Invalid devices JSON: {error}"))
        })?;
        let mut args = vec!["--yes".to_string(), vg_name];
        args.extend(devices);
        Self::run_cmd("vgcreate", &args)?;
        Self::emit_topology_changed(connection, "lvm-create-vg").await
    }

    #[authorized_interface(action = "org.cosmic.ext.storage.service.logical-modify")]
    async fn lvm_delete_volume_group(
        &self,
        #[zbus(connection)] connection: &Connection,
        #[zbus(header)] _header: MessageHeader<'_>,
        vg_name: String,
    ) -> zbus::fdo::Result<()> {
        self.domain.require_modify()?;
        Self::run_cmd("vgremove", &["-f".to_string(), vg_name])?;
        Self::emit_topology_changed(connection, "lvm-delete-vg").await
    }

    #[authorized_interface(action = "org.cosmic.ext.storage.service.logical-modify")]
    async fn lvm_add_physical_volume(
        &self,
        #[zbus(connection)] connection: &Connection,
        #[zbus(header)] _header: MessageHeader<'_>,
        vg_name: String,
        pv_device: String,
    ) -> zbus::fdo::Result<()> {
        self.domain.require_modify()?;
        Self::run_cmd("vgextend", &[vg_name, pv_device])?;
        Self::emit_topology_changed(connection, "lvm-add-pv").await
    }

    #[authorized_interface(action = "org.cosmic.ext.storage.service.logical-modify")]
    async fn lvm_remove_physical_volume(
        &self,
        #[zbus(connection)] connection: &Connection,
        #[zbus(header)] _header: MessageHeader<'_>,
        vg_name: String,
        pv_device: String,
    ) -> zbus::fdo::Result<()> {
        self.domain.require_modify()?;
        Self::run_cmd("vgreduce", &[vg_name, pv_device])?;
        Self::emit_topology_changed(connection, "lvm-remove-pv").await
    }

    #[authorized_interface(action = "org.cosmic.ext.storage.service.logical-modify")]
    async fn lvm_create_logical_volume(
        &self,
        #[zbus(connection)] connection: &Connection,
        #[zbus(header)] _header: MessageHeader<'_>,
        vg_name: String,
        lv_name: String,
        size_bytes: u64,
    ) -> zbus::fdo::Result<()> {
        self.domain.require_modify()?;
        Self::run_cmd(
            "lvcreate",
            &[
                "-L".to_string(),
                format!("{size_bytes}B"),
                "-y".to_string(),
                "-Zn".to_string(),
                "-n".to_string(),
                lv_name,
                vg_name,
            ],
        )?;
        Self::emit_topology_changed(connection, "lvm-create-lv").await
    }

    #[authorized_interface(action = "org.cosmic.ext.storage.service.logical-modify")]
    async fn lvm_delete_logical_volume(
        &self,
        #[zbus(connection)] connection: &Connection,
        #[zbus(header)] _header: MessageHeader<'_>,
        lv_path: String,
    ) -> zbus::fdo::Result<()> {
        self.domain.require_modify()?;
        Self::run_cmd("lvremove", &["-f".to_string(), lv_path])?;
        Self::emit_topology_changed(connection, "lvm-delete-lv").await
    }

    #[authorized_interface(action = "org.cosmic.ext.storage.service.logical-modify")]
    async fn lvm_resize_logical_volume(
        &self,
        #[zbus(connection)] connection: &Connection,
        #[zbus(header)] _header: MessageHeader<'_>,
        lv_path: String,
        size_bytes: u64,
    ) -> zbus::fdo::Result<()> {
        self.domain.require_modify()?;
        Self::run_cmd(
            "lvresize",
            &["-L".to_string(), format!("{size_bytes}B"), lv_path],
        )?;
        Self::emit_topology_changed(connection, "lvm-resize-lv").await
    }

    #[authorized_interface(action = "org.cosmic.ext.storage.service.logical-modify")]
    async fn lvm_activate_logical_volume(
        &self,
        #[zbus(connection)] connection: &Connection,
        #[zbus(header)] _header: MessageHeader<'_>,
        lv_path: String,
    ) -> zbus::fdo::Result<()> {
        self.domain.require_modify()?;
        Self::run_cmd("lvchange", &["-ay".to_string(), lv_path])?;
        Self::emit_topology_changed(connection, "lvm-activate-lv").await
    }

    #[authorized_interface(action = "org.cosmic.ext.storage.service.logical-modify")]
    async fn lvm_deactivate_logical_volume(
        &self,
        #[zbus(connection)] connection: &Connection,
        #[zbus(header)] _header: MessageHeader<'_>,
        lv_path: String,
    ) -> zbus::fdo::Result<()> {
        self.domain.require_modify()?;
        Self::run_cmd("lvchange", &["-an".to_string(), lv_path])?;
        Self::emit_topology_changed(connection, "lvm-deactivate-lv").await
    }

    #[authorized_interface(action = "org.cosmic.ext.storage.service.logical-modify")]
    async fn mdraid_create_array(
        &self,
        #[zbus(connection)] connection: &Connection,
        #[zbus(header)] _header: MessageHeader<'_>,
        array_device: String,
        level: String,
        devices_json: String,
    ) -> zbus::fdo::Result<()> {
        self.domain.require_modify()?;
        let devices: Vec<String> = serde_json::from_str(&devices_json).map_err(|error| {
            zbus::fdo::Error::InvalidArgs(format!("Invalid devices JSON: {error}"))
        })?;
        let mut args = vec![
            "--create".to_string(),
            "--force".to_string(),
            "--run".to_string(),
            array_device,
            "--level".to_string(),
            level,
            "--metadata=0.90".to_string(),
            "--raid-devices".to_string(),
            devices.len().to_string(),
        ];
        args.extend(devices);
        Self::run_cmd("mdadm", &args)?;
        Self::emit_topology_changed(connection, "mdraid-create").await
    }

    #[authorized_interface(action = "org.cosmic.ext.storage.service.logical-modify")]
    async fn mdraid_stop_array(
        &self,
        #[zbus(connection)] connection: &Connection,
        #[zbus(header)] _header: MessageHeader<'_>,
        array_device: String,
    ) -> zbus::fdo::Result<()> {
        self.domain.require_modify()?;
        Self::run_cmd("mdadm", &["--stop".to_string(), array_device])?;
        Self::emit_topology_changed(connection, "mdraid-stop").await
    }

    #[authorized_interface(action = "org.cosmic.ext.storage.service.logical-modify")]
    async fn mdraid_start_array(
        &self,
        #[zbus(connection)] connection: &Connection,
        #[zbus(header)] _header: MessageHeader<'_>,
        array_device: String,
    ) -> zbus::fdo::Result<()> {
        self.domain.require_modify()?;
        Self::run_cmd("mdadm", &["--assemble".to_string(), array_device])?;
        Self::emit_topology_changed(connection, "mdraid-start").await
    }

    #[authorized_interface(action = "org.cosmic.ext.storage.service.logical-modify")]
    async fn mdraid_add_member(
        &self,
        #[zbus(connection)] connection: &Connection,
        #[zbus(header)] _header: MessageHeader<'_>,
        array_device: String,
        member_device: String,
    ) -> zbus::fdo::Result<()> {
        self.domain.require_modify()?;
        Self::run_cmd("mdadm", &["--add".to_string(), array_device, member_device])?;
        Self::emit_topology_changed(connection, "mdraid-add-member").await
    }

    #[authorized_interface(action = "org.cosmic.ext.storage.service.logical-modify")]
    async fn mdraid_remove_member(
        &self,
        #[zbus(connection)] connection: &Connection,
        #[zbus(header)] _header: MessageHeader<'_>,
        array_device: String,
        member_device: String,
    ) -> zbus::fdo::Result<()> {
        self.domain.require_modify()?;
        Self::run_cmd(
            "mdadm",
            &[
                "--manage".to_string(),
                array_device,
                "--remove".to_string(),
                member_device,
            ],
        )?;
        Self::emit_topology_changed(connection, "mdraid-remove-member").await
    }

    #[authorized_interface(action = "org.cosmic.ext.storage.service.logical-modify")]
    async fn mdraid_delete_array(
        &self,
        #[zbus(connection)] connection: &Connection,
        #[zbus(header)] _header: MessageHeader<'_>,
        array_device: String,
    ) -> zbus::fdo::Result<()> {
        self.domain.require_modify()?;
        Self::run_cmd("mdadm", &["--stop".to_string(), array_device.clone()])?;
        Self::run_cmd("mdadm", &["--zero-superblock".to_string(), array_device])?;
        Self::emit_topology_changed(connection, "mdraid-delete").await
    }

    #[authorized_interface(action = "org.cosmic.ext.storage.service.logical-modify")]
    async fn mdraid_request_sync_action(
        &self,
        #[zbus(connection)] connection: &Connection,
        #[zbus(header)] _header: MessageHeader<'_>,
        md_name: String,
        action: String,
    ) -> zbus::fdo::Result<()> {
        self.domain.require_modify()?;
        let sync_path = format!("/sys/block/{md_name}/md/sync_action");
        std::fs::write(&sync_path, action).map_err(|error| {
            zbus::fdo::Error::Failed(format!("Failed writing sync action: {error}"))
        })?;
        Self::emit_topology_changed(connection, "mdraid-sync-action").await
    }

    #[authorized_interface(action = "org.cosmic.ext.storage.service.logical-modify")]
    async fn btrfs_add_device(
        &self,
        #[zbus(connection)] connection: &Connection,
        #[zbus(header)] _header: MessageHeader<'_>,
        member_device: String,
        mount_point: String,
    ) -> zbus::fdo::Result<()> {
        self.domain.require_modify()?;
        Self::run_cmd(
            "btrfs",
            &[
                "device".to_string(),
                "add".to_string(),
                member_device,
                mount_point,
            ],
        )?;
        Self::emit_topology_changed(connection, "btrfs-add-device").await
    }

    #[authorized_interface(action = "org.cosmic.ext.storage.service.logical-modify")]
    async fn btrfs_remove_device(
        &self,
        #[zbus(connection)] connection: &Connection,
        #[zbus(header)] _header: MessageHeader<'_>,
        member_device: String,
        mount_point: String,
    ) -> zbus::fdo::Result<()> {
        self.domain.require_modify()?;
        Self::run_cmd(
            "btrfs",
            &[
                "device".to_string(),
                "remove".to_string(),
                member_device,
                mount_point,
            ],
        )?;
        Self::emit_topology_changed(connection, "btrfs-remove-device").await
    }

    #[authorized_interface(action = "org.cosmic.ext.storage.service.logical-modify")]
    async fn btrfs_resize(
        &self,
        #[zbus(connection)] connection: &Connection,
        #[zbus(header)] _header: MessageHeader<'_>,
        size_spec: String,
        mount_point: String,
    ) -> zbus::fdo::Result<()> {
        self.domain.require_modify()?;
        Self::run_cmd(
            "btrfs",
            &[
                "filesystem".to_string(),
                "resize".to_string(),
                size_spec,
                mount_point,
            ],
        )?;
        Self::emit_topology_changed(connection, "btrfs-resize").await
    }

    #[authorized_interface(action = "org.cosmic.ext.storage.service.logical-modify")]
    async fn btrfs_set_label(
        &self,
        #[zbus(connection)] connection: &Connection,
        #[zbus(header)] _header: MessageHeader<'_>,
        mount_point: String,
        label: String,
    ) -> zbus::fdo::Result<()> {
        self.domain.require_modify()?;
        Self::run_cmd(
            "btrfs",
            &[
                "filesystem".to_string(),
                "label".to_string(),
                mount_point,
                label,
            ],
        )?;
        Self::emit_topology_changed(connection, "btrfs-set-label").await
    }

    #[authorized_interface(action = "org.cosmic.ext.storage.service.logical-modify")]
    async fn btrfs_set_default_subvolume(
        &self,
        #[zbus(connection)] connection: &Connection,
        #[zbus(header)] _header: MessageHeader<'_>,
        subvolume_id: u64,
        mount_point: String,
    ) -> zbus::fdo::Result<()> {
        self.domain.require_modify()?;
        Self::run_cmd(
            "btrfs",
            &[
                "subvolume".to_string(),
                "set-default".to_string(),
                subvolume_id.to_string(),
                mount_point,
            ],
        )?;
        Self::emit_topology_changed(connection, "btrfs-set-default-subvolume").await
    }
}

#[cfg(test)]
mod tests {
    use storage_types::{LogicalCapabilities, LogicalEntity, LogicalEntityKind};

    #[test]
    fn logical_entity_json_serializes() {
        let entity = LogicalEntity {
            id: "lvm-vg:vg0".to_string(),
            kind: LogicalEntityKind::LvmVolumeGroup,
            name: "vg0".to_string(),
            uuid: None,
            parent_id: None,
            device_path: None,
            size_bytes: 1,
            used_bytes: Some(1),
            free_bytes: Some(0),
            health_status: Some("ok".to_string()),
            progress_fraction: None,
            members: vec![],
            capabilities: LogicalCapabilities::default(),
            metadata: Default::default(),
        };

        let json = serde_json::to_string(&entity).expect("serialize logical entity");
        assert!(json.contains("lvm-vg:vg0"));
    }
}
