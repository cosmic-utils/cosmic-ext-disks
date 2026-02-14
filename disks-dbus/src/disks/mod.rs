mod btrfs_native;
mod drive;
mod gpt;
pub mod image;
mod lvm;
mod manager;
mod ops;
mod process_finder;
mod smart;
mod volume;
mod volume_model;

pub use btrfs_native::{BtrfsFilesystem, BtrfsSubvolume};
pub use drive::DriveModel;
pub use gpt::{fallback_gpt_usable_range_bytes, probe_gpt_usable_range_bytes};
pub use lvm::list_lvs_for_pv;
pub use manager::{DeviceEvent, DeviceEventStream, DiskManager};
pub use process_finder::{find_processes_using_mount, kill_processes};
pub use smart::{SmartInfo, SmartSelfTestKind};
use thiserror::Error;
pub use volume::{BlockIndex, VolumeNode};
pub use volume_model::VolumeModel;

// Re-export these from storage_models (they're defined there now)
pub use storage_models::{CreatePartitionInfo, GPT_ALIGNMENT_BYTES, VolumeKind, VolumeType};

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct MountOptionsSettings {
    pub identify_as: String,
    pub mount_point: String,
    pub filesystem_type: String,
    pub mount_at_startup: bool,
    pub require_auth: bool,
    pub show_in_ui: bool,
    pub other_options: String,
    pub display_name: String,
    pub icon_name: String,
    pub symbolic_icon_name: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct EncryptionOptionsSettings {
    pub name: String,
    pub unlock_at_startup: bool,
    pub require_auth: bool,
    pub other_options: String,
}

// async fn get_size(path: impl Into<String> + std::fmt::Display) -> Result<String> {
//     let client = udisks2::Client::new().await?;
//     let object = client
//         .object(format!(
//             "/org/freedesktop/UDisks2/block_devices/{}",
//             path.to_string()
//         ))
//         .expect(&format!("No {} device found", path));
//     let block = object.block().await?;
//     let drive = client.drive_for_block(&block).await?;
//     Ok(client.size_for_display(drive.size().await?, true, true))
// }

#[derive(Error, Debug)]
pub enum DiskError {
    #[error("The model {0} is not connected")]
    NotConnected(String),

    #[error("Device is busy: {device} at {mount_point}")]
    ResourceBusy { device: String, mount_point: String },

    #[error("Zbus Error")]
    ZbusError(#[from] zbus::Error),
}
