use std::collections::HashMap;

use anyhow::Result;
use tracing::{error, info, warn};
use udisks2::{
    Client, block::BlockProxy, drive::DriveProxy, partition::PartitionProxy,
    partitiontable::PartitionTableProxy,
};
use zbus::zvariant::OwnedValue;
use zbus::zvariant::Value;
use zbus::{Connection, zvariant::OwnedObjectPath};

use super::{SmartInfo, SmartSelfTestKind};

fn is_dbus_not_supported(err: &zbus::Error) -> bool {
    match err {
        zbus::Error::MethodError(name, _msg, _info) => matches!(
            name.as_str(),
            "org.freedesktop.DBus.Error.UnknownInterface"
                | "org.freedesktop.DBus.Error.UnknownMethod"
                | "org.freedesktop.DBus.Error.UnknownProperty"
        ),
        _ => false,
    }
}

fn is_dbus_device_busy(err: &zbus::Error) -> bool {
    match err {
        zbus::Error::MethodError(name, _msg, _info) => {
            name.as_str() == "org.freedesktop.UDisks2.Error.DeviceBusy"
        }
        _ => false,
    }
}

fn is_anyhow_not_supported(err: &anyhow::Error) -> bool {
    err.downcast_ref::<zbus::Error>()
        .is_some_and(is_dbus_not_supported)
}

fn is_anyhow_device_busy(err: &anyhow::Error) -> bool {
    err.downcast_ref::<zbus::Error>()
        .is_some_and(is_dbus_device_busy)
}

use crate::CreatePartitionInfo;

use super::{
    ByteRange, VolumeModel, fallback_gpt_usable_range_bytes, manager::UDisks2ManagerProxy,
    probe_gpt_usable_range_bytes,
};

use super::{BlockIndex, VolumeKind, VolumeNode};

use super::ops::{RealDiskBackend, drive_create_partition};

#[derive(Debug, Clone)]
pub struct DriveModel {
    pub can_power_off: bool,
    pub ejectable: bool,
    pub media_available: bool,
    pub media_change_detected: bool,
    pub media_removable: bool,
    pub optical: bool,
    pub optical_blank: bool,
    pub removable: bool,
    pub id: String,
    pub model: String,
    pub revision: String,
    pub serial: String,
    pub vendor: String,
    pub size: u64,
    pub name: String,
    pub block_path: String,
    pub is_loop: bool,
    pub backing_file: Option<String>,
    pub volumes_flat: Vec<VolumeModel>,
    pub volumes: Vec<VolumeNode>,
    pub path: String,
    pub partition_table_type: Option<String>,
    pub gpt_usable_range: Option<ByteRange>,
    connection: Connection,
}

#[derive(Debug, Clone)]
struct DriveBlockPair {
    block_path: OwnedObjectPath,
    drive_path: Option<OwnedObjectPath>,
    backing_file: Option<String>,
}

impl DriveModel {
    fn is_missing_encrypted_interface(err: &anyhow::Error) -> bool {
        let msg = err.to_string();
        msg.contains("No such interface")
            && msg.contains("org.freedesktop.UDisks2.Encrypted")
            && msg.contains("InvalidArgs")
    }

    pub async fn from_proxy(
        path: &str,
        block_path: &str,
        drive_proxy: &DriveProxy<'_>,
    ) -> Result<Self> {
        let mut size = drive_proxy.size().await?;
        if size == 0 {
            let connection = Connection::system().await?;
            let block_proxy = BlockProxy::builder(&connection)
                .path(block_path)?
                .build()
                .await?;
            size = block_proxy.size().await?;
        }

        Ok(DriveModel {
            name: path.to_owned(),
            path: path.to_string(),
            size,
            id: drive_proxy.id().await?,
            model: drive_proxy.model().await?,
            serial: drive_proxy.serial().await?,
            vendor: drive_proxy.vendor().await?,
            block_path: block_path.to_string(),
            is_loop: false,
            backing_file: None,
            volumes_flat: vec![],
            volumes: vec![],
            can_power_off: drive_proxy.can_power_off().await?,
            ejectable: drive_proxy.ejectable().await?,
            media_available: drive_proxy.media_available().await?,
            media_change_detected: drive_proxy.media_change_detected().await?,
            media_removable: drive_proxy.media_removable().await?,
            optical: drive_proxy.optical().await?,
            optical_blank: drive_proxy.optical_blank().await?,
            removable: drive_proxy.removable().await?,
            revision: drive_proxy.revision().await?,
            partition_table_type: None,
            gpt_usable_range: None,
            connection: Connection::system().await?,
        })
    }

    async fn loop_backing_file(
        connection: &Connection,
        block_path: &OwnedObjectPath,
    ) -> Option<String> {
        let proxy = match zbus::Proxy::new(
            connection,
            "org.freedesktop.UDisks2",
            block_path.as_str(),
            "org.freedesktop.UDisks2.Loop",
        )
        .await
        {
            Ok(p) => p,
            Err(_) => return None,
        };

        // UDisks2 commonly exposes BackingFile as ay (C-string bytes). Be tolerant.
        if let Ok(bytes) = proxy.get_property::<Vec<u8>>("BackingFile").await {
            let raw = bytes.split(|b| *b == 0).next().unwrap_or(&bytes);
            let s = String::from_utf8_lossy(raw).to_string();
            if !s.trim().is_empty() {
                return Some(s);
            }
        }

        if let Ok(s) = proxy.get_property::<String>("BackingFile").await
            && !s.trim().is_empty()
        {
            return Some(s);
        }

        None
    }

    pub async fn from_block_only(
        block_path: &OwnedObjectPath,
        block_proxy: &BlockProxy<'_>,
        backing_file: Option<String>,
    ) -> Result<Self> {
        let size = block_proxy.size().await?;

        Ok(DriveModel {
            name: block_path.to_string(),
            path: String::new(),
            size,
            id: String::new(),
            model: String::new(),
            serial: String::new(),
            vendor: String::new(),
            block_path: block_path.to_string(),
            is_loop: backing_file.is_some(),
            backing_file,
            volumes_flat: vec![],
            volumes: vec![],
            can_power_off: false,
            ejectable: false,
            media_available: true,
            media_change_detected: false,
            media_removable: false,
            optical: false,
            optical_blank: false,
            removable: false,
            revision: String::new(),
            partition_table_type: None,
            gpt_usable_range: None,
            connection: Connection::system().await?,
        })
    }

    async fn get_drive_paths(connection: &Connection) -> Result<Vec<DriveBlockPair>> {
        let manager_proxy = UDisks2ManagerProxy::new(connection).await?;
        let block_paths = manager_proxy.get_block_devices(HashMap::new()).await?;

        let mut drive_paths: Vec<DriveBlockPair> = vec![];

        for path in block_paths {
            let block_device = match BlockProxy::builder(connection).path(&path)?.build().await {
                Ok(d) => d,
                Err(e) => {
                    info!("Could not get block device: {}", e);
                    continue;
                }
            };

            // Drive nodes don't have a .Partition interface assigned.
            // If we can build a Partition proxy AND it has a table, it's a partition.
            if let Ok(partition_proxy) = PartitionProxy::builder(connection)
                .path(&path)?
                .build()
                .await
            {
                if partition_proxy.table().await.is_ok() {
                    continue;
                }
                // Otherwise, we've found a drive.
            } else {
                // If we can't build the proxy, treat it as a drive.
            }

            match block_device.drive().await {
                Ok(dp) if dp.as_str() != "/" => drive_paths.push(DriveBlockPair {
                    block_path: path,
                    drive_path: Some(dp),
                    backing_file: None,
                }),
                _ => {
                    // Loop devices have no associated Drive object; include them if they implement
                    // org.freedesktop.UDisks2.Loop.
                    let backing = Self::loop_backing_file(connection, &path).await;
                    if backing.is_some() {
                        drive_paths.push(DriveBlockPair {
                            block_path: path,
                            drive_path: None,
                            backing_file: backing,
                        });
                    }
                }
            }
        }

        Ok(drive_paths)
    }

    pub async fn get_drives() -> Result<Vec<DriveModel>> {
        let connection = Connection::system().await?;
        let client = Client::new_for_connection(connection.clone()).await?;
        let drive_paths = Self::get_drive_paths(&connection).await?;

        // Build a device-node → object-path lookup for nested volumes (LUKS cleartext, LVM LVs).
        let manager_proxy = UDisks2ManagerProxy::new(&connection).await?;
        let all_block_objects = manager_proxy.get_block_devices(HashMap::new()).await?;
        let block_index = BlockIndex::build(&connection, &all_block_objects).await?;

        async fn build_volume_nodes_for_drive(
            drive: &mut DriveModel,
            connection: &Connection,
            block_index: &BlockIndex,
        ) -> Result<()> {
            // Build nested volumes for UI presentation/actions.
            drive.volumes = Vec::with_capacity(drive.volumes_flat.len());
            for p in &drive.volumes_flat {
                let label = if p.name.is_empty() {
                    p.name()
                } else {
                    p.name.clone()
                };

                // LUKS: treat as a container; children are cleartext filesystem or LVM PV.
                let encrypted_probe =
                    udisks2::encrypted::EncryptedProxy::builder(connection).path(&p.path);

                let volume = if let Ok(builder) = encrypted_probe {
                    match builder.build().await {
                        Ok(_) => match VolumeNode::crypto_container_for_partition(
                            connection,
                            p.path.clone(),
                            label.clone(),
                            block_index,
                        )
                        .await
                        {
                            Ok(v) => v,
                            Err(e) if DriveModel::is_missing_encrypted_interface(&e) => {
                                if p.id_type == "LVM2_member" {
                                    VolumeNode::from_block_object(
                                        connection,
                                        p.path.clone(),
                                        label,
                                        VolumeKind::LvmPhysicalVolume,
                                        Some(block_index),
                                    )
                                    .await?
                                } else if p.has_filesystem {
                                    VolumeNode::from_block_object(
                                        connection,
                                        p.path.clone(),
                                        label,
                                        VolumeKind::Filesystem,
                                        Some(block_index),
                                    )
                                    .await?
                                } else {
                                    VolumeNode::from_block_object(
                                        connection,
                                        p.path.clone(),
                                        label,
                                        VolumeKind::Partition,
                                        Some(block_index),
                                    )
                                    .await?
                                }
                            }
                            Err(e) => return Err(e),
                        },
                        Err(_) => {
                            // Not actually encrypted; fall back below.
                            if p.id_type == "LVM2_member" {
                                VolumeNode::from_block_object(
                                    connection,
                                    p.path.clone(),
                                    label,
                                    VolumeKind::LvmPhysicalVolume,
                                    Some(block_index),
                                )
                                .await?
                            } else if p.has_filesystem {
                                VolumeNode::from_block_object(
                                    connection,
                                    p.path.clone(),
                                    label,
                                    VolumeKind::Filesystem,
                                    Some(block_index),
                                )
                                .await?
                            } else {
                                VolumeNode::from_block_object(
                                    connection,
                                    p.path.clone(),
                                    label,
                                    VolumeKind::Partition,
                                    Some(block_index),
                                )
                                .await?
                            }
                        }
                    }
                } else if p.id_type == "LVM2_member" {
                    VolumeNode::from_block_object(
                        connection,
                        p.path.clone(),
                        label,
                        VolumeKind::LvmPhysicalVolume,
                        Some(block_index),
                    )
                    .await?
                } else if p.has_filesystem {
                    VolumeNode::from_block_object(
                        connection,
                        p.path.clone(),
                        label,
                        VolumeKind::Filesystem,
                        Some(block_index),
                    )
                    .await?
                } else {
                    VolumeNode::from_block_object(
                        connection,
                        p.path.clone(),
                        label,
                        VolumeKind::Partition,
                        Some(block_index),
                    )
                    .await?
                };

                drive.volumes.push(volume);
            }

            Ok(())
        }

        async fn partitions_by_table(
            connection: &Connection,
            client: &Client,
            all_block_objects: &[OwnedObjectPath],
            table_path: OwnedObjectPath,
            drive_path_for_partition: String,
        ) -> Result<Vec<VolumeModel>> {
            let mut out: Vec<VolumeModel> = Vec::new();

            for obj in all_block_objects {
                let partition_proxy =
                    match PartitionProxy::builder(connection).path(obj)?.build().await {
                        Ok(p) => p,
                        Err(_) => continue,
                    };

                let table = match partition_proxy.table().await {
                    Ok(t) => t,
                    Err(_) => continue,
                };

                if table != table_path {
                    continue;
                }

                let block_proxy = match BlockProxy::builder(connection).path(obj)?.build().await {
                    Ok(p) => p,
                    Err(e) => {
                        warn!("Error getting partition block proxy for {}: {}", obj, e);
                        continue;
                    }
                };

                match VolumeModel::from_proxy(
                    client,
                    drive_path_for_partition.clone(),
                    obj.clone(),
                    &partition_proxy,
                    &block_proxy,
                )
                .await
                {
                    Ok(p) => out.push(p),
                    Err(e) => {
                        warn!("Error building partition model for {}: {}", obj, e);
                        continue;
                    }
                }
            }

            Ok(out)
        }

        let mut drives: HashMap<String, DriveModel> = HashMap::new();

        for pair in drive_paths {
            let mut drive = if let Some(drive_path) = &pair.drive_path {
                let drive_proxy = DriveProxy::builder(&connection)
                    .path(drive_path)?
                    .build()
                    .await?;

                match DriveModel::from_proxy(drive_path, &pair.block_path, &drive_proxy).await {
                    Ok(d) => d,
                    Err(e) => {
                        warn!("Could not get drive: {}", e);
                        continue;
                    }
                }
            } else {
                let block_proxy = BlockProxy::builder(&connection)
                    .path(&pair.block_path)?
                    .build()
                    .await?;

                match DriveModel::from_block_only(
                    &pair.block_path,
                    &block_proxy,
                    pair.backing_file.clone(),
                )
                .await
                {
                    Ok(d) => d,
                    Err(e) => {
                        warn!("Could not get loop device: {}", e);
                        continue;
                    }
                }
            };

            let drive_path_for_partition = pair
                .drive_path
                .as_ref()
                .map(|p| p.to_string())
                .unwrap_or_else(|| pair.block_path.to_string());

            let partition_table_proxy = match PartitionTableProxy::builder(&connection)
                .path(&pair.block_path)?
                .build()
                .await
            {
                Ok(p) => p,
                Err(e) => {
                    // Not all devices (notably loop-backed images) have a partition table.
                    // Treat this as "no partition table" instead of failing enumeration.
                    warn!("No partition table proxy for {}: {}", pair.block_path, e);

                    if drive.is_loop
                        && let Ok(parts) = partitions_by_table(
                            &connection,
                            &client,
                            &all_block_objects,
                            pair.block_path.clone(),
                            drive_path_for_partition.clone(),
                        )
                        .await
                    {
                        drive.volumes_flat = parts;
                        if drive.partition_table_type.is_none() {
                            drive.partition_table_type = drive
                                .volumes_flat
                                .iter()
                                .find(|p| !p.table_type.is_empty())
                                .map(|p| p.table_type.clone());
                        }
                    }

                    if drive.volumes_flat.is_empty() {
                        let drive_block_proxy = BlockProxy::builder(&connection)
                            .path(&pair.block_path)?
                            .build()
                            .await?;
                        if let Ok(v) = VolumeModel::filesystem_from_block(
                            &connection,
                            drive_path_for_partition.clone(),
                            pair.block_path.clone(),
                            &drive_block_proxy,
                        )
                        .await
                            && v.has_filesystem
                        {
                            drive.volumes_flat.push(v);
                        }
                    }

                    build_volume_nodes_for_drive(&mut drive, &connection, &block_index).await?;

                    drives.insert(drive.name.clone(), drive);
                    continue;
                }
            };

            drive.partition_table_type = match partition_table_proxy.type_().await {
                Ok(t) => Some(t),
                Err(e) => {
                    warn!(
                        "No partition table interface for {}: {}",
                        pair.block_path, e
                    );

                    if drive.is_loop
                        && let Ok(parts) = partitions_by_table(
                            &connection,
                            &client,
                            &all_block_objects,
                            pair.block_path.clone(),
                            drive_path_for_partition.clone(),
                        )
                        .await
                    {
                        drive.volumes_flat = parts;
                        if drive.partition_table_type.is_none() {
                            drive.partition_table_type = drive
                                .volumes_flat
                                .iter()
                                .find(|p| !p.table_type.is_empty())
                                .map(|p| p.table_type.clone());
                        }
                    }

                    if drive.volumes_flat.is_empty() {
                        let drive_block_proxy = BlockProxy::builder(&connection)
                            .path(&pair.block_path)?
                            .build()
                            .await?;
                        if let Ok(v) = VolumeModel::filesystem_from_block(
                            &connection,
                            drive_path_for_partition.clone(),
                            pair.block_path.clone(),
                            &drive_block_proxy,
                        )
                        .await
                            && v.has_filesystem
                        {
                            drive.volumes_flat.push(v);
                        }
                    }

                    build_volume_nodes_for_drive(&mut drive, &connection, &block_index).await?;

                    drives.insert(drive.name.clone(), drive);
                    continue;
                }
            };

            if drive.partition_table_type.as_deref() == Some("gpt") {
                let drive_block_proxy = BlockProxy::builder(&connection)
                    .path(&pair.block_path)?
                    .build()
                    .await?;

                match probe_gpt_usable_range_bytes(&drive_block_proxy, drive.size).await {
                    Ok(Some(range)) => {
                        drive.gpt_usable_range = Some(range);
                    }
                    Ok(None) => {
                        warn!(
                            "Could not parse GPT usable range for {}; falling back to conservative 1MiB bands",
                            pair.block_path
                        );
                        drive.gpt_usable_range = fallback_gpt_usable_range_bytes(drive.size);
                    }
                    Err(e) => {
                        warn!(
                            "Error probing GPT usable range for {}: {}; falling back to conservative 1MiB bands",
                            pair.block_path, e
                        );
                        drive.gpt_usable_range = fallback_gpt_usable_range_bytes(drive.size);
                    }
                }
            }

            let partition_paths = match partition_table_proxy.partitions().await {
                Ok(p) => p,
                Err(e) => {
                    warn!("No partitions for {}: {}", pair.block_path, e);
                    drives.insert(drive.name.clone(), drive);
                    continue;
                }
            };

            for partition_path in partition_paths {
                let partition_proxy = match PartitionProxy::builder(&connection)
                    .path(&partition_path)?
                    .build()
                    .await
                {
                    Ok(p) => p,
                    Err(e) => {
                        error!("Error getting partition info: {}", e);
                        continue;
                    }
                };

                let block_proxy = match BlockProxy::builder(&connection)
                    .path(&partition_path)?
                    .build()
                    .await
                {
                    Ok(p) => p,
                    Err(e) => {
                        warn!(
                            "Error getting partition block proxy for {}: {}",
                            partition_path, e
                        );
                        continue;
                    }
                };

                let drive_path_for_partition = pair
                    .drive_path
                    .as_ref()
                    .map(|p| p.to_string())
                    .unwrap_or_else(|| pair.block_path.to_string());

                match VolumeModel::from_proxy(
                    &client,
                    drive_path_for_partition,
                    partition_path.clone(),
                    &partition_proxy,
                    &block_proxy,
                )
                .await
                {
                    Ok(p) => drive.volumes_flat.push(p),
                    Err(e) => {
                        warn!(
                            "Error building partition model for {}: {}",
                            partition_path, e
                        );
                        continue;
                    }
                }
            }

            if drive.volumes_flat.is_empty() {
                let drive_block_proxy = BlockProxy::builder(&connection)
                    .path(&pair.block_path)?
                    .build()
                    .await?;
                if let Ok(v) = VolumeModel::filesystem_from_block(
                    &connection,
                    drive_path_for_partition.clone(),
                    pair.block_path.clone(),
                    &drive_block_proxy,
                )
                .await
                    && v.has_filesystem
                {
                    drive.volumes_flat.push(v);
                }
            }

            build_volume_nodes_for_drive(&mut drive, &connection, &block_index).await?;

            drives.insert(drive.name.clone(), drive);
        }

        //Order b
        let mut drives: Vec<DriveModel> = drives.into_values().collect();
        drives.sort_by(|d1, d2| {
            d1.removable.cmp(&d2.removable).then_with(|| {
                d2.block_path.cmp(&d1.block_path) //TODO: understand this. d1 SHOULD come first in this compare...
            })
        });

        Ok(drives)
    }

    pub fn name(&self) -> String {
        self.name.split("/").last().unwrap().replace("_", " ") //TODO: Handle unwrap
    }

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
        crate::open_for_backup(block_object_path).await
    }

    pub async fn open_for_restore(&self) -> Result<std::os::fd::OwnedFd> {
        let block_object_path: OwnedObjectPath = self.block_path.as_str().try_into()?;
        crate::open_for_restore(block_object_path).await
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

    pub async fn smart_info(&self) -> Result<SmartInfo> {
        match self.nvme_smart_info().await {
            Ok(info) => Ok(info),
            Err(e) if is_anyhow_not_supported(&e) => match self.ata_smart_info().await {
                Ok(info) => Ok(info),
                Err(e2) if is_anyhow_not_supported(&e2) => {
                    Err(anyhow::anyhow!("Not supported by this drive"))
                }
                Err(e2) => Err(e2),
            },
            Err(e) => Err(e),
        }
    }

    pub async fn smart_selftest_start(&self, kind: SmartSelfTestKind) -> Result<()> {
        match self.nvme_selftest_start(kind).await {
            Ok(()) => Ok(()),
            Err(e) if is_anyhow_not_supported(&e) => match self.ata_selftest_start(kind).await {
                Ok(()) => Ok(()),
                Err(e2) if is_anyhow_not_supported(&e2) => {
                    Err(anyhow::anyhow!("Not supported by this drive"))
                }
                Err(e2) => Err(e2),
            },
            Err(e) => Err(e),
        }
    }

    pub async fn smart_selftest_abort(&self) -> Result<()> {
        match self.nvme_selftest_abort().await {
            Ok(()) => Ok(()),
            Err(e) if is_anyhow_not_supported(&e) => match self.ata_selftest_abort().await {
                Ok(()) => Ok(()),
                Err(e2) if is_anyhow_not_supported(&e2) => {
                    Err(anyhow::anyhow!("Not supported by this drive"))
                }
                Err(e2) => Err(e2),
            },
            Err(e) => Err(e),
        }
    }

    async fn nvme_smart_info(&self) -> Result<SmartInfo> {
        let proxy = zbus::Proxy::new(
            &self.connection,
            "org.freedesktop.UDisks2",
            self.path.as_str(),
            "org.freedesktop.UDisks2.NVMe.Controller",
        )
        .await?;

        // If the interface isn't present on this drive, properties/methods will error.
        let _state: String = proxy.get_property("State").await?;

        let options: HashMap<&str, Value<'_>> = HashMap::new();
        let _: () = proxy.call("SmartUpdate", &(options)).await?;

        let updated_at: Option<u64> = proxy.get_property::<u64>("SmartUpdated").await.ok();
        let temp_k: Option<u16> = proxy.get_property::<u16>("SmartTemperature").await.ok();
        let power_on_hours: Option<u64> = proxy.get_property::<u64>("SmartPowerOnHours").await.ok();
        let selftest_status: Option<String> = proxy
            .get_property::<String>("SmartSelftestStatus")
            .await
            .ok();

        let attrs: HashMap<String, OwnedValue> = proxy
            .call("SmartGetAttributes", &(HashMap::<&str, Value<'_>>::new()))
            .await?;

        let mut attributes = std::collections::BTreeMap::new();
        for (k, v) in attrs {
            attributes.insert(k, format!("{v:?}"));
        }

        Ok(SmartInfo {
            device_type: "NVMe".to_string(),
            updated_at,
            temperature_c: temp_k.map(|k| (k as u64).saturating_sub(273)),
            power_on_hours,
            selftest_status,
            attributes,
        })
    }

    async fn nvme_selftest_start(&self, kind: SmartSelfTestKind) -> Result<()> {
        let proxy = zbus::Proxy::new(
            &self.connection,
            "org.freedesktop.UDisks2",
            self.path.as_str(),
            "org.freedesktop.UDisks2.NVMe.Controller",
        )
        .await?;

        let _state: String = proxy.get_property("State").await?;

        let options: HashMap<&str, Value<'_>> = HashMap::new();
        let _: () = proxy
            .call("SmartSelftestStart", &(kind.as_udisks_str(), options))
            .await?;
        Ok(())
    }

    async fn nvme_selftest_abort(&self) -> Result<()> {
        let proxy = zbus::Proxy::new(
            &self.connection,
            "org.freedesktop.UDisks2",
            self.path.as_str(),
            "org.freedesktop.UDisks2.NVMe.Controller",
        )
        .await?;

        let _state: String = proxy.get_property("State").await?;

        let options: HashMap<&str, Value<'_>> = HashMap::new();
        let _: () = proxy.call("SmartSelftestAbort", &(options)).await?;
        Ok(())
    }

    async fn ata_smart_info(&self) -> Result<SmartInfo> {
        let proxy = zbus::Proxy::new(
            &self.connection,
            "org.freedesktop.UDisks2",
            self.path.as_str(),
            "org.freedesktop.UDisks2.Drive.Ata",
        )
        .await?;

        // If the interface isn't present on this drive, this will error.
        let _smart_enabled: bool = proxy.get_property("SmartEnabled").await?;

        let options: HashMap<&str, Value<'_>> = HashMap::new();
        let _: () = proxy.call("SmartUpdate", &(options)).await?;

        let updated_at: Option<u64> = proxy.get_property::<u64>("SmartUpdated").await.ok();
        let temperature: Option<u64> = proxy.get_property::<u64>("SmartTemperature").await.ok();
        let power_on_seconds: Option<u64> =
            proxy.get_property::<u64>("SmartPowerOnSeconds").await.ok();
        let selftest_status: Option<String> = proxy
            .get_property::<String>("SmartSelftestStatus")
            .await
            .ok();

        let attrs: HashMap<String, OwnedValue> = proxy
            .call("SmartGetAttributes", &(HashMap::<&str, Value<'_>>::new()))
            .await?;

        let mut attributes = std::collections::BTreeMap::new();
        for (k, v) in attrs {
            attributes.insert(k, format!("{v:?}"));
        }

        Ok(SmartInfo {
            device_type: "ATA".to_string(),
            updated_at,
            temperature_c: temperature,
            power_on_hours: power_on_seconds.map(|s| s / 3600),
            selftest_status,
            attributes,
        })
    }

    async fn ata_selftest_start(&self, kind: SmartSelfTestKind) -> Result<()> {
        let proxy = zbus::Proxy::new(
            &self.connection,
            "org.freedesktop.UDisks2",
            self.path.as_str(),
            "org.freedesktop.UDisks2.Drive.Ata",
        )
        .await?;

        let _smart_enabled: bool = proxy.get_property("SmartEnabled").await?;

        let options: HashMap<&str, Value<'_>> = HashMap::new();
        let _: () = proxy
            .call("SmartSelftestStart", &(kind.as_udisks_str(), options))
            .await?;
        Ok(())
    }

    async fn ata_selftest_abort(&self) -> Result<()> {
        let proxy = zbus::Proxy::new(
            &self.connection,
            "org.freedesktop.UDisks2",
            self.path.as_str(),
            "org.freedesktop.UDisks2.Drive.Ata",
        )
        .await?;

        let _smart_enabled: bool = proxy.get_property("SmartEnabled").await?;

        let options: HashMap<&str, Value<'_>> = HashMap::new();
        let _: () = proxy.call("SmartSelftestAbort", &(options)).await?;
        Ok(())
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

#[cfg(test)]
mod tests {
    #[test]
    fn dos_table_type_is_supported_and_not_msdos() {
        assert!(crate::COMMON_DOS_TYPES[0].table_type == "dos");
    }

    #[test]
    fn gpt_table_type_is_supported() {
        assert!(crate::COMMON_GPT_TYPES[0].table_type == "gpt");
    }
}
