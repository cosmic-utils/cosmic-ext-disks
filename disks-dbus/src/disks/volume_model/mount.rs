use super::VolumeModel;
use super::config::{extract_prefixed_value, find_configuration_item};
use crate::disks::MountOptionsSettings;
use crate::dbus::bytestring as bs;
use crate::udisks_block_config::UDisks2BlockConfigurationProxy;
use crate::{
    join_options, remove_prefixed, remove_token, set_prefixed_value, set_token_present,
    split_options, stable_dedup,
};
use anyhow::Result;
use zbus::zvariant::OwnedValue;
use crate::disks::ops::{RealDiskBackend, partition_mount, partition_unmount};

impl VolumeModel {
    pub async fn mount(&self) -> Result<()> {
        if self.connection.is_none() {
            return Err(crate::disks::DiskError::NotConnected(self.name.clone()).into());
        }

        let backend = RealDiskBackend::new(self.connection.as_ref().unwrap().clone());
        partition_mount(&backend, self.path.clone()).await
    }

    pub async fn unmount(&self) -> Result<()> {
        if self.connection.is_none() {
            return Err(crate::disks::DiskError::NotConnected(self.name.clone()).into());
        }

        let backend = RealDiskBackend::new(self.connection.as_ref().unwrap().clone());
        partition_unmount(&backend, self.path.clone()).await
    }

    pub async fn get_mount_options_settings(&self) -> Result<Option<MountOptionsSettings>> {
        if self.connection.is_none() {
            return Err(crate::disks::DiskError::NotConnected(self.name.clone()).into());
        }

        let proxy = UDisks2BlockConfigurationProxy::builder(self.connection.as_ref().unwrap())
            .path(&self.path)?
            .build()
            .await?;

        let items = proxy.configuration().await?;
        let Some((_, dict)) = find_configuration_item(&items, "fstab") else {
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

        let display_name = extract_prefixed_value(&tokens, "x-gvfs-name=");
        let icon_name = extract_prefixed_value(&tokens, "x-gvfs-icon=");
        let symbolic_icon_name = extract_prefixed_value(&tokens, "x-gvfs-symbolic-icon=");

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

    pub async fn default_mount_options(&self) -> Result<()> {
        if self.connection.is_none() {
            return Err(crate::disks::DiskError::NotConnected(self.name.clone()).into());
        }

        let proxy = UDisks2BlockConfigurationProxy::builder(self.connection.as_ref().unwrap())
            .path(&self.path)?
            .build()
            .await?;

        let items = proxy.configuration().await?;
        if let Some(old_item) = find_configuration_item(&items, "fstab") {
            proxy
                .remove_configuration_item(old_item, std::collections::HashMap::new())
                .await?;
        }

        Ok(())
    }

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
            return Err(crate::disks::DiskError::NotConnected(self.name.clone()).into());
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
        let old_item = find_configuration_item(&items, "fstab");

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

        let new_item = ("fstab".to_string(), dict);

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
}
