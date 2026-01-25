use crate::Usage;
use crate::udisks_block_config::{ConfigurationItem, UDisks2BlockConfigurationProxy};
use crate::{
    join_options, remove_prefixed, remove_token, set_prefixed_value, set_token_present,
    split_options, stable_dedup,
};
use anyhow::Result;
use std::collections::HashMap;
use std::path::Path;
use udisks2::{
    block::BlockProxy, encrypted::EncryptedProxy, filesystem::FilesystemProxy,
    partitiontable::PartitionTableProxy,
};
use zbus::zvariant::OwnedValue;
use zbus::zvariant::Value;
use zbus::{Connection, zvariant::OwnedObjectPath};

use super::MountOptionsSettings;

use super::{
    DiskError, lvm,
    ops::{RealDiskBackend, crypto_lock, crypto_unlock, partition_mount, partition_unmount},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VolumeKind {
    Partition,
    CryptoContainer,
    Filesystem,
    LvmPhysicalVolume,
    LvmLogicalVolume,
    Block,
}

#[derive(Debug, Clone)]
pub struct VolumeNode {
    pub kind: VolumeKind,
    pub label: String,
    pub size: u64,
    pub id_type: String,
    pub object_path: OwnedObjectPath,
    pub device_path: Option<String>,
    pub has_filesystem: bool,
    pub mount_points: Vec<String>,
    pub usage: Option<Usage>,

    pub locked: bool,
    pub children: Vec<VolumeNode>,

    connection: Option<Connection>,
}

impl VolumeNode {
    fn encode_bytestring(value: &str) -> Vec<u8> {
        let mut bytes = value.as_bytes().to_vec();
        bytes.push(0);
        bytes
    }

    fn bytestring_owned_value(value: &str) -> OwnedValue {
        Value::from(Self::encode_bytestring(value))
            .try_into()
            .expect("zvariant Value<Vec<u8>> should convert into OwnedValue")
    }

    fn owned_value_to_bytestring(value: &OwnedValue) -> Option<String> {
        let bytes: Vec<u8> = value.clone().try_into().ok()?;
        Some(Self::decode_c_string_bytes(&bytes))
    }

    fn extract_prefixed_value(tokens: &[String], prefix: &str) -> String {
        tokens
            .iter()
            .find_map(|t| t.strip_prefix(prefix).map(|v| v.to_string()))
            .unwrap_or_default()
    }

    fn find_configuration_item(
        items: &[ConfigurationItem],
        kind: &str,
    ) -> Option<ConfigurationItem> {
        items.iter().find(|(t, _)| t == kind).cloned()
    }

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

    pub fn can_unlock(&self) -> bool {
        self.kind == VolumeKind::CryptoContainer && self.locked
    }

    pub fn can_lock(&self) -> bool {
        self.kind == VolumeKind::CryptoContainer && !self.locked
    }

    async fn probe_basic_block(
        connection: &Connection,
        object_path: OwnedObjectPath,
        label: String,
        kind: VolumeKind,
    ) -> Result<Self> {
        let block_proxy = BlockProxy::builder(connection)
            .path(&object_path)?
            .build()
            .await?;

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
            let proposed = format!("/dev/{}", object_path.split("/").last().unwrap());
            if Path::new(&proposed).exists() {
                device_path = Some(proposed);
            }
        }

        let (has_filesystem, mount_points) = match FilesystemProxy::builder(connection)
            .path(&object_path)?
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

        let id_type = block_proxy.id_type().await?;
        let size = block_proxy.size().await?;

        let node = Self {
            kind,
            label,
            size,
            id_type,
            object_path,
            device_path,
            has_filesystem,
            mount_points,
            usage,
            locked: false,
            children: Vec::new(),
            connection: Some(connection.clone()),
        };

        Ok(node)
    }

    pub async fn from_block_object(
        connection: &Connection,
        object_path: OwnedObjectPath,
        label: String,
        kind: VolumeKind,
        block_index: Option<&BlockIndex>,
    ) -> Result<Self> {
        let mut node = Self::probe_basic_block(connection, object_path, label, kind).await?;

        // If this is an LVM PV, enumerate LVs as children.
        if node.kind == VolumeKind::LvmPhysicalVolume || node.id_type == "LVM2_member" {
            node.kind = VolumeKind::LvmPhysicalVolume;
            if let (Some(pv_device), Some(index)) = (node.device_path.as_deref(), block_index)
                && let Ok(lvs) = lvm::list_lvs_for_pv(pv_device)
            {
                node.children = lvs
                    .into_iter()
                    .filter_map(|lv| {
                        let lv_obj_path = index.object_path_for_device(&lv.lv_path)?;
                        Some(VolumeNode {
                            kind: VolumeKind::LvmLogicalVolume,
                            label: lv.display_name(),
                            size: lv.size_bytes,
                            id_type: String::new(),
                            object_path: lv_obj_path,
                            device_path: Some(lv.lv_path),
                            has_filesystem: false,
                            mount_points: Vec::new(),
                            usage: None,
                            locked: false,
                            children: Vec::new(),
                            connection: Some(connection.clone()),
                        })
                    })
                    .collect();

                // Populate filesystem/mount info for each LV.
                for child in &mut node.children {
                    if let Ok(lv_info) = VolumeNode::probe_basic_block(
                        connection,
                        child.object_path.clone(),
                        child.label.clone(),
                        VolumeKind::LvmLogicalVolume,
                    )
                    .await
                    {
                        child.id_type = lv_info.id_type;
                        child.size = lv_info.size;
                        child.device_path = lv_info.device_path;
                        child.has_filesystem = lv_info.has_filesystem;
                        child.mount_points = lv_info.mount_points;
                        child.usage = lv_info.usage;
                    }
                }
            }
        }

        Ok(node)
    }

    pub async fn crypto_container_for_partition(
        connection: &Connection,
        partition_object_path: OwnedObjectPath,
        label: String,
        block_index: &BlockIndex,
    ) -> Result<Self> {
        let encrypted = EncryptedProxy::builder(connection)
            .path(&partition_object_path)?
            .build()
            .await?;

        let mut node = Self::from_block_object(
            connection,
            partition_object_path.clone(),
            label,
            VolumeKind::CryptoContainer,
            Some(block_index),
        )
        .await?;

        let cleartext = encrypted.cleartext_device().await?;
        let cleartext_str = cleartext.to_string();
        let unlocked = cleartext_str != "/";

        node.locked = !unlocked;

        if unlocked {
            let child_label = String::new();
            let mut cleartext_node = Self::from_block_object(
                connection,
                cleartext,
                child_label,
                VolumeKind::Block,
                Some(block_index),
            )
            .await?;

            // If cleartext has a filesystem, treat it as mountable.
            if cleartext_node.has_filesystem {
                cleartext_node.kind = VolumeKind::Filesystem;
                if cleartext_node.label.trim().is_empty() {
                    cleartext_node.label = "Filesystem".to_string();
                }
            } else {
                if cleartext_node.label.trim().is_empty() {
                    cleartext_node.label = "Cleartext".to_string();
                }
                // If cleartext contains a partition table, enumerate its partitions as children.
                if let Ok(pt) = PartitionTableProxy::builder(connection)
                    .path(&cleartext_node.object_path)?
                    .build()
                    .await
                    && let Ok(parts) = pt.partitions().await
                {
                    for part_path in parts {
                        // Best-effort: treat nested partitions as regular blocks.
                        let label = part_path
                            .split("/")
                            .last()
                            .unwrap_or("Partition")
                            .replace('_', " ");
                        if let Ok(child) = VolumeNode::from_block_object(
                            connection,
                            part_path.clone(),
                            label,
                            VolumeKind::Partition,
                            Some(block_index),
                        )
                        .await
                        {
                            cleartext_node.children.push(child);
                        }
                    }
                }
            }

            // If cleartext is an LVM PV, it will be converted to LvmPhysicalVolume with children.
            node.children.push(cleartext_node);
        }

        Ok(node)
    }

    pub async fn mount(&self) -> Result<()> {
        if self.connection.is_none() {
            return Err(DiskError::NotConnected(self.label.clone()).into());
        }

        let backend = RealDiskBackend::new(self.connection.as_ref().unwrap().clone());
        partition_mount(&backend, self.object_path.clone()).await
    }

    pub async fn unmount(&self) -> Result<()> {
        if self.connection.is_none() {
            return Err(DiskError::NotConnected(self.label.clone()).into());
        }

        let backend = RealDiskBackend::new(self.connection.as_ref().unwrap().clone());
        partition_unmount(&backend, self.object_path.clone()).await
    }

    pub async fn default_mount_options(&self) -> Result<()> {
        let Some(connection) = self.connection.as_ref() else {
            return Err(DiskError::NotConnected(self.label.clone()).into());
        };

        let proxy = UDisks2BlockConfigurationProxy::builder(connection)
            .path(&self.object_path)?
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

    pub async fn get_mount_options_settings(&self) -> Result<Option<MountOptionsSettings>> {
        let Some(connection) = self.connection.as_ref() else {
            return Err(DiskError::NotConnected(self.label.clone()).into());
        };

        let proxy = UDisks2BlockConfigurationProxy::builder(connection)
            .path(&self.object_path)?
            .build()
            .await?;

        let items = proxy.configuration().await?;
        let Some((_, dict)) = Self::find_configuration_item(&items, "fstab") else {
            return Ok(None);
        };

        let identify_as = dict
            .get("fsname")
            .and_then(Self::owned_value_to_bytestring)
            .unwrap_or_default();
        let mount_point = dict
            .get("dir")
            .and_then(Self::owned_value_to_bytestring)
            .unwrap_or_default();
        let filesystem_type = dict
            .get("type")
            .and_then(Self::owned_value_to_bytestring)
            .unwrap_or_default();
        let opts = dict
            .get("opts")
            .and_then(Self::owned_value_to_bytestring)
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

    #[allow(clippy::too_many_arguments)]
    pub async fn edit_mount_options(
        &self,
        mount_at_startup: bool,
        show_in_ui: bool,
        require_auth: bool,
        display_name: Option<String>,
        icon_name: Option<String>,
        symbolic_icon_name: Option<String>,
        options: String,
        mount_point: String,
        identify_as: String,
        file_system_type: String,
    ) -> Result<()> {
        let Some(connection) = self.connection.as_ref() else {
            return Err(DiskError::NotConnected(self.label.clone()).into());
        };

        if mount_point.trim().is_empty() {
            return Err(anyhow::anyhow!("Mount point must not be empty"));
        }
        if identify_as.trim().is_empty() {
            return Err(anyhow::anyhow!("Identify As must not be empty"));
        }
        if file_system_type.trim().is_empty() {
            return Err(anyhow::anyhow!("Filesystem type must not be empty"));
        }

        let mut tokens = split_options(&options);
        tokens = set_token_present(tokens, "noauto", !mount_at_startup);
        tokens = set_token_present(tokens, "x-udisks-auth", require_auth);
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

        let proxy = UDisks2BlockConfigurationProxy::builder(connection)
            .path(&self.object_path)?
            .build()
            .await?;

        let items = proxy.configuration().await?;
        let old_item = Self::find_configuration_item(&items, "fstab");

        let mut dict: std::collections::HashMap<String, OwnedValue> =
            std::collections::HashMap::new();
        dict.insert(
            "fsname".to_string(),
            Self::bytestring_owned_value(identify_as.trim()),
        );
        dict.insert(
            "dir".to_string(),
            Self::bytestring_owned_value(mount_point.trim()),
        );
        dict.insert(
            "type".to_string(),
            Self::bytestring_owned_value(file_system_type.trim()),
        );
        dict.insert(
            "opts".to_string(),
            Self::bytestring_owned_value(opts.trim()),
        );
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

    pub async fn unlock(&self, passphrase: &str) -> Result<OwnedObjectPath> {
        if self.connection.is_none() {
            return Err(DiskError::NotConnected(self.label.clone()).into());
        }
        let backend = RealDiskBackend::new(self.connection.as_ref().unwrap().clone());
        crypto_unlock(&backend, self.object_path.clone(), passphrase).await
    }

    pub async fn lock(&self) -> Result<()> {
        if self.connection.is_none() {
            return Err(DiskError::NotConnected(self.label.clone()).into());
        }
        let backend = RealDiskBackend::new(self.connection.as_ref().unwrap().clone());
        crypto_lock(&backend, self.object_path.clone()).await
    }

    pub async fn edit_filesystem_label(&self, label: &str) -> Result<()> {
        if self.connection.is_none() {
            return Err(DiskError::NotConnected(self.label.clone()).into());
        }

        let proxy = FilesystemProxy::builder(self.connection.as_ref().unwrap())
            .path(&self.object_path)?
            .build()
            .await?;

        let options: HashMap<&str, Value<'_>> = HashMap::new();
        proxy.set_label(label, options).await?;
        Ok(())
    }

    pub async fn check_filesystem(&self) -> Result<()> {
        if self.connection.is_none() {
            return Err(DiskError::NotConnected(self.label.clone()).into());
        }

        let proxy = FilesystemProxy::builder(self.connection.as_ref().unwrap())
            .path(&self.object_path)?
            .build()
            .await?;

        let options: HashMap<&str, Value<'_>> = HashMap::new();
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
            return Err(DiskError::NotConnected(self.label.clone()).into());
        }

        let proxy = FilesystemProxy::builder(self.connection.as_ref().unwrap())
            .path(&self.object_path)?
            .build()
            .await?;

        let options: HashMap<&str, Value<'_>> = HashMap::new();
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
            return Err(DiskError::NotConnected(self.label.clone()).into());
        }

        let proxy = FilesystemProxy::builder(self.connection.as_ref().unwrap())
            .path(&self.object_path)?
            .build()
            .await?;

        let mut options: HashMap<&str, Value<'_>> = HashMap::new();
        options.insert("recursive", Value::from(recursive));
        proxy.take_ownership(options).await?;
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct BlockIndex {
    by_device: HashMap<String, OwnedObjectPath>,
}

impl BlockIndex {
    fn canonicalize_best_effort(p: &str) -> Option<String> {
        match std::fs::canonicalize(p) {
            Ok(c) => Some(c.to_string_lossy().to_string()),
            Err(_) => None,
        }
    }

    pub async fn build(connection: &Connection, block_objects: &[OwnedObjectPath]) -> Result<Self> {
        let mut by_device = HashMap::new();

        for obj in block_objects {
            let proxy = match BlockProxy::builder(connection).path(obj)?.build().await {
                Ok(p) => p,
                Err(_) => continue,
            };

            let preferred_device =
                VolumeNode::decode_c_string_bytes(&proxy.preferred_device().await?);
            let device = if preferred_device.is_empty() {
                VolumeNode::decode_c_string_bytes(&proxy.device().await?)
            } else {
                preferred_device
            };

            if !device.is_empty() {
                by_device.insert(device.clone(), obj.clone());

                // Also index the canonical path if it differs (helps match /dev/vg/* and symlinks).
                if let Some(canon) = Self::canonicalize_best_effort(&device) {
                    by_device.entry(canon).or_insert_with(|| obj.clone());
                }
            }
        }

        Ok(Self { by_device })
    }

    pub fn object_path_for_device(&self, dev: &str) -> Option<OwnedObjectPath> {
        if let Some(p) = self.by_device.get(dev) {
            return Some(p.clone());
        }

        if let Some(canon) = Self::canonicalize_best_effort(dev) {
            return self.by_device.get(&canon).cloned();
        }

        None
    }
}
