//! UDisks2 BTRFS D-Bus interface wrapper
//!
//! Provides Rust wrappers around the org.freedesktop.UDisks2.Filesystem.BTRFS D-Bus interface.
//! This replaces CLI subprocess calls with proper D-Bus integration.

use anyhow::{Context, Result};
use std::collections::HashMap;
use zbus::{Connection, zvariant::OwnedObjectPath};
use zbus::zvariant::Value;

/// Represents a BTRFS subvolume from D-Bus GetSubvolumes() call
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BtrfsSubvolume {
    /// Subvolume ID
    pub id: u64,
    /// Parent subvolume ID
    pub parent_id: u64,
    /// Full path relative to mount point
    pub path: String,
}

impl BtrfsSubvolume {
    /// Get the name (last component of path)
    pub fn name(&self) -> &str {
        self.path.rsplit('/').next().unwrap_or(&self.path)
    }
}

/// BTRFS filesystem D-Bus proxy
///
/// Wraps the org.freedesktop.UDisks2.Filesystem.BTRFS interface for a specific block device.
pub struct BtrfsFilesystem<'a> {
    connection: &'a Connection,
    block_path: OwnedObjectPath,
}

impl<'a> BtrfsFilesystem<'a> {
    /// Create a new BTRFS filesystem proxy for the given block device path
    pub fn new(connection: &'a Connection, block_path: OwnedObjectPath) -> Self {
        Self {
            connection,
            block_path,
        }
    }

    /// Check if the BTRFS interface is available on this block device
    pub async fn is_available(&self) -> bool {
        // Try to create the proxy and check if interface exists
        zbus::Proxy::new(
            self.connection,
            "org.freedesktop.UDisks2",
            &self.block_path,
            "org.freedesktop.UDisks2.Filesystem.BTRFS",
        )
        .await
        .is_ok()
    }

    /// Get list of subvolumes
    ///
    /// # Arguments
    /// * `snapshots_only` - If true, only return snapshots; if false, return all subvolumes
    pub async fn get_subvolumes(&self, snapshots_only: bool) -> Result<Vec<BtrfsSubvolume>> {
        let proxy: zbus::Proxy<'_> = zbus::Proxy::new(
            self.connection,
            "org.freedesktop.UDisks2",
            &self.block_path,
            "org.freedesktop.UDisks2.Filesystem.BTRFS",
        )
        .await
        .context("Failed to create BTRFS interface proxy")?;
        
        let options: HashMap<String, Value> = HashMap::new();

        // Call GetSubvolumes(b snapshots_only, a{sv} options) -> (a(tts) subvols, i count)
        let result: (Vec<(u64, u64, String)>, i32) = proxy
            .call("GetSubvolumes", &(snapshots_only, options))
            .await
            .context("Failed to call GetSubvolumes")?;

        let (subvols, _count) = result;
        Ok(subvols
            .into_iter()
            .map(|(id, parent_id, path)| BtrfsSubvolume {
                id,
                parent_id,
                path,
            })
            .collect())
    }

    /// Create a new subvolume
    ///
    /// # Arguments
    /// * `name` - Name/path of the subvolume to create
    pub async fn create_subvolume(&self, name: &str) -> Result<()> {
        let proxy: zbus::Proxy<'_> = zbus::Proxy::new(
            self.connection,
            "org.freedesktop.UDisks2",
            &self.block_path,
            "org.freedesktop.UDisks2.Filesystem.BTRFS",
        )
        .await
        .context("Failed to create BTRFS interface proxy")?;
        
        let options: HashMap<String, Value> = HashMap::new();

        // Call CreateSubvolume(s name, a{sv} options) -> nothing
        let _: () = proxy
            .call("CreateSubvolume", &(name, options))
            .await
            .context(format!("Failed to create subvolume '{}'", name))?;

        Ok(())
    }

    /// Remove a subvolume
    ///
    /// # Arguments
    /// * `name` - Name/path of the subvolume to remove
    pub async fn remove_subvolume(&self, name: &str) -> Result<()> {
        let proxy: zbus::Proxy<'_> = zbus::Proxy::new(
            self.connection,
            "org.freedesktop.UDisks2",
            &self.block_path,
            "org.freedesktop.UDisks2.Filesystem.BTRFS",
        )
        .await
        .context("Failed to create BTRFS interface proxy")?;
        
        let options: HashMap<String, Value> = HashMap::new();

        // Call RemoveSubvolume(s name, a{sv} options) -> nothing
        let _: () = proxy
            .call("RemoveSubvolume", &(name, options))
            .await
            .context(format!("Failed to remove subvolume '{}'", name))?;

        Ok(())
    }

    /// Create a snapshot
    ///
    /// # Arguments
    /// * `source` - Source subvolume path
    /// * `dest` - Destination snapshot path
    /// * `read_only` - Whether the snapshot should be read-only
    pub async fn create_snapshot(
        &self,
        source: &str,
        dest: &str,
        read_only: bool,
    ) -> Result<()> {
        let proxy: zbus::Proxy<'_> = zbus::Proxy::new(
            self.connection,
            "org.freedesktop.UDisks2",
            &self.block_path,
            "org.freedesktop.UDisks2.Filesystem.BTRFS",
        )
        .await
        .context("Failed to create BTRFS interface proxy")?;
        
        let options: HashMap<String, Value> = HashMap::new();

        // Call CreateSnapshot(s source, s dest, b ro, a{sv} options) -> nothing
        let _: () = proxy
            .call("CreateSnapshot", &(source, dest, read_only, options))
            .await
            .context(format!(
                "Failed to create snapshot from '{}' to '{}'",
                source, dest
            ))?;

        Ok(())
    }

    /// Get the used space in bytes
    pub async fn get_used_space(&self) -> Result<u64> {
        let proxy: zbus::Proxy<'_> = zbus::Proxy::new(
            self.connection,
            "org.freedesktop.UDisks2",
            &self.block_path,
            "org.freedesktop.UDisks2.Filesystem.BTRFS",
        )
        .await
        .context("Failed to create BTRFS interface proxy")?;
        
        let used: u64 = proxy
            .get_property("used")
            .await
            .context("Failed to get 'used' property")?;
        Ok(used)
    }

    /// Get the filesystem label
    pub async fn get_label(&self) -> Result<String> {
        let proxy: zbus::Proxy<'_> = zbus::Proxy::new(
            self.connection,
            "org.freedesktop.UDisks2",
            &self.block_path,
            "org.freedesktop.UDisks2.Filesystem.BTRFS",
        )
        .await
        .context("Failed to create BTRFS interface proxy")?;
        
        let label: String = proxy
            .get_property("label")
            .await
            .context("Failed to get 'label' property")?;
        Ok(label)
    }

    /// Get the filesystem UUID
    pub async fn get_uuid(&self) -> Result<String> {
        let proxy: zbus::Proxy<'_> = zbus::Proxy::new(
            self.connection,
            "org.freedesktop.UDisks2",
            &self.block_path,
            "org.freedesktop.UDisks2.Filesystem.BTRFS",
        )
        .await
        .context("Failed to create BTRFS interface proxy")?;
        
        let uuid: String = proxy
            .get_property("uuid")
            .await
            .context("Failed to get 'uuid' property")?;
        Ok(uuid)
    }

    /// Get the number of devices in the filesystem
    pub async fn get_num_devices(&self) -> Result<u32> {
        let proxy: zbus::Proxy<'_> = zbus::Proxy::new(
            self.connection,
            "org.freedesktop.UDisks2",
            &self.block_path,
            "org.freedesktop.UDisks2.Filesystem.BTRFS",
        )
        .await
        .context("Failed to create BTRFS interface proxy")?;
        
        let num_devices: u32 = proxy
            .get_property("num_devices")
            .await
            .context("Failed to get 'num_devices' property")?;
        Ok(num_devices)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_subvolume_name() {
        let subvol = BtrfsSubvolume {
            id: 256,
            parent_id: 5,
            path: "/home/user/data".to_string(),
        };
        assert_eq!(subvol.name(), "data");

        let root_subvol = BtrfsSubvolume {
            id: 5,
            parent_id: 0,
            path: "/".to_string(),
        };
        assert_eq!(root_subvol.name(), "/");

        let simple_subvol = BtrfsSubvolume {
            id: 257,
            parent_id: 5,
            path: "snapshots".to_string(),
        };
        assert_eq!(simple_subvol.name(), "snapshots");
    }

    #[test]
    fn test_subvolume_clone() {
        let subvol = BtrfsSubvolume {
            id: 256,
            parent_id: 5,
            path: "/test".to_string(),
        };
        let cloned = subvol.clone();
        assert_eq!(subvol, cloned);
    }
}
