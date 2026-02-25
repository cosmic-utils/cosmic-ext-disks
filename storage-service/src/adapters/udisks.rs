// SPDX-License-Identifier: GPL-3.0-only

use async_trait::async_trait;
use std::os::fd::OwnedFd;
use std::sync::Arc;

use storage_contracts::traits::{
    DiskOpsAdapter, DiskQueryAdapter, FilesystemOpsAdapter, ImageOpsAdapter, LuksOpsAdapter,
    PartitionOpsAdapter,
};
use storage_contracts::{StorageError, StorageErrorKind};
use storage_types::{
    CreatePartitionInfo, DiskInfo, EncryptionOptionsSettings, FormatOptions, LuksInfo,
    MountOptions, MountOptionsSettings, PartitionInfo, ProcessInfo, SmartInfo, SmartSelfTestKind,
    VolumeInfo,
};
use storage_udisks::DiskManager;

pub struct DefaultUdisksAdapters {
    pub disk_query: Arc<dyn DiskQueryAdapter>,
    pub disk_ops: Arc<dyn DiskOpsAdapter>,
    pub partition_ops: Arc<dyn PartitionOpsAdapter>,
    pub filesystem_ops: Arc<dyn FilesystemOpsAdapter>,
    pub luks_ops: Arc<dyn LuksOpsAdapter>,
    pub image_ops: Arc<dyn ImageOpsAdapter>,
}

pub async fn build_default_adapters() -> anyhow::Result<DefaultUdisksAdapters> {
    let disk_manager = DiskManager::new().await?;

    Ok(DefaultUdisksAdapters {
        disk_query: Arc::new(UdisksDiskQueryAdapter::new(disk_manager.clone())),
        disk_ops: Arc::new(UdisksDiskOpsAdapter::new(disk_manager.clone())),
        partition_ops: Arc::new(UdisksPartitionOpsAdapter::new(disk_manager.clone())),
        filesystem_ops: Arc::new(UdisksFilesystemOpsAdapter::new(disk_manager)),
        luks_ops: Arc::new(UdisksLuksOpsAdapter::new()),
        image_ops: Arc::new(UdisksImageOpsAdapter::new()),
    })
}

#[derive(Clone)]
pub struct UdisksDiskQueryAdapter {
    manager: DiskManager,
}

impl UdisksDiskQueryAdapter {
    pub fn new(manager: DiskManager) -> Self {
        Self { manager }
    }
}

#[derive(Clone)]
pub struct UdisksPartitionOpsAdapter {
    manager: DiskManager,
}

impl UdisksPartitionOpsAdapter {
    pub fn new(manager: DiskManager) -> Self {
        Self { manager }
    }
}

#[derive(Clone)]
pub struct UdisksDiskOpsAdapter;

impl UdisksDiskOpsAdapter {
    pub fn new(_manager: DiskManager) -> Self {
        Self
    }
}

#[derive(Clone)]
pub struct UdisksFilesystemOpsAdapter {
    manager: DiskManager,
}

impl UdisksFilesystemOpsAdapter {
    pub fn new(manager: DiskManager) -> Self {
        Self { manager }
    }
}

#[derive(Clone)]
pub struct UdisksLuksOpsAdapter;

impl UdisksLuksOpsAdapter {
    pub fn new() -> Self {
        Self
    }
}

#[derive(Clone)]
pub struct UdisksImageOpsAdapter;

impl UdisksImageOpsAdapter {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl DiskQueryAdapter for UdisksDiskQueryAdapter {
    async fn list_disks(&self) -> Result<Vec<DiskInfo>, StorageError> {
        storage_udisks::disk::get_disks(&self.manager)
            .await
            .map_err(|e| {
                StorageError::new(
                    StorageErrorKind::Internal,
                    format!("Failed to enumerate disks via UDisks adapter: {e}"),
                )
            })
    }

    async fn list_disks_with_volumes(
        &self,
    ) -> Result<Vec<(DiskInfo, Vec<VolumeInfo>)>, StorageError> {
        storage_udisks::disk::get_disks_with_volumes(&self.manager)
            .await
            .map_err(|e| {
                StorageError::new(
                    StorageErrorKind::Internal,
                    format!("Failed to enumerate disks with volumes via UDisks adapter: {e}"),
                )
            })
    }

    async fn get_disk_info_for_drive_path(
        &self,
        object_path: &str,
    ) -> Result<DiskInfo, StorageError> {
        storage_udisks::get_disk_info_for_drive_path(&self.manager, object_path)
            .await
            .map_err(|e| {
                StorageError::new(
                    StorageErrorKind::Internal,
                    format!("Failed to get disk info for drive path via UDisks adapter: {e}"),
                )
            })
    }
}

#[async_trait]
impl PartitionOpsAdapter for UdisksPartitionOpsAdapter {
    async fn list_disks_with_partitions(
        &self,
    ) -> Result<Vec<(DiskInfo, Vec<PartitionInfo>)>, StorageError> {
        storage_udisks::disk::get_disks_with_partitions(&self.manager)
            .await
            .map_err(|e| {
                StorageError::new(
                    StorageErrorKind::Internal,
                    format!("Failed to enumerate disks with partitions via UDisks adapter: {e}"),
                )
            })
    }

    async fn resolve_block_path_for_device(&self, device: &str) -> Result<String, StorageError> {
        storage_udisks::block_object_path_for_device(device)
            .await
            .map_err(|e| {
                StorageError::new(
                    StorageErrorKind::NotFound,
                    format!("Failed to resolve block path for device {device}: {e}"),
                )
            })
    }

    async fn create_partition_table(
        &self,
        block_path: &str,
        table_type: &str,
    ) -> Result<(), StorageError> {
        storage_udisks::create_partition_table(block_path, table_type)
            .await
            .map_err(|e| {
                StorageError::new(
                    StorageErrorKind::Internal,
                    format!("Failed to create partition table {table_type} at {block_path}: {e}"),
                )
            })
    }

    async fn create_partition(
        &self,
        block_path: &str,
        offset: u64,
        size: u64,
        type_id: &str,
    ) -> Result<String, StorageError> {
        storage_udisks::create_partition(block_path, offset, size, type_id)
            .await
            .map_err(|e| {
                StorageError::new(
                    StorageErrorKind::Internal,
                    format!("Failed to create partition at {block_path}: {e}"),
                )
            })
    }

    async fn create_partition_with_filesystem(
        &self,
        block_path: &str,
        info: &CreatePartitionInfo,
    ) -> Result<String, StorageError> {
        storage_udisks::create_partition_with_filesystem(block_path, info)
            .await
            .map_err(|e| {
                StorageError::new(
                    StorageErrorKind::Internal,
                    format!("Failed to create partition with filesystem at {block_path}: {e}"),
                )
            })
    }

    async fn delete_partition(&self, partition_path: &str) -> Result<(), StorageError> {
        storage_udisks::delete_partition(partition_path)
            .await
            .map_err(|e| {
                StorageError::new(
                    StorageErrorKind::Internal,
                    format!("Failed to delete partition at {partition_path}: {e}"),
                )
            })
    }

    async fn resize_partition(
        &self,
        partition_path: &str,
        new_size: u64,
    ) -> Result<(), StorageError> {
        storage_udisks::resize_partition(partition_path, new_size)
            .await
            .map_err(|e| {
                StorageError::new(
                    StorageErrorKind::Internal,
                    format!("Failed to resize partition at {partition_path}: {e}"),
                )
            })
    }

    async fn set_partition_type(
        &self,
        partition_path: &str,
        type_id: &str,
    ) -> Result<(), StorageError> {
        storage_udisks::set_partition_type(partition_path, type_id)
            .await
            .map_err(|e| {
                StorageError::new(
                    StorageErrorKind::Internal,
                    format!("Failed to set partition type for {partition_path}: {e}"),
                )
            })
    }

    async fn set_partition_flags(
        &self,
        partition_path: &str,
        flags: u64,
    ) -> Result<(), StorageError> {
        storage_udisks::set_partition_flags(partition_path, flags)
            .await
            .map_err(|e| {
                StorageError::new(
                    StorageErrorKind::Internal,
                    format!("Failed to set partition flags for {partition_path}: {e}"),
                )
            })
    }

    async fn set_partition_name(
        &self,
        partition_path: &str,
        name: &str,
    ) -> Result<(), StorageError> {
        storage_udisks::set_partition_name(partition_path, name)
            .await
            .map_err(|e| {
                StorageError::new(
                    StorageErrorKind::Internal,
                    format!("Failed to set partition name for {partition_path}: {e}"),
                )
            })
    }
}

#[async_trait]
impl DiskOpsAdapter for UdisksDiskOpsAdapter {
    async fn get_smart_info_by_device(&self, device: &str) -> Result<SmartInfo, StorageError> {
        storage_udisks::get_smart_info_by_device(device)
            .await
            .map_err(|e| {
                StorageError::new(
                    StorageErrorKind::Internal,
                    format!("Failed to get SMART info for {device}: {e}"),
                )
            })
    }

    async fn eject_drive_by_device(
        &self,
        device: &str,
        ejectable: bool,
    ) -> Result<(), StorageError> {
        storage_udisks::eject_drive_by_device(device, ejectable)
            .await
            .map_err(|e| {
                StorageError::new(
                    StorageErrorKind::Internal,
                    format!("Failed to eject drive {device}: {e}"),
                )
            })
    }

    async fn power_off_drive_by_device(
        &self,
        device: &str,
        can_power_off: bool,
    ) -> Result<(), StorageError> {
        storage_udisks::power_off_drive_by_device(device, can_power_off)
            .await
            .map_err(|e| {
                StorageError::new(
                    StorageErrorKind::Internal,
                    format!("Failed to power off drive {device}: {e}"),
                )
            })
    }

    async fn standby_drive_by_device(&self, device: &str) -> Result<(), StorageError> {
        storage_udisks::standby_drive_by_device(device)
            .await
            .map_err(|e| {
                StorageError::new(
                    StorageErrorKind::Internal,
                    format!("Failed to put drive {device} into standby: {e}"),
                )
            })
    }

    async fn wakeup_drive_by_device(&self, device: &str) -> Result<(), StorageError> {
        storage_udisks::wakeup_drive_by_device(device)
            .await
            .map_err(|e| {
                StorageError::new(
                    StorageErrorKind::Internal,
                    format!("Failed to wake drive {device}: {e}"),
                )
            })
    }

    async fn remove_drive_by_device(
        &self,
        device: &str,
        is_loop: bool,
        removable: bool,
        can_power_off: bool,
    ) -> Result<(), StorageError> {
        storage_udisks::remove_drive_by_device(device, is_loop, removable, can_power_off)
            .await
            .map_err(|e| {
                StorageError::new(
                    StorageErrorKind::Internal,
                    format!("Failed to remove drive {device}: {e}"),
                )
            })
    }

    async fn start_drive_smart_selftest_by_device(
        &self,
        device: &str,
        kind: SmartSelfTestKind,
    ) -> Result<(), StorageError> {
        storage_udisks::start_drive_smart_selftest_by_device(device, kind)
            .await
            .map_err(|e| {
                StorageError::new(
                    StorageErrorKind::Internal,
                    format!("Failed to start SMART self-test for {device}: {e}"),
                )
            })
    }
}

#[async_trait]
impl FilesystemOpsAdapter for UdisksFilesystemOpsAdapter {
    async fn list_disks_with_volumes(
        &self,
    ) -> Result<Vec<(DiskInfo, Vec<VolumeInfo>)>, StorageError> {
        storage_udisks::disk::get_disks_with_volumes(&self.manager)
            .await
            .map_err(|e| {
                StorageError::new(
                    StorageErrorKind::Internal,
                    format!("Failed to enumerate disks with volumes via UDisks adapter: {e}"),
                )
            })
    }

    async fn get_filesystem_label(&self, device: &str) -> Result<String, StorageError> {
        storage_udisks::get_filesystem_label(device)
            .await
            .map_err(|e| {
                StorageError::new(
                    StorageErrorKind::Internal,
                    format!("Failed to get filesystem label for {device}: {e}"),
                )
            })
    }

    async fn format_filesystem(
        &self,
        device_path: &str,
        fs_type: &str,
        label: &str,
        options: FormatOptions,
    ) -> Result<(), StorageError> {
        storage_udisks::format_filesystem(device_path, fs_type, label, options)
            .await
            .map_err(|e| {
                StorageError::new(
                    StorageErrorKind::Internal,
                    format!("Failed to format filesystem {device_path} as {fs_type}: {e}"),
                )
            })
    }

    async fn mount_filesystem(
        &self,
        device_path: &str,
        mount_point: &str,
        options: MountOptions,
        caller_uid: Option<u32>,
    ) -> Result<String, StorageError> {
        storage_udisks::mount_filesystem(device_path, mount_point, options, caller_uid)
            .await
            .map_err(|e| {
                StorageError::new(
                    StorageErrorKind::Internal,
                    format!("Failed to mount filesystem {device_path}: {e}"),
                )
            })
    }

    async fn get_mount_point(&self, device: &str) -> Result<String, StorageError> {
        storage_udisks::get_mount_point(device).await.map_err(|e| {
            StorageError::new(
                StorageErrorKind::NotFound,
                format!("Failed to get mount point for {device}: {e}"),
            )
        })
    }

    async fn unmount_filesystem(
        &self,
        device_or_mount: &str,
        force: bool,
    ) -> Result<(), StorageError> {
        storage_udisks::unmount_filesystem(device_or_mount, force)
            .await
            .map_err(|e| {
                StorageError::new(
                    StorageErrorKind::Internal,
                    format!("Failed to unmount filesystem {device_or_mount}: {e}"),
                )
            })
    }

    async fn find_processes_using_mount(
        &self,
        mount_point: &str,
    ) -> Result<Vec<ProcessInfo>, StorageError> {
        storage_udisks::find_processes_using_mount(mount_point)
            .await
            .map_err(|e| {
                StorageError::new(
                    StorageErrorKind::Internal,
                    format!("Failed to find processes using mount {mount_point}: {e}"),
                )
            })
    }

    async fn kill_processes(&self, pids: &[i32]) -> Result<(), StorageError> {
        let failed: Vec<_> = storage_udisks::kill_processes(pids)
            .into_iter()
            .filter(|r| !r.success)
            .collect();

        if failed.is_empty() {
            return Ok(());
        }

        let errors = failed
            .into_iter()
            .map(|r| {
                format!(
                    "pid {}: {}",
                    r.pid,
                    r.error.unwrap_or_else(|| "unknown error".to_string())
                )
            })
            .collect::<Vec<_>>()
            .join(", ");

        Err(StorageError::new(
            StorageErrorKind::PermissionDenied,
            format!("Failed to kill one or more processes: {errors}"),
        ))
    }

    async fn check_filesystem(&self, device: &str, repair: bool) -> Result<bool, StorageError> {
        storage_udisks::check_filesystem(device, repair)
            .await
            .map_err(|e| {
                StorageError::new(
                    StorageErrorKind::Internal,
                    format!("Failed to check filesystem {device}: {e}"),
                )
            })
    }

    async fn set_filesystem_label(&self, device: &str, label: &str) -> Result<(), StorageError> {
        storage_udisks::set_filesystem_label(device, label)
            .await
            .map_err(|e| {
                StorageError::new(
                    StorageErrorKind::Internal,
                    format!("Failed to set filesystem label for {device}: {e}"),
                )
            })
    }

    async fn get_mount_options(
        &self,
        device: &str,
    ) -> Result<Option<MountOptionsSettings>, StorageError> {
        storage_udisks::get_mount_options(device)
            .await
            .map_err(|e| {
                StorageError::new(
                    StorageErrorKind::Internal,
                    format!("Failed to get mount options for {device}: {e}"),
                )
            })
    }

    async fn reset_mount_options(&self, device: &str) -> Result<(), StorageError> {
        storage_udisks::reset_mount_options(device)
            .await
            .map_err(|e| {
                StorageError::new(
                    StorageErrorKind::Internal,
                    format!("Failed to reset mount options for {device}: {e}"),
                )
            })
    }

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
    ) -> Result<(), StorageError> {
        storage_udisks::set_mount_options(
            device,
            mount_at_startup,
            show_in_ui,
            require_auth,
            display_name,
            icon_name,
            symbolic_icon_name,
            options,
            mount_point,
            identify_as,
            filesystem_type,
        )
        .await
        .map_err(|e| {
            StorageError::new(
                StorageErrorKind::Internal,
                format!("Failed to set mount options for {device}: {e}"),
            )
        })
    }

    async fn take_filesystem_ownership(
        &self,
        device: &str,
        recursive: bool,
    ) -> Result<(), StorageError> {
        storage_udisks::take_filesystem_ownership(device, recursive)
            .await
            .map_err(|e| {
                StorageError::new(
                    StorageErrorKind::Internal,
                    format!("Failed to take ownership for {device}: {e}"),
                )
            })
    }
}

#[async_trait]
impl LuksOpsAdapter for UdisksLuksOpsAdapter {
    async fn list_luks_devices(&self) -> Result<Vec<LuksInfo>, StorageError> {
        storage_udisks::list_luks_devices().await.map_err(|e| {
            StorageError::new(
                StorageErrorKind::Internal,
                format!("Failed to list LUKS devices via UDisks adapter: {e}"),
            )
        })
    }

    async fn format_luks(
        &self,
        device: &str,
        passphrase: &str,
        version: &str,
    ) -> Result<(), StorageError> {
        storage_udisks::format_luks(device, passphrase, version)
            .await
            .map_err(|e| {
                StorageError::new(
                    StorageErrorKind::Internal,
                    format!("Failed to format LUKS container on {device}: {e}"),
                )
            })
    }

    async fn unlock_luks(&self, device: &str, passphrase: &str) -> Result<String, StorageError> {
        storage_udisks::unlock_luks(device, passphrase)
            .await
            .map_err(|e| {
                StorageError::new(
                    StorageErrorKind::PermissionDenied,
                    format!("Failed to unlock LUKS container {device}: {e}"),
                )
            })
    }

    async fn lock_luks(&self, device: &str) -> Result<(), StorageError> {
        storage_udisks::lock_luks(device).await.map_err(|e| {
            StorageError::new(
                StorageErrorKind::Internal,
                format!("Failed to lock LUKS container {device}: {e}"),
            )
        })
    }

    async fn change_luks_passphrase(
        &self,
        device: &str,
        current_passphrase: &str,
        new_passphrase: &str,
    ) -> Result<(), StorageError> {
        storage_udisks::change_luks_passphrase(device, current_passphrase, new_passphrase)
            .await
            .map_err(|e| {
                StorageError::new(
                    StorageErrorKind::Internal,
                    format!("Failed to change LUKS passphrase for {device}: {e}"),
                )
            })
    }

    async fn get_encryption_options(
        &self,
        device: &str,
    ) -> Result<Option<EncryptionOptionsSettings>, StorageError> {
        storage_udisks::get_encryption_options(device)
            .await
            .map_err(|e| {
                StorageError::new(
                    StorageErrorKind::Internal,
                    format!("Failed to get encryption options for {device}: {e}"),
                )
            })
    }

    async fn set_encryption_options(
        &self,
        device: &str,
        settings: &EncryptionOptionsSettings,
    ) -> Result<(), StorageError> {
        storage_udisks::set_encryption_options(device, settings)
            .await
            .map_err(|e| {
                StorageError::new(
                    StorageErrorKind::Internal,
                    format!("Failed to set encryption options for {device}: {e}"),
                )
            })
    }

    async fn clear_encryption_options(&self, device: &str) -> Result<(), StorageError> {
        storage_udisks::clear_encryption_options(device)
            .await
            .map_err(|e| {
                StorageError::new(
                    StorageErrorKind::Internal,
                    format!("Failed to clear encryption options for {device}: {e}"),
                )
            })
    }
}

#[async_trait]
impl ImageOpsAdapter for UdisksImageOpsAdapter {
    async fn open_for_backup_by_device(&self, device: &str) -> Result<OwnedFd, StorageError> {
        storage_udisks::open_for_backup_by_device(device)
            .await
            .map_err(|e| {
                StorageError::new(
                    StorageErrorKind::Internal,
                    format!("Failed to open backup device {device}: {e}"),
                )
            })
    }

    async fn open_for_restore_by_device(&self, device: &str) -> Result<OwnedFd, StorageError> {
        storage_udisks::open_for_restore_by_device(device)
            .await
            .map_err(|e| {
                StorageError::new(
                    StorageErrorKind::Internal,
                    format!("Failed to open restore device {device}: {e}"),
                )
            })
    }

    async fn loop_setup_device_path(&self, image_path: &str) -> Result<String, StorageError> {
        storage_udisks::loop_setup_device_path(image_path)
            .await
            .map_err(|e| {
                StorageError::new(
                    StorageErrorKind::Internal,
                    format!("Failed to setup loop device for {image_path}: {e}"),
                )
            })
    }
}
