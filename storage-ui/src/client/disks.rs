// SPDX-License-Identifier: GPL-3.0-only

use crate::client::connection::shared_connection;
use crate::client::error::ClientError;
use storage_common::{DiskInfo, SmartAttribute, SmartStatus, VolumeInfo};
use zbus::proxy;

/// D-Bus proxy interface for disk discovery and SMART operations
#[proxy(
    interface = "org.cosmic.ext.StorageService.Disks",
    default_service = "org.cosmic.ext.StorageService",
    default_path = "/org/cosmic/ext/StorageService/disks"
)]
pub trait DisksInterface {
    /// List all disks in the system
    async fn list_disks(&self) -> zbus::Result<String>;

    /// Get detailed information about a specific disk
    async fn get_disk_info(&self, device: &str) -> zbus::Result<String>;

    /// List all volumes across all disks with parent_path references
    async fn list_volumes(&self) -> zbus::Result<String>;

    /// Get information about a specific volume
    async fn get_volume_info(&self, device: &str) -> zbus::Result<String>;

    /// Get SMART status for a disk
    async fn get_smart_status(&self, device: &str) -> zbus::Result<String>;

    /// Get detailed SMART attributes
    async fn get_smart_attributes(&self, device: &str) -> zbus::Result<String>;

    /// Start a SMART self-test
    async fn start_smart_test(&self, device: &str, test_type: &str) -> zbus::Result<()>;

    /// Eject removable media
    async fn eject(&self, device: &str) -> zbus::Result<()>;

    /// Power off a drive
    async fn power_off(&self, device: &str) -> zbus::Result<()>;

    /// Put drive in low-power standby mode
    async fn standby_now(&self, device: &str) -> zbus::Result<()>;

    /// Wake drive from standby mode
    async fn wakeup(&self, device: &str) -> zbus::Result<()>;

    /// Safely remove a drive (unmount all, lock LUKS, eject, power off)
    async fn remove(&self, device: &str) -> zbus::Result<()>;

    /// Signal emitted when a disk is added (hotplug)
    #[zbus(signal)]
    async fn disk_added(&self, device: &str, info_json: &str) -> zbus::Result<()>;

    /// Signal emitted when a disk is removed
    #[zbus(signal)]
    async fn disk_removed(&self, device: &str) -> zbus::Result<()>;

    /// Signal emitted when a SMART test completes
    #[zbus(signal)]
    async fn smart_test_completed(&self, device: &str, success: bool) -> zbus::Result<()>;
}

/// Client for disk discovery and SMART operations
pub struct DisksClient {
    proxy: DisksInterfaceProxy<'static>,
}

impl std::fmt::Debug for DisksClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DisksClient").finish_non_exhaustive()
    }
}

impl DisksClient {
    /// Create a new disks client connected to the storage service
    pub async fn new() -> Result<Self, ClientError> {
        let conn = shared_connection().await?;

        let proxy = DisksInterfaceProxy::new(conn)
            .await
            .map_err(|e| ClientError::Connection(format!("Failed to create disks proxy: {}", e)))?;

        Ok(Self { proxy })
    }

    /// List all disks in the system
    pub async fn list_disks(&self) -> Result<Vec<DiskInfo>, ClientError> {
        let json = self.proxy.list_disks().await?;
        let disks: Vec<DiskInfo> = serde_json::from_str(&json)
            .map_err(|e| ClientError::ParseError(format!("Failed to parse disk list: {}", e)))?;
        Ok(disks)
    }

    /// Get detailed information about a specific disk
    pub async fn get_disk_info(&self, device: &str) -> Result<DiskInfo, ClientError> {
        let json = self.proxy.get_disk_info(device).await?;
        let disk: DiskInfo = serde_json::from_str(&json)
            .map_err(|e| ClientError::ParseError(format!("Failed to parse disk info: {}", e)))?;
        Ok(disk)
    }

    /// List all volumes across all disks
    ///
    /// Returns a flat list of all volumes with parent_path populated for building hierarchies.
    pub async fn list_volumes(&self) -> Result<Vec<VolumeInfo>, ClientError> {
        let json = self.proxy.list_volumes().await?;
        let volumes: Vec<VolumeInfo> = serde_json::from_str(&json)
            .map_err(|e| ClientError::ParseError(format!("Failed to parse volume list: {}", e)))?;
        Ok(volumes)
    }

    /// Get information about a specific volume
    ///
    /// This supports atomic updates - query a single volume instead of refreshing the entire list.
    #[allow(dead_code)]
    pub async fn get_volume_info(&self, device: &str) -> Result<VolumeInfo, ClientError> {
        let json = self.proxy.get_volume_info(device).await?;
        let volume: VolumeInfo = serde_json::from_str(&json)
            .map_err(|e| ClientError::ParseError(format!("Failed to parse volume info: {}", e)))?;
        Ok(volume)
    }

    /// Get SMART status for a disk
    pub async fn get_smart_status(&self, device: &str) -> Result<SmartStatus, ClientError> {
        let json = self.proxy.get_smart_status(device).await?;
        let status: SmartStatus = serde_json::from_str(&json)
            .map_err(|e| ClientError::ParseError(format!("Failed to parse SMART status: {}", e)))?;
        Ok(status)
    }

    /// Get detailed SMART attributes
    pub async fn get_smart_attributes(
        &self,
        device: &str,
    ) -> Result<Vec<SmartAttribute>, ClientError> {
        let json = self.proxy.get_smart_attributes(device).await?;
        let attributes: Vec<SmartAttribute> = serde_json::from_str(&json).map_err(|e| {
            ClientError::ParseError(format!("Failed to parse SMART attributes: {}", e))
        })?;
        Ok(attributes)
    }

    /// Start a SMART self-test (short, long, or conveyance)
    pub async fn start_smart_test(&self, device: &str, test_type: &str) -> Result<(), ClientError> {
        Ok(self.proxy.start_smart_test(device, test_type).await?)
    }

    /// Eject removable media (optical drives, USB sticks)
    ///
    /// Requires no authentication for active sessions.
    #[allow(dead_code)]
    pub async fn eject(&self, device: &str) -> Result<(), ClientError> {
        Ok(self.proxy.eject(device).await?)
    }

    /// Power off an external drive
    ///
    /// Requires administrator authentication (cached for session).
    pub async fn power_off(&self, device: &str) -> Result<(), ClientError> {
        Ok(self.proxy.power_off(device).await?)
    }

    /// Put drive in low-power standby mode (ATA drives)
    ///
    /// Requires no authentication for active sessions.
    pub async fn standby_now(&self, device: &str) -> Result<(), ClientError> {
        Ok(self.proxy.standby_now(device).await?)
    }

    /// Wake drive from standby mode (ATA drives)
    ///
    /// Requires no authentication for active sessions.
    pub async fn wakeup(&self, device: &str) -> Result<(), ClientError> {
        Ok(self.proxy.wakeup(device).await?)
    }

    /// Safely remove a drive
    ///
    /// This performs a complete safe removal workflow:
    /// 1. Unmount all filesystems on the drive
    /// 2. Lock any LUKS encrypted containers
    /// 3. Eject the media if ejectable
    /// 4. Power off the drive if possible
    ///
    /// Requires administrator authentication (cached for session).
    pub async fn remove(&self, device: &str) -> Result<(), ClientError> {
        Ok(self.proxy.remove(device).await?)
    }

    /// Get the underlying proxy for signal subscriptions
    pub fn proxy(&self) -> &DisksInterfaceProxy<'static> {
        &self.proxy
    }
}
