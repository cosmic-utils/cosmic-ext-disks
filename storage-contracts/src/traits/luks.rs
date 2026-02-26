// SPDX-License-Identifier: GPL-3.0-only

use async_trait::async_trait;

use storage_types::{EncryptionOptionsSettings, LuksInfo};

use crate::StorageError;

#[async_trait]
pub trait LuksOpsAdapter: Send + Sync {
    async fn list_luks_devices(&self) -> Result<Vec<LuksInfo>, StorageError>;

    async fn format_luks(
        &self,
        device: &str,
        passphrase: &str,
        version: &str,
    ) -> Result<(), StorageError>;

    async fn unlock_luks(&self, device: &str, passphrase: &str) -> Result<String, StorageError>;

    async fn lock_luks(&self, device: &str) -> Result<(), StorageError>;

    async fn change_luks_passphrase(
        &self,
        device: &str,
        current_passphrase: &str,
        new_passphrase: &str,
    ) -> Result<(), StorageError>;

    async fn get_encryption_options(
        &self,
        device: &str,
    ) -> Result<Option<EncryptionOptionsSettings>, StorageError>;

    async fn set_encryption_options(
        &self,
        device: &str,
        settings: &EncryptionOptionsSettings,
    ) -> Result<(), StorageError>;

    async fn clear_encryption_options(&self, device: &str) -> Result<(), StorageError>;
}
