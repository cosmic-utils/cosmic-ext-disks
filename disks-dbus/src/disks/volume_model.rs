use super::DiskError;
use super::ops::{
    PartitionFormatArgs, RealDiskBackend, crypto_lock, crypto_unlock, partition_delete,
    partition_format, partition_mount, partition_unmount,
};
use crate::Usage;
use crate::dbus::bytestring as bs;
use crate::udisks_block_config::{ConfigurationItem, UDisks2BlockConfigurationProxy};
use crate::{
    join_options, remove_prefixed, remove_token, set_prefixed_value, set_token_present,
    split_options, stable_dedup,
};
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
use zbus::zvariant::OwnedValue;
use zbus::{Connection, zvariant::OwnedObjectPath};

use super::{EncryptionOptionsSettings, MountOptionsSettings};

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
    fn find_configuration_item(
        items: &[ConfigurationItem],
        kind: &str,
    ) -> Option<ConfigurationItem> {
        items.iter().find(|(t, _)| t == kind).cloned()
    }

    fn extract_prefixed_value(tokens: &[String], prefix: &str) -> String {
        tokens
            .iter()
            .find_map(|t| t.strip_prefix(prefix).map(|v| v.to_string()))
            .unwrap_or_default()
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

        println!("{type_str}");

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

    pub async fn open_for_backup(&self) -> Result<std::os::fd::OwnedFd> {
        crate::open_for_backup(self.path.clone()).await
    }

    pub async fn open_for_restore(&self) -> Result<std::os::fd::OwnedFd> {
        crate::open_for_restore(self.path.clone()).await
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

    pub async fn edit_partition(
        &self,
        partition_type: String,
        name: String,
        flags: u64,
    ) -> Result<()> {
        if self.connection.is_none() {
            return Err(DiskError::NotConnected(self.name.clone()).into());
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

    pub async fn edit_filesystem_label(&self, label: String) -> Result<()> {
        if self.connection.is_none() {
            return Err(DiskError::NotConnected(self.name.clone()).into());
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

    pub async fn change_passphrase(&self, current: &str, new: &str) -> Result<()> {
        if self.connection.is_none() {
            return Err(DiskError::NotConnected(self.name.clone()).into());
        }

        let proxy = udisks2::encrypted::EncryptedProxy::builder(self.connection.as_ref().unwrap())
            .path(&self.path)?
            .build()
            .await?;

        let options: std::collections::HashMap<&str, zbus::zvariant::Value<'_>> =
            std::collections::HashMap::new();
        proxy.change_passphrase(current, new, options).await?;
        Ok(())
    }

    pub async fn resize(&self, new_size_bytes: u64) -> Result<()> {
        if self.connection.is_none() {
            return Err(DiskError::NotConnected(self.name.clone()).into());
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

    pub async fn check_filesystem(&self) -> Result<()> {
        if self.connection.is_none() {
            return Err(DiskError::NotConnected(self.name.clone()).into());
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
            return Err(DiskError::NotConnected(self.name.clone()).into());
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
            return Err(DiskError::NotConnected(self.name.clone()).into());
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

    pub async fn get_mount_options_settings(&self) -> Result<Option<MountOptionsSettings>> {
        if self.connection.is_none() {
            return Err(DiskError::NotConnected(self.name.clone()).into());
        }

        let proxy = UDisks2BlockConfigurationProxy::builder(self.connection.as_ref().unwrap())
            .path(&self.path)?
            .build()
            .await?;

        let items = proxy.configuration().await?;
        let Some((_, dict)) = Self::find_configuration_item(&items, "fstab") else {
            return Ok(None);
        };

        let identify_as = dict
            .get("fsname")
            .and_then(bs::owned_value_to_bytestring)
            .unwrap_or_default();
        let mount_point = dict
            .get("dir")
            .and_then(bs::owned_value_to_bytestring)
            .unwrap_or_default();
        let filesystem_type = dict
            .get("type")
            .and_then(bs::owned_value_to_bytestring)
            .unwrap_or_default();
        let opts = dict
            .get("opts")
            .and_then(bs::owned_value_to_bytestring)
            .unwrap_or_default();

        let tokens = split_options(&opts);
        let mount_at_startup = !tokens.iter().any(|t| t == "noauto");
        let require_auth = tokens.iter().any(|t| t == "x-udisks-auth");
        let show_in_ui = tokens.iter().any(|t| t == "x-gvfs-show");

        let display_name = Self::extract_prefixed_value(&tokens, "x-gvfs-name=");
        let icon_name = Self::extract_prefixed_value(&tokens, "x-gvfs-icon=");
        let symbolic_icon_name = Self::extract_prefixed_value(&tokens, "x-gvfs-symbolic-icon=");

        let mut other_tokens = tokens;
        other_tokens = remove_token(other_tokens, "noauto");
        other_tokens = remove_token(other_tokens, "x-udisks-auth");
        other_tokens = remove_token(other_tokens, "x-gvfs-show");
        other_tokens = remove_prefixed(other_tokens, "x-gvfs-name=");
        other_tokens = remove_prefixed(other_tokens, "x-gvfs-icon=");
        other_tokens = remove_prefixed(other_tokens, "x-gvfs-symbolic-icon=");

        let other_options = join_options(&stable_dedup(other_tokens));

        Ok(Some(MountOptionsSettings {
            identify_as,
            mount_point,
            filesystem_type,
            mount_at_startup,
            require_auth,
            show_in_ui,
            other_options,
            display_name,
            icon_name,
            symbolic_icon_name,
        }))
    }

    pub async fn get_encryption_options_settings(
        &self,
    ) -> Result<Option<EncryptionOptionsSettings>> {
        if self.connection.is_none() {
            return Err(DiskError::NotConnected(self.name.clone()).into());
        }

        let proxy = UDisks2BlockConfigurationProxy::builder(self.connection.as_ref().unwrap())
            .path(&self.path)?
            .build()
            .await?;

        let items = proxy.configuration().await?;
        let Some((_, dict)) = Self::find_configuration_item(&items, "crypttab") else {
            return Ok(None);
        };

        let name = dict
            .get("name")
            .and_then(bs::owned_value_to_bytestring)
            .unwrap_or_default();
        let options = dict
            .get("options")
            .and_then(bs::owned_value_to_bytestring)
            .unwrap_or_default();

        let tokens = split_options(&options);
        let unlock_at_startup = !tokens.iter().any(|t| t == "noauto");
        let require_auth = tokens.iter().any(|t| t == "x-udisks-auth");

        let mut other_tokens = tokens;
        other_tokens = remove_token(other_tokens, "noauto");
        other_tokens = remove_token(other_tokens, "x-udisks-auth");
        let other_options = join_options(&stable_dedup(other_tokens));

        Ok(Some(EncryptionOptionsSettings {
            name,
            unlock_at_startup,
            require_auth,
            other_options,
        }))
    }

    //TODO: implement. See how edit mount options -> User session defaults works in gnome-disks.
    pub async fn default_mount_options(&self) -> Result<()> {
        if self.connection.is_none() {
            return Err(DiskError::NotConnected(self.name.clone()).into());
        }

        let proxy = UDisks2BlockConfigurationProxy::builder(self.connection.as_ref().unwrap())
            .path(&self.path)?
            .build()
            .await?;

        let items = proxy.configuration().await?;
        if let Some(old_item) = Self::find_configuration_item(&items, "fstab") {
            proxy
                .remove_configuration_item(old_item, std::collections::HashMap::new())
                .await?;
        }

        Ok(())
    }

    //TODO: implement. Look at gnome-disks -> partition -> edit mount options. Likely make all params optional.
    #[allow(clippy::too_many_arguments)]
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

        if mount_point.trim().is_empty() {
            return Err(anyhow::anyhow!("Mount point must not be empty"));
        }
        if identify_as.trim().is_empty() {
            return Err(anyhow::anyhow!("Identify As must not be empty"));
        }
        if file_system_type.trim().is_empty() {
            return Err(anyhow::anyhow!("Filesystem type must not be empty"));
        }

        // Build the final `opts` token list matching GNOME Disks behavior.
        let mut tokens = split_options(&options);
        tokens = set_token_present(tokens, "noauto", !mount_at_startup);
        tokens = set_token_present(tokens, "x-udisks-auth", requre_auth);
        tokens = set_token_present(tokens, "x-gvfs-show", show_in_ui);
        tokens = set_prefixed_value(tokens, "x-gvfs-name=", display_name.as_deref());
        tokens = set_prefixed_value(tokens, "x-gvfs-icon=", icon_name.as_deref());
        tokens = set_prefixed_value(
            tokens,
            "x-gvfs-symbolic-icon=",
            symbolic_icon_name.as_deref(),
        );
        let opts = join_options(&stable_dedup(tokens));

        if opts.trim().is_empty() {
            return Err(anyhow::anyhow!("Mount options must not be empty"));
        }

        let proxy = UDisks2BlockConfigurationProxy::builder(self.connection.as_ref().unwrap())
            .path(&self.path)?
            .build()
            .await?;

        let items = proxy.configuration().await?;
        let old_item = Self::find_configuration_item(&items, "fstab");

        let mut dict: std::collections::HashMap<String, OwnedValue> =
            std::collections::HashMap::new();
        dict.insert(
            "fsname".to_string(),
            bs::bytestring_owned_value(identify_as.trim()),
        );
        dict.insert(
            "dir".to_string(),
            bs::bytestring_owned_value(mount_point.trim()),
        );
        dict.insert(
            "type".to_string(),
            bs::bytestring_owned_value(file_system_type.trim()),
        );
        dict.insert("opts".to_string(), bs::bytestring_owned_value(opts.trim()));
        dict.insert("freq".to_string(), OwnedValue::from(0i32));
        dict.insert("passno".to_string(), OwnedValue::from(0i32));

        let new_item: ConfigurationItem = ("fstab".to_string(), dict);

        if let Some(old_item) = old_item {
            proxy
                .update_configuration_item(old_item, new_item, std::collections::HashMap::new())
                .await?;
        } else {
            proxy
                .add_configuration_item(new_item, std::collections::HashMap::new())
                .await?;
        }

        Ok(())
    }

    pub async fn edit_encryption_options(
        &self,
        unlock_at_startup: bool,
        require_auth: bool,
        other_options: String,
        name: String,
        passphrase: String,
    ) -> Result<()> {
        if self.connection.is_none() {
            return Err(DiskError::NotConnected(self.name.clone()).into());
        }

        if name.trim().is_empty() {
            return Err(anyhow::anyhow!("Name must not be empty"));
        }

        // GNOME Disks forces `device` to `UUID=<block-uuid>`.
        let block_proxy = BlockProxy::builder(self.connection.as_ref().unwrap())
            .path(&self.path)?
            .build()
            .await?;
        let id_uuid = block_proxy.id_uuid().await.unwrap_or_default();
        if id_uuid.trim().is_empty() {
            return Err(anyhow::anyhow!(
                "Missing block UUID; cannot write crypttab entry"
            ));
        }
        let device = format!("UUID={id_uuid}");

        let mut tokens = split_options(&other_options);
        tokens = set_token_present(tokens, "noauto", !unlock_at_startup);
        tokens = set_token_present(tokens, "x-udisks-auth", require_auth);
        let options = join_options(&stable_dedup(tokens));

        let proxy = UDisks2BlockConfigurationProxy::builder(self.connection.as_ref().unwrap())
            .path(&self.path)?
            .build()
            .await?;

        let items = proxy.configuration().await?;
        let old_item = Self::find_configuration_item(&items, "crypttab");

        // If we have an existing non-empty, non-/dev* passphrase path, keep it.
        let existing_passphrase_path = old_item
            .as_ref()
            .and_then(|(_, d)| d.get("passphrase-path"))
            .and_then(bs::owned_value_to_bytestring)
            .unwrap_or_default();

        let mut passphrase_path = String::new();
        let mut passphrase_contents = String::new();
        if !passphrase.is_empty() {
            passphrase_contents = passphrase;
            if !existing_passphrase_path.is_empty() && !existing_passphrase_path.starts_with("/dev")
            {
                passphrase_path = existing_passphrase_path;
            } else {
                passphrase_path = format!("/etc/luks-keys/{}", name.trim());
            }
        }

        let mut dict: std::collections::HashMap<String, OwnedValue> =
            std::collections::HashMap::new();
        dict.insert("device".to_string(), bs::bytestring_owned_value(&device));
        dict.insert("name".to_string(), bs::bytestring_owned_value(name.trim()));
        dict.insert(
            "options".to_string(),
            bs::bytestring_owned_value(options.trim()),
        );
        dict.insert(
            "passphrase-path".to_string(),
            bs::bytestring_owned_value(&passphrase_path),
        );
        dict.insert(
            "passphrase-contents".to_string(),
            bs::bytestring_owned_value(&passphrase_contents),
        );

        let new_item: ConfigurationItem = ("crypttab".to_string(), dict);

        if let Some(old_item) = old_item {
            proxy
                .update_configuration_item(old_item, new_item, std::collections::HashMap::new())
                .await?;
        } else {
            proxy
                .add_configuration_item(new_item, std::collections::HashMap::new())
                .await?;
        }

        Ok(())
    }

    pub async fn default_encryption_options(&self) -> Result<()> {
        if self.connection.is_none() {
            return Err(DiskError::NotConnected(self.name.clone()).into());
        }

        let proxy = UDisks2BlockConfigurationProxy::builder(self.connection.as_ref().unwrap())
            .path(&self.path)?
            .build()
            .await?;

        let items = proxy.configuration().await?;
        if let Some(old_item) = Self::find_configuration_item(&items, "crypttab") {
            proxy
                .remove_configuration_item(old_item, std::collections::HashMap::new())
                .await?;
        }

        Ok(())
    }

    // Backwards-compat (typo in API): keep the old name but make it explicit.
    pub async fn edit_encrytion_options(&self) -> Result<()> {
        Err(anyhow::anyhow!(
            "edit_encrytion_options() is deprecated; use edit_encryption_options(...)"
        ))
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
