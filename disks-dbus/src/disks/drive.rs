use std::collections::HashMap;

use anyhow::Context;
use anyhow::Result;
use tracing::{error, info, warn};
use udisks2::{
    Client, block::BlockProxy, drive::DriveProxy, partition::PartitionProxy,
    partitiontable::PartitionTableProxy,
};
use zbus::{
    Connection,
    zvariant::{OwnedObjectPath, Value},
};

use crate::{COMMON_DOS_TYPES, COMMON_GPT_TYPES, CreatePartitionInfo, get_usage_data};

use super::{
    ByteRange, PartitionModel, fallback_gpt_usable_range_bytes, manager::UDisks2ManagerProxy,
    probe_gpt_usable_range_bytes,
};

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
    pub partitions: Vec<PartitionModel>,
    pub path: String,
    pub partition_table_type: Option<String>,
    pub gpt_usable_range: Option<ByteRange>,
    connection: Connection,
}

#[derive(Debug, Clone)]
struct DriveBlockPair {
    block_path: OwnedObjectPath,
    drive_path: OwnedObjectPath,
}

impl DriveModel {
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
            partitions: vec![],
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

            //Drive nodes don't have a .Partition interface assigned.
            match PartitionProxy::builder(connection)
                .path(&path)?
                .build()
                .await
            {
                Ok(e) => match e.table().await {
                    Ok(_) => {
                        continue;
                    }
                    Err(_) => {} //We've found a drive
                },
                Err(_) => {} //We've found a drive
            };

            match block_device.drive().await {
                Ok(dp) => drive_paths.push(DriveBlockPair {
                    block_path: path,
                    drive_path: dp,
                }),
                Err(_) => continue,
            }
        }

        Ok(drive_paths)
    }

    pub async fn get_drives() -> Result<Vec<DriveModel>> {
        let connection = Connection::system().await?;
        let client = Client::new_for_connection(Connection::system().await?).await?;
        let drive_paths = Self::get_drive_paths(&connection).await?;

        let mut drives: HashMap<String, DriveModel> = HashMap::new();
        let mut usage_data = get_usage_data()?;

        for pair in drive_paths {
            let drive_proxy = DriveProxy::builder(&connection)
                .path(&pair.drive_path)?
                .build()
                .await?;
            let mut drive = match DriveModel::from_proxy(
                &pair.drive_path,
                &pair.block_path,
                &drive_proxy,
            )
            .await
            {
                Ok(d) => d,
                Err(e) => {
                    warn!("Could not get drive: {}", e);
                    continue;
                }
            };

            let partition_table_proxy = match PartitionTableProxy::builder(&connection)
                .path(&pair.block_path)?
                .build()
                .await
            {
                Ok(p) => p,
                Err(e) => {
                    error!("Error getting partition table: {}", e);
                    drives.insert(drive.name.clone(), drive);
                    continue;
                }
            };

            drive.partition_table_type = Some(partition_table_proxy.type_().await?);

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
                    error!("Error getting partitions for {}: {}", pair.block_path, e);
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

                let short_name = partition_path.as_str().split("/").last();

                let usage = match short_name {
                    Some(sn) => usage_data
                        .iter_mut()
                        .find(|u| u.filesystem.ends_with(sn))
                        .map(|u| u.clone()),
                    None => None,
                };

                let block_proxy = BlockProxy::builder(&connection)
                    .path(&partition_path)?
                    .build()
                    .await?;

                drive.partitions.push(
                    PartitionModel::from_proxy(
                        &client,
                        pair.drive_path.to_string(),
                        partition_path.clone(),
                        usage,
                        &partition_proxy,
                        &block_proxy,
                    )
                    .await?,
                );
            }

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
        let proxy = DriveProxy::builder(&self.connection)
            .path(self.path.clone())?
            .build()
            .await?;
        proxy.eject(HashMap::new()).await?;
        Ok(())
    }

    pub async fn power_off(&self) -> Result<()> {
        let proxy = DriveProxy::builder(&self.connection)
            .path(self.path.clone())?
            .build()
            .await?;
        proxy.power_off(HashMap::new()).await?;
        Ok(())
    }

    pub async fn create_partition(&self, info: CreatePartitionInfo) -> Result<()> {
        let partition_table_proxy = PartitionTableProxy::builder(&self.connection)
            .path(self.block_path.clone())?
            .build()
            .await?;

        // Get the current partition table type
        let table_type = self
            .partition_table_type
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("No partition table type available"))?;

        // UDisks2 expects bytes. When the user requests the maximum size for a free-space segment,
        // pass 0 to let the backend pick the maximal size after alignment/geometry constraints.
        let requested_size = if info.size >= info.max_size {
            0
        } else {
            info.size
        };

        // DOS/MBR typically reserves the beginning of the disk (MBR + alignment). Avoid targeting
        // offset 0.
        const DOS_RESERVED_START_BYTES: u64 = 1024 * 1024;
        if table_type == "dos" && info.offset < DOS_RESERVED_START_BYTES {
            return Err(anyhow::anyhow!(
                "Requested offset {} is inside reserved DOS/MBR start region (< {} bytes)",
                info.offset,
                DOS_RESERVED_START_BYTES
            ));
        }

        if table_type == "gpt" {
            if let Some(range) = self.gpt_usable_range {
                if info.offset < range.start || info.offset >= range.end {
                    return Err(anyhow::anyhow!(
                        "Requested partition offset {} is outside GPT usable range [{}, {})",
                        info.offset,
                        range.start,
                        range.end
                    ));
                }

                if requested_size != 0 {
                    let requested_end = info.offset.saturating_add(requested_size);
                    if requested_end > range.end {
                        return Err(anyhow::anyhow!(
                            "Requested partition range [{}, {}) is outside GPT usable range [{}, {})",
                            info.offset,
                            requested_end,
                            range.start,
                            range.end
                        ));
                    }
                }
            }
        }

        // Find a partition type that matches the table type.
        // Note: UDisks2 reports DOS/MBR partition tables as "dos".
        let partition_info = common_partition_info_for(table_type, info.selected_partitition_type)?;

        // Verify the selected partition type is compatible with the table type
        if partition_info.table_type != table_type {
            return Err(anyhow::anyhow!(
                "Partition type '{}' is not compatible with partition table type '{}'",
                partition_info.name,
                table_type
            ));
        }

        let partition_type = partition_info.ty;

        // DOS/MBR partition tables do not support per-partition names. Use filesystem label
        // (format option) instead.
        let create_name = if table_type == "dos" {
            ""
        } else {
            info.name.as_str()
        };

        // Partition creation options.
        let mut create_options: HashMap<&str, Value<'_>> = HashMap::new();
        if table_type == "dos" {
            // UDisks2 expects this option (for DOS/MBR tables) to control primary/extended/logical.
            // Default to primary until the UI supports selecting otherwise.
            create_options.insert("partition-type", Value::from("primary"));
        }

        // Format options.
        let mut format_options: HashMap<&str, Value<'_>> = HashMap::new();
        if info.erase {
            format_options.insert("erase", Value::from("zero"));
        }
        if !info.name.is_empty() {
            format_options.insert("label", Value::from(info.name.clone()));
        }

        // Use the combined call so we format the returned object and avoid races relying on
        // PartitionTable.Partitions ordering.
        let _created_partition = partition_table_proxy
            .create_partition_and_format(
                info.offset,
                requested_size,
                partition_type,
                create_name,
                create_options,
                partition_info.filesystem_type,
                format_options,
            )
            .await
            .with_context(|| {
                format!(
                    "UDisks2 CreatePartitionAndFormat failed (table_type={table_type}, offset={}, size={}, part_type={}, fs={})",
                    info.offset,
                    requested_size,
                    partition_type,
                    partition_info.filesystem_type
                )
            })?;

        Ok(())
    }
}

fn common_partition_info_for(
    table_type: &str,
    selected_partition_type: usize,
) -> Result<&'static crate::PartitionTypeInfo> {
    match table_type {
        "gpt" => COMMON_GPT_TYPES
            .get(selected_partition_type)
            .ok_or_else(|| anyhow::anyhow!("Invalid partition type index for GPT")),
        "dos" => COMMON_DOS_TYPES
            .get(selected_partition_type)
            .ok_or_else(|| anyhow::anyhow!("Invalid partition type index for DOS/MBR")),
        _ => Err(anyhow::anyhow!(
            "Unsupported partition table type: {table_type}"
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dos_table_type_is_supported_and_not_msdos() {
        let info = common_partition_info_for("dos", 0).expect("dos should be supported");
        assert_eq!(info.table_type, "dos");

        assert!(common_partition_info_for("msdos", 0).is_err());
    }

    #[test]
    fn gpt_table_type_is_supported() {
        let info = common_partition_info_for("gpt", 0).expect("gpt should be supported");
        assert_eq!(info.table_type, "gpt");
    }
}
