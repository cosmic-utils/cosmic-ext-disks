// SPDX-License-Identifier: GPL-3.0-only

use async_trait::async_trait;

use storage_types::{DiskInfo, FilesystemInfo, PartitionInfo};

use crate::StorageError;

#[async_trait]
pub trait DiskDiscovery: Send + Sync {
    async fn list_disks(&self) -> Result<Vec<DiskInfo>, StorageError>;
}

#[async_trait]
pub trait Partitioning: Send + Sync {
    async fn list_partitions(&self, disk_path: &str) -> Result<Vec<PartitionInfo>, StorageError>;
}

#[async_trait]
pub trait FilesystemDiscovery: Send + Sync {
    async fn list_filesystems(&self) -> Result<Vec<FilesystemInfo>, StorageError>;
}
