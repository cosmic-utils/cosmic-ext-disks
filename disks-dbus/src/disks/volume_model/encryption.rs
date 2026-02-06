use super::VolumeModel;
use super::config::find_configuration_item;
use crate::disks::EncryptionOptionsSettings;
use crate::disks::ops::{RealDiskBackend, crypto_lock, crypto_unlock};
use crate::dbus::bytestring as bs;
use crate::udisks_block_config::UDisks2BlockConfigurationProxy;
use crate::{
    join_options, remove_token, set_token_present, split_options, stable_dedup,
};
use anyhow::Result;
use udisks2::block::BlockProxy;
use zbus::zvariant::{OwnedObjectPath, OwnedValue};

impl VolumeModel {
    pub async fn unlock(&self, passphrase: &str) -> Result<OwnedObjectPath> {
        if self.connection.is_none() {
            return Err(crate::disks::DiskError::NotConnected(self.name.clone()).into());
        }

        let backend = RealDiskBackend::new(self.connection.as_ref().unwrap().clone());
        crypto_unlock(&backend, self.path.clone(), passphrase).await
    }

    pub async fn lock(&self) -> Result<()> {
        if self.connection.is_none() {
            return Err(crate::disks::DiskError::NotConnected(self.name.clone()).into());
        }

        let backend = RealDiskBackend::new(self.connection.as_ref().unwrap().clone());
        crypto_lock(&backend, self.path.clone()).await
    }

    pub async fn change_passphrase(&self, current: &str, new: &str) -> Result<()> {
        if self.connection.is_none() {
            return Err(crate::disks::DiskError::NotConnected(self.name.clone()).into());
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

    pub async fn get_encryption_options_settings(
        &self,
    ) -> Result<Option<EncryptionOptionsSettings>> {
        if self.connection.is_none() {
            return Err(crate::disks::DiskError::NotConnected(self.name.clone()).into());
        }

        let proxy = UDisks2BlockConfigurationProxy::builder(self.connection.as_ref().unwrap())
            .path(&self.path)?
            .build()
            .await?;

        let items = proxy.configuration().await?;
        let Some((_, dict)) = find_configuration_item(&items, "crypttab") else {
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

    pub async fn default_encryption_options(&self) -> Result<()> {
        if self.connection.is_none() {
            return Err(crate::disks::DiskError::NotConnected(self.name.clone()).into());
        }

        let proxy = UDisks2BlockConfigurationProxy::builder(self.connection.as_ref().unwrap())
            .path(&self.path)?
            .build()
            .await?;

        let items = proxy.configuration().await?;
        if let Some(old_item) = find_configuration_item(&items, "crypttab") {
            proxy
                .remove_configuration_item(old_item, std::collections::HashMap::new())
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
            return Err(crate::disks::DiskError::NotConnected(self.name.clone()).into());
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
        let old_item = find_configuration_item(&items, "crypttab");

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

        let new_item = ("crypttab".to_string(), dict);

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
