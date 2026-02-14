// SPDX-License-Identifier: GPL-3.0-only

use zbus::{proxy, Connection};
use storage_models::btrfs::{FilesystemUsage, SubvolumeList, DeletedSubvolume};
use crate::client::error::ClientError;

/// D-Bus proxy interface for BTRFS operations
#[proxy(
    interface = "org.cosmic.ext.StorageService.Btrfs",
    default_service = "org.cosmic.ext.StorageService",
    default_path = "/org/cosmic/ext/StorageService/btrfs"
)]
trait BtrfsInterface {
    /// List all subvolumes in a BTRFS filesystem
    async fn list_subvolumes(&self, mountpoint: &str) -> zbus::Result<String>;
    
    /// Create a new subvolume
    async fn create_subvolume(&self, mountpoint: &str, name: &str) -> zbus::Result<()>;
    
    /// Create a snapshot of a subvolume
    async fn create_snapshot(
        &self,
        mountpoint: &str,
        source_path: &str,
        dest_path: &str,
        readonly: bool,
    ) -> zbus::Result<()>;
    
    /// Delete a subvolume
    async fn delete_subvolume(&self, mountpoint: &str, path: &str, recursive: bool) -> zbus::Result<()>;
    
    /// Set or unset the read-only flag on a subvolume
    async fn set_readonly(&self, mountpoint: &str, path: &str, readonly: bool) -> zbus::Result<()>;
    
    /// Set a subvolume as the default
    async fn set_default(&self, mountpoint: &str, path: &str) -> zbus::Result<()>;
    
    /// Get the default subvolume ID
    async fn get_default(&self, mountpoint: &str) -> zbus::Result<u64>;
    
    /// List deleted subvolumes pending cleanup
    async fn list_deleted(&self, mountpoint: &str) -> zbus::Result<String>;
    
    /// Get filesystem usage information
    async fn get_usage(&self, mountpoint: &str) -> zbus::Result<String>;
}

/// Client for BTRFS operations via D-Bus
pub struct BtrfsClient {
    proxy: BtrfsInterfaceProxy<'static>,
}

impl std::fmt::Debug for BtrfsClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BtrfsClient").finish_non_exhaustive()
    }
}

impl BtrfsClient {
    /// Create a new BTRFS client connected to the storage service
    pub async fn new() -> Result<Self, ClientError> {
        let conn = Connection::system().await.map_err(|e| {
            ClientError::Connection(format!("Failed to connect to system bus: {}", e))
        })?;
        
        let proxy = BtrfsInterfaceProxy::new(&conn).await.map_err(|e| {
            ClientError::Connection(format!("Failed to create proxy: {}", e))
        })?;
        
        Ok(Self { proxy })
    }
    
    /// List all subvolumes in a BTRFS filesystem
    pub async fn list_subvolumes(&self, mountpoint: &str) -> Result<SubvolumeList, ClientError> {
        let json = self.proxy.list_subvolumes(mountpoint).await?;
        let list: SubvolumeList = serde_json::from_str(&json)?;
        Ok(list)
    }
    
    /// Create a new subvolume
    pub async fn create_subvolume(&self, mountpoint: &str, name: &str) -> Result<(), ClientError> {
        Ok(self.proxy.create_subvolume(mountpoint, name).await?)
    }
    
    /// Create a snapshot of a subvolume
    pub async fn create_snapshot(
        &self,
        mountpoint: &str,
        source_path: &str,
        dest_path: &str,
        readonly: bool,
    ) -> Result<(), ClientError> {
        Ok(self.proxy.create_snapshot(mountpoint, source_path, dest_path, readonly).await?)
    }
    
    /// Delete a subvolume
    pub async fn delete_subvolume(
        &self,
        mountpoint: &str,
        path: &str,
        recursive: bool,
    ) -> Result<(), ClientError> {
        Ok(self.proxy.delete_subvolume(mountpoint, path, recursive).await?)
    }
    
    /// Set or unset the read-only flag on a subvolume
    pub async fn set_readonly(
        &self,
        mountpoint: &str,
        path: &str,
        readonly: bool,
    ) -> Result<(), ClientError> {
        Ok(self.proxy.set_readonly(mountpoint, path, readonly).await?)
    }
    
    /// Set a subvolume as the default
    pub async fn set_default(&self, mountpoint: &str, path: &str) -> Result<(), ClientError> {
        Ok(self.proxy.set_default(mountpoint, path).await?)
    }
    
    /// Get the default subvolume ID
    pub async fn get_default(&self, mountpoint: &str) -> Result<u64, ClientError> {
        Ok(self.proxy.get_default(mountpoint).await?)
    }
    
    /// List deleted subvolumes pending cleanup
    pub async fn list_deleted(&self, mountpoint: &str) -> Result<Vec<DeletedSubvolume>, ClientError> {
        let json = self.proxy.list_deleted(mountpoint).await?;
        let deleted: Vec<DeletedSubvolume> = serde_json::from_str(&json)?;
        Ok(deleted)
    }
    
    /// Get filesystem usage information
    pub async fn get_usage(&self, mountpoint: &str) -> Result<FilesystemUsage, ClientError> {
        let json = self.proxy.get_usage(mountpoint).await?;
        let usage: FilesystemUsage = serde_json::from_str(&json)?;
        Ok(usage)
    }
}
