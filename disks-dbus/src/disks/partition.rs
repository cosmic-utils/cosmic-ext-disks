use super::DiskError;
use super::ops::{
    PartitionFormatArgs, RealDiskBackend, crypto_lock, crypto_unlock, partition_delete,
    partition_format, partition_mount, partition_unmount,
};
use crate::Usage;
use anyhow::Result;
use enumflags2::BitFlags;
use std::path::Path;
use udisks2::{
    Client,
    block::BlockProxy,
    filesystem::FilesystemProxy,
    partition::{PartitionFlags, PartitionProxy},
};
use zbus::{Connection, zvariant::OwnedObjectPath};

#[derive(Debug, Clone)]
pub struct PartitionModel {
    pub is_contained: bool,
    pub is_container: bool,
    pub table_path: OwnedObjectPath,
    pub name: String,
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

impl PartitionModel {
    fn decode_c_string_bytes(bytes: &[u8]) -> String {
        let raw = match bytes.split(|b| *b == 0).next() {
            Some(v) => v,
            None => bytes,
        };

        String::from_utf8_lossy(raw).to_string()
    }

    fn decode_mount_points(mount_points: Vec<Vec<u8>>) -> Vec<String> {
        mount_points
            .into_iter()
            .filter_map(|mp| {
                let decoded = Self::decode_c_string_bytes(&mp);
                if decoded.is_empty() {
                    None
                } else {
                    Some(decoded)
                }
            })
            .collect()
    }

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

        let preferred_device = Self::decode_c_string_bytes(&block_proxy.preferred_device().await?);
        let device = if preferred_device.is_empty() {
            Self::decode_c_string_bytes(&block_proxy.device().await?)
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
                Ok(mps) => (true, Self::decode_mount_points(mps)),
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

        let table_proxy = client.partition_table(partition_proxy).await?;
        let type_str = match client.partition_type_for_display(
            &table_proxy.type_().await?,
            &partition_proxy.type_().await?,
        ) {
            Some(val) => val
                .to_owned()
                .replace("part-type", "")
                .replace("\u{004}", ""),
            _ => partition_proxy.type_().await?,
        };

        Ok(Self {
            is_contained: partition_proxy.is_contained().await?,
            is_container: partition_proxy.is_container().await?,
            table_path: partition_proxy.table().await?,
            name: partition_proxy.name().await?,
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
            table_type: table_proxy.type_().await?,
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

        println!("{type_str}");

        Ok(type_str)
    }

    pub fn name(&self) -> String {
        format!("Partition {}", &self.number)
    }

    pub async fn connect(&mut self) -> Result<()> {
        if self.connection.is_none() {
            self.connection = Some(Connection::system().await?);
        }

        Ok(())
    }

    pub async fn mount(&self) -> Result<()> {
        if self.connection.is_none() {
            return Err(DiskError::NotConnected(self.name.clone()).into());
        }

        let backend = RealDiskBackend::new(self.connection.as_ref().unwrap().clone());
        partition_mount(&backend, self.path.clone()).await
    }

    pub async fn unmount(&self) -> Result<()> {
        if self.connection.is_none() {
            return Err(DiskError::NotConnected(self.name.clone()).into());
        }

        let backend = RealDiskBackend::new(self.connection.as_ref().unwrap().clone());
        partition_unmount(&backend, self.path.clone()).await
    }

    pub async fn unlock(&self, passphrase: &str) -> Result<OwnedObjectPath> {
        if self.connection.is_none() {
            return Err(DiskError::NotConnected(self.name.clone()).into());
        }

        let backend = RealDiskBackend::new(self.connection.as_ref().unwrap().clone());
        crypto_unlock(&backend, self.path.clone(), passphrase).await
    }

    pub async fn lock(&self) -> Result<()> {
        if self.connection.is_none() {
            return Err(DiskError::NotConnected(self.name.clone()).into());
        }

        let backend = RealDiskBackend::new(self.connection.as_ref().unwrap().clone());
        crypto_lock(&backend, self.path.clone()).await
    }

    pub async fn delete(&self) -> Result<()> {
        if self.connection.is_none() {
            return Err(DiskError::NotConnected(self.name.clone()).into());
        }

        //try to unmount first. If it fails, it's likely because it's already unmounted.
        //any other error with the partition should be caught by the delete operation.
        let _ = self.unmount().await;

        let backend = RealDiskBackend::new(self.connection.as_ref().unwrap().clone());
        partition_delete(&backend, self.path.clone()).await
    }

    pub async fn format(&self, name: String, erase: bool, partion_type: String) -> Result<()> {
        if self.connection.is_none() {
            return Err(DiskError::NotConnected(self.name.clone()).into());
        }

        let backend = RealDiskBackend::new(self.connection.as_ref().unwrap().clone());

        let label = if name.is_empty() { None } else { Some(name) };
        let args = PartitionFormatArgs {
            block_path: self.path.clone(),
            filesystem_type: partion_type,
            erase,
            label,
        };

        partition_format(&backend, args).await
    }

    //TODO: implement
    pub async fn edit_partition(
        &self,
        _partition_type: String,
        _name: String,
        _flags: u64,
    ) -> Result<()> {
        if self.connection.is_none() {
            return Err(DiskError::NotConnected(self.name.clone()).into());
        }

        Ok(())
    }

    //TODO: implement
    pub async fn edit_filesystem_label(&self, _label: String) -> Result<()> {
        if self.connection.is_none() {
            return Err(DiskError::NotConnected(self.name.clone()).into());
        }

        Ok(())
    }

    //TODO: implement
    pub async fn change_passphrase(&self) -> Result<()> {
        if self.connection.is_none() {
            return Err(DiskError::NotConnected(self.name.clone()).into());
        }
        Ok(())
    }

    //TODO: implement
    pub async fn resize(&self, _new_size_bytes: u64) -> Result<()> {
        if self.connection.is_none() {
            return Err(DiskError::NotConnected(self.name.clone()).into());
        }
        Ok(())
    }

    //TODO: implement
    pub async fn check_filesystem(&self) -> Result<()> {
        if self.connection.is_none() {
            return Err(DiskError::NotConnected(self.name.clone()).into());
        }
        Ok(())
    }

    //TODO: implement
    pub async fn repair_filesystem(&self) -> Result<()> {
        if self.connection.is_none() {
            return Err(DiskError::NotConnected(self.name.clone()).into());
        }
        Ok(())
    }

    //TODO: implement
    pub async fn take_ownership(&self, _recursive: bool) -> Result<()> {
        if self.connection.is_none() {
            return Err(DiskError::NotConnected(self.name.clone()).into());
        }
        Ok(())
    }

    //TODO: implement. See how edit mount options -> User session defaults works in gnome-disks.
    pub async fn default_mount_options(&self) -> Result<()> {
        if self.connection.is_none() {
            return Err(DiskError::NotConnected(self.name.clone()).into());
        }
        Ok(())
    }

    //TODO: implement. Look at gnome-disks -> partition -> edit mount options. Likely make all params optional.
    #[allow(clippy::too_many_arguments)]
    pub async fn edit_mount_options(
        &self,
        _mount_at_startup: bool,
        _show_in_ui: bool,
        _requre_auth: bool,
        _display_name: Option<String>,
        _icon_name: Option<String>,
        _symbolic_icon_name: Option<String>,
        _options: String,
        _mount_point: String,
        _identify_as: String,
        _file_system_type: String,
    ) -> Result<()> {
        if self.connection.is_none() {
            return Err(DiskError::NotConnected(self.name.clone()).into());
        }
        Ok(())
    }

    //TODO: implement
    pub async fn edit_encrytion_options(&self) -> Result<()> {
        if self.connection.is_none() {
            return Err(DiskError::NotConnected(self.name.clone()).into());
        }
        Ok(())
    }

    //TODO: implement. creates a *.img of self.
    pub async fn create_image(&self, _output_path: String) -> Result<()> {
        if self.connection.is_none() {
            return Err(DiskError::NotConnected(self.name.clone()).into());
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::PartitionModel;

    #[test]
    fn decode_c_string_bytes_truncates_nul() {
        let bytes = b"/run/media/user/DISK\0garbage";
        assert_eq!(
            PartitionModel::decode_c_string_bytes(bytes),
            "/run/media/user/DISK"
        );
    }

    #[test]
    fn decode_mount_points_filters_empty_entries() {
        let decoded = PartitionModel::decode_mount_points(vec![
            b"/mnt/a\0".to_vec(),
            b"\0".to_vec(),
            Vec::new(),
            b"/mnt/b".to_vec(),
        ]);

        assert_eq!(decoded, vec!["/mnt/a".to_string(), "/mnt/b".to_string()]);
    }

    #[test]
    fn can_mount_tracks_filesystem_interface() {
        let mut p = PartitionModel {
            is_contained: false,
            is_container: false,
            table_path: "/".try_into().unwrap(),
            name: String::new(),
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
