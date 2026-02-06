use anyhow::Result;
use udisks2::{
    block::BlockProxy,
    drive::{DriveProxy, RotationRate},
};
use zbus::{Connection, zvariant::OwnedObjectPath};

use crate::disks::{ByteRange, VolumeModel, VolumeNode};

#[derive(Debug, Clone)]
pub struct DriveModel {
    pub can_power_off: bool,
    pub ejectable: bool,
    pub media_available: bool,
    pub media_change_detected: bool,
    pub media_removable: bool,
    pub optical: bool,
    pub optical_blank: bool,
    pub removable: bool,
    pub rotation_rate: i32,
    pub id: String,
    pub model: String,
    pub revision: String,
    pub serial: String,
    pub vendor: String,
    pub size: u64,
    pub name: String,
    pub block_path: String,
    pub is_loop: bool,
    pub backing_file: Option<String>,
    pub volumes_flat: Vec<VolumeModel>,
    pub volumes: Vec<VolumeNode>,
    pub path: String,
    pub partition_table_type: Option<String>,
    pub gpt_usable_range: Option<ByteRange>,
    pub(super) connection: Connection,
}

impl DriveModel {
    pub(super) fn is_missing_encrypted_interface(err: &anyhow::Error) -> bool {
        let msg = err.to_string();
        msg.contains("No such interface")
            && msg.contains("org.freedesktop.UDisks2.Encrypted")
            && msg.contains("InvalidArgs")
    }

    pub async fn from_proxy(
        path: &str,
        block_path: &str,
        drive_proxy: &DriveProxy<'_>,
    ) -> Result<Self> {
        let mut size = drive_proxy.size().await?;
        if size == 0 {
            let connection = Connection::system().await?;
            let block_proxy = BlockProxy::builder(&connection)
                .path(block_path)?
                .build()
                .await?;
            size = block_proxy.size().await?;
        }

        Ok(DriveModel {
            name: path.to_owned(),
            path: path.to_string(),
            size,
            id: drive_proxy.id().await?,
            model: drive_proxy.model().await?,
            serial: drive_proxy.serial().await?,
            vendor: drive_proxy.vendor().await?,
            block_path: block_path.to_string(),
            is_loop: false,
            backing_file: None,
            volumes_flat: vec![],
            volumes: vec![],
            can_power_off: drive_proxy.can_power_off().await?,
            ejectable: drive_proxy.ejectable().await?,
            media_available: drive_proxy.media_available().await?,
            media_change_detected: drive_proxy.media_change_detected().await?,
            media_removable: drive_proxy.media_removable().await?,
            optical: drive_proxy.optical().await?,
            optical_blank: drive_proxy.optical_blank().await?,
            removable: drive_proxy.removable().await?,
            revision: drive_proxy.revision().await?,
            rotation_rate: match drive_proxy.rotation_rate().await {
                Ok(rate) => match rate {
                    RotationRate::Rotating(rpm) => rpm,
                    RotationRate::NonRotating => 0,
                    RotationRate::Unknown => -1,
                },
                Err(_) => 0,
            },
            partition_table_type: None,
            gpt_usable_range: None,
            connection: Connection::system().await?,
        })
    }

    pub async fn from_block_only(
        block_path: &OwnedObjectPath,
        block_proxy: &BlockProxy<'_>,
        backing_file: Option<String>,
    ) -> Result<Self> {
        let size = block_proxy.size().await?;

        Ok(DriveModel {
            name: block_path.to_string(),
            path: String::new(),
            size,
            id: String::new(),
            model: String::new(),
            serial: String::new(),
            vendor: String::new(),
            block_path: block_path.to_string(),
            is_loop: backing_file.is_some(),
            backing_file,
            volumes_flat: vec![],
            volumes: vec![],
            can_power_off: false,
            ejectable: false,
            media_available: true,
            media_change_detected: false,
            media_removable: false,
            optical: false,
            optical_blank: false,
            removable: false,
            revision: String::new(),
            rotation_rate: 0,
            partition_table_type: None,
            gpt_usable_range: None,
            connection: Connection::system().await?,
        })
    }

    pub fn name(&self) -> String {
        self.name
            .split('/')
            .next_back()
            .unwrap_or(&self.name)
            .replace('_', " ")
    }

    /// Check if the drive supports power management (spin down/standby).
    /// Returns true for spinning disks (rotation_rate > 0), false for SSDs and NVMe drives.
    pub fn supports_power_management(&self) -> bool {
        // Loop devices don't support power management
        if self.is_loop {
            return false;
        }

        // Only rotating media (HDDs) support power management
        // rotation_rate: -1 = unknown, 0 = SSD/NVMe, >0 = HDD
        self.rotation_rate != 0
    }
}
