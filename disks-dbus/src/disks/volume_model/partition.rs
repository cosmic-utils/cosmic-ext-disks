use super::VolumeModel;
use anyhow::Result;
use enumflags2::BitFlags;
use udisks2::partition::{PartitionFlags, PartitionProxy};

impl VolumeModel {
    pub async fn edit_partition(
        &self,
        partition_type: String,
        name: String,
        flags: u64,
    ) -> Result<()> {
        if self.connection.is_none() {
            return Err(crate::disks::DiskError::NotConnected(self.name.clone()).into());
        }

        let proxy = PartitionProxy::builder(self.connection.as_ref().unwrap())
            .path(&self.path)?
            .build()
            .await?;

        let options: std::collections::HashMap<&str, zbus::zvariant::Value<'_>> =
            std::collections::HashMap::new();

        let flags = BitFlags::<PartitionFlags>::from_bits_truncate(flags);

        proxy.set_type(&partition_type, options.clone()).await?;
        proxy.set_name(&name, options.clone()).await?;
        proxy.set_flags(flags, options).await?;

        Ok(())
    }

    pub fn is_legacy_bios_bootable(&self) -> bool {
        self.flags.contains(PartitionFlags::LegacyBIOSBootable)
    }

    pub fn is_system_partition(&self) -> bool {
        self.flags.contains(PartitionFlags::SystemPartition)
    }

    pub fn is_hidden(&self) -> bool {
        self.flags.contains(PartitionFlags::Hidden)
    }

    pub fn make_partition_flags_bits(
        legacy_bios_bootable: bool,
        system_partition: bool,
        hidden: bool,
    ) -> u64 {
        let mut bits: u64 = 0;
        if system_partition {
            bits |= PartitionFlags::SystemPartition as u64;
        }
        if legacy_bios_bootable {
            bits |= PartitionFlags::LegacyBIOSBootable as u64;
        }
        if hidden {
            bits |= PartitionFlags::Hidden as u64;
        }
        bits
    }

    pub async fn resize(&self, new_size_bytes: u64) -> Result<()> {
        if self.connection.is_none() {
            return Err(crate::disks::DiskError::NotConnected(self.name.clone()).into());
        }

        let proxy = PartitionProxy::builder(self.connection.as_ref().unwrap())
            .path(&self.path)?
            .build()
            .await?;

        let options: std::collections::HashMap<&str, zbus::zvariant::Value<'_>> =
            std::collections::HashMap::new();
        proxy.resize(new_size_bytes, options).await?;
        Ok(())
    }

    pub async fn delete(&self) -> Result<()> {
        if self.connection.is_none() {
            return Err(crate::disks::DiskError::NotConnected(self.name.clone()).into());
        }

        //try to unmount first. If it fails, it's likely because it's already unmounted.
        //any other error with the partition should be caught by the delete operation.
        let _ = self.unmount().await;

        let backend =
            crate::disks::ops::RealDiskBackend::new(self.connection.as_ref().unwrap().clone());
        crate::disks::ops::partition_delete(&backend, self.path.clone()).await
    }
}
