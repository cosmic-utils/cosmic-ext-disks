// SPDX-License-Identifier: GPL-3.0-only

use zbus::{proxy, Connection};
use storage_models::PartitionInfo;
use crate::client::error::ClientError;

/// D-Bus proxy interface for partition management
#[proxy(
    interface = "org.cosmic.ext.StorageService.Partitions",
    default_service = "org.cosmic.ext.StorageService",
    default_path = "/org/cosmic/ext/StorageService/partitions"
)]
trait PartitionsInterface {
    /// List all partitions on a disk
    async fn list_partitions(&self, disk: &str) -> zbus::Result<String>;
    
    /// Create a new partition table (destroys existing partitions)
    async fn create_partition_table(&self, disk: &str, table_type: &str) -> zbus::Result<()>;
    
    /// Create a new partition
    async fn create_partition(
        &self,
        disk: &str,
        offset: u64,
        size: u64,
        type_id: &str,
    ) -> zbus::Result<String>;
    
    /// Delete a partition
    async fn delete_partition(&self, partition: &str) -> zbus::Result<()>;
    
    /// Resize a partition
    async fn resize_partition(&self, partition: &str, new_size: u64) -> zbus::Result<()>;
    
    /// Set partition type (GPT GUID or MBR code)
    async fn set_partition_type(&self, partition: &str, type_id: &str) -> zbus::Result<()>;
    
    /// Set partition flags
    async fn set_partition_flags(&self, partition: &str, flags: u64) -> zbus::Result<()>;
    
    /// Set partition name (GPT only)
    async fn set_partition_name(&self, partition: &str, name: &str) -> zbus::Result<()>;
    
    /// Signal emitted when a partition table is created
    #[zbus(signal)]
    async fn partition_table_created(&self, disk: &str, table_type: &str) -> zbus::Result<()>;
    
    /// Signal emitted when a partition is created
    #[zbus(signal)]
    async fn partition_created(&self, disk: &str, partition: &str) -> zbus::Result<()>;
    
    /// Signal emitted when a partition is deleted
    #[zbus(signal)]
    async fn partition_deleted(&self, disk: &str, partition: &str) -> zbus::Result<()>;
    
    /// Signal emitted when a partition is modified
    #[zbus(signal)]
    async fn partition_modified(&self, partition: &str) -> zbus::Result<()>;
}

/// Client for partition management operations
pub struct PartitionsClient {
    proxy: PartitionsInterfaceProxy<'static>,
}

impl PartitionsClient {
    /// Create a new partitions client connected to the storage service
    pub async fn new() -> Result<Self, ClientError> {
        let conn = Connection::system().await.map_err(|e| {
            ClientError::Connection(format!("Failed to connect to system bus: {}", e))
        })?;
        
        let proxy = PartitionsInterfaceProxy::new(&conn).await.map_err(|e| {
            ClientError::Connection(format!("Failed to create partitions proxy: {}", e))
        })?;
        
        Ok(Self { proxy })
    }
    
    /// List all partitions on a disk
    pub async fn list_partitions(&self, disk: &str) -> Result<Vec<PartitionInfo>, ClientError> {
        let json = self.proxy.list_partitions(disk).await?;
        let partitions: Vec<PartitionInfo> = serde_json::from_str(&json)
            .map_err(|e| ClientError::ParseError(format!("Failed to parse partition list: {}", e)))?;
        Ok(partitions)
    }
    
    /// Create a new partition table (gpt or dos/mbr)
    pub async fn create_partition_table(&self, disk: &str, table_type: &str) -> Result<(), ClientError> {
        Ok(self.proxy.create_partition_table(disk, table_type).await?)
    }
    
    /// Create a new partition, returns the device path (e.g., /dev/sda1)
    pub async fn create_partition(
        &self,
        disk: &str,
        offset: u64,
        size: u64,
        type_id: &str,
    ) -> Result<String, ClientError> {
        Ok(self.proxy.create_partition(disk, offset, size, type_id).await?)
    }
    
    /// Delete a partition
    pub async fn delete_partition(&self, partition: &str) -> Result<(), ClientError> {
        Ok(self.proxy.delete_partition(partition).await?)
    }
    
    /// Resize a partition
    pub async fn resize_partition(&self, partition: &str, new_size: u64) -> Result<(), ClientError> {
        Ok(self.proxy.resize_partition(partition, new_size).await?)
    }
    
    /// Set partition type (GPT GUID or MBR hex code)
    pub async fn set_partition_type(&self, partition: &str, type_id: &str) -> Result<(), ClientError> {
        Ok(self.proxy.set_partition_type(partition, type_id).await?)
    }
    
    /// Set partition flags (e.g., bootable)
    pub async fn set_partition_flags(&self, partition: &str, flags: u64) -> Result<(), ClientError> {
        Ok(self.proxy.set_partition_flags(partition, flags).await?)
    }
    
    /// Set partition name (GPT only)
    pub async fn set_partition_name(&self, partition: &str, name: &str) -> Result<(), ClientError> {
        Ok(self.proxy.set_partition_name(partition, name).await?)
    }
    
    /// Get the underlying proxy for signal subscriptions
    pub fn proxy(&self) -> &PartitionsInterfaceProxy<'static> {
        &self.proxy
    }
}
