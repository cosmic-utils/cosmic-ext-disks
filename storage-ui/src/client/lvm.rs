// SPDX-License-Identifier: GPL-3.0-only

use crate::client::error::ClientError;
use storage_common::{LogicalVolumeInfo, PhysicalVolumeInfo, VolumeGroupInfo};
use zbus::{Connection, proxy};

/// D-Bus proxy interface for LVM operations
#[proxy(
    interface = "org.cosmic.ext.StorageService.Lvm",
    default_service = "org.cosmic.ext.StorageService",
    default_path = "/org/cosmic/ext/StorageService/lvm"
)]
pub trait LvmInterface {
    /// List all volume groups
    async fn list_volume_groups(&self) -> zbus::Result<String>;

    /// List logical volumes in a volume group
    async fn list_logical_volumes(&self, vg_name: &str) -> zbus::Result<String>;

    /// List all physical volumes
    async fn list_physical_volumes(&self) -> zbus::Result<String>;

    /// Create a new volume group
    async fn create_volume_group(&self, name: &str, devices_json: &str) -> zbus::Result<()>;

    /// Create a logical volume
    async fn create_logical_volume(
        &self,
        vg_name: &str,
        lv_name: &str,
        size: u64,
    ) -> zbus::Result<String>;

    /// Resize a logical volume
    async fn resize_logical_volume(
        &self,
        vg_name: &str,
        lv_name: &str,
        new_size: u64,
    ) -> zbus::Result<()>;

    /// Delete a volume group
    async fn delete_volume_group(&self, vg_name: &str) -> zbus::Result<()>;

    /// Delete a logical volume
    async fn delete_logical_volume(&self, vg_name: &str, lv_name: &str) -> zbus::Result<()>;

    /// Remove a physical volume from a volume group
    async fn remove_physical_volume(&self, vg_name: &str, device: &str) -> zbus::Result<()>;

    /// Signal emitted when a volume group is created
    #[zbus(signal)]
    async fn volume_group_created(&self, vg_name: &str) -> zbus::Result<()>;

    /// Signal emitted when a logical volume is created
    #[zbus(signal)]
    async fn logical_volume_created(&self, vg_name: &str, lv_name: &str) -> zbus::Result<()>;

    /// Signal emitted when a logical volume is resized
    #[zbus(signal)]
    async fn logical_volume_resized(
        &self,
        vg_name: &str,
        lv_name: &str,
        new_size: u64,
    ) -> zbus::Result<()>;

    /// Signal emitted when a logical volume is deleted
    #[zbus(signal)]
    async fn logical_volume_deleted(&self, vg_name: &str, lv_name: &str) -> zbus::Result<()>;

    /// Signal emitted when a volume group is deleted
    #[zbus(signal)]
    async fn volume_group_deleted(&self, vg_name: &str) -> zbus::Result<()>;
}

/// Client for LVM operations
#[allow(dead_code)]
pub struct LvmClient {
    proxy: LvmInterfaceProxy<'static>,
}

#[allow(dead_code)]
impl LvmClient {
    /// Create a new LVM client connected to the storage service
    pub async fn new() -> Result<Self, ClientError> {
        let conn = Connection::system().await.map_err(|e| {
            ClientError::Connection(format!("Failed to connect to system bus: {}", e))
        })?;

        let proxy = LvmInterfaceProxy::new(&conn)
            .await
            .map_err(|e| ClientError::Connection(format!("Failed to create LVM proxy: {}", e)))?;

        Ok(Self { proxy })
    }

    /// List all volume groups
    pub async fn list_volume_groups(&self) -> Result<Vec<VolumeGroupInfo>, ClientError> {
        let json = self.proxy.list_volume_groups().await?;
        let vgs: Vec<VolumeGroupInfo> = serde_json::from_str(&json).map_err(|e| {
            ClientError::ParseError(format!("Failed to parse volume group list: {}", e))
        })?;
        Ok(vgs)
    }

    /// List logical volumes in a volume group
    pub async fn list_logical_volumes(
        &self,
        vg_name: &str,
    ) -> Result<Vec<LogicalVolumeInfo>, ClientError> {
        let json = self.proxy.list_logical_volumes(vg_name).await?;
        let lvs: Vec<LogicalVolumeInfo> = serde_json::from_str(&json).map_err(|e| {
            ClientError::ParseError(format!("Failed to parse logical volume list: {}", e))
        })?;
        Ok(lvs)
    }

    /// List all physical volumes
    pub async fn list_physical_volumes(&self) -> Result<Vec<PhysicalVolumeInfo>, ClientError> {
        let json = self.proxy.list_physical_volumes().await?;
        let pvs: Vec<PhysicalVolumeInfo> = serde_json::from_str(&json).map_err(|e| {
            ClientError::ParseError(format!("Failed to parse physical volume list: {}", e))
        })?;
        Ok(pvs)
    }

    /// Create a new volume group from devices
    pub async fn create_volume_group(
        &self,
        name: &str,
        devices: &[String],
    ) -> Result<(), ClientError> {
        let devices_json = serde_json::to_string(devices)
            .map_err(|e| ClientError::ParseError(format!("Failed to serialize devices: {}", e)))?;
        Ok(self.proxy.create_volume_group(name, &devices_json).await?)
    }

    /// Create a logical volume, returns device path
    pub async fn create_logical_volume(
        &self,
        vg_name: &str,
        lv_name: &str,
        size: u64,
    ) -> Result<String, ClientError> {
        Ok(self
            .proxy
            .create_logical_volume(vg_name, lv_name, size)
            .await?)
    }

    /// Resize a logical volume
    pub async fn resize_logical_volume(
        &self,
        vg_name: &str,
        lv_name: &str,
        new_size: u64,
    ) -> Result<(), ClientError> {
        Ok(self
            .proxy
            .resize_logical_volume(vg_name, lv_name, new_size)
            .await?)
    }

    /// Delete a volume group
    pub async fn delete_volume_group(&self, vg_name: &str) -> Result<(), ClientError> {
        Ok(self.proxy.delete_volume_group(vg_name).await?)
    }

    /// Delete a logical volume
    pub async fn delete_logical_volume(
        &self,
        vg_name: &str,
        lv_name: &str,
    ) -> Result<(), ClientError> {
        Ok(self.proxy.delete_logical_volume(vg_name, lv_name).await?)
    }

    /// Remove a physical volume from a volume group
    pub async fn remove_physical_volume(
        &self,
        vg_name: &str,
        device: &str,
    ) -> Result<(), ClientError> {
        Ok(self.proxy.remove_physical_volume(vg_name, device).await?)
    }

    /// Get the underlying proxy for signal subscriptions
    pub fn proxy(&self) -> &LvmInterfaceProxy<'static> {
        &self.proxy
    }
}
