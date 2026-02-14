// SPDX-License-Identifier: GPL-3.0-only

use crate::client::error::ClientError;
use storage_models::{
    FilesystemInfo, FilesystemUsage, MountOptionsSettings, ProcessInfo, UnmountResult,
};
use zbus::{Connection, proxy};

/// D-Bus proxy interface for filesystem operations
#[proxy(
    interface = "org.cosmic.ext.StorageService.Filesystems",
    default_service = "org.cosmic.ext.StorageService",
    default_path = "/org/cosmic/ext/StorageService/filesystems"
)]
pub trait FilesystemsInterface {
    /// List all filesystems
    async fn list_filesystems(&self) -> zbus::Result<String>;

    /// Get list of supported filesystem types
    async fn get_supported_filesystems(&self) -> zbus::Result<String>;

    /// Format a device with a filesystem
    async fn format(
        &self,
        device: &str,
        fs_type: &str,
        label: &str,
        options_json: &str,
    ) -> zbus::Result<()>;

    /// Mount a filesystem
    async fn mount(
        &self,
        device: &str,
        mount_point: &str,
        options_json: &str,
    ) -> zbus::Result<String>;

    /// Unmount a filesystem
    async fn unmount(
        &self,
        device_or_mount: &str,
        force: bool,
        kill_processes: bool,
    ) -> zbus::Result<String>;

    /// Get processes blocking unmount
    async fn get_blocking_processes(&self, device_or_mount: &str) -> zbus::Result<String>;

    /// Check and repair a filesystem
    async fn check(&self, device: &str, repair: bool) -> zbus::Result<String>;

    /// Set filesystem label
    async fn set_label(&self, device: &str, label: &str) -> zbus::Result<()>;

    /// Get filesystem usage statistics
    async fn get_usage(&self, mount_point: &str) -> zbus::Result<String>;

    /// Get persistent mount options (fstab) for a device
    async fn get_mount_options(&self, device: &str) -> zbus::Result<String>;

    /// Clear persistent mount options for a device
    async fn default_mount_options(&self, device: &str) -> zbus::Result<()>;

    /// Set persistent mount options for a device
    async fn edit_mount_options(
        &self,
        device: &str,
        mount_at_startup: bool,
        show_in_ui: bool,
        require_auth: bool,
        display_name: &str,
        icon_name: &str,
        symbolic_icon_name: &str,
        other_options: &str,
        mount_point: &str,
        identify_as: &str,
        filesystem_type: &str,
    ) -> zbus::Result<()>;

    /// Take ownership of a mounted filesystem (e.g. for fstab)
    async fn take_ownership(&self, device: &str, recursive: bool) -> zbus::Result<()>;

    /// Signal emitted during format operation with progress
    #[zbus(signal)]
    async fn format_progress(&self, device: &str, percent: u8) -> zbus::Result<()>;

    /// Signal emitted when format completes
    #[zbus(signal)]
    async fn formatted(&self, device: &str, fs_type: &str) -> zbus::Result<()>;

    /// Signal emitted when filesystem is mounted
    #[zbus(signal)]
    async fn mounted(&self, device: &str, mount_point: &str) -> zbus::Result<()>;

    /// Signal emitted when filesystem is unmounted
    #[zbus(signal)]
    async fn unmounted(&self, device_or_mount: &str) -> zbus::Result<()>;
}

/// Client for filesystem operations
pub struct FilesystemsClient {
    proxy: FilesystemsInterfaceProxy<'static>,
}

impl std::fmt::Debug for FilesystemsClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FilesystemsClient").finish_non_exhaustive()
    }
}

impl FilesystemsClient {
    /// Create a new filesystems client connected to the storage service
    pub async fn new() -> Result<Self, ClientError> {
        let conn = Connection::system().await.map_err(|e| {
            ClientError::Connection(format!("Failed to connect to system bus: {}", e))
        })?;

        let proxy = FilesystemsInterfaceProxy::new(&conn).await.map_err(|e| {
            ClientError::Connection(format!("Failed to create filesystems proxy: {}", e))
        })?;

        Ok(Self { proxy })
    }

    /// List all filesystems
    pub async fn list_filesystems(&self) -> Result<Vec<FilesystemInfo>, ClientError> {
        let json = self.proxy.list_filesystems().await?;
        let filesystems: Vec<FilesystemInfo> = serde_json::from_str(&json).map_err(|e| {
            ClientError::ParseError(format!("Failed to parse filesystem list: {}", e))
        })?;
        Ok(filesystems)
    }

    /// Get list of supported filesystem types
    pub async fn get_supported_filesystems(&self) -> Result<Vec<String>, ClientError> {
        let json = self.proxy.get_supported_filesystems().await?;
        let types: Vec<String> = serde_json::from_str(&json).map_err(|e| {
            ClientError::ParseError(format!("Failed to parse filesystem types: {}", e))
        })?;
        Ok(types)
    }

    /// Format a device with a filesystem
    pub async fn format(
        &self,
        device: &str,
        fs_type: &str,
        label: &str,
        options: Option<&str>,
    ) -> Result<(), ClientError> {
        let options_json = options.unwrap_or("{}");
        Ok(self
            .proxy
            .format(device, fs_type, label, options_json)
            .await?)
    }

    /// Mount a filesystem, returns actual mount point used
    pub async fn mount(
        &self,
        device: &str,
        mount_point: &str,
        options: Option<&str>,
    ) -> Result<String, ClientError> {
        let options_json = options.unwrap_or("{}");
        Ok(self.proxy.mount(device, mount_point, options_json).await?)
    }

    /// Unmount a filesystem
    pub async fn unmount(
        &self,
        device_or_mount: &str,
        force: bool,
        kill_processes: bool,
    ) -> Result<UnmountResult, ClientError> {
        let json = self
            .proxy
            .unmount(device_or_mount, force, kill_processes)
            .await?;
        let result: UnmountResult = serde_json::from_str(&json).map_err(|e| {
            ClientError::ParseError(format!("Failed to parse unmount result: {}", e))
        })?;
        Ok(result)
    }

    /// Get processes blocking unmount
    pub async fn get_blocking_processes(
        &self,
        device_or_mount: &str,
    ) -> Result<Vec<ProcessInfo>, ClientError> {
        let json = self.proxy.get_blocking_processes(device_or_mount).await?;
        let processes: Vec<ProcessInfo> = serde_json::from_str(&json)
            .map_err(|e| ClientError::ParseError(format!("Failed to parse process list: {}", e)))?;
        Ok(processes)
    }

    /// Check and repair a filesystem
    pub async fn check(&self, device: &str, repair: bool) -> Result<String, ClientError> {
        Ok(self.proxy.check(device, repair).await?)
    }

    /// Set filesystem label
    pub async fn set_label(&self, device: &str, label: &str) -> Result<(), ClientError> {
        Ok(self.proxy.set_label(device, label).await?)
    }

    /// Get filesystem usage statistics
    pub async fn get_usage(&self, mount_point: &str) -> Result<FilesystemUsage, ClientError> {
        let json = self.proxy.get_usage(mount_point).await?;
        let usage: FilesystemUsage = serde_json::from_str(&json)
            .map_err(|e| ClientError::ParseError(format!("Failed to parse usage stats: {}", e)))?;
        Ok(usage)
    }

    /// Get persistent mount options (fstab) for a device
    pub async fn get_mount_options(
        &self,
        device: &str,
    ) -> Result<Option<MountOptionsSettings>, ClientError> {
        let json = self.proxy.get_mount_options(device).await?;
        let opt: Option<MountOptionsSettings> = serde_json::from_str(&json).map_err(|e| {
            ClientError::ParseError(format!("Failed to parse mount options: {}", e))
        })?;
        Ok(opt)
    }

    /// Clear persistent mount options for a device
    pub async fn default_mount_options(&self, device: &str) -> Result<(), ClientError> {
        Ok(self.proxy.default_mount_options(device).await?)
    }

    /// Set persistent mount options for a device
    pub async fn edit_mount_options(
        &self,
        device: &str,
        mount_at_startup: bool,
        show_in_ui: bool,
        require_auth: bool,
        display_name: Option<&str>,
        icon_name: Option<&str>,
        symbolic_icon_name: Option<&str>,
        other_options: &str,
        mount_point: &str,
        identify_as: &str,
        filesystem_type: &str,
    ) -> Result<(), ClientError> {
        Ok(self
            .proxy
            .edit_mount_options(
                device,
                mount_at_startup,
                show_in_ui,
                require_auth,
                display_name.unwrap_or(""),
                icon_name.unwrap_or(""),
                symbolic_icon_name.unwrap_or(""),
                other_options,
                mount_point,
                identify_as,
                filesystem_type,
            )
            .await?)
    }

    /// Take ownership of a mounted filesystem
    pub async fn take_ownership(&self, device: &str, recursive: bool) -> Result<(), ClientError> {
        Ok(self.proxy.take_ownership(device, recursive).await?)
    }

    /// Get the underlying proxy for signal subscriptions
    pub fn proxy(&self) -> &FilesystemsInterfaceProxy<'static> {
        &self.proxy
    }
}
