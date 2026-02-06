mod backup;
mod config;
mod encryption;
mod filesystem;
mod mount;
mod partition;

use crate::Usage;
use crate::dbus::bytestring as bs;
use anyhow::Result;
use enumflags2::BitFlags;
use std::path::Path;
use udisks2::partitiontable::PartitionTableProxy;
use udisks2::{
    Client,
    block::BlockProxy,
    filesystem::FilesystemProxy,
    partition::{PartitionFlags, PartitionProxy},
};
use zbus::{Connection, zvariant::OwnedObjectPath};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VolumeType {
    Container,
    Partition,
    Filesystem,
}

#[derive(Debug, Clone)]
pub struct VolumeModel {
    pub volume_type: VolumeType,
    pub table_path: OwnedObjectPath,
    pub name: String,
    pub partition_type_id: String,
    pub partition_type: String,
    pub id_type: String,
    pub uuid: String,
    pub number: u32,
    pub flags: BitFlags<PartitionFlags>,
    pub offset: u64,
    pub size: u64,
    pub path: OwnedObjectPath,
    pub device_path: Option<String>,
    pub has_filesystem: bool,
    pub mount_points: Vec<String>,
    pub usage: Option<Usage>,
    connection: Option<Connection>,
    pub drive_path: String,
    pub table_type: String,
}

impl VolumeModel {
    pub fn is_mounted(&self) -> bool {
        self.has_filesystem && !self.mount_points.is_empty()
    }

    pub fn can_mount(&self) -> bool {
        self.has_filesystem
    }

    pub async fn from_proxy(
        client: &Client,
        drive_path: String,
        partition_path: OwnedObjectPath,
        partition_proxy: &PartitionProxy<'_>,
        block_proxy: &BlockProxy<'_>,
    ) -> Result<Self> {
        let connection = Connection::system().await?;

        let preferred_device = bs::decode_c_string_bytes(&block_proxy.preferred_device().await?);
        let device = if preferred_device.is_empty() {
            bs::decode_c_string_bytes(&block_proxy.device().await?)
        } else {
            preferred_device
        };

        let mut device_path = if device.is_empty() {
            None
        } else {
            Some(device)
        };
        if device_path.is_none() {
            let proposed = format!("/dev/{}", partition_path.split("/").last().unwrap());
            if Path::new(&proposed).exists() {
                device_path = Some(proposed);
            }
        }

        let (has_filesystem, mount_points) = match FilesystemProxy::builder(&connection)
            .path(&partition_path)?
            .build()
            .await
        {
            Ok(proxy) => match proxy.mount_points().await {
                Ok(mps) => (true, bs::decode_mount_points(mps)),
                Err(_) => (false, Vec::new()),
            },
            Err(_) => (false, Vec::new()),
        };

        let usage = match mount_points.first() {
            Some(mount_point) => {
                crate::usage_for_mount_point(mount_point, device_path.as_deref()).ok()
            }
            None => None,
        };

        let table_path = partition_proxy.table().await?;

        // Not all table objects actually expose org.freedesktop.UDisks2.PartitionTable
        // (notably for some loop-backed devices). Treat missing interface as "unknown".
        let table_type = match PartitionTableProxy::builder(&connection)
            .path(&table_path)?
            .build()
            .await
        {
            Ok(proxy) => proxy.type_().await.unwrap_or_default(),
            Err(_) => String::new(),
        };

        let partition_type_id = partition_proxy.type_().await?;

        let type_str = if table_type.is_empty() {
            partition_type_id.clone()
        } else {
            match client.partition_type_for_display(&table_type, &partition_type_id) {
                Some(val) => val
                    .to_owned()
                    .replace("part-type", "")
                    .replace("\u{004}", ""),
                _ => partition_type_id.clone(),
            }
        };

        let volume_type = if partition_proxy.is_container().await? {
            VolumeType::Container
        } else {
            VolumeType::Partition
        };

        Ok(Self {
            volume_type,
            table_path,
            name: partition_proxy.name().await?,
            partition_type_id,
            partition_type: type_str,
            id_type: block_proxy.id_type().await?,
            uuid: partition_proxy.uuid().await?,
            number: partition_proxy.number().await?,
            flags: partition_proxy.flags().await?,
            offset: partition_proxy.offset().await?,
            size: partition_proxy.size().await?,
            path: partition_path.clone(),
            device_path,
            has_filesystem,
            mount_points,
            usage,
            connection: Some(connection),
            drive_path,
            table_type,
        })
    }

    pub async fn filesystem_from_block(
        connection: &Connection,
        drive_path: String,
        block_object_path: OwnedObjectPath,
        block_proxy: &BlockProxy<'_>,
    ) -> Result<Self> {
        let preferred_device = bs::decode_c_string_bytes(&block_proxy.preferred_device().await?);
        let device = if preferred_device.is_empty() {
            bs::decode_c_string_bytes(&block_proxy.device().await?)
        } else {
            preferred_device
        };

        let mut device_path = if device.is_empty() {
            None
        } else {
            Some(device)
        };
        if device_path.is_none()
            && let Some(last) = block_object_path.split('/').next_back()
        {
            let proposed = format!("/dev/{}", last);
            if Path::new(&proposed).exists() {
                device_path = Some(proposed);
            }
        }

        let (has_filesystem, mount_points) = match FilesystemProxy::builder(connection)
            .path(&block_object_path)?
            .build()
            .await
        {
            Ok(proxy) => match proxy.mount_points().await {
                Ok(mps) => (true, bs::decode_mount_points(mps)),
                Err(_) => (false, Vec::new()),
            },
            Err(_) => (false, Vec::new()),
        };

        let usage = match mount_points.first() {
            Some(mount_point) => {
                crate::usage_for_mount_point(mount_point, device_path.as_deref()).ok()
            }
            None => None,
        };

        let uuid: String = (block_proxy.id_uuid().await).unwrap_or_default();

        Ok(Self {
            volume_type: VolumeType::Filesystem,
            table_path: "/".try_into().unwrap(),
            name: String::new(),
            partition_type_id: String::new(),
            partition_type: "Filesystem".to_string(),
            id_type: block_proxy.id_type().await?,
            uuid,
            number: 0,
            flags: Default::default(),
            offset: 0,
            size: block_proxy.size().await?,
            path: block_object_path.clone(),
            device_path,
            has_filesystem,
            mount_points,
            usage,
            connection: Some(connection.clone()),
            drive_path,
            table_type: String::new(),
        })
    }

    /// Returns informating about the given partition that is suitable for presentation in an user
    /// interface in a single line of text.
    ///
    /// The returned string is localized and includes things like the partition type, flags (if
    /// any) and name (if any).
    ///
    /// # Errors
    /// Returns an errors if it fails to read any of the aforementioned information.
    pub async fn partition_info(client: &Client, partition: &PartitionProxy<'_>) -> Result<String> {
        let _flags = partition.flags().await?;
        let table = client.partition_table(partition).await?;
        let _flags_str = String::new();

        let type_str = match client
            .partition_type_for_display(&table.type_().await?, &partition.type_().await?)
        {
            Some(val) => val.to_owned(),
            _ => partition.type_().await?,
        };

        Ok(type_str)
    }

    pub fn name(&self) -> String {
        if self.number > 0 {
            format!("Partition {}", &self.number)
        } else {
            "Filesystem".to_string()
        }
    }

    pub async fn connect(&mut self) -> Result<()> {
        if self.connection.is_none() {
            self.connection = Some(Connection::system().await?);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::VolumeModel;
    use crate::dbus::bytestring;

    #[test]
    fn decode_c_string_bytes_truncates_nul() {
        let bytes = b"/run/media/user/DISK\0garbage";
        assert_eq!(
            bytestring::decode_c_string_bytes(bytes),
            "/run/media/user/DISK"
        );
    }

    #[test]
    fn decode_mount_points_filters_empty_entries() {
        let decoded = bytestring::decode_mount_points(vec![
            b"/mnt/a\0".to_vec(),
            b"\0".to_vec(),
            Vec::new(),
            b"/mnt/b".to_vec(),
        ]);

        assert_eq!(decoded, vec!["/mnt/a".to_string(), "/mnt/b".to_string()]);
    }

    #[test]
    fn can_mount_tracks_filesystem_interface() {
        let mut p = VolumeModel {
            volume_type: super::VolumeType::Partition,
            table_path: "/".try_into().unwrap(),
            name: String::new(),
            partition_type_id: String::new(),
            partition_type: String::new(),
            id_type: String::new(),
            uuid: String::new(),
            number: 1,
            flags: Default::default(),
            offset: 0,
            size: 0,
            path: "/".try_into().unwrap(),
            device_path: None,
            has_filesystem: false,
            mount_points: Vec::new(),
            usage: None,
            connection: None,
            drive_path: String::new(),
            table_type: String::new(),
        };

        assert!(!p.can_mount());
        assert!(!p.is_mounted());

        p.has_filesystem = true;
        assert!(p.can_mount());
        assert!(!p.is_mounted());

        p.mount_points = vec!["/mnt/a".to_string()];
        assert!(p.is_mounted());
    }
}
