// SPDX-License-Identifier: GPL-3.0-only

use async_trait::async_trait;

use storage_types::{DiskInfo, SmartInfo, SmartSelfTestKind, VolumeInfo};

use crate::StorageError;

#[async_trait]
pub trait DiskQueryAdapter: Send + Sync {
    async fn list_disks(&self) -> Result<Vec<DiskInfo>, StorageError>;

    async fn list_disks_with_volumes(
        &self,
    ) -> Result<Vec<(DiskInfo, Vec<VolumeInfo>)>, StorageError>;

    async fn get_disk_info_for_drive_path(
        &self,
        object_path: &str,
    ) -> Result<DiskInfo, StorageError>;
}

#[async_trait]
pub trait DiskOpsAdapter: Send + Sync {
    async fn get_smart_info_by_device(&self, device: &str) -> Result<SmartInfo, StorageError>;

    async fn eject_drive_by_device(
        &self,
        device: &str,
        ejectable: bool,
    ) -> Result<(), StorageError>;

    async fn power_off_drive_by_device(
        &self,
        device: &str,
        can_power_off: bool,
    ) -> Result<(), StorageError>;

    async fn standby_drive_by_device(&self, device: &str) -> Result<(), StorageError>;

    async fn wakeup_drive_by_device(&self, device: &str) -> Result<(), StorageError>;

    async fn remove_drive_by_device(
        &self,
        device: &str,
        is_loop: bool,
        removable: bool,
        can_power_off: bool,
    ) -> Result<(), StorageError>;

    async fn start_drive_smart_selftest_by_device(
        &self,
        device: &str,
        kind: SmartSelfTestKind,
    ) -> Result<(), StorageError>;
}
