// SPDX-License-Identifier: GPL-3.0-only

pub mod discovery;
pub mod disk;
pub mod filesystem;
pub mod image;
pub mod luks;
pub mod partition;

pub use discovery::{DiskDiscovery, FilesystemDiscovery, Partitioning};
pub use disk::{DiskOpsAdapter, DiskQueryAdapter};
pub use filesystem::FilesystemOpsAdapter;
pub use image::ImageOpsAdapter;
pub use luks::LuksOpsAdapter;
pub use partition::PartitionOpsAdapter;
