use std::collections::HashMap;

use anyhow::Result;
use udisks2::{
    Client, block::BlockProxy, drive::DriveProxy, partition::PartitionProxy,
    partitiontable::PartitionTableProxy,
};
use zbus::{Connection, zvariant::OwnedObjectPath};

use crate::disks::{
    BlockIndex, VolumeModel, fallback_gpt_usable_range_bytes, manager::UDisks2ManagerProxy,
    probe_gpt_usable_range_bytes,
};
use super::model::DriveModel;

#[derive(Debug, Clone)]
struct DriveBlockPair {
    block_path: OwnedObjectPath,
    drive_path: Option<OwnedObjectPath>,
    backing_file: Option<String>,
}

impl DriveModel {
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

    async fn get_drive_paths(connection: &Connection) -> Result<Vec<DriveBlockPair>> {
        let manager_proxy = UDisks2ManagerProxy::new(connection).await?;
        let block_paths = manager_proxy.get_block_devices(HashMap::new()).await?;

        let mut drive_paths: Vec<DriveBlockPair> = vec![];

        for path in block_paths {
            let block_device = match BlockProxy::builder(connection).path(&path)?.build().await {
                Ok(d) => d,
                Err(e) => {
                    tracing::info!("Could not get block device: {}", e);
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
                        tracing::warn!("Error getting partition block proxy for {}: {}", obj, e);
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
                        tracing::warn!("Error building partition model for {}: {}", obj, e);
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
                        tracing::warn!("Could not get drive: {}", e);
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
                        tracing::warn!("Could not get loop device: {}", e);
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
                    tracing::warn!("No partition table proxy for {}: {}", pair.block_path, e);

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

                    drive
                        .build_volume_nodes_for_drive(&connection, &block_index)
                        .await?;

                    drives.insert(drive.name.clone(), drive);
                    continue;
                }
            };

            drive.partition_table_type = match partition_table_proxy.type_().await {
                Ok(t) => Some(t),
                Err(e) => {
                    tracing::warn!(
                        "No partition table interface for {}: {}",
                        pair.block_path,
                        e
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

                    drive
                        .build_volume_nodes_for_drive(&connection, &block_index)
                        .await?;

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
                        tracing::warn!(
                            "Could not parse GPT usable range for {}; falling back to conservative 1MiB bands",
                            pair.block_path
                        );
                        drive.gpt_usable_range = fallback_gpt_usable_range_bytes(drive.size);
                    }
                    Err(e) => {
                        tracing::warn!(
                            "Error probing GPT usable range for {}: {}; falling back to conservative 1MiB bands",
                            pair.block_path,
                            e
                        );
                        drive.gpt_usable_range = fallback_gpt_usable_range_bytes(drive.size);
                    }
                }
            }

            let partition_paths = match partition_table_proxy.partitions().await {
                Ok(p) => p,
                Err(e) => {
                    tracing::warn!("No partitions for {}: {}", pair.block_path, e);
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
                        tracing::error!("Error getting partition info: {}", e);
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
                        tracing::warn!(
                            "Error getting partition block proxy for {}: {}",
                            partition_path,
                            e
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
                        tracing::warn!(
                            "Error building partition model for {}: {}",
                            partition_path,
                            e
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

            drive
                .build_volume_nodes_for_drive(&connection, &block_index)
                .await?;

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
}
