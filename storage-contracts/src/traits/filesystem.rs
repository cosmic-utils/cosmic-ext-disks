// SPDX-License-Identifier: GPL-3.0-only

use async_trait::async_trait;

use storage_types::{
    DiskInfo, FormatOptions, MountOptions, MountOptionsSettings, ProcessInfo, VolumeInfo,
};

use crate::StorageError;

#[async_trait]
pub trait FilesystemOpsAdapter: Send + Sync {
    async fn list_disks_with_volumes(
        &self,
    ) -> Result<Vec<(DiskInfo, Vec<VolumeInfo>)>, StorageError>;

    async fn get_filesystem_label(&self, device: &str) -> Result<String, StorageError>;

    async fn format_filesystem(
        &self,
        device_path: &str,
        fs_type: &str,
        label: &str,
        options: FormatOptions,
    ) -> Result<(), StorageError>;

    async fn mount_filesystem(
        &self,
        device_path: &str,
        mount_point: &str,
        options: MountOptions,
        caller_uid: Option<u32>,
    ) -> Result<String, StorageError>;

    async fn get_mount_point(&self, device: &str) -> Result<String, StorageError>;

    async fn unmount_filesystem(
        &self,
        device_or_mount: &str,
        force: bool,
    ) -> Result<(), StorageError>;

    async fn find_processes_using_mount(
        &self,
        mount_point: &str,
    ) -> Result<Vec<ProcessInfo>, StorageError>;

    async fn kill_processes(&self, pids: &[i32]) -> Result<(), StorageError>;

    async fn check_filesystem(&self, device: &str, repair: bool) -> Result<bool, StorageError>;

    async fn set_filesystem_label(&self, device: &str, label: &str) -> Result<(), StorageError>;

    async fn get_mount_options(
        &self,
        device: &str,
    ) -> Result<Option<MountOptionsSettings>, StorageError>;

    async fn reset_mount_options(&self, device: &str) -> Result<(), StorageError>;

    #[allow(clippy::too_many_arguments)]
    async fn set_mount_options(
        &self,
        device: &str,
        mount_at_startup: bool,
        show_in_ui: bool,
        require_auth: bool,
        display_name: Option<String>,
        icon_name: Option<String>,
        symbolic_icon_name: Option<String>,
        options: String,
        mount_point: String,
        identify_as: String,
        filesystem_type: String,
    ) -> Result<(), StorageError>;

    async fn take_filesystem_ownership(
        &self,
        device: &str,
        recursive: bool,
    ) -> Result<(), StorageError>;
}
