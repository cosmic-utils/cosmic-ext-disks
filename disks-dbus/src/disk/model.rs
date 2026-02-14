use anyhow::Result;
use storage_models::{ByteRange, DiskInfo};
use udisks2::{
    block::BlockProxy,
    drive::{DriveProxy, RotationRate},
};
use zbus::{Connection, zvariant::OwnedObjectPath};

use crate::volume::{VolumeNode, BlockIndex};

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
        self.rotation_rate > 0
    }

    /// Get volume tree as canonical storage-models types.
    /// 
    /// This converts the internal VolumeNode tree structure to storage_models::VolumeInfo,
    /// which is the recommended type for clients.
    pub fn get_volumes(&self) -> Vec<storage_models::VolumeInfo> {
        self.volumes.iter().map(|v| v.clone().into()).collect()
    }

    /// Get flat list of partitions as canonical storage-models types.
    /// 
    /// This converts the internal VolumeModel list to storage_models::PartitionInfo,
    /// which is the recommended type for partition operations.
    pub fn get_partitions(&self) -> Vec<storage_models::PartitionInfo> {
        self.volumes_flat.iter().map(|v| v.clone().into()).collect()
    }
}

/// Convert DriveModel to storage-models DiskInfo (extracts domain data, drops connection)
impl From<DriveModel> for DiskInfo {
    fn from(drive: DriveModel) -> Self {
        // Infer connection bus from drive properties
        let connection_bus = infer_connection_bus(&drive);
        
        // Convert rotation_rate (-1/0/rpm) to Option<u16>
        let rotation_rate = if drive.rotation_rate > 0 {
            Some(drive.rotation_rate as u16)
        } else {
            None
        };
        
        DiskInfo {
            // Identity
            device: drive.block_path,
            id: drive.id,
            model: drive.model,
            serial: drive.serial,
            vendor: drive.vendor,
            revision: drive.revision,
            
            // Physical properties
            size: drive.size,
            connection_bus,
            rotation_rate,
            
            // Media properties
            removable: drive.removable,
            ejectable: drive.ejectable,
            media_removable: drive.media_removable,
            media_available: drive.media_available,
            optical: drive.optical,
            optical_blank: drive.optical_blank,
            can_power_off: drive.can_power_off,
            
            // Loop device
            is_loop: drive.is_loop,
            backing_file: drive.backing_file,
            
            // Partitioning
            partition_table_type: drive.partition_table_type,
            gpt_usable_range: drive.gpt_usable_range,
        }
    }
}

/// Infer connection bus type from drive properties
fn infer_connection_bus(drive: &DriveModel) -> String {
    if drive.is_loop {
        return "loop".to_string();
    }
    
    let path_lower = drive.block_path.to_lowercase();
    let model_lower = drive.model.to_lowercase();
    let vendor_lower = drive.vendor.to_lowercase();
    
    // Check block device path patterns
    if path_lower.contains("nvme") {
        return "nvme".to_string();
    }
    if path_lower.contains("mmc") || path_lower.contains("mmcblk") {
        return "mmc".to_string();
    }
    if path_lower.contains("sr") || drive.optical {
        return "optical".to_string();
    }
    
    // Check vendor/model for USB indicators
    if model_lower.contains("usb") || vendor_lower.contains("usb") {
        return "usb".to_string();
    }
    
    // Default to ata/sata for traditional disks (sd*)
    "ata".to_string()
}
