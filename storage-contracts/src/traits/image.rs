// SPDX-License-Identifier: GPL-3.0-only

use async_trait::async_trait;
use std::os::fd::OwnedFd;

use crate::StorageError;

#[async_trait]
pub trait ImageOpsAdapter: Send + Sync {
    async fn open_for_backup_by_device(&self, device: &str) -> Result<OwnedFd, StorageError>;

    async fn open_for_restore_by_device(&self, device: &str) -> Result<OwnedFd, StorageError>;

    async fn loop_setup_device_path(&self, image_path: &str) -> Result<String, StorageError>;
}
