mod create_partition_info;
mod drive;
mod gpt;
mod lvm;
mod manager;
mod ops;
mod partition;
mod smart;
mod volume;

pub use create_partition_info::*;
pub use drive::*;
pub use gpt::*;
pub use lvm::*;
pub use manager::*;
pub use partition::PartitionModel;
pub use smart::*;
use thiserror::Error;
pub use volume::*;

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

    #[error("Zbus Error")]
    ZbusError(#[from] zbus::Error),
}
