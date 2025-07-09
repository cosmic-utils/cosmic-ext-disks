use super::DiskError;
use crate::Usage;
use anyhow::Result;
use enumflags2::{BitFlags, bitflags};
use std::{collections::HashMap, path::Path};
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
    pub usage: Option<Usage>,
    connection: Option<Connection>,
    pub drive_path: String,
    pub table_type: String,
}

impl PartitionModel {
    pub async fn from_proxy(
        client: &Client,
        drive_path: String,
        partition_path: OwnedObjectPath,
        usage: Option<Usage>,
        partition_proxy: &PartitionProxy<'_>,
        block_proxy: &BlockProxy<'_>,
    ) -> Result<Self> {
        let device_path = match &usage {
            Some(usage) => Some(usage.filesystem.clone()),
            None => {
                let proposed = format!("/dev/{}", partition_path.split("/").last().unwrap());

                match Path::new(&proposed).exists() {
                    true => Some(proposed),
                    false => None,
                }
            }
        };

        let table_proxy = client.partition_table(&partition_proxy).await?;
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
            device_path: device_path,
            usage,
            connection: Some(Connection::system().await?),
            drive_path: drive_path,
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
        let flags = partition.flags().await?;
        let table = client.partition_table(partition).await?;
        let mut flags_str = String::new();

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

        let proxy = FilesystemProxy::builder(&self.connection.as_ref().unwrap())
            .path(&self.path)?
            .build()
            .await?;

        proxy.mount(HashMap::new()).await?;

        Ok(())
    }

    pub async fn unmount(&self) -> Result<()> {
        if self.connection.is_none() {
            return Err(DiskError::NotConnected(self.name.clone()).into());
        }

        let proxy = FilesystemProxy::builder(&self.connection.as_ref().unwrap())
            .path(&self.path)?
            .build()
            .await?;

        proxy.unmount(HashMap::new()).await?;

        Ok(())
    }

    pub async fn delete(&self) -> Result<()> {
        if self.connection.is_none() {
            return Err(DiskError::NotConnected(self.name.clone()).into());
        }

        //try to unmount first. If it fails, it's likely because it's already unmounted.
        //any other error with the partition should be caught by the delete operation.
        let _ = self.unmount().await;

        let proxy = PartitionProxy::builder(&self.connection.as_ref().unwrap())
            .path(&self.path)?
            .build()
            .await?;

        proxy.delete(HashMap::new()).await?;

        Ok(())
    }

    pub async fn format(&self, name: String, erase: bool, partion_type: String) -> Result<()> {
        if self.connection.is_none() {
            return Err(DiskError::NotConnected(self.name.clone()).into());
        }

        Ok(())
    }

    //TODO: implement
    pub async fn edit_partition(
        &self,
        partition_type: String,
        name: String,
        flags: u64,
    ) -> Result<()> {
        if self.connection.is_none() {
            return Err(DiskError::NotConnected(self.name.clone()).into());
        }

        Ok(())
    }

    //TODO: implement
    pub async fn edit_filesystem_label(&self, label: String) -> Result<()> {
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
    pub async fn resize(&self, new_size_bytes: u64) -> Result<()> {
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
    pub async fn take_ownership(&self, recursive: bool) -> Result<()> {
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
    pub async fn edit_mount_options(
        &self,
        mount_at_startup: bool,
        show_in_ui: bool,
        requre_auth: bool,
        display_name: Option<String>,
        icon_name: Option<String>,
        symbolic_icon_name: Option<String>,
        options: String,
        mount_point: String,
        identify_as: String,
        file_system_type: String,
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
    pub async fn create_image(&self, output_path: String) -> Result<()> {
        if self.connection.is_none() {
            return Err(DiskError::NotConnected(self.name.clone()).into());
        }
        Ok(())
    }
}
