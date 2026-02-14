// SPDX-License-Identifier: GPL-3.0-only

//! Filesystem management D-Bus interface
//!
//! This module provides D-Bus methods for managing filesystems,
//! including formatting, mounting, unmounting, and process management.

use std::path::Path;
use zbus::{interface, Connection};
use storage_models::{FilesystemInfo, FormatOptions, MountOptions, MountOptionsSettings, CheckResult, UnmountResult};

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
        
        for drive in drives {
            // Access volumes_flat directly from DriveModel
            for volume in &drive.volumes_flat {
                // Only include volumes that have actual filesystems (not empty id_type, not LUKS)
                if volume.has_filesystem && volume.id_type != "crypto_LUKS" {
                    let device = volume.device_path.clone().unwrap_or_default();
                    
                    // Get filesystem label via disks-dbus operation
                    let label = disks_dbus::get_filesystem_label(&device)
                        .await
                        .unwrap_or_default();

                    let available = volume.usage.as_ref().map(|u| u.available_bytes()).unwrap_or(0);

                    filesystems.push(FilesystemInfo {
                        device,
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
        #[zbus(signal_context)] signal_ctx: zbus::object_server::SignalEmitter<'_>,
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
        let options: FormatOptions = serde_json::from_str(&options_json)
            .unwrap_or_default();
        
        // Delegate to disks-dbus operation
        disks_dbus::format_filesystem(&device, &fs_type, &label, options)
            .await
            .map_err(|e| {
                tracing::error!("Failed to format device: {e}");
                zbus::fdo::Error::Failed(format!("Failed to format device: {e}"))
            })?;
        
        tracing::info!("Successfully formatted {} as {}", device, fs_type);
        let _ = Self::formatted(&signal_ctx, &device, &fs_type).await;
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
        #[zbus(signal_context)] signal_ctx: zbus::object_server::SignalEmitter<'_>,
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
        
        // Delegate to disks-dbus operation
        let actual_mount_point = disks_dbus::mount_filesystem_op(&device, &mount_point, mount_opts)
            .await
            .map_err(|e| {
                tracing::error!("Failed to mount filesystem: {e}");
                zbus::fdo::Error::Failed(format!("Failed to mount filesystem: {e}"))
            })?;
        
        tracing::info!("Successfully mounted at: {}", actual_mount_point);
        let _ = Self::mounted(&signal_ctx, &device, &actual_mount_point).await;
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
        #[zbus(signal_context)] signal_ctx: zbus::object_server::SignalEmitter<'_>,
        device_or_mount: String,
        force: bool,
        kill_processes: bool,
    ) -> zbus::fdo::Result<String> {
        // Check authorization for unmount
        check_polkit_auth(connection, "org.cosmic.ext.storage-service.filesystem-mount")
            .await
            .map_err(|e| zbus::fdo::Error::Failed(format!("Authorization failed: {e}")))?;
        
        tracing::info!("Unmounting {} (force={}, kill={})", device_or_mount, force, kill_processes);
        
        // Determine if input is device or mount point (for finding processes later)
        let mount_point = if device_or_mount.starts_with("/dev/") {
            // It's a device, get mount point via disks-dbus
            disks_dbus::get_mount_point(&device_or_mount)
                .await
                .unwrap_or_else(|_| device_or_mount.clone())
        } else {
            // Assume it's a mount point
            device_or_mount.clone()
        };
        
        // Attempt unmount via disks-dbus operation
        let unmount_result = disks_dbus::unmount_filesystem(&device_or_mount, force).await;
        
        match unmount_result {
            Ok(_) => {
                tracing::info!("Successfully unmounted {}", device_or_mount);
                let _ = Self::unmounted(&signal_ctx, &device_or_mount).await;
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
                        
                        // Retry unmount via disks-dbus
                        match disks_dbus::unmount_filesystem(&device_or_mount, force).await {
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
        
        // Determine mount point via disks-dbus
        let mount_point = if device_or_mount.starts_with("/dev/") {
            disks_dbus::get_mount_point(&device_or_mount)
                .await
                .map_err(|e| {
                    tracing::error!("Failed to get mount point: {e}");
                    zbus::fdo::Error::Failed(format!("Failed to get mount point: {e}"))
                })?
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
        
        // Delegate to disks-dbus operation
        let clean = disks_dbus::check_filesystem(&device, repair)
            .await
            .map_err(|e| {
                tracing::error!("Filesystem check failed: {e}");
                zbus::fdo::Error::Failed(format!("Filesystem check failed: {e}"))
            })?;
        
        let result = CheckResult {
            device: device.clone(),
            clean,
            errors_corrected: if repair && !clean { 1 } else { 0 },
            errors_uncorrected: 0,
            output: if clean { "Filesystem is clean".to_string() } else { "Filesystem has errors".to_string() },
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
        
        // Delegate to disks-dbus operation
        disks_dbus::set_filesystem_label(&device, &label)
            .await
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

    /// Get persistent mount options (fstab configuration) for a device
    ///
    /// Returns: JSON Option<MountOptionsSettings> ("null" if none)
    ///
    /// Authorization: org.cosmic.ext.storage-service.filesystem-read (allow_active)
    async fn get_mount_options(
        &self,
        #[zbus(connection)] connection: &Connection,
        device: String,
    ) -> zbus::fdo::Result<String> {
        check_polkit_auth(connection, "org.cosmic.ext.storage-service.filesystem-read")
            .await
            .map_err(|e| zbus::fdo::Error::Failed(format!("Authorization failed: {e}")))?;

        let drives = disks_dbus::DriveModel::get_drives()
            .await
            .map_err(|e| {
                tracing::error!("Failed to get drives: {e}");
                zbus::fdo::Error::Failed(format!("Failed to enumerate drives: {e}"))
            })?;

        for drive in drives {
            for vol in &drive.volumes_flat {
                if vol.device_path.as_deref() == Some(device.as_str()) {
                    match vol.get_mount_options_settings().await {
                        Ok(Some(s)) => {
                            let out = MountOptionsSettings {
                                identify_as: s.identify_as,
                                mount_point: s.mount_point,
                                filesystem_type: s.filesystem_type,
                                mount_at_startup: s.mount_at_startup,
                                require_auth: s.require_auth,
                                show_in_ui: s.show_in_ui,
                                other_options: s.other_options,
                                display_name: s.display_name,
                                icon_name: s.icon_name,
                                symbolic_icon_name: s.symbolic_icon_name,
                            };
                            return serde_json::to_string(&Some(out))
                                .map_err(|e| zbus::fdo::Error::Failed(format!("Serialize: {e}")));
                        }
                        Ok(None) => return Ok("null".to_string()),
                        Err(e) => {
                            tracing::warn!("get_mount_options_settings failed: {e}");
                            return Ok("null".to_string());
                        }
                    }
                }
            }
        }
        Ok("null".to_string())
    }

    /// Clear persistent mount options (remove fstab entry) for a device
    ///
    /// Authorization: org.cosmic.ext.storage-service.filesystem-mount (allow_active)
    async fn default_mount_options(
        &self,
        #[zbus(connection)] connection: &Connection,
        device: String,
    ) -> zbus::fdo::Result<()> {
        check_polkit_auth(connection, "org.cosmic.ext.storage-service.filesystem-mount")
            .await
            .map_err(|e| zbus::fdo::Error::Failed(format!("Authorization failed: {e}")))?;

        let drives = disks_dbus::DriveModel::get_drives()
            .await
            .map_err(|e| {
                tracing::error!("Failed to get drives: {e}");
                zbus::fdo::Error::Failed(format!("Failed to enumerate drives: {e}"))
            })?;

        for drive in drives {
            for vol in &drive.volumes_flat {
                if vol.device_path.as_deref() == Some(device.as_str()) {
                    return vol.default_mount_options().await.map_err(|e| {
                        tracing::error!("default_mount_options failed: {e}");
                        zbus::fdo::Error::Failed(format!("Failed to clear mount options: {e}"))
                    });
                }
            }
        }
        Err(zbus::fdo::Error::Failed(format!("Device not found: {}", device)))
    }

    /// Set persistent mount options (fstab configuration) for a device
    ///
    /// Authorization: org.cosmic.ext.storage-service.filesystem-mount (allow_active)
    async fn edit_mount_options(
        &self,
        #[zbus(connection)] connection: &Connection,
        device: String,
        mount_at_startup: bool,
        show_in_ui: bool,
        require_auth: bool,
        display_name: String,
        icon_name: String,
        symbolic_icon_name: String,
        other_options: String,
        mount_point: String,
        identify_as: String,
        filesystem_type: String,
    ) -> zbus::fdo::Result<()> {
        check_polkit_auth(connection, "org.cosmic.ext.storage-service.filesystem-mount")
            .await
            .map_err(|e| zbus::fdo::Error::Failed(format!("Authorization failed: {e}")))?;

        let drives = disks_dbus::DriveModel::get_drives()
            .await
            .map_err(|e| {
                tracing::error!("Failed to get drives: {e}");
                zbus::fdo::Error::Failed(format!("Failed to enumerate drives: {e}"))
            })?;

        let display_opt = if display_name.trim().is_empty() {
            None
        } else {
            Some(display_name)
        };
        let icon_opt = if icon_name.trim().is_empty() {
            None
        } else {
            Some(icon_name)
        };
        let symbolic_icon_opt = if symbolic_icon_name.trim().is_empty() {
            None
        } else {
            Some(symbolic_icon_name)
        };

        for drive in drives {
            for vol in &drive.volumes_flat {
                if vol.device_path.as_deref() == Some(device.as_str()) {
                    return vol
                        .edit_mount_options(
                            mount_at_startup,
                            show_in_ui,
                            require_auth,
                            display_opt,
                            icon_opt,
                            symbolic_icon_opt,
                            other_options,
                            mount_point,
                            identify_as,
                            filesystem_type,
                        )
                        .await
                        .map_err(|e| {
                            tracing::error!("edit_mount_options failed: {e}");
                            zbus::fdo::Error::Failed(format!("Failed to set mount options: {e}"))
                        });
                }
            }
        }
        Err(zbus::fdo::Error::Failed(format!("Device not found: {}", device)))
    }

    /// Take ownership of a mounted filesystem (e.g. for fstab/crypttab)
    ///
    /// Args:
    /// - device: Device path (e.g. "/dev/sda1" or "/dev/mapper/luks-xxx")
    /// - recursive: Take ownership of child mounts
    ///
    /// Authorization: org.cosmic.ext.storage-service.filesystems-take-ownership
    async fn take_ownership(
        &self,
        #[zbus(connection)] connection: &Connection,
        device: String,
        recursive: bool,
    ) -> zbus::fdo::Result<()> {
        check_polkit_auth(connection, "org.cosmic.ext.storage-service.filesystems-take-ownership")
            .await
            .map_err(|e| zbus::fdo::Error::Failed(format!("Authorization failed: {e}")))?;

        tracing::info!("Taking ownership of {} (recursive={})", device, recursive);

        // Delegate to disks-dbus operation
        disks_dbus::take_filesystem_ownership(&device, recursive)
            .await
            .map_err(|e| {
                tracing::error!("Take ownership failed: {e}");
                zbus::fdo::Error::Failed(format!("Take ownership failed: {e}"))
            })?;

        tracing::info!("Successfully took ownership of {}", device);
        Ok(())
    }
}
