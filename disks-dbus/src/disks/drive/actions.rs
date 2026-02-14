use std::collections::HashMap;

use anyhow::Result;
use udisks2::{block::BlockProxy, drive::DriveProxy};
use zbus::zvariant::OwnedObjectPath;
use zbus::zvariant::Value;

use super::is_anyhow_device_busy;
use super::is_anyhow_not_supported;
use super::model::DriveModel;
use crate::CreatePartitionInfo;
use crate::disks::volume::VolumeNode;
use crate::disks::ops::{RealDiskBackend, drive_create_partition};

impl DriveModel {
    pub async fn eject(&self) -> Result<()> {
        if !self.ejectable {
            return Err(anyhow::anyhow!("Not supported by this drive"));
        }

        let proxy = DriveProxy::builder(&self.connection)
            .path(self.path.clone())?
            .build()
            .await?;

        match proxy.eject(HashMap::new()).await.map_err(Into::into) {
            Ok(()) => Ok(()),
            Err(e) if is_anyhow_not_supported(&e) => {
                Err(anyhow::anyhow!("Not supported by this drive"))
            }
            Err(e) if is_anyhow_device_busy(&e) => Err(anyhow::anyhow!(
                "Device is busy. Unmount any volumes on it and try again."
            )),
            Err(e) => Err(e),
        }
    }

    pub async fn power_off(&self) -> Result<()> {
        if !self.can_power_off {
            return Err(anyhow::anyhow!("Not supported by this drive"));
        }

        let proxy = DriveProxy::builder(&self.connection)
            .path(self.path.clone())?
            .build()
            .await?;
        proxy.power_off(HashMap::new()).await?;
        Ok(())
    }

    pub async fn open_for_backup(&self) -> Result<std::os::fd::OwnedFd> {
        let block_object_path: OwnedObjectPath = self.block_path.as_str().try_into()?;
        crate::disks::image::open_for_backup(block_object_path).await
    }

    pub async fn open_for_restore(&self) -> Result<std::os::fd::OwnedFd> {
        let block_object_path: OwnedObjectPath = self.block_path.as_str().try_into()?;
        crate::disks::image::open_for_restore(block_object_path).await
    }

    pub async fn standby_now(&self) -> Result<()> {
        let proxy = zbus::Proxy::new(
            &self.connection,
            "org.freedesktop.UDisks2",
            self.path.as_str(),
            "org.freedesktop.UDisks2.Drive.Ata",
        )
        .await?;

        let options: HashMap<&str, Value<'_>> = HashMap::new();
        let res: Result<()> = proxy
            .call("StandbyNow", &(options))
            .await
            .map_err(Into::into);
        match res {
            Ok(()) => Ok(()),
            Err(e) if is_anyhow_not_supported(&e) => {
                Err(anyhow::anyhow!("Not supported by this drive"))
            }
            Err(e) => Err(e),
        }
    }

    pub async fn wakeup(&self) -> Result<()> {
        let proxy = zbus::Proxy::new(
            &self.connection,
            "org.freedesktop.UDisks2",
            self.path.as_str(),
            "org.freedesktop.UDisks2.Drive.Ata",
        )
        .await?;

        let options: HashMap<&str, Value<'_>> = HashMap::new();
        let res: Result<()> = proxy.call("Wakeup", &(options)).await.map_err(Into::into);
        match res {
            Ok(()) => Ok(()),
            Err(e) if is_anyhow_not_supported(&e) => {
                Err(anyhow::anyhow!("Not supported by this drive"))
            }
            Err(e) => Err(e),
        }
    }

    pub async fn create_partition(&self, info: CreatePartitionInfo) -> Result<()> {
        let table_type = self
            .partition_table_type
            .as_deref()
            .ok_or_else(|| anyhow::anyhow!("No partition table type available"))?;

        let backend = RealDiskBackend::new(self.connection.clone());
        drive_create_partition(
            &backend,
            self.block_path.clone(),
            table_type,
            self.gpt_usable_range,
            info,
        )
        .await
    }

    /// Format the entire disk (drive block device) via UDisks2.
    ///
    /// `format_type` is passed directly to `org.freedesktop.UDisks2.Block.Format`, and may be
    /// values like `"gpt"`, `"dos"`, or `"empty"` (depending on UDisks support).
    ///
    /// If `erase` is true, request a zero-fill erase (slow) via the `erase=zero` option.
    pub async fn format_disk(&self, format_type: &str, erase: bool) -> Result<()> {
        // Preflight: ensure no mounted filesystems (including nested/LUKS/LVM children) keep the
        // disk busy, and teardown unlocked encrypted containers.
        self.preflight_unmount_and_lock().await?;

        let block_proxy = BlockProxy::builder(&self.connection)
            .path(self.block_path.clone())?
            .build()
            .await?;

        let mut format_options: HashMap<&str, Value<'_>> = HashMap::new();

        if erase {
            format_options.insert("erase", Value::from("zero"));
        }

        block_proxy.format(format_type, format_options).await?;
        Ok(())
    }

    async fn preflight_unmount_and_lock(&self) -> Result<()> {
        let mut first_err: Option<anyhow::Error> = None;

        for v in &self.volumes {
            // Post-order traversal (children before parent lock), but unmount as soon as we see a
            // mounted filesystem.
            let mut stack: Vec<(VolumeNode, bool)> = vec![(v.clone(), false)];
            while let Some((node, visited)) = stack.pop() {
                if !visited {
                    if node.is_mounted()
                        && let Err(e) = node.unmount().await
                    {
                        first_err.get_or_insert(e);
                    }

                    stack.push((node.clone(), true));
                    for child in node.children.iter().rev() {
                        stack.push((child.clone(), false));
                    }
                } else if node.can_lock()
                    && let Err(e) = node.lock().await
                {
                    first_err.get_or_insert(e);
                }
            }
        }

        if let Some(e) = first_err {
            return Err(e);
        }

        Ok(())
    }

    pub async fn remove(&self) -> Result<()> {
        // Always unmount any child volumes first so the device isn't busy.
        // Also lock (close) unlocked encrypted containers where possible.
        self.preflight_unmount_and_lock().await?;

        if self.is_loop {
            let proxy = zbus::Proxy::new(
                &self.connection,
                "org.freedesktop.UDisks2",
                self.block_path.as_str(),
                "org.freedesktop.UDisks2.Loop",
            )
            .await?;

            let options: HashMap<&str, Value<'_>> = HashMap::new();
            let res: Result<()> = proxy.call("Delete", &(options)).await.map_err(Into::into);
            match res {
                Ok(()) => Ok(()),
                Err(e) if is_anyhow_not_supported(&e) => Err(anyhow::anyhow!(
                    "Remove not supported: device does not implement org.freedesktop.UDisks2.Loop"
                )),
                Err(e) if is_anyhow_device_busy(&e) => Err(anyhow::anyhow!(
                    "Device is busy. Unmount any volumes on it and try again."
                )),
                Err(e) => Err(e),
            }
        } else if self.removable {
            // For removable drives, the expected "safe remove" behavior is power off.
            if !self.can_power_off {
                return Err(anyhow::anyhow!(
                    "Remove not supported: drive is removable but does not support power off"
                ));
            }
            self.power_off().await
        } else {
            Err(anyhow::anyhow!(
                "Remove not supported: device is neither a loop-backed image nor a removable drive"
            ))
        }
    }
}
