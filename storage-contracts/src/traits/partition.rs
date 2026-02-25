// SPDX-License-Identifier: GPL-3.0-only

use async_trait::async_trait;

use storage_types::{CreatePartitionInfo, DiskInfo, PartitionInfo};

use crate::StorageError;

#[async_trait]
pub trait PartitionOpsAdapter: Send + Sync {
    async fn list_disks_with_partitions(
        &self,
    ) -> Result<Vec<(DiskInfo, Vec<PartitionInfo>)>, StorageError>;

    async fn resolve_block_path_for_device(&self, device: &str) -> Result<String, StorageError>;

    async fn create_partition_table(
        &self,
        block_path: &str,
        table_type: &str,
    ) -> Result<(), StorageError>;

    async fn create_partition(
        &self,
        block_path: &str,
        offset: u64,
        size: u64,
        type_id: &str,
    ) -> Result<String, StorageError>;

    async fn create_partition_with_filesystem(
        &self,
        block_path: &str,
        info: &CreatePartitionInfo,
    ) -> Result<String, StorageError>;

    async fn delete_partition(&self, partition_path: &str) -> Result<(), StorageError>;

    async fn resize_partition(
        &self,
        partition_path: &str,
        new_size: u64,
    ) -> Result<(), StorageError>;

    async fn set_partition_type(
        &self,
        partition_path: &str,
        type_id: &str,
    ) -> Result<(), StorageError>;

    async fn set_partition_flags(
        &self,
        partition_path: &str,
        flags: u64,
    ) -> Result<(), StorageError>;

    async fn set_partition_name(
        &self,
        partition_path: &str,
        name: &str,
    ) -> Result<(), StorageError>;
}
