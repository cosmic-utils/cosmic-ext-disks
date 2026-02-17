// SPDX-License-Identifier: GPL-3.0-only

use disks_btrfs::SubvolumeManager;
use std::path::PathBuf;
use storage_common::btrfs::SubvolumeList;
use storage_service_macros::authorized_interface;
use zbus::message::Header as MessageHeader;
use zbus::object_server::SignalEmitter;
use zbus::{Connection, interface};

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
    #[authorized_interface(action = "org.cosmic.ext.storage-service.btrfs-read")]
    async fn list_subvolumes(
        &self,
        #[zbus(connection)] _connection: &Connection,
        #[zbus(header)] _header: MessageHeader<'_>,
        #[zbus(signal_context)] _ctx: SignalEmitter<'_>,
        mountpoint: &str,
    ) -> zbus::fdo::Result<String> {
        tracing::info!("Listing subvolumes at {} (UID {})", mountpoint, caller.uid);

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
    #[authorized_interface(action = "org.cosmic.ext.storage-service.btrfs-modify")]
    async fn create_subvolume(
        &self,
        #[zbus(connection)] _connection: &Connection,
        #[zbus(header)] _header: MessageHeader<'_>,
        #[zbus(signal_context)] ctx: SignalEmitter<'_>,
        mountpoint: &str,
        name: &str,
    ) -> zbus::fdo::Result<()> {
        tracing::info!(
            "Creating subvolume {} at {} (UID {})",
            name,
            mountpoint,
            caller.uid
        );

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
    #[authorized_interface(action = "org.cosmic.ext.storage-service.btrfs-modify")]
    #[allow(clippy::too_many_arguments)]
    async fn create_snapshot(
        &self,
        #[zbus(connection)] _connection: &Connection,
        #[zbus(header)] _header: MessageHeader<'_>,
        #[zbus(signal_context)] ctx: SignalEmitter<'_>,
        mountpoint: &str,
        source_path: &str,
        dest_path: &str,
        readonly: bool,
    ) -> zbus::fdo::Result<()> {
        tracing::info!(
            "Creating snapshot from {} to {}, readonly={} (UID {})",
            source_path,
            dest_path,
            readonly,
            caller.uid
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
    #[authorized_interface(action = "org.cosmic.ext.storage-service.btrfs-modify")]
    async fn delete_subvolume(
        &self,
        #[zbus(connection)] _connection: &Connection,
        #[zbus(header)] _header: MessageHeader<'_>,
        #[zbus(signal_context)] ctx: SignalEmitter<'_>,
        mountpoint: &str,
        path: &str,
        recursive: bool,
    ) -> zbus::fdo::Result<()> {
        tracing::info!(
            "Deleting subvolume at {}, recursive={} (UID {})",
            path,
            recursive,
            caller.uid
        );

        let manager = SubvolumeManager::new(mountpoint)
            .map_err(|e| zbus::fdo::Error::Failed(e.to_string()))?;

        manager
            .delete(&PathBuf::from(path), recursive)
            .map_err(|e| zbus::fdo::Error::Failed(e.to_string()))?;

        // Emit signal
        Self::subvolume_changed(&ctx, path, "deleted").await.ok();

        Ok(())
    }

    /// Set or unset the read-only flag on a subvolume
    #[authorized_interface(action = "org.cosmic.ext.storage-service.btrfs-modify")]
    async fn set_readonly(
        &self,
        #[zbus(connection)] _connection: &Connection,
        #[zbus(header)] _header: MessageHeader<'_>,
        #[zbus(signal_context)] ctx: SignalEmitter<'_>,
        mountpoint: &str,
        path: &str,
        readonly: bool,
    ) -> zbus::fdo::Result<()> {
        tracing::info!(
            "Setting readonly={} on {} (UID {})",
            readonly,
            path,
            caller.uid
        );

        let manager = SubvolumeManager::new(mountpoint)
            .map_err(|e| zbus::fdo::Error::Failed(e.to_string()))?;

        manager
            .set_readonly(&PathBuf::from(path), readonly)
            .map_err(|e| zbus::fdo::Error::Failed(e.to_string()))?;

        // Emit signal
        Self::subvolume_changed(&ctx, path, "modified").await.ok();

        Ok(())
    }

    /// Set a subvolume as the default
    #[authorized_interface(action = "org.cosmic.ext.storage-service.btrfs-modify")]
    async fn set_default(
        &self,
        #[zbus(connection)] _connection: &Connection,
        #[zbus(header)] _header: MessageHeader<'_>,
        #[zbus(signal_context)] ctx: SignalEmitter<'_>,
        mountpoint: &str,
        path: &str,
    ) -> zbus::fdo::Result<()> {
        tracing::info!("Setting default subvolume to {} (UID {})", path, caller.uid);

        let manager = SubvolumeManager::new(mountpoint)
            .map_err(|e| zbus::fdo::Error::Failed(e.to_string()))?;

        manager
            .set_default(&PathBuf::from(path))
            .map_err(|e| zbus::fdo::Error::Failed(e.to_string()))?;

        // Emit signal
        Self::subvolume_changed(&ctx, path, "modified").await.ok();

        Ok(())
    }

    /// Get the default subvolume ID
    #[authorized_interface(action = "org.cosmic.ext.storage-service.btrfs-read")]
    async fn get_default(
        &self,
        #[zbus(connection)] _connection: &Connection,
        #[zbus(header)] _header: MessageHeader<'_>,
        mountpoint: &str,
    ) -> zbus::fdo::Result<u64> {
        let manager = SubvolumeManager::new(mountpoint)
            .map_err(|e| zbus::fdo::Error::Failed(e.to_string()))?;

        let default_id = manager
            .get_default()
            .map_err(|e| zbus::fdo::Error::Failed(e.to_string()))?;

        Ok(default_id)
    }

    /// List deleted subvolumes pending cleanup
    #[authorized_interface(action = "org.cosmic.ext.storage-service.btrfs-read")]
    async fn list_deleted(
        &self,
        #[zbus(connection)] _connection: &Connection,
        #[zbus(header)] _header: MessageHeader<'_>,
        mountpoint: &str,
    ) -> zbus::fdo::Result<String> {
        tracing::info!(
            "Listing deleted subvolumes at {} (UID {})",
            mountpoint,
            caller.uid
        );

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
    #[authorized_interface(action = "org.cosmic.ext.storage-service.btrfs-read")]
    async fn get_usage(
        &self,
        #[zbus(connection)] _connection: &Connection,
        #[zbus(header)] _header: MessageHeader<'_>,
        mountpoint: &str,
    ) -> zbus::fdo::Result<String> {
        tracing::info!("Getting usage for {} (UID {})", mountpoint, caller.uid);

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
