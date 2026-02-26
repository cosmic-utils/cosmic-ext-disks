// SPDX-License-Identifier: GPL-3.0-only

//! D-Bus client wrappers for storage-service operations

pub mod btrfs;
pub mod connection;
pub mod disks;
pub mod error;
pub mod filesystems;
pub mod image;
pub mod logical;
pub mod luks;
pub mod partitions;
pub mod rclone;

pub use btrfs::BtrfsClient;
pub use disks::DisksClient;
pub use filesystems::FilesystemsClient;
pub use image::ImageClient;
pub use logical::LogicalClient;
pub use luks::LuksClient;
pub use partitions::PartitionsClient;
#[allow(unused_imports)]
pub use rclone::RcloneClient;
