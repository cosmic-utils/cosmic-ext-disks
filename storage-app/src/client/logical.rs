// SPDX-License-Identifier: GPL-3.0-only

use crate::client::connection::shared_connection;
use crate::client::error::ClientError;
use storage_types::LogicalEntity;
use zbus::proxy;

#[proxy(
    interface = "org.cosmic.ext.Storage.Service.Logical",
    default_service = "org.cosmic.ext.Storage.Service",
    default_path = "/org/cosmic/ext/Storage/Service/logical"
)]
pub trait LogicalInterface {
    async fn list_logical_entities(&self) -> zbus::Result<String>;

    async fn lvm_create_volume_group(
        &self,
        vg_name: String,
        devices_json: String,
    ) -> zbus::Result<()>;
    async fn lvm_delete_volume_group(&self, vg_name: String) -> zbus::Result<()>;
    async fn lvm_add_physical_volume(&self, vg_name: String, pv_device: String)
    -> zbus::Result<()>;
    async fn lvm_remove_physical_volume(
        &self,
        vg_name: String,
        pv_device: String,
    ) -> zbus::Result<()>;
    async fn lvm_create_logical_volume(
        &self,
        vg_name: String,
        lv_name: String,
        size_bytes: u64,
    ) -> zbus::Result<()>;
    async fn lvm_delete_logical_volume(&self, lv_path: String) -> zbus::Result<()>;
    async fn lvm_resize_logical_volume(&self, lv_path: String, size_bytes: u64)
    -> zbus::Result<()>;
    async fn lvm_activate_logical_volume(&self, lv_path: String) -> zbus::Result<()>;
    async fn lvm_deactivate_logical_volume(&self, lv_path: String) -> zbus::Result<()>;

    async fn mdraid_create_array(
        &self,
        array_device: String,
        level: String,
        devices_json: String,
    ) -> zbus::Result<()>;
    async fn mdraid_delete_array(&self, array_device: String) -> zbus::Result<()>;
    async fn mdraid_start_array(&self, array_device: String) -> zbus::Result<()>;
    async fn mdraid_stop_array(&self, array_device: String) -> zbus::Result<()>;
    async fn mdraid_add_member(
        &self,
        array_device: String,
        member_device: String,
    ) -> zbus::Result<()>;
    async fn mdraid_remove_member(
        &self,
        array_device: String,
        member_device: String,
    ) -> zbus::Result<()>;
    async fn mdraid_request_sync_action(&self, md_name: String, action: String)
    -> zbus::Result<()>;

    async fn btrfs_add_device(
        &self,
        member_device: String,
        mount_point: String,
    ) -> zbus::Result<()>;
    async fn btrfs_remove_device(
        &self,
        member_device: String,
        mount_point: String,
    ) -> zbus::Result<()>;
    async fn btrfs_resize(&self, size_spec: String, mount_point: String) -> zbus::Result<()>;
    async fn btrfs_set_label(&self, mount_point: String, label: String) -> zbus::Result<()>;
    async fn btrfs_set_default_subvolume(
        &self,
        subvolume_id: u64,
        mount_point: String,
    ) -> zbus::Result<()>;
}

pub struct LogicalClient {
    proxy: LogicalInterfaceProxy<'static>,
}

impl std::fmt::Debug for LogicalClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LogicalClient").finish_non_exhaustive()
    }
}

impl LogicalClient {
    pub async fn new() -> Result<Self, ClientError> {
        let conn = shared_connection().await?;
        let proxy = LogicalInterfaceProxy::new(conn).await.map_err(|error| {
            ClientError::Connection(format!("Failed to create logical proxy: {error}"))
        })?;

        Ok(Self { proxy })
    }

    pub async fn list_logical_entities(&self) -> Result<Vec<LogicalEntity>, ClientError> {
        let json = self.proxy.list_logical_entities().await?;
        serde_json::from_str(&json).map_err(|error| {
            ClientError::ParseError(format!("Failed to parse logical entities: {error}"))
        })
    }

    pub async fn lvm_create_volume_group(
        &self,
        vg_name: String,
        devices_json: String,
    ) -> Result<(), ClientError> {
        self.proxy
            .lvm_create_volume_group(vg_name, devices_json)
            .await
            .map_err(Into::into)
    }

    pub async fn lvm_delete_volume_group(&self, vg_name: String) -> Result<(), ClientError> {
        self.proxy
            .lvm_delete_volume_group(vg_name)
            .await
            .map_err(Into::into)
    }

    pub async fn lvm_add_physical_volume(
        &self,
        vg_name: String,
        pv_device: String,
    ) -> Result<(), ClientError> {
        self.proxy
            .lvm_add_physical_volume(vg_name, pv_device)
            .await
            .map_err(Into::into)
    }

    pub async fn lvm_remove_physical_volume(
        &self,
        vg_name: String,
        pv_device: String,
    ) -> Result<(), ClientError> {
        self.proxy
            .lvm_remove_physical_volume(vg_name, pv_device)
            .await
            .map_err(Into::into)
    }

    pub async fn lvm_create_logical_volume(
        &self,
        vg_name: String,
        lv_name: String,
        size_bytes: u64,
    ) -> Result<(), ClientError> {
        self.proxy
            .lvm_create_logical_volume(vg_name, lv_name, size_bytes)
            .await
            .map_err(Into::into)
    }

    pub async fn lvm_delete_logical_volume(&self, lv_path: String) -> Result<(), ClientError> {
        self.proxy
            .lvm_delete_logical_volume(lv_path)
            .await
            .map_err(Into::into)
    }

    pub async fn lvm_resize_logical_volume(
        &self,
        lv_path: String,
        size_bytes: u64,
    ) -> Result<(), ClientError> {
        self.proxy
            .lvm_resize_logical_volume(lv_path, size_bytes)
            .await
            .map_err(Into::into)
    }

    pub async fn lvm_activate_logical_volume(&self, lv_path: String) -> Result<(), ClientError> {
        self.proxy
            .lvm_activate_logical_volume(lv_path)
            .await
            .map_err(Into::into)
    }

    pub async fn lvm_deactivate_logical_volume(&self, lv_path: String) -> Result<(), ClientError> {
        self.proxy
            .lvm_deactivate_logical_volume(lv_path)
            .await
            .map_err(Into::into)
    }

    pub async fn mdraid_create_array(
        &self,
        array_device: String,
        level: String,
        devices_json: String,
    ) -> Result<(), ClientError> {
        self.proxy
            .mdraid_create_array(array_device, level, devices_json)
            .await
            .map_err(Into::into)
    }

    pub async fn mdraid_delete_array(&self, array_device: String) -> Result<(), ClientError> {
        self.proxy
            .mdraid_delete_array(array_device)
            .await
            .map_err(Into::into)
    }

    pub async fn mdraid_start_array(&self, array_device: String) -> Result<(), ClientError> {
        self.proxy
            .mdraid_start_array(array_device)
            .await
            .map_err(Into::into)
    }

    pub async fn mdraid_stop_array(&self, array_device: String) -> Result<(), ClientError> {
        self.proxy
            .mdraid_stop_array(array_device)
            .await
            .map_err(Into::into)
    }

    pub async fn mdraid_add_member(
        &self,
        array_device: String,
        member_device: String,
    ) -> Result<(), ClientError> {
        self.proxy
            .mdraid_add_member(array_device, member_device)
            .await
            .map_err(Into::into)
    }

    pub async fn mdraid_remove_member(
        &self,
        array_device: String,
        member_device: String,
    ) -> Result<(), ClientError> {
        self.proxy
            .mdraid_remove_member(array_device, member_device)
            .await
            .map_err(Into::into)
    }

    pub async fn mdraid_request_sync_action(
        &self,
        md_name: String,
        action: String,
    ) -> Result<(), ClientError> {
        self.proxy
            .mdraid_request_sync_action(md_name, action)
            .await
            .map_err(Into::into)
    }

    pub async fn btrfs_add_device(
        &self,
        member_device: String,
        mount_point: String,
    ) -> Result<(), ClientError> {
        self.proxy
            .btrfs_add_device(member_device, mount_point)
            .await
            .map_err(Into::into)
    }

    pub async fn btrfs_remove_device(
        &self,
        member_device: String,
        mount_point: String,
    ) -> Result<(), ClientError> {
        self.proxy
            .btrfs_remove_device(member_device, mount_point)
            .await
            .map_err(Into::into)
    }

    pub async fn btrfs_resize(
        &self,
        size_spec: String,
        mount_point: String,
    ) -> Result<(), ClientError> {
        self.proxy
            .btrfs_resize(size_spec, mount_point)
            .await
            .map_err(Into::into)
    }

    pub async fn btrfs_set_label(
        &self,
        mount_point: String,
        label: String,
    ) -> Result<(), ClientError> {
        self.proxy
            .btrfs_set_label(mount_point, label)
            .await
            .map_err(Into::into)
    }

    pub async fn btrfs_set_default_subvolume(
        &self,
        subvolume_id: u64,
        mount_point: String,
    ) -> Result<(), ClientError> {
        self.proxy
            .btrfs_set_default_subvolume(subvolume_id, mount_point)
            .await
            .map_err(Into::into)
    }
}
