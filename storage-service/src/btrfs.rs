// SPDX-License-Identifier: GPL-3.0-only

use crate::auth::require_authorization;
use disks_btrfs::{SubvolumeList, SubvolumeManager};
use std::path::PathBuf;
use zbus::{interface, Connection};
use zbus::message::Header as MessageHeader;
use zbus::object_server::SignalEmitter;

/// BTRFS operations handler
pub struct BtrfsHandler;

impl BtrfsHandler {
    pub fn new() -> Self {
        Self
    }
}

#[interface(name = "org.cosmic.ext.StorageService.Btrfs")]
impl BtrfsHandler {
    /// List all subvolumes in a BTRFS filesystem
    async fn list_subvolumes(
        &self,
        #[zbus(connection)] connection: &Connection,
        #[zbus(header)] header: MessageHeader<'_>,
        #[zbus(signal_context)] _ctx: SignalEmitter<'_>,
        mountpoint: &str,
    ) -> zbus::fdo::Result<String> {
        let sender = header
            .sender()
            .ok_or_else(|| zbus::fdo::Error::Failed("No sender".to_string()))?
            .as_str();
        
        // Check read authorization (less restrictive)
        require_authorization(connection, sender, "org.cosmic.ext.storage-service.btrfs-read")
            .await?;
        
        tracing::info!("Listing subvolumes at {}", mountpoint);
        
        let manager = SubvolumeManager::new(mountpoint)
            .map_err(|e| zbus::fdo::Error::Failed(e.to_string()))?;
        
        let subvolumes = manager
            .list_all()
            .map_err(|e| zbus::fdo::Error::Failed(e.to_string()))?;
        
        let default_id = manager.get_default().unwrap_or(5);
        
        let list = SubvolumeList {
            subvolumes,
            default_id,
        };
        
        let json = serde_json::to_string(&list)
            .map_err(|e| zbus::fdo::Error::Failed(format!("Serialization error: {}", e)))?;
        
        Ok(json)
    }
    
    /// Create a new subvolume
    async fn create_subvolume(
        &self,
        #[zbus(connection)] connection: &Connection,
        #[zbus(header)] header: MessageHeader<'_>,
        #[zbus(signal_context)] ctx: SignalEmitter<'_>,
        mountpoint: &str,
        name: &str,
    ) -> zbus::fdo::Result<()> {
        let sender = header
            .sender()
            .ok_or_else(|| zbus::fdo::Error::Failed("No sender".to_string()))?
            .as_str();
        
        require_authorization(connection, sender, "org.cosmic.ext.storage-service.btrfs-modify")
            .await?;
        
        tracing::info!("Creating subvolume {} at {}", name, mountpoint);
        
        let manager = SubvolumeManager::new(mountpoint)
            .map_err(|e| zbus::fdo::Error::Failed(e.to_string()))?;
        
        manager
            .create(name)
            .map_err(|e| zbus::fdo::Error::Failed(e.to_string()))?;
        
        // Emit signal
        Self::subvolume_changed(&ctx, mountpoint, "created")
            .await
            .ok();
        
        Ok(())
    }
    
    /// Create a snapshot of a subvolume
    async fn create_snapshot(
        &self,
        #[zbus(connection)] connection: &Connection,
        #[zbus(header)] header: MessageHeader<'_>,
        #[zbus(signal_context)] ctx: SignalEmitter<'_>,
        mountpoint: &str,
        source_path: &str,
        dest_path: &str,
        readonly: bool,
    ) -> zbus::fdo::Result<()> {
        let sender = header
            .sender()
            .ok_or_else(|| zbus::fdo::Error::Failed("No sender".to_string()))?
            .as_str();
        
        require_authorization(connection, sender, "org.cosmic.ext.storage-service.btrfs-modify")
            .await?;
        
        tracing::info!(
            "Creating snapshot from {} to {}, readonly={}",
            source_path,
            dest_path,
            readonly
        );
        
        let manager = SubvolumeManager::new(mountpoint)
            .map_err(|e| zbus::fdo::Error::Failed(e.to_string()))?;
        
        manager
            .snapshot(
                &PathBuf::from(source_path),
                &PathBuf::from(dest_path),
                readonly,
                false, // recursive
            )
            .map_err(|e| zbus::fdo::Error::Failed(e.to_string()))?;
        
        // Emit signal
        Self::subvolume_changed(&ctx, dest_path, "created")
            .await
            .ok();
        
        Ok(())
    }
    
    /// Delete a subvolume
    async fn delete_subvolume(
        &self,
        #[zbus(connection)] connection: &Connection,
        #[zbus(header)] header: MessageHeader<'_>,
        #[zbus(signal_context)] ctx: SignalEmitter<'_>,
        mountpoint: &str,
        path: &str,
        recursive: bool,
    ) -> zbus::fdo::Result<()> {
        let sender = header
            .sender()
            .ok_or_else(|| zbus::fdo::Error::Failed("No sender".to_string()))?
            .as_str();
        
        require_authorization(connection, sender, "org.cosmic.ext.storage-service.btrfs-modify")
            .await?;
        
        tracing::info!("Deleting subvolume at {}, recursive={}", path, recursive);
        
        let manager = SubvolumeManager::new(mountpoint)
            .map_err(|e| zbus::fdo::Error::Failed(e.to_string()))?;
        
        manager
            .delete(&PathBuf::from(path), recursive)
            .map_err(|e| zbus::fdo::Error::Failed(e.to_string()))?;
        
        // Emit signal
        Self::subvolume_changed(&ctx, path, "deleted")
            .await
            .ok();
        
        Ok(())
    }
    
    /// Set or unset the read-only flag on a subvolume
    async fn set_readonly(
        &self,
        #[zbus(connection)] connection: &Connection,
        #[zbus(header)] header: MessageHeader<'_>,
        #[zbus(signal_context)] ctx: SignalEmitter<'_>,
        mountpoint: &str,
        path: &str,
        readonly: bool,
    ) -> zbus::fdo::Result<()> {
        let sender = header
            .sender()
            .ok_or_else(|| zbus::fdo::Error::Failed("No sender".to_string()))?
            .as_str();
        
        require_authorization(connection, sender, "org.cosmic.ext.storage-service.btrfs-modify")
            .await?;
        
        tracing::info!("Setting readonly={} on {}", readonly, path);
        
        let manager = SubvolumeManager::new(mountpoint)
            .map_err(|e| zbus::fdo::Error::Failed(e.to_string()))?;
        
        manager
            .set_readonly(&PathBuf::from(path), readonly)
            .map_err(|e| zbus::fdo::Error::Failed(e.to_string()))?;
        
        // Emit signal
        Self::subvolume_changed(&ctx, path, "modified")
            .await
            .ok();
        
        Ok(())
    }
    
    /// Set a subvolume as the default
    async fn set_default(
        &self,
        #[zbus(connection)] connection: &Connection,
        #[zbus(header)] header: MessageHeader<'_>,
        #[zbus(signal_context)] ctx: SignalEmitter<'_>,
        mountpoint: &str,
        path: &str,
    ) -> zbus::fdo::Result<()> {
        let sender = header
            .sender()
            .ok_or_else(|| zbus::fdo::Error::Failed("No sender".to_string()))?
            .as_str();
        
        require_authorization(connection, sender, "org.cosmic.ext.storage-service.btrfs-modify")
            .await?;
        
        tracing::info!("Setting default subvolume to {}", path);
        
        let manager = SubvolumeManager::new(mountpoint)
            .map_err(|e| zbus::fdo::Error::Failed(e.to_string()))?;
        
        manager
            .set_default(&PathBuf::from(path))
            .map_err(|e| zbus::fdo::Error::Failed(e.to_string()))?;
        
        // Emit signal
        Self::subvolume_changed(&ctx, path, "modified")
            .await
            .ok();
        
        Ok(())
    }
    
    /// Get the default subvolume ID
    async fn get_default(
        &self,
        #[zbus(connection)] connection: &Connection,
        #[zbus(header)] header: MessageHeader<'_>,
        mountpoint: &str,
    ) -> zbus::fdo::Result<u64> {
        let sender = header
            .sender()
            .ok_or_else(|| zbus::fdo::Error::Failed("No sender".to_string()))?
            .as_str();
        
        require_authorization(connection, sender, "org.cosmic.ext.storage-service.btrfs-read")
            .await?;
        
        let manager = SubvolumeManager::new(mountpoint)
            .map_err(|e| zbus::fdo::Error::Failed(e.to_string()))?;
        
        let default_id = manager
            .get_default()
            .map_err(|e| zbus::fdo::Error::Failed(e.to_string()))?;
        
        Ok(default_id)
    }
    
    /// List deleted subvolumes pending cleanup
    async fn list_deleted(
        &self,
        #[zbus(connection)] connection: &Connection,
        #[zbus(header)] header: MessageHeader<'_>,
        mountpoint: &str,
    ) -> zbus::fdo::Result<String> {
        let sender = header
            .sender()
            .ok_or_else(|| zbus::fdo::Error::Failed("No sender".to_string()))?
            .as_str();
        
        require_authorization(connection, sender, "org.cosmic.ext.storage-service.btrfs-read")
            .await?;
        
        tracing::info!("Listing deleted subvolumes at {}", mountpoint);
        
        let manager = SubvolumeManager::new(mountpoint)
            .map_err(|e| zbus::fdo::Error::Failed(e.to_string()))?;
        
        let deleted = manager
            .list_deleted()
            .map_err(|e| zbus::fdo::Error::Failed(e.to_string()))?;
        
        let json = serde_json::to_string(&deleted)
            .map_err(|e| zbus::fdo::Error::Failed(format!("Serialization error: {}", e)))?;
        
        Ok(json)
    }
    
    /// Get filesystem usage information
    async fn get_usage(
        &self,
        #[zbus(connection)] connection: &Connection,
        #[zbus(header)] header: MessageHeader<'_>,
        mountpoint: &str,
    ) -> zbus::fdo::Result<String> {
        let sender = header
            .sender()
            .ok_or_else(|| zbus::fdo::Error::Failed("No sender".to_string()))?
            .as_str();
        
        require_authorization(connection, sender, "org.cosmic.ext.storage-service.btrfs-read")
            .await?;
        
        tracing::info!("Getting usage for {}", mountpoint);
        
        let usage = disks_btrfs::get_filesystem_usage(&PathBuf::from(mountpoint))
            .map_err(|e| zbus::fdo::Error::Failed(e.to_string()))?;
        
        let json = serde_json::to_string(&usage)
            .map_err(|e| zbus::fdo::Error::Failed(format!("Serialization error: {}", e)))?;
        
        Ok(json)
    }
    
    /// Signal: Subvolume was modified
    #[zbus(signal)]
    async fn subvolume_changed(
        ctx: &SignalEmitter<'_>,
        path: &str,
        change_type: &str,
    ) -> zbus::Result<()>;
}
