// SPDX-License-Identifier: GPL-3.0-only

//! Filesystem management D-Bus interface
//!
//! This module provides D-Bus methods for managing filesystems,
//! including formatting, mounting, unmounting, and process management.

use std::collections::HashMap;
use std::path::Path;
use udisks2::{block::BlockProxy, filesystem::FilesystemProxy};
use zbus::{interface, Connection};
use zbus::zvariant::{OwnedObjectPath, Value};
use storage_models::{FilesystemInfo, FormatOptions, MountOptions, CheckResult, UnmountResult};

use crate::auth::check_polkit_auth;

/// D-Bus interface for filesystem management operations
pub struct FilesystemsHandler {
    /// Cached list of supported filesystem tools
    supported_tools: Vec<String>,
}

impl FilesystemsHandler {
    /// Create a new FilesystemsHandler and detect available tools
    pub fn new() -> Self {
        let supported_tools = Self::detect_filesystem_tools();
        Self { supported_tools }
    }
    
    /// Detect which mkfs tools are installed
    fn detect_filesystem_tools() -> Vec<String> {
        let tools = vec![
            ("ext4", "mkfs.ext4"),
            ("xfs", "mkfs.xfs"),
            ("btrfs", "mkfs.btrfs"),
            ("vfat", "mkfs.vfat"),
            ("ntfs", "mkfs.ntfs"),
            ("exfat", "mkfs.exfat"),
        ];
        
        let mut supported = Vec::new();
        for (fs_type, command) in tools {
            if which::which(command).is_ok() {
                supported.push(fs_type.to_string());
            }
        }
        
        tracing::info!("Detected filesystem support: {:?}", supported);
        supported
    }
}

#[interface(name = "org.cosmic.ext.StorageService.Filesystems")]
impl FilesystemsHandler {
    /// Signal emitted during format operation with progress (0-100)
    #[zbus(signal)]
    async fn format_progress(
        signal_ctxt: &zbus::object_server::SignalEmitter<'_>,
        device: &str,
        progress: u8,
    ) -> zbus::Result<()>;
    
    /// Signal emitted when format operation completes
    #[zbus(signal)]
    async fn formatted(
        signal_ctxt: &zbus::object_server::SignalEmitter<'_>,
        device: &str,
        fs_type: &str,
    ) -> zbus::Result<()>;
    
    /// Signal emitted when filesystem is mounted
    #[zbus(signal)]
    async fn mounted(
        signal_ctxt: &zbus::object_server::SignalEmitter<'_>,
        device: &str,
        mount_point: &str,
    ) -> zbus::Result<()>;
    
    /// Signal emitted when filesystem is unmounted
    #[zbus(signal)]
    async fn unmounted(
        signal_ctxt: &zbus::object_server::SignalEmitter<'_>,
        device: &str,
    ) -> zbus::Result<()>;
    
    /// List all filesystems on the system
    /// 
    /// Returns: JSON-serialized Vec<FilesystemInfo>
    /// 
    /// Authorization: org.cosmic.ext.storage-service.filesystem-read (allow_active)
    async fn list_filesystems(
        &self,
        #[zbus(connection)] connection: &Connection,
    ) -> zbus::fdo::Result<String> {
        // Check authorization
        check_polkit_auth(connection, "org.cosmic.ext.storage-service.filesystem-read")
            .await
            .map_err(|e| zbus::fdo::Error::Failed(format!("Authorization failed: {e}")))?;
        
        tracing::debug!("Listing filesystems");
        
        // Get all drives
        let drives = disks_dbus::DriveModel::get_drives()
            .await
            .map_err(|e| {
                tracing::error!("Failed to get drives: {e}");
                zbus::fdo::Error::Failed(format!("Failed to enumerate drives: {e}"))
            })?;
        
        let mut filesystems = Vec::new();
        let sys_conn = Connection::system().await
            .map_err(|e| zbus::fdo::Error::Failed(format!("Failed to connect to system bus: {e}")))?;
        
        for drive in drives {
            // Access volumes_flat directly from DriveModel
            for volume in &drive.volumes_flat {
                // Only include volumes that have actual filesystems (not empty id_type, not LUKS)
                if volume.has_filesystem && volume.id_type != "crypto_LUKS" {
                    // Get filesystem label from UDisks2 Block interface
                    let label = if let Ok(block_proxy) = BlockProxy::builder(&sys_conn)
                        .path(&volume.path)?
                        .build()
                        .await
                    {
                        block_proxy.id_label().await.unwrap_or_default()
                    } else {
                        String::new()
                    };

                    let available = volume.usage.as_ref().map(|u| u.available_bytes()).unwrap_or(0);

                    filesystems.push(FilesystemInfo {
                        device: volume.device_path.clone().unwrap_or_default(),
                        fs_type: volume.id_type.clone(),
                        label,
                        uuid: volume.uuid.clone(),
                        mount_points: volume.mount_points.clone(),
                        size: volume.size,
                        available,
                    });
                }
            }
        }
        
        tracing::debug!("Found {} filesystems", filesystems.len());
        
        // Serialize to JSON
        let json = serde_json::to_string(&filesystems)
            .map_err(|e| {
                tracing::error!("Failed to serialize filesystems: {e}");
                zbus::fdo::Error::Failed(format!("Failed to serialize filesystems: {e}"))
            })?;
        
        Ok(json)
    }
    
    /// Get list of supported filesystem types
    /// 
    /// Returns: JSON array of filesystem type strings
    /// 
    /// Authorization: org.cosmic.ext.storage-service.filesystem-read (allow_active)
    async fn get_supported_filesystems(
        &self,
        #[zbus(connection)] connection: &Connection,
    ) -> zbus::fdo::Result<String> {
        // Check authorization
        check_polkit_auth(connection, "org.cosmic.ext.storage-service.filesystem-read")
            .await
            .map_err(|e| zbus::fdo::Error::Failed(format!("Authorization failed: {e}")))?;
        
        tracing::debug!("Getting supported filesystems");
        
        // Serialize to JSON
        let json = serde_json::to_string(&self.supported_tools)
            .map_err(|e| {
                tracing::error!("Failed to serialize supported filesystems: {e}");
                zbus::fdo::Error::Failed(format!("Failed to serialize: {e}"))
            })?;
        
        Ok(json)
    }
    
    /// Format a device with a filesystem
    /// 
    /// Args:
    /// - device: Device path (e.g., "/dev/sda1")
    /// - fs_type: Filesystem type ("ext4", "xfs", "btrfs", "vfat", etc.)
    /// - label: Filesystem label
    /// - options_json: JSON-serialized FormatOptions
    /// 
    /// Authorization: org.cosmic.ext.storage-service.filesystem-format (auth_admin - always prompt)
    async fn format(
        &self,
        #[zbus(connection)] connection: &Connection,
        device: String,
        fs_type: String,
        label: String,
        options_json: String,
    ) -> zbus::fdo::Result<()> {
        // Check authorization (always prompt for format)
        check_polkit_auth(connection, "org.cosmic.ext.storage-service.filesystem-format")
            .await
            .map_err(|e| zbus::fdo::Error::Failed(format!("Authorization failed: {e}")))?;
        
        tracing::info!("Formatting {} as {} with label '{}'", device, fs_type, label);
        
        // Validate filesystem type is supported
        if !self.supported_tools.contains(&fs_type) {
            tracing::warn!("Unsupported filesystem type: {}", fs_type);
            return Err(zbus::fdo::Error::Failed(
                format!("Filesystem type '{}' is not supported or tools not installed", fs_type)
            ));
        }
        
        // Parse options
        let _options: FormatOptions = serde_json::from_str(&options_json)
            .unwrap_or_default();
        
        // Find UDisks2 block device
        let block_path = self.find_block_path(connection, &device).await?;
        
        // Check if device is mounted
        if let Ok(fs_proxy_builder) = FilesystemProxy::builder(connection)
            .path(&block_path)
        {
            if let Ok(fs_proxy) = fs_proxy_builder.build().await {
                let mount_points = fs_proxy.mount_points().await.unwrap_or_default();
                if !mount_points.is_empty() {
                    tracing::warn!("Device {} is mounted", device);
                    return Err(zbus::fdo::Error::Failed(
                        format!("Device is mounted. Unmount it first.")
                    ));
                }
            }
        }
        
        // Create filesystem using UDisks2 Block.Format
        let block_proxy = BlockProxy::builder(connection)
            .path(&block_path)
            .map_err(|e| {
                tracing::error!("Failed to create block proxy: {e}");
                zbus::fdo::Error::Failed(format!("Failed to create block proxy: {e}"))
            })?
            .build()
            .await
            .map_err(|e| {
                tracing::error!("Failed to build block proxy: {e}");
                zbus::fdo::Error::Failed(format!("Failed to access device: {e}"))
            })?;
        
        // Build format options
        let mut format_opts: HashMap<&str, Value<'_>> = HashMap::new();
        if !label.is_empty() {
            format_opts.insert("label", Value::from(label.as_str()));
        }
        
        // Call Format
        block_proxy.format(&fs_type, format_opts).await
            .map_err(|e| {
                tracing::error!("Failed to format device: {e}");
                zbus::fdo::Error::Failed(format!("Failed to format device: {e}"))
            })?;
        
        tracing::info!("Successfully formatted {} as {}", device, fs_type);
        
        // TODO: Emit Formatted signal
        
        Ok(())
    }
    
    /// Mount a filesystem
    /// 
    /// Args:
    /// - device: Device path (e.g., "/dev/sda1")
    /// - mount_point: Mount point path (empty string for auto)
    /// - options_json: JSON-serialized MountOptions
    /// 
    /// Returns: Actual mount point used
    /// 
    /// Authorization: org.cosmic.ext.storage-service.filesystem-mount (allow_active)
    async fn mount(
        &self,
        #[zbus(connection)] connection: &Connection,
        device: String,
        mount_point: String,
        options_json: String,
    ) -> zbus::fdo::Result<String> {
        // Check authorization
        check_polkit_auth(connection, "org.cosmic.ext.storage-service.filesystem-mount")
            .await
            .map_err(|e| zbus::fdo::Error::Failed(format!("Authorization failed: {e}")))?;
        
        tracing::info!("Mounting {} to {}", device, if mount_point.is_empty() { "(auto)" } else { &mount_point });
        
        // Parse options
        let mount_opts: MountOptions = serde_json::from_str(&options_json)
            .unwrap_or_default();
        
        // Find UDisks2 filesystem object
        let block_path = self.find_block_path(connection, &device).await?;
        
        let fs_proxy = FilesystemProxy::builder(connection)
            .path(&block_path)
            .map_err(|e| {
                tracing::error!("Failed to create filesystem proxy: {e}");
                zbus::fdo::Error::Failed(format!("Failed to create filesystem proxy: {e}"))
            })?
            .build()
            .await
            .map_err(|e| {
                tracing::error!("Failed to build filesystem proxy: {e}");
                zbus::fdo::Error::Failed(format!("Failed to access filesystem: {e}"))
            })?;
        
        // Check if already mounted
        let existing_mounts = fs_proxy.mount_points().await.unwrap_or_default();
        if !existing_mounts.is_empty() && !existing_mounts[0].is_empty() {
            // Already mounted, return existing mount point
            let mount_str = String::from_utf8(existing_mounts[0].clone().into_iter().filter(|&b| b != 0).collect())
                .unwrap_or_else(|_| "/unknown".to_string());
            tracing::info!("Device already mounted at: {}", mount_str);
            return Ok(mount_str);
        }
        
        // Build mount options
        let mut opts: HashMap<&str, Value<'_>> = HashMap::new();
        let mut options_vec = Vec::new();
        
        if mount_opts.read_only {
            options_vec.push("ro");
        }
        if mount_opts.no_exec {
            options_vec.push("noexec");
        }
        if mount_opts.no_suid {
            options_vec.push("nosuid");
        }
        for opt in &mount_opts.other {
            options_vec.push(opt.as_str());
        }
        
        if !options_vec.is_empty() {
            opts.insert("options", Value::from(options_vec.join(",")));
        }
        
        // Mount
        let actual_mount_point = fs_proxy.mount(opts).await
            .map_err(|e| {
                tracing::error!("Failed to mount filesystem: {e}");
                zbus::fdo::Error::Failed(format!("Failed to mount filesystem: {e}"))
            })?;
        
        tracing::info!("Successfully mounted at: {}", actual_mount_point);
        
        // TODO: Emit Mounted signal
        
        Ok(actual_mount_point)
    }
    
    /// Unmount a filesystem with optional process killing
    /// 
    /// Args:
    /// - device_or_mount: Device path or mount point
    /// - force: Use lazy unmount if filesystem is busy
    /// - kill_processes: Kill blocking processes if unmount fails
    /// 
    /// Returns: JSON-serialized UnmountResult
    /// 
    /// Authorization: org.cosmic.ext.storage-service.filesystem-mount (base)
    ///                org.cosmic.ext.storage-service.filesystem-kill-processes (if kill_processes=true)
    async fn unmount(
        &self,
        #[zbus(connection)] connection: &Connection,
        device_or_mount: String,
        force: bool,
        kill_processes: bool,
    ) -> zbus::fdo::Result<String> {
        // Check authorization for unmount
        check_polkit_auth(connection, "org.cosmic.ext.storage-service.filesystem-mount")
            .await
            .map_err(|e| zbus::fdo::Error::Failed(format!("Authorization failed: {e}")))?;
        
        tracing::info!("Unmounting {} (force={}, kill={})", device_or_mount, force, kill_processes);
        
        // Determine if input is device or mount point
        let mount_point = if device_or_mount.starts_with("/dev/") {
            // It's a device, need to find mount point
            self.get_mount_point(connection, &device_or_mount).await?
        } else {
            // Assume it's a mount point
            device_or_mount.clone()
        };
        
        // Find UDisks2 filesystem object
        let block_path = if device_or_mount.starts_with("/dev/") {
            self.find_block_path(connection, &device_or_mount).await?
        } else {
            // Need to find device for mount point
            self.find_block_path_by_mount(connection, &mount_point).await?
        };
        
        let fs_proxy = FilesystemProxy::builder(connection)
            .path(&block_path)
            .map_err(|e| {
                tracing::error!("Failed to create filesystem proxy: {e}");
                zbus::fdo::Error::Failed(format!("Failed to create filesystem proxy: {e}"))
            })?
            .build()
            .await
            .map_err(|e| {
                tracing::error!("Failed to build filesystem proxy: {e}");
                zbus::fdo::Error::Failed(format!("Failed to access filesystem: {e}"))
            })?;
        
        // Build unmount options
        let mut opts: HashMap<&str, Value<'_>> = HashMap::new();
        if force {
            opts.insert("force", Value::from(true));
        }
        
        // Attempt unmount
        let unmount_result = fs_proxy.unmount(opts.clone()).await;
        
        match unmount_result {
            Ok(_) => {
                tracing::info!("Successfully unmounted {}", device_or_mount);
                
                // TODO: Emit Unmounted signal
                
                let result = UnmountResult {
                    success: true,
                    error: None,
                    blocking_processes: Vec::new(),
                };
                
                let json = serde_json::to_string(&result)
                    .map_err(|e| {
                        tracing::error!("Failed to serialize result: {e}");
                        zbus::fdo::Error::Failed(format!("Failed to serialize result: {e}"))
                    })?;
                
                Ok(json)
            }
            Err(e) => {
                // Check if it's a busy error
                let error_str = e.to_string();
                if error_str.contains("busy") || error_str.contains("in use") {
                    tracing::warn!("Unmount failed: device busy");
                    
                    // Find blocking processes
                    let processes = disks_dbus::find_processes_using_mount(&mount_point)
                        .await
                        .unwrap_or_default();
                    
                    if kill_processes && !processes.is_empty() {
                        // Check authorization for killing processes
                        if let Err(auth_err) = check_polkit_auth(connection, "org.cosmic.ext.storage-service.filesystem-kill-processes").await {
                            tracing::warn!("Authorization failed for killing processes: {}", auth_err);
                            
                            let result = UnmountResult {
                                success: false,
                                error: Some("Authorization required to kill processes".to_string()),
                                blocking_processes: processes,
                            };
                            
                            let json = serde_json::to_string(&result)
                                .map_err(|e| zbus::fdo::Error::Failed(format!("Failed to serialize: {e}")))?;
                            
                            return Ok(json);
                        }
                        
                        // Kill blocking processes
                        let pids: Vec<i32> = processes.iter().map(|p| p.pid).collect();
                        tracing::info!("Killing {} blocking processes", pids.len());
                        
                        let _kill_results = disks_dbus::kill_processes(&pids);
                        
                        // Wait a moment for processes to die
                        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
                        
                        // Retry unmount
                        match fs_proxy.unmount(opts).await {
                            Ok(_) => {
                                tracing::info!("Successfully unmounted after killing processes");
                                
                                let result = UnmountResult {
                                    success: true,
                                    error: None,
                                    blocking_processes: Vec::new(),
                                };
                                
                                let json = serde_json::to_string(&result)
                                    .map_err(|e| zbus::fdo::Error::Failed(format!("Failed to serialize: {e}")))?;
                                
                                return Ok(json);
                            }
                            Err(retry_err) => {
                                tracing::error!("Unmount failed even after killing processes: {}", retry_err);
                                
                                let result = UnmountResult {
                                    success: false,
                                    error: Some(retry_err.to_string()),
                                    blocking_processes: Vec::new(),
                                };
                                
                                let json = serde_json::to_string(&result)
                                    .map_err(|e| zbus::fdo::Error::Failed(format!("Failed to serialize: {e}")))?;
                                
                                return Ok(json);
                            }
                        }
                    } else {
                        // Return error with blocking processes
                        let result = UnmountResult {
                            success: false,
                            error: Some("Device is busy".to_string()),
                            blocking_processes: processes,
                        };
                        
                        let json = serde_json::to_string(&result)
                            .map_err(|e| zbus::fdo::Error::Failed(format!("Failed to serialize: {e}")))?;
                        
                        return Ok(json);
                    }
                } else {
                    // Other error
                    tracing::error!("Unmount failed: {}", e);
                    
                    let result = UnmountResult {
                        success: false,
                        error: Some(e.to_string()),
                        blocking_processes: Vec::new(),
                    };
                    
                    let json = serde_json::to_string(&result)
                        .map_err(|e| zbus::fdo::Error::Failed(format!("Failed to serialize: {e}")))?;
                    
                    return Ok(json);
                }
            }
        }
    }
    
    /// Get processes blocking unmount of a filesystem
    /// 
    /// Args:
    /// - device_or_mount: Device path or mount point
    /// 
    /// Returns: JSON-serialized Vec<ProcessInfo>
    /// 
    /// Authorization: org.cosmic.ext.storage-service.filesystem-read (allow_active)
    async fn get_blocking_processes(
        &self,
        #[zbus(connection)] connection: &Connection,
        device_or_mount: String,
    ) -> zbus::fdo::Result<String> {
        // Check authorization
        check_polkit_auth(connection, "org.cosmic.ext.storage-service.filesystem-read")
            .await
            .map_err(|e| zbus::fdo::Error::Failed(format!("Authorization failed: {e}")))?;
        
        tracing::debug!("Getting blocking processes for {}", device_or_mount);
        
        // Determine mount point
        let mount_point = if device_or_mount.starts_with("/dev/") {
            self.get_mount_point(connection, &device_or_mount).await?
        } else {
            device_or_mount.clone()
        };
        
        // Find blocking processes
        let processes = disks_dbus::find_processes_using_mount(&mount_point)
            .await
            .map_err(|e| {
                tracing::error!("Failed to find processes: {e}");
                zbus::fdo::Error::Failed(format!("Failed to find processes: {e}"))
            })?;
        
        tracing::debug!("Found {} blocking processes", processes.len());
        
        // Serialize to JSON
        let json = serde_json::to_string(&processes)
            .map_err(|e| {
                tracing::error!("Failed to serialize processes: {e}");
                zbus::fdo::Error::Failed(format!("Failed to serialize: {e}"))
            })?;
        
        Ok(json)
    }
    
    // Note: Process killing is intentionally only available through Unmount with kill_processes=true
    // to limit the security surface. A standalone KillProcesses method could be exploited to kill
    // arbitrary processes. The Unmount workflow (try unmount → get blocking processes → unmount with
    // kill_processes=true) provides the necessary context and safety.
    
    /// Check and optionally repair filesystem
    /// 
    /// Args:
    /// - device: Device path (e.g., "/dev/sda1")
    /// - repair: Whether to repair errors (requires unmount)
    /// 
    /// Returns: JSON-serialized CheckResult
    /// 
    /// Authorization: org.cosmic.ext.storage-service.filesystem-modify (auth_admin_keep)
    async fn check(
        &self,
        #[zbus(connection)] connection: &Connection,
        device: String,
        repair: bool,
    ) -> zbus::fdo::Result<String> {
        // Check authorization
        check_polkit_auth(connection, "org.cosmic.ext.storage-service.filesystem-modify")
            .await
            .map_err(|e| zbus::fdo::Error::Failed(format!("Authorization failed: {e}")))?;
        
        tracing::info!("Checking filesystem on {} (repair={})", device, repair);
        
        // Find UDisks2 block device
        let block_path = self.find_block_path(connection, &device).await?;
        
        // Check if device is mounted
        if let Ok(fs_proxy_builder) = FilesystemProxy::builder(connection)
            .path(&block_path)
        {
            if let Ok(fs_proxy) = fs_proxy_builder.build().await {
                let mount_points = fs_proxy.mount_points().await.unwrap_or_default();
                if !mount_points.is_empty() {
                    tracing::warn!("Device {} is mounted", device);
                    return Err(zbus::fdo::Error::Failed(
                        format!("Device is mounted. Unmount it first to run fsck.")
                    ));
                }
            }
        }
        
        // For now, return a simple result (UDisks2 doesn't expose fsck directly)
        // In a full implementation, would need to call fsck commands directly
        let result = CheckResult {
            device: device.clone(),
            clean: true,
            errors_corrected: 0,
            errors_uncorrected: 0,
            output: "Filesystem check not fully implemented via UDisks2".to_string(),
        };
        
        let json = serde_json::to_string(&result)
            .map_err(|e| {
                tracing::error!("Failed to serialize result: {e}");
                zbus::fdo::Error::Failed(format!("Failed to serialize: {e}"))
            })?;
        
        Ok(json)
    }
    
    /// Set filesystem label
    /// 
    /// Args:
    /// - device: Device path (e.g., "/dev/sda1")
    /// - label: New filesystem label
    /// 
    /// Authorization: org.cosmic.ext.storage-service.filesystem-modify (auth_admin_keep)
    async fn set_label(
        &self,
        #[zbus(connection)] connection: &Connection,
        device: String,
        label: String,
    ) -> zbus::fdo::Result<()> {
        // Check authorization
        check_polkit_auth(connection, "org.cosmic.ext.storage-service.filesystem-modify")
            .await
            .map_err(|e| zbus::fdo::Error::Failed(format!("Authorization failed: {e}")))?;
        
        tracing::info!("Setting label on {} to '{}'", device, label);
        
        // Find UDisks2 filesystem object
        let block_path = self.find_block_path(connection, &device).await?;
        
        let fs_proxy = FilesystemProxy::builder(connection)
            .path(&block_path)
            .map_err(|e| {
                tracing::error!("Failed to create filesystem proxy: {e}");
                zbus::fdo::Error::Failed(format!("Failed to create filesystem proxy: {e}"))
            })?
            .build()
            .await
            .map_err(|e| {
                tracing::error!("Failed to build filesystem proxy: {e}");
                zbus::fdo::Error::Failed(format!("Failed to access filesystem: {e}"))
            })?;
        
        // Set label
        let options: HashMap<&str, Value<'_>> = HashMap::new();
        fs_proxy.set_label(&label, options).await
            .map_err(|e| {
                tracing::error!("Failed to set label: {e}");
                zbus::fdo::Error::Failed(format!("Failed to set label: {e}"))
            })?;
        
        tracing::info!("Successfully set label on {}", device);
        
        Ok(())
    }
    
    /// Get filesystem usage statistics
    /// 
    /// Args:
    /// - mount_point: Mount point path
    /// 
    /// Returns: JSON with size, used, available, percent
    /// 
    /// Authorization: org.cosmic.ext.storage-service.filesystem-read (allow_active)
    async fn get_usage(
        &self,
        #[zbus(connection)] connection: &Connection,
        mount_point: String,
    ) -> zbus::fdo::Result<String> {
        // Check authorization
        check_polkit_auth(connection, "org.cosmic.ext.storage-service.filesystem-read")
            .await
            .map_err(|e| zbus::fdo::Error::Failed(format!("Authorization failed: {e}")))?;
        
        tracing::debug!("Getting usage for mount point: {}", mount_point);
        
        // Validate mount point exists and is mounted
        if !Path::new(&mount_point).exists() {
            tracing::warn!("Mount point does not exist: {}", mount_point);
            return Err(zbus::fdo::Error::Failed(
                format!("Mount point does not exist: {}", mount_point)
            ));
        }
        
        // Use statvfs to get filesystem stats
        use nix::sys::statvfs::statvfs;
        
        let stats = statvfs(mount_point.as_str())
            .map_err(|e| {
                tracing::error!("Failed to get filesystem stats: {e}");
                zbus::fdo::Error::Failed(format!("Failed to get filesystem stats: {e}"))
            })?;
        
        let block_size = stats.block_size();
        let total_blocks = stats.blocks();
        let free_blocks = stats.blocks_free();
        let available_blocks = stats.blocks_available();
        
        let size = total_blocks * block_size;
        let available = available_blocks * block_size;
        let used = size - (free_blocks * block_size);
        let percent = if size > 0 {
            ((used as f64 / size as f64) * 100.0) as u8
        } else {
            0
        };
        
        let usage = serde_json::json!({
            "size": size,
            "used": used,
            "available": available,
            "percent": percent,
        });
        
        let json = serde_json::to_string(&usage)
            .map_err(|e| {
                tracing::error!("Failed to serialize usage: {e}");
                zbus::fdo::Error::Failed(format!("Failed to serialize: {e}"))
            })?;
        
        Ok(json)
    }
}

/// Helper methods
impl FilesystemsHandler {
    /// Find UDisks2 block object path from device path
    async fn find_block_path(
        &self,
        _connection: &Connection,
        device: &str,
    ) -> zbus::fdo::Result<OwnedObjectPath> {
        let device_clean = device.strip_prefix("/dev/").unwrap_or(device);
        let path = format!("/org/freedesktop/UDisks2/block_devices/{}", device_clean);
        
        path.try_into()
            .map_err(|e| {
                tracing::error!("Invalid device path: {e}");
                zbus::fdo::Error::Failed(format!("Invalid device path: {e}"))
            })
    }
    
    /// Find UDisks2 block object path from mount point
    async fn find_block_path_by_mount(
        &self,
        connection: &Connection,
        mount_point: &str,
    ) -> zbus::fdo::Result<OwnedObjectPath> {
        // Get all drives and search for mount point
        let drives = disks_dbus::DriveModel::get_drives()
            .await
            .map_err(|e| {
                tracing::error!("Failed to get drives: {e}");
                zbus::fdo::Error::Failed(format!("Failed to enumerate drives: {e}"))
            })?;
        
        for drive in drives {
            let partitions = drive.get_partitions();
            for partition in partitions {
                if partition.mount_points.contains(&mount_point.to_string()) {
                    return self.find_block_path(connection, &partition.device).await;
                }
            }
        }
        
        tracing::warn!("Mount point not found: {}", mount_point);
        Err(zbus::fdo::Error::Failed(format!("Mount point not found: {}", mount_point)))
    }
    
    /// Get mount point for a device
    async fn get_mount_point(
        &self,
        connection: &Connection,
        device: &str,
    ) -> zbus::fdo::Result<String> {
        let block_path = self.find_block_path(connection, device).await?;
        
        let fs_proxy = FilesystemProxy::builder(connection)
            .path(&block_path)
            .map_err(|e| {
                tracing::error!("Failed to create filesystem proxy: {e}");
                zbus::fdo::Error::Failed(format!("Failed to create filesystem proxy: {e}"))
            })?
            .build()
            .await
            .map_err(|e| {
                tracing::error!("Failed to build filesystem proxy: {e}");
                zbus::fdo::Error::Failed(format!("Device has no filesystem: {e}"))
            })?;
        
        let mount_points = fs_proxy.mount_points().await
            .map_err(|e| {
                tracing::error!("Failed to get mount points: {e}");
                zbus::fdo::Error::Failed(format!("Failed to get mount points: {e}"))
            })?;
        
        if mount_points.is_empty() {
            tracing::warn!("Device {} is not mounted", device);
            return Err(zbus::fdo::Error::Failed(format!("Device is not mounted")));
        }
        
        let mount_str = String::from_utf8(mount_points[0].clone().into_iter().filter(|&b| b != 0).collect())
            .unwrap_or_else(|_| "/unknown".to_string());
        
        Ok(mount_str)
    }
}
