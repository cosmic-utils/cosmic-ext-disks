// SPDX-License-Identifier: GPL-3.0-only

//! D-Bus client wrappers for storage-service operations

pub mod btrfs;
pub mod disks;
pub mod error;
pub mod filesystems;
pub mod image;
pub mod luks;
pub mod lvm;
pub mod partitions;

pub use btrfs::BtrfsClient;
pub use disks::DisksClient;
pub use error::ClientError;
pub use filesystems::FilesystemsClient;
pub use image::ImageClient;
pub use luks::LuksClient;
pub use lvm::LvmClient;
pub use partitions::PartitionsClient;
