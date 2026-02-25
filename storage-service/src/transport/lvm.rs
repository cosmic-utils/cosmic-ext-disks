// SPDX-License-Identifier: GPL-3.0-only

//! LVM (Logical Volume Manager) D-Bus interface
//!
//! This module provides D-Bus methods for managing LVM volume groups,
//! logical volumes, and physical volumes.

use std::process::Command;
use std::sync::Arc;
use storage_macros::authorized_interface;
use storage_types::{LogicalVolumeInfo, PhysicalVolumeInfo, VolumeGroupInfo};
use zbus::message::Header as MessageHeader;
use zbus::{Connection, interface};

use crate::domain::lvm::{DefaultLvmDomain, LvmDomain};

/// D-Bus interface for LVM management operations
pub struct LVMHandler {
    domain: Arc<dyn LvmDomain>,
}

impl LVMHandler {
    /// Create a new LVMHandler
    pub fn new() -> Self {
        let domain: Arc<dyn LvmDomain> = Arc::new(DefaultLvmDomain::new());
        if let Err(error) = domain.require_lvm() {
            tracing::warn!("LVM operations will be disabled: {error}");
        }
        Self { domain }
    }

    /// Ensure LVM tools are available
    fn require_lvm(&self) -> Result<(), zbus::fdo::Error> {
        self.domain.require_lvm()
    }
}

#[interface(name = "org.cosmic.ext.Storage.Service.LVM")]
impl LVMHandler {
    /// Signal emitted when a volume group is created
    #[zbus(signal)]
    async fn volume_group_created(
        signal_ctxt: &zbus::object_server::SignalEmitter<'_>,
        vg_name: &str,
    ) -> zbus::Result<()>;

    /// Signal emitted when a volume group is removed
    #[zbus(signal)]
    async fn volume_group_removed(
        signal_ctxt: &zbus::object_server::SignalEmitter<'_>,
        vg_name: &str,
    ) -> zbus::Result<()>;

    /// Signal emitted when a logical volume is created
    #[zbus(signal)]
    async fn logical_volume_created(
        signal_ctxt: &zbus::object_server::SignalEmitter<'_>,
        vg_name: &str,
        lv_name: &str,
    ) -> zbus::Result<()>;

    /// Signal emitted when a logical volume is removed
    #[zbus(signal)]
    async fn logical_volume_removed(
        signal_ctxt: &zbus::object_server::SignalEmitter<'_>,
        vg_name: &str,
        lv_name: &str,
    ) -> zbus::Result<()>;

    /// List all volume groups
    ///
    /// Returns: JSON-serialized Vec<VolumeGroupInfo>
    ///
    /// Authorization: org.cosmic.ext.storage.service.lvm-read (allow_active)
    #[authorized_interface(action = "org.cosmic.ext.storage.service.lvm-read")]
    async fn list_volume_groups(
        &self,
        #[zbus(connection)] _connection: &Connection,
        #[zbus(header)] _header: MessageHeader<'_>,
    ) -> zbus::fdo::Result<String> {
        self.require_lvm()?;

        tracing::debug!("Listing volume groups (UID {})", caller.uid);

        // Call vgs command
        let output = Command::new("vgs")
            .args([
                "--noheadings",
                "--units",
                "b",
                "--nosuffix",
                "-o",
                "vg_name,vg_uuid,vg_size,vg_free,pv_count,lv_count",
                "--separator",
                "\t",
            ])
            .output()
            .map_err(|e| {
                tracing::error!("Failed to run vgs: {e}");
                zbus::fdo::Error::Failed(format!("Failed to run vgs: {e}"))
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            tracing::error!("vgs failed: {stderr}");
            return Err(zbus::fdo::Error::Failed(format!(
                "vgs command failed: {stderr}"
            )));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut vgs = Vec::new();

        for line in stdout.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }

            let mut parts = line.split('\t');
            let name = match parts.next() {
                Some(v) => v.trim().to_string(),
                None => continue,
            };
            let uuid = parts.next().unwrap_or("").trim().to_string();
            let size: u64 = parts.next().unwrap_or("0").trim().parse().unwrap_or(0);
            let free: u64 = parts.next().unwrap_or("0").trim().parse().unwrap_or(0);
            let pv_count: u32 = parts.next().unwrap_or("0").trim().parse().unwrap_or(0);
            let lv_count: u32 = parts.next().unwrap_or("0").trim().parse().unwrap_or(0);

            vgs.push(VolumeGroupInfo {
                name,
                uuid,
                size,
                free,
                pv_count,
                lv_count,
            });
        }

        tracing::debug!("Found {} volume groups", vgs.len());

        let json = serde_json::to_string(&vgs)
            .map_err(|e| zbus::fdo::Error::Failed(format!("Serialize error: {e}")))?;

        Ok(json)
    }

    /// List all logical volumes
    ///
    /// Returns: JSON-serialized Vec<LogicalVolumeInfo>
    ///
    /// Authorization: org.cosmic.ext.storage.service.lvm-read (allow_active)
    #[authorized_interface(action = "org.cosmic.ext.storage.service.lvm-read")]
    async fn list_logical_volumes(
        &self,
        #[zbus(connection)] _connection: &Connection,
        #[zbus(header)] _header: MessageHeader<'_>,
    ) -> zbus::fdo::Result<String> {
        self.require_lvm()?;

        tracing::debug!("Listing logical volumes (UID {})", caller.uid);

        // Call lvs command
        let output = Command::new("lvs")
            .args([
                "--noheadings",
                "--units",
                "b",
                "--nosuffix",
                "-o",
                "lv_name,vg_name,lv_uuid,lv_size,lv_path,lv_active",
                "--separator",
                "\t",
            ])
            .output()
            .map_err(|e| {
                tracing::error!("Failed to run lvs: {e}");
                zbus::fdo::Error::Failed(format!("Failed to run lvs: {e}"))
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            tracing::error!("lvs failed: {stderr}");
            return Err(zbus::fdo::Error::Failed(format!(
                "lvs command failed: {stderr}"
            )));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut lvs = Vec::new();

        for line in stdout.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }

            let mut parts = line.split('\t');
            let name = match parts.next() {
                Some(v) => v.trim().to_string(),
                None => continue,
            };
            let vg_name = parts.next().unwrap_or("").trim().to_string();
            let uuid = parts.next().unwrap_or("").trim().to_string();
            let size: u64 = parts.next().unwrap_or("0").trim().parse().unwrap_or(0);
            let device_path = parts.next().unwrap_or("").trim().to_string();
            let active_str = parts.next().unwrap_or("").trim();
            let active = active_str == "active";

            lvs.push(LogicalVolumeInfo {
                name,
                vg_name,
                uuid,
                size,
                device_path,
                active,
            });
        }

        tracing::debug!("Found {} logical volumes", lvs.len());

        let json = serde_json::to_string(&lvs)
            .map_err(|e| zbus::fdo::Error::Failed(format!("Serialize error: {e}")))?;

        Ok(json)
    }

    /// List all physical volumes
    ///
    /// Returns: JSON-serialized Vec<PhysicalVolumeInfo>
    ///
    /// Authorization: org.cosmic.ext.storage.service.lvm-read (allow_active)
    #[authorized_interface(action = "org.cosmic.ext.storage.service.lvm-read")]
    async fn list_physical_volumes(
        &self,
        #[zbus(connection)] _connection: &Connection,
        #[zbus(header)] _header: MessageHeader<'_>,
    ) -> zbus::fdo::Result<String> {
        self.require_lvm()?;

        tracing::debug!("Listing physical volumes (UID {})", caller.uid);

        // Call pvs command
        let output = Command::new("pvs")
            .args([
                "--noheadings",
                "--units",
                "b",
                "--nosuffix",
                "-o",
                "pv_name,vg_name,pv_size,pv_free",
                "--separator",
                "\t",
            ])
            .output()
            .map_err(|e| {
                tracing::error!("Failed to run pvs: {e}");
                zbus::fdo::Error::Failed(format!("Failed to run pvs: {e}"))
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            tracing::error!("pvs failed: {stderr}");
            return Err(zbus::fdo::Error::Failed(format!(
                "pvs command failed: {stderr}"
            )));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut pvs = Vec::new();

        for line in stdout.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }

            let mut parts = line.split('\t');
            let device = match parts.next() {
                Some(v) => v.trim().to_string(),
                None => continue,
            };
            let vg_name_str = parts.next().unwrap_or("").trim();
            let vg_name = if vg_name_str.is_empty() {
                None
            } else {
                Some(vg_name_str.to_string())
            };
            let size: u64 = parts.next().unwrap_or("0").trim().parse().unwrap_or(0);
            let free: u64 = parts.next().unwrap_or("0").trim().parse().unwrap_or(0);

            pvs.push(PhysicalVolumeInfo {
                device,
                vg_name,
                size,
                free,
            });
        }

        tracing::debug!("Found {} physical volumes", pvs.len());

        let json = serde_json::to_string(&pvs)
            .map_err(|e| zbus::fdo::Error::Failed(format!("Serialize error: {e}")))?;

        Ok(json)
    }

    /// Create a new volume group
    ///
    /// Args:
    /// - vg_name: Name for the new volume group
    /// - devices_json: JSON-serialized Vec<String> of device paths (e.g., ["/dev/sda1", "/dev/sdb1"])
    ///
    /// Authorization: org.cosmic.ext.storage.service.lvm-modify (auth_admin_keep)
    #[authorized_interface(action = "org.cosmic.ext.storage.service.lvm-modify")]
    async fn create_volume_group(
        &self,
        #[zbus(connection)] _connection: &Connection,
        #[zbus(header)] _header: MessageHeader<'_>,
        #[zbus(signal_context)] signal_ctx: zbus::object_server::SignalEmitter<'_>,
        vg_name: String,
        devices_json: String,
    ) -> zbus::fdo::Result<()> {
        self.require_lvm()?;

        // Parse devices
        let devices: Vec<String> = serde_json::from_str(&devices_json)
            .map_err(|e| zbus::fdo::Error::InvalidArgs(format!("Invalid devices JSON: {e}")))?;

        if devices.is_empty() {
            return Err(zbus::fdo::Error::InvalidArgs(
                "At least one device required".to_string(),
            ));
        }

        tracing::info!(
            "Creating volume group '{}' with devices: {:?} (UID {})",
            vg_name,
            devices,
            caller.uid
        );

        // Run vgcreate
        let output = Command::new("vgcreate")
            .arg(&vg_name)
            .args(&devices)
            .output()
            .map_err(|e| {
                tracing::error!("Failed to run vgcreate: {e}");
                zbus::fdo::Error::Failed(format!("Failed to run vgcreate: {e}"))
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            tracing::error!("vgcreate failed: {stderr}");
            return Err(zbus::fdo::Error::Failed(format!(
                "vgcreate failed: {stderr}"
            )));
        }

        tracing::info!("Volume group '{}' created successfully", vg_name);
        let _ = Self::volume_group_created(&signal_ctx, &vg_name).await;
        Ok(())
    }

    /// Create a new logical volume
    ///
    /// Args:
    /// - vg_name: Name of the parent volume group
    /// - lv_name: Name for the new logical volume
    /// - size_bytes: Size in bytes
    ///
    /// Authorization: org.cosmic.ext.storage.service.lvm-modify (auth_admin_keep)
    #[authorized_interface(action = "org.cosmic.ext.storage.service.lvm-modify")]
    async fn create_logical_volume(
        &self,
        #[zbus(connection)] _connection: &Connection,
        #[zbus(header)] _header: MessageHeader<'_>,
        #[zbus(signal_context)] signal_ctx: zbus::object_server::SignalEmitter<'_>,
        vg_name: String,
        lv_name: String,
        size_bytes: u64,
    ) -> zbus::fdo::Result<String> {
        self.require_lvm()?;

        tracing::info!(
            "Creating logical volume '{}/{}' with size {} bytes (UID {})",
            vg_name,
            lv_name,
            size_bytes,
            caller.uid
        );

        // Run lvcreate with size in bytes
        let size_arg = format!("{}B", size_bytes);
        let output = Command::new("lvcreate")
            .args(["-L", &size_arg, "-n", &lv_name, &vg_name])
            .output()
            .map_err(|e| {
                tracing::error!("Failed to run lvcreate: {e}");
                zbus::fdo::Error::Failed(format!("Failed to run lvcreate: {e}"))
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            tracing::error!("lvcreate failed: {stderr}");
            return Err(zbus::fdo::Error::Failed(format!(
                "lvcreate failed: {stderr}"
            )));
        }

        let device_path = format!("/dev/{}/{}", vg_name, lv_name);
        tracing::info!("Logical volume created: {}", device_path);
        let _ = Self::logical_volume_created(&signal_ctx, &vg_name, &lv_name).await;
        Ok(device_path)
    }

    /// Resize a logical volume
    ///
    /// Args:
    /// - lv_path: Logical volume path (e.g., "/dev/vg0/lv0" or "vg0/lv0")
    /// - new_size_bytes: New size in bytes
    ///
    /// Authorization: org.cosmic.ext.storage.service.lvm-modify (auth_admin_keep)
    #[authorized_interface(action = "org.cosmic.ext.storage.service.lvm-modify")]
    async fn resize_logical_volume(
        &self,
        #[zbus(connection)] _connection: &Connection,
        #[zbus(header)] _header: MessageHeader<'_>,
        lv_path: String,
        new_size_bytes: u64,
    ) -> zbus::fdo::Result<()> {
        self.require_lvm()?;

        tracing::info!(
            "Resizing logical volume '{}' to {} bytes (UID {})",
            lv_path,
            new_size_bytes,
            caller.uid
        );

        // Run lvresize with new size in bytes
        let size_arg = format!("{}B", new_size_bytes);
        let output = Command::new("lvresize")
            .args(["-L", &size_arg, &lv_path])
            .output()
            .map_err(|e| {
                tracing::error!("Failed to run lvresize: {e}");
                zbus::fdo::Error::Failed(format!("Failed to run lvresize: {e}"))
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            tracing::error!("lvresize failed: {stderr}");
            return Err(zbus::fdo::Error::Failed(format!(
                "lvresize failed: {stderr}"
            )));
        }

        tracing::info!("Logical volume '{}' resized successfully", lv_path);

        Ok(())
    }

    /// Delete a volume group
    ///
    /// Args:
    /// - vg_name: Name of the volume group to delete
    ///
    /// Authorization: org.cosmic.ext.storage.service.lvm-modify (auth_admin_keep)
    #[authorized_interface(action = "org.cosmic.ext.storage.service.lvm-modify")]
    async fn delete_volume_group(
        &self,
        #[zbus(connection)] _connection: &Connection,
        #[zbus(header)] _header: MessageHeader<'_>,
        #[zbus(signal_context)] signal_ctx: zbus::object_server::SignalEmitter<'_>,
        vg_name: String,
    ) -> zbus::fdo::Result<()> {
        self.require_lvm()?;

        tracing::info!("Deleting volume group '{}' (UID {})", vg_name, caller.uid);

        // Run vgremove
        let output = Command::new("vgremove")
            .args(["-f", &vg_name])
            .output()
            .map_err(|e| {
                tracing::error!("Failed to run vgremove: {e}");
                zbus::fdo::Error::Failed(format!("Failed to run vgremove: {e}"))
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            tracing::error!("vgremove failed: {stderr}");
            return Err(zbus::fdo::Error::Failed(format!(
                "vgremove failed: {stderr}"
            )));
        }

        tracing::info!("Volume group '{}' deleted successfully", vg_name);
        let _ = Self::volume_group_removed(&signal_ctx, &vg_name).await;
        Ok(())
    }

    /// Delete a logical volume
    ///
    /// Args:
    /// - lv_path: Logical volume path (e.g., "/dev/vg0/lv0" or "vg0/lv0")
    ///
    /// Authorization: org.cosmic.ext.storage.service.lvm-modify (auth_admin_keep)
    #[authorized_interface(action = "org.cosmic.ext.storage.service.lvm-modify")]
    async fn delete_logical_volume(
        &self,
        #[zbus(connection)] _connection: &Connection,
        #[zbus(header)] _header: MessageHeader<'_>,
        #[zbus(signal_context)] signal_ctx: zbus::object_server::SignalEmitter<'_>,
        lv_path: String,
    ) -> zbus::fdo::Result<()> {
        self.require_lvm()?;

        tracing::info!("Deleting logical volume '{}' (UID {})", lv_path, caller.uid);

        // Run lvremove
        let output = Command::new("lvremove")
            .args(["-f", &lv_path])
            .output()
            .map_err(|e| {
                tracing::error!("Failed to run lvremove: {e}");
                zbus::fdo::Error::Failed(format!("Failed to run lvremove: {e}"))
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            tracing::error!("lvremove failed: {stderr}");
            return Err(zbus::fdo::Error::Failed(format!(
                "lvremove failed: {stderr}"
            )));
        }

        tracing::info!("Logical volume '{}' deleted successfully", lv_path);
        let (vg_name, lv_name) = lv_path
            .trim_start_matches("/dev/")
            .rsplit_once('/')
            .map(|(vg, lv)| (vg.to_string(), lv.to_string()))
            .unwrap_or_else(|| (String::new(), lv_path.clone()));
        let _ = Self::logical_volume_removed(&signal_ctx, &vg_name, &lv_name).await;
        Ok(())
    }

    /// Remove a physical volume from a volume group
    ///
    /// Args:
    /// - vg_name: Name of the volume group
    /// - pv_device: Physical volume device path (e.g., "/dev/sda1")
    ///
    /// Authorization: org.cosmic.ext.storage.service.lvm-modify (auth_admin_keep)
    #[authorized_interface(action = "org.cosmic.ext.storage.service.lvm-modify")]
    async fn remove_physical_volume(
        &self,
        #[zbus(connection)] _connection: &Connection,
        #[zbus(header)] _header: MessageHeader<'_>,
        vg_name: String,
        pv_device: String,
    ) -> zbus::fdo::Result<()> {
        self.require_lvm()?;

        tracing::info!(
            "Removing physical volume '{}' from volume group '{}' (UID {})",
            pv_device,
            vg_name,
            caller.uid
        );

        // Run vgreduce
        let output = Command::new("vgreduce")
            .args([&vg_name, &pv_device])
            .output()
            .map_err(|e| {
                tracing::error!("Failed to run vgreduce: {e}");
                zbus::fdo::Error::Failed(format!("Failed to run vgreduce: {e}"))
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            tracing::error!("vgreduce failed: {stderr}");
            return Err(zbus::fdo::Error::Failed(format!(
                "vgreduce failed: {stderr}"
            )));
        }

        tracing::info!(
            "Physical volume '{}' removed from volume group '{}'",
            pv_device,
            vg_name
        );

        Ok(())
    }
}
