mod drive;
mod manager;
mod partition;
mod create_partition_info;

pub use drive::*;
pub use manager::*;
pub use create_partition_info::*;
pub use partition::PartitionModel;
use thiserror::Error;

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
