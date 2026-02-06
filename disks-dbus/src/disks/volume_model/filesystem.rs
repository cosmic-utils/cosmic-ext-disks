use super::VolumeModel;
use crate::disks::ops::{PartitionFormatArgs, RealDiskBackend, partition_format};
use anyhow::Result;
use udisks2::filesystem::FilesystemProxy;

impl VolumeModel {
    pub async fn format(&self, name: String, erase: bool, partion_type: String) -> Result<()> {
        if self.connection.is_none() {
            return Err(crate::disks::DiskError::NotConnected(self.name.clone()).into());
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

    pub async fn edit_filesystem_label(&self, label: String) -> Result<()> {
        if self.connection.is_none() {
            return Err(crate::disks::DiskError::NotConnected(self.name.clone()).into());
        }

        let proxy = FilesystemProxy::builder(self.connection.as_ref().unwrap())
            .path(&self.path)?
            .build()
            .await?;

        let options: std::collections::HashMap<&str, zbus::zvariant::Value<'_>> =
            std::collections::HashMap::new();
        proxy.set_label(&label, options).await?;
        Ok(())
    }

    pub async fn check_filesystem(&self) -> Result<()> {
        if self.connection.is_none() {
            return Err(crate::disks::DiskError::NotConnected(self.name.clone()).into());
        }

        let proxy = FilesystemProxy::builder(self.connection.as_ref().unwrap())
            .path(&self.path)?
            .build()
            .await?;

        let options: std::collections::HashMap<&str, zbus::zvariant::Value<'_>> =
            std::collections::HashMap::new();
        let ok = proxy.check(options).await?;
        if ok {
            Ok(())
        } else {
            Err(anyhow::anyhow!(
                "Filesystem check completed but reported problems"
            ))
        }
    }

    pub async fn repair_filesystem(&self) -> Result<()> {
        if self.connection.is_none() {
            return Err(crate::disks::DiskError::NotConnected(self.name.clone()).into());
        }

        let proxy = FilesystemProxy::builder(self.connection.as_ref().unwrap())
            .path(&self.path)?
            .build()
            .await?;

        let options: std::collections::HashMap<&str, zbus::zvariant::Value<'_>> =
            std::collections::HashMap::new();
        let ok = proxy.repair(options).await?;
        if ok {
            Ok(())
        } else {
            Err(anyhow::anyhow!(
                "Filesystem repair completed but reported failure"
            ))
        }
    }

    pub async fn take_ownership(&self, recursive: bool) -> Result<()> {
        if self.connection.is_none() {
            return Err(crate::disks::DiskError::NotConnected(self.name.clone()).into());
        }

        let proxy = FilesystemProxy::builder(self.connection.as_ref().unwrap())
            .path(&self.path)?
            .build()
            .await?;

        let mut options: std::collections::HashMap<&str, zbus::zvariant::Value<'_>> =
            std::collections::HashMap::new();
        options.insert("recursive", zbus::zvariant::Value::from(recursive));

        proxy.take_ownership(options).await?;
        Ok(())
    }
}
