// SPDX-License-Identifier: GPL-3.0-only

//! Filesystem management D-Bus interface
//!
//! This module provides D-Bus methods for managing filesystems,
//! including formatting, mounting, unmounting, and process management.

use std::ffi::{CStr, CString};
use std::fs;
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::time::Duration;
use storage_common::{
    CheckResult, FilesystemInfo, FilesystemToolInfo, FormatOptions, MountOptions,
    MountOptionsSettings, UnmountResult, UsageDeleteFailure, UsageDeleteResult,
    UsageScanParallelismPreset, UsageScanResult,
};
use storage_dbus::DiskManager;
use storage_service_macros::authorized_interface;
use zbus::message::Header as MessageHeader;
use zbus::{Connection, interface};

/// D-Bus interface for filesystem management operations
pub struct FilesystemsHandler {
    /// Cached list of supported filesystem tools (simple list for backward compat)
    supported_tools: Vec<String>,
    /// Detailed filesystem tool information
    filesystem_tools: Vec<FilesystemToolInfo>,
    /// DiskManager for disk enumeration (cached connection)
    manager: DiskManager,
}

impl FilesystemsHandler {
    /// Create a new FilesystemsHandler and detect available tools
    pub async fn new() -> Self {
        let filesystem_tools = Self::detect_all_filesystem_tools();
        let supported_tools: Vec<String> = filesystem_tools
            .iter()
            .filter(|t| t.available)
            .map(|t| t.fs_type.clone())
            .collect();
        let manager = DiskManager::new()
            .await
            .expect("Failed to create DiskManager");
        Self {
            supported_tools,
            filesystem_tools,
            manager,
        }
    }

    /// Detect all filesystem tools with detailed information
    fn detect_all_filesystem_tools() -> Vec<FilesystemToolInfo> {
        let tools = vec![
            ("ext4", "EXT4", "mkfs.ext4", "e2fsprogs"),
            ("xfs", "XFS", "mkfs.xfs", "xfsprogs"),
            ("btrfs", "Btrfs", "mkfs.btrfs", "btrfs-progs"),
            ("vfat", "FAT32", "mkfs.vfat", "dosfstools"),
            ("ntfs", "NTFS", "mkfs.ntfs", "ntfs-3g"),
            ("exfat", "exFAT", "mkfs.exfat", "exfat-utils"),
        ];

        let mut results = Vec::new();
        for (fs_type, fs_name, command, package_hint) in tools {
            let available = which::which(command).is_ok();
            results.push(FilesystemToolInfo {
                fs_type: fs_type.to_string(),
                fs_name: fs_name.to_string(),
                command: command.to_string(),
                package_hint: package_hint.to_string(),
                available,
            });
        }

        tracing::info!(
            "Detected filesystem tools: {:?}",
            results
                .iter()
                .filter(|t| t.available)
                .map(|t| &t.fs_type)
                .collect::<Vec<_>>()
        );
        results
    }

    /// Legacy method for backward compatibility
    #[allow(dead_code)]
    fn detect_filesystem_tools() -> Vec<String> {
        let tools = Self::detect_all_filesystem_tools();
        tools
            .iter()
            .filter(|t| t.available)
            .map(|t| t.fs_type.clone())
            .collect()
    }
}

fn current_process_groups() -> Vec<u32> {
    let mut gids = vec![unsafe { libc::getegid() } as u32];

    let group_count = unsafe { libc::getgroups(0, std::ptr::null_mut()) };
    if group_count > 0 {
        let mut groups = vec![0 as libc::gid_t; group_count as usize];
        let read_count = unsafe { libc::getgroups(group_count, groups.as_mut_ptr()) };
        if read_count > 0 {
            groups.truncate(read_count as usize);
            gids.extend(groups.into_iter().map(|gid| gid as u32));
        }
    }

    gids.sort_unstable();
    gids.dedup();
    gids
}

fn resolve_caller_groups(uid: u32, username: Option<&str>) -> Vec<u32> {
    let process_uid = unsafe { libc::geteuid() } as u32;
    if uid == process_uid {
        return current_process_groups();
    }

    let mut pwd = std::mem::MaybeUninit::<libc::passwd>::uninit();
    let mut pwd_ptr: *mut libc::passwd = std::ptr::null_mut();
    let mut buffer = vec![0_u8; 4096];

    let lookup_result = unsafe {
        libc::getpwuid_r(
            uid,
            pwd.as_mut_ptr(),
            buffer.as_mut_ptr() as *mut libc::c_char,
            buffer.len(),
            &mut pwd_ptr,
        )
    };

    if lookup_result != 0 || pwd_ptr.is_null() {
        tracing::warn!("Failed to resolve passwd entry for UID {}", uid);
        return Vec::new();
    }

    let passwd = unsafe { pwd.assume_init() };
    let primary_gid = passwd.pw_gid;
    let username_cstr = if let Some(name) = username {
        CString::new(name).ok()
    } else {
        unsafe { Some(CStr::from_ptr(passwd.pw_name).to_owned()) }
    };

    let Some(username_cstr) = username_cstr else {
        tracing::warn!("Failed to construct username for UID {}", uid);
        return vec![primary_gid as u32];
    };

    let mut ngroups = 32_i32;
    let mut groups = vec![0 as libc::gid_t; ngroups as usize];

    let result = unsafe {
        libc::getgrouplist(
            username_cstr.as_ptr(),
            primary_gid,
            groups.as_mut_ptr(),
            &mut ngroups,
        )
    };

    if result == -1 {
        if ngroups <= 0 {
            return vec![primary_gid as u32];
        }

        groups.resize(ngroups as usize, 0);
        let retry = unsafe {
            libc::getgrouplist(
                username_cstr.as_ptr(),
                primary_gid,
                groups.as_mut_ptr(),
                &mut ngroups,
            )
        };

        if retry == -1 {
            return vec![primary_gid as u32];
        }
    }

    groups.truncate(ngroups.max(0) as usize);
    let mut gids: Vec<u32> = groups.into_iter().map(|gid| gid as u32).collect();
    gids.push(primary_gid as u32);
    gids.sort_unstable();
    gids.dedup();
    gids
}

fn is_owned_tree(path: &Path, uid: u32) -> std::io::Result<bool> {
    let metadata = fs::symlink_metadata(path)?;
    if metadata.uid() != uid {
        return Ok(false);
    }

    if metadata.is_dir() {
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            if !is_owned_tree(&entry.path(), uid)? {
                return Ok(false);
            }
        }
    }

    Ok(true)
}

fn caller_can_unlink(path: &Path, uid: u32, caller_gids: &[u32]) -> std::io::Result<bool> {
    let Some(parent) = path.parent() else {
        return Ok(false);
    };

    let metadata = fs::symlink_metadata(parent)?;
    let mode = metadata.mode();

    if metadata.uid() == uid {
        return Ok(mode & 0o300 == 0o300);
    }

    if caller_gids.contains(&metadata.gid()) {
        return Ok(mode & 0o030 == 0o030);
    }

    Ok(mode & 0o003 == 0o003)
}

fn path_requires_admin_delete(path: &Path, caller_uid: u32, caller_gids: &[u32]) -> bool {
    let metadata = match fs::symlink_metadata(path) {
        Ok(metadata) => metadata,
        Err(_) => return true,
    };

    if !metadata.is_file() {
        return false;
    }

    if metadata.uid() != caller_uid {
        return true;
    }

    !matches!(caller_can_unlink(path, caller_uid, caller_gids), Ok(true))
}

fn map_parallelism_threads(preset: UsageScanParallelismPreset, cpu_count: usize) -> usize {
    let cpus = cpu_count.max(1);
    match preset {
        UsageScanParallelismPreset::Low => cpus.div_ceil(4).max(1),
        UsageScanParallelismPreset::Balanced => cpus.div_ceil(2).max(1),
        UsageScanParallelismPreset::High => cpus,
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

    /// Signal emitted during usage scan with processed and estimated total bytes.
    #[zbus(signal)]
    async fn usage_scan_progress(
        signal_ctxt: &zbus::object_server::SignalEmitter<'_>,
        scan_id: &str,
        processed_bytes: u64,
        estimated_total_bytes: u64,
    ) -> zbus::Result<()>;

    /// List all filesystems on the system
    ///
    /// Returns: JSON-serialized Vec<FilesystemInfo>
    ///
    /// Authorization: org.cosmic.ext.storage-service.filesystem-read (allow_active)
    #[authorized_interface(action = "org.cosmic.ext.storage-service.filesystem-read")]
    async fn list_filesystems(
        &self,
        #[zbus(connection)] _connection: &Connection,
        #[zbus(header)] _header: MessageHeader<'_>,
    ) -> zbus::fdo::Result<String> {
        // `caller` is injected by the macro
        tracing::debug!("Listing filesystems for UID {}", caller.uid);

        // Get all drives using cached connection
        let drives = storage_dbus::disk::get_disks_with_volumes(&self.manager)
            .await
            .map_err(|e| {
                tracing::error!("Failed to get drives: {e}");
                zbus::fdo::Error::Failed(format!("Failed to enumerate drives: {e}"))
            })?;

        let mut filesystems = Vec::new();

        fn collect_volumes(volumes: &[storage_common::VolumeInfo], out: &mut Vec<FilesystemInfo>) {
            for volume in volumes {
                if volume.has_filesystem
                    && volume.id_type != "crypto_LUKS"
                    && let Some(ref device) = volume.device_path
                {
                    let available = volume
                        .usage
                        .as_ref()
                        .map(|u| u.available_bytes())
                        .unwrap_or(0);
                    out.push(FilesystemInfo {
                        device: device.clone(),
                        fs_type: volume.id_type.clone(),
                        label: String::new(), // filled below
                        uuid: String::new(),
                        mount_points: volume.mount_points.clone(),
                        size: volume.size,
                        available,
                    });
                }
                collect_volumes(&volume.children, out);
            }
        }

        for (_disk_info, volumes) in &drives {
            let mut batch = Vec::new();
            collect_volumes(volumes, &mut batch);
            for mut fs in batch {
                fs.label = storage_dbus::get_filesystem_label(&fs.device)
                    .await
                    .unwrap_or_default();
                filesystems.push(fs);
            }
        }

        tracing::debug!("Found {} filesystems", filesystems.len());

        // Serialize to JSON
        let json = serde_json::to_string(&filesystems).map_err(|e| {
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
    #[authorized_interface(action = "org.cosmic.ext.storage-service.filesystem-read")]
    async fn get_supported_filesystems(
        &self,
        #[zbus(connection)] _connection: &Connection,
        #[zbus(header)] _header: MessageHeader<'_>,
    ) -> zbus::fdo::Result<String> {
        tracing::debug!("Getting supported filesystems for UID {}", caller.uid);

        // Serialize to JSON
        let json = serde_json::to_string(&self.supported_tools).map_err(|e| {
            tracing::error!("Failed to serialize supported filesystems: {e}");
            zbus::fdo::Error::Failed(format!("Failed to serialize: {e}"))
        })?;

        Ok(json)
    }

    /// Get detailed information about available filesystem tools
    ///
    /// Returns: JSON array of FilesystemToolInfo objects with availability status
    ///
    /// Authorization: org.cosmic.ext.storage-service.filesystem-read (allow_active)
    #[authorized_interface(action = "org.cosmic.ext.storage-service.filesystem-read")]
    async fn get_filesystem_tools(
        &self,
        #[zbus(connection)] _connection: &Connection,
        #[zbus(header)] _header: MessageHeader<'_>,
    ) -> zbus::fdo::Result<String> {
        tracing::debug!("Getting filesystem tool details for UID {}", caller.uid);

        // Serialize to JSON
        let json = serde_json::to_string(&self.filesystem_tools).map_err(|e| {
            tracing::error!("Failed to serialize filesystem tools: {e}");
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
    #[authorized_interface(action = "org.cosmic.ext.storage-service.filesystem-format")]
    async fn format(
        &self,
        #[zbus(connection)] _connection: &Connection,
        #[zbus(header)] _header: MessageHeader<'_>,
        #[zbus(signal_context)] signal_ctx: zbus::object_server::SignalEmitter<'_>,
        device: String,
        fs_type: String,
        label: String,
        options_json: String,
    ) -> zbus::fdo::Result<()> {
        tracing::info!(
            "Formatting {} as {} with label '{}' (requested by UID {})",
            device,
            fs_type,
            label,
            caller.uid
        );

        // Validate filesystem type is supported
        if !self.supported_tools.contains(&fs_type) {
            tracing::warn!("Unsupported filesystem type: {}", fs_type);
            return Err(zbus::fdo::Error::Failed(format!(
                "Filesystem type '{}' is not supported or tools not installed",
                fs_type
            )));
        }

        // Parse options
        let options: FormatOptions = serde_json::from_str(&options_json).unwrap_or_default();

        // Delegate to storage-dbus operation
        storage_dbus::format_filesystem(&device, &fs_type, &label, options)
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
    #[authorized_interface(action = "org.cosmic.ext.storage-service.filesystem-mount")]
    async fn mount(
        &self,
        #[zbus(connection)] _connection: &Connection,
        #[zbus(header)] _header: MessageHeader<'_>,
        #[zbus(signal_context)] signal_ctx: zbus::object_server::SignalEmitter<'_>,
        device: String,
        mount_point: String,
        options_json: String,
    ) -> zbus::fdo::Result<String> {
        tracing::info!(
            "Mounting {} to {} (as UID {})",
            device,
            if mount_point.is_empty() {
                "(auto)"
            } else {
                &mount_point
            },
            caller.uid
        );

        // Parse options
        let mount_opts: MountOptions = serde_json::from_str(&options_json).unwrap_or_default();

        // Delegate to storage-dbus operation with caller UID
        let actual_mount_point =
            storage_dbus::mount_filesystem(&device, &mount_point, mount_opts, Some(caller.uid))
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
    #[authorized_interface(action = "org.cosmic.ext.storage-service.filesystem-mount")]
    async fn unmount(
        &self,
        #[zbus(connection)] connection: &Connection,
        #[zbus(header)] header: MessageHeader<'_>,
        #[zbus(signal_context)] signal_ctx: zbus::object_server::SignalEmitter<'_>,
        device_or_mount: String,
        force: bool,
        kill_processes: bool,
    ) -> zbus::fdo::Result<String> {
        tracing::info!(
            "Unmounting {} (force={}, kill={}) for UID {}",
            device_or_mount,
            force,
            kill_processes,
            caller.uid
        );

        // Determine if input is device or mount point (for finding processes later)
        let mount_point = if device_or_mount.starts_with("/dev/") {
            // It's a device, get mount point via storage-dbus
            storage_dbus::get_mount_point(&device_or_mount)
                .await
                .unwrap_or_else(|_| device_or_mount.clone())
        } else {
            // Assume it's a mount point
            device_or_mount.clone()
        };

        // Attempt unmount via storage-dbus operation
        let unmount_result = storage_dbus::unmount_filesystem(&device_or_mount, force).await;

        match unmount_result {
            Ok(_) => {
                tracing::info!("Successfully unmounted {}", device_or_mount);
                let _ = Self::unmounted(&signal_ctx, &device_or_mount).await;
                let result = UnmountResult {
                    success: true,
                    error: None,
                    blocking_processes: Vec::new(),
                };

                let json = serde_json::to_string(&result).map_err(|e| {
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
                    let processes = storage_dbus::find_processes_using_mount(&mount_point)
                        .await
                        .unwrap_or_default();

                    if kill_processes && !processes.is_empty() {
                        // Check if this is a protected system path
                        use std::path::Path;
                        if crate::protected_paths::is_protected_path(Path::new(&mount_point)) {
                            tracing::warn!(
                                "Refusing to kill processes on protected system path: {}",
                                mount_point
                            );

                            let result = UnmountResult {
                                success: false,
                                error: Some(format!(
                                    "Cannot kill processes on protected system path: {}",
                                    mount_point
                                )),
                                blocking_processes: processes,
                            };

                            let json = serde_json::to_string(&result).map_err(|e| {
                                zbus::fdo::Error::Failed(format!("Failed to serialize: {e}"))
                            })?;

                            return Ok(json);
                        }

                        // Check authorization for killing processes
                        // Note: We use the connection and header from the macro for secondary auth
                        let auth_result = crate::auth::check_authorization(
                            connection,
                            header
                                .sender()
                                .ok_or_else(|| zbus::fdo::Error::Failed("No sender".to_string()))?
                                .as_str(),
                            "org.cosmic.ext.storage-service.filesystem-kill-processes",
                        )
                        .await;

                        if auth_result.is_err() {
                            tracing::warn!("Authorization failed for killing processes");

                            let result = UnmountResult {
                                success: false,
                                error: Some("Authorization required to kill processes".to_string()),
                                blocking_processes: processes,
                            };

                            let json = serde_json::to_string(&result).map_err(|e| {
                                zbus::fdo::Error::Failed(format!("Failed to serialize: {e}"))
                            })?;

                            return Ok(json);
                        }

                        // Kill blocking processes
                        let pids: Vec<i32> = processes.iter().map(|p| p.pid).collect();
                        tracing::info!("Killing {} blocking processes", pids.len());

                        let _kill_results = storage_dbus::kill_processes(&pids);

                        // Wait a moment for processes to die
                        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

                        // Retry unmount via storage-dbus
                        match storage_dbus::unmount_filesystem(&device_or_mount, force).await {
                            Ok(_) => {
                                tracing::info!("Successfully unmounted after killing processes");

                                let result = UnmountResult {
                                    success: true,
                                    error: None,
                                    blocking_processes: Vec::new(),
                                };

                                let json = serde_json::to_string(&result).map_err(|e| {
                                    zbus::fdo::Error::Failed(format!("Failed to serialize: {e}"))
                                })?;

                                Ok(json)
                            }
                            Err(retry_err) => {
                                tracing::error!(
                                    "Unmount failed even after killing processes: {}",
                                    retry_err
                                );

                                let result = UnmountResult {
                                    success: false,
                                    error: Some(retry_err.to_string()),
                                    blocking_processes: Vec::new(),
                                };

                                let json = serde_json::to_string(&result).map_err(|e| {
                                    zbus::fdo::Error::Failed(format!("Failed to serialize: {e}"))
                                })?;

                                Ok(json)
                            }
                        }
                    } else {
                        // Return error with blocking processes
                        let result = UnmountResult {
                            success: false,
                            error: Some("Device is busy".to_string()),
                            blocking_processes: processes,
                        };

                        let json = serde_json::to_string(&result).map_err(|e| {
                            zbus::fdo::Error::Failed(format!("Failed to serialize: {e}"))
                        })?;

                        Ok(json)
                    }
                } else {
                    // Other error
                    tracing::error!("Unmount failed: {}", e);

                    let result = UnmountResult {
                        success: false,
                        error: Some(e.to_string()),
                        blocking_processes: Vec::new(),
                    };

                    let json = serde_json::to_string(&result).map_err(|e| {
                        zbus::fdo::Error::Failed(format!("Failed to serialize: {e}"))
                    })?;

                    Ok(json)
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
    #[authorized_interface(action = "org.cosmic.ext.storage-service.filesystem-read")]
    async fn get_blocking_processes(
        &self,
        #[zbus(connection)] _connection: &Connection,
        #[zbus(header)] _header: MessageHeader<'_>,
        device_or_mount: String,
    ) -> zbus::fdo::Result<String> {
        tracing::debug!(
            "Getting blocking processes for {} (UID {})",
            device_or_mount,
            caller.uid
        );

        // Determine mount point via storage-dbus
        let mount_point = if device_or_mount.starts_with("/dev/") {
            storage_dbus::get_mount_point(&device_or_mount)
                .await
                .map_err(|e| {
                    tracing::error!("Failed to get mount point: {e}");
                    zbus::fdo::Error::Failed(format!("Failed to get mount point: {e}"))
                })?
        } else {
            device_or_mount.clone()
        };

        // Find blocking processes
        let processes = storage_dbus::find_processes_using_mount(&mount_point)
            .await
            .map_err(|e| {
                tracing::error!("Failed to find processes: {e}");
                zbus::fdo::Error::Failed(format!("Failed to find processes: {e}"))
            })?;

        tracing::debug!("Found {} blocking processes", processes.len());

        // Serialize to JSON
        let json = serde_json::to_string(&processes).map_err(|e| {
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
    #[authorized_interface(action = "org.cosmic.ext.storage-service.filesystem-modify")]
    async fn check(
        &self,
        #[zbus(connection)] _connection: &Connection,
        #[zbus(header)] _header: MessageHeader<'_>,
        device: String,
        repair: bool,
    ) -> zbus::fdo::Result<String> {
        tracing::info!(
            "Checking filesystem on {} (repair={}) for UID {}",
            device,
            repair,
            caller.uid
        );

        // Delegate to storage-dbus operation
        let clean = storage_dbus::check_filesystem(&device, repair)
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
            output: if clean {
                "Filesystem is clean".to_string()
            } else {
                "Filesystem has errors".to_string()
            },
        };

        let json = serde_json::to_string(&result).map_err(|e| {
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
    #[authorized_interface(action = "org.cosmic.ext.storage-service.filesystem-modify")]
    async fn set_label(
        &self,
        #[zbus(connection)] _connection: &Connection,
        #[zbus(header)] _header: MessageHeader<'_>,
        device: String,
        label: String,
    ) -> zbus::fdo::Result<()> {
        tracing::info!(
            "Setting label on {} to '{}' for UID {}",
            device,
            label,
            caller.uid
        );

        // Delegate to storage-dbus operation
        storage_dbus::set_filesystem_label(&device, &label)
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
    #[authorized_interface(action = "org.cosmic.ext.storage-service.filesystem-read")]
    async fn get_usage(
        &self,
        #[zbus(connection)] _connection: &Connection,
        #[zbus(header)] _header: MessageHeader<'_>,
        mount_point: String,
    ) -> zbus::fdo::Result<String> {
        tracing::debug!(
            "Getting usage for mount point: {} (UID {})",
            mount_point,
            caller.uid
        );

        // Validate mount point exists and is mounted
        if !Path::new(&mount_point).exists() {
            tracing::warn!("Mount point does not exist: {}", mount_point);
            return Err(zbus::fdo::Error::Failed(format!(
                "Mount point does not exist: {}",
                mount_point
            )));
        }

        // Use statvfs to get filesystem stats
        use nix::sys::statvfs::statvfs;

        let stats = statvfs(mount_point.as_str()).map_err(|e| {
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

        let json = serde_json::to_string(&usage).map_err(|e| {
            tracing::error!("Failed to serialize usage: {e}");
            zbus::fdo::Error::Failed(format!("Failed to serialize: {e}"))
        })?;

        Ok(json)
    }

    /// Run a global local-mount usage scan and return category/top-file results.
    ///
    /// Emits `usage_scan_progress` while the scan is running.
    ///
    /// Authorization: org.cosmic.ext.storage-service.filesystem-read (allow_active)
    #[authorized_interface(action = "org.cosmic.ext.storage-service.filesystem-read")]
    async fn get_usage_scan(
        &self,
        #[zbus(connection)] connection: &Connection,
        #[zbus(header)] header: MessageHeader<'_>,
        #[zbus(signal_context)] signal_ctx: zbus::object_server::SignalEmitter<'_>,
        scan_id: String,
        top_files_per_category: u32,
        mount_points_json: String,
        show_all_files: bool,
        parallelism_preset: String,
    ) -> zbus::fdo::Result<String> {
        let parallelism_preset = UsageScanParallelismPreset::from_str(&parallelism_preset)
            .ok_or_else(|| {
                zbus::fdo::Error::Failed("Invalid usage scan parallelism preset".to_string())
            })?;
        let cpu_count = std::thread::available_parallelism()
            .map(|parallelism| parallelism.get())
            .unwrap_or(1);
        let scan_threads = map_parallelism_threads(parallelism_preset, cpu_count);

        tracing::info!(
            "Starting usage scan id={} top_files_per_category={} show_all_files={} preset={} threads={} (UID {})",
            scan_id,
            top_files_per_category,
            show_all_files,
            parallelism_preset.as_str(),
            scan_threads,
            caller.uid
        );

        let selected_mounts: Vec<String> =
            serde_json::from_str(&mount_points_json).map_err(|e| {
                zbus::fdo::Error::Failed(format!("Failed to parse mount points payload: {e}"))
            })?;
        if selected_mounts.is_empty() {
            return Err(zbus::fdo::Error::Failed(
                "At least one mount point must be selected".to_string(),
            ));
        }

        let mounts: Vec<PathBuf> = selected_mounts
            .iter()
            .map(PathBuf::from)
            .filter(|path| path.is_absolute())
            .collect();

        if mounts.is_empty() {
            return Err(zbus::fdo::Error::Failed(
                "Selected mount points must be absolute paths".to_string(),
            ));
        }

        if show_all_files {
            let sender = header
                .sender()
                .ok_or_else(|| zbus::fdo::Error::Failed("No sender in message header".to_string()))?
                .as_str()
                .to_string();

            let authorized = crate::auth::check_authorization(
                connection,
                &sender,
                "org.cosmic.ext.storage-service.filesystem-modify",
            )
            .await
            .map_err(|e| zbus::fdo::Error::Failed(format!("Authorization check failed: {e}")))?;

            if !authorized {
                return Err(zbus::fdo::Error::AccessDenied(
                    "Not authorized to list all files".to_string(),
                ));
            }
        }

        let estimate = storage_sys::usage::mounts::estimate_used_bytes_for_mounts(&mounts);
        let estimated_total_bytes = estimate.used_bytes.max(1);
        let scan_config = storage_sys::usage::ScanConfig {
            threads: Some(scan_threads),
            top_files_per_category: top_files_per_category as usize,
            show_all_files,
            caller_uid: Some(caller.uid),
            caller_gids: Some(resolve_caller_groups(
                caller.uid,
                caller.username.as_deref(),
            )),
        };

        let (progress_tx, progress_rx) = mpsc::channel::<u64>();
        let scan_mounts = mounts.clone();
        let scan_task = tokio::task::spawn_blocking(move || {
            storage_sys::usage::scan_paths_with_progress(
                &scan_mounts,
                &scan_config,
                Some(progress_tx),
            )
        });

        let mut processed_bytes = 0_u64;

        loop {
            while let Ok(delta) = progress_rx.try_recv() {
                processed_bytes = processed_bytes.saturating_add(delta);
            }

            let _ = Self::usage_scan_progress(
                &signal_ctx,
                &scan_id,
                processed_bytes,
                estimated_total_bytes,
            )
            .await;

            if scan_task.is_finished() {
                break;
            }

            tokio::time::sleep(Duration::from_millis(120)).await;
        }

        while let Ok(delta) = progress_rx.try_recv() {
            processed_bytes = processed_bytes.saturating_add(delta);
        }

        let mut scan_result = scan_task
            .await
            .map_err(|e| zbus::fdo::Error::Failed(format!("Usage scan join error: {e}")))?
            .map_err(|e| zbus::fdo::Error::Failed(format!("Usage scan failed: {e}")))?;

        scan_result.total_free_bytes = estimate.free_bytes;

        let final_processed = processed_bytes.max(scan_result.total_bytes);
        let _ = Self::usage_scan_progress(
            &signal_ctx,
            &scan_id,
            final_processed,
            estimated_total_bytes,
        )
        .await;

        let json = serde_json::to_string::<UsageScanResult>(&scan_result).map_err(|e| {
            tracing::error!("Failed to serialize usage scan result: {e}");
            zbus::fdo::Error::Failed(format!("Failed to serialize usage scan result: {e}"))
        })?;

        Ok(json)
    }

    /// List local mount points available for usage scans.
    ///
    /// Authorization: org.cosmic.ext.storage-service.filesystem-read (allow_active)
    #[authorized_interface(action = "org.cosmic.ext.storage-service.filesystem-read")]
    async fn list_usage_mount_points(
        &self,
        #[zbus(connection)] _connection: &Connection,
        #[zbus(header)] _header: MessageHeader<'_>,
    ) -> zbus::fdo::Result<String> {
        let mounts = storage_sys::usage::mounts::discover_local_mounts_under(Path::new("/"))
            .map_err(|e| zbus::fdo::Error::Failed(format!("Failed to discover mounts: {e}")))?;

        let mount_strings: Vec<String> = mounts
            .into_iter()
            .map(|mount| mount.to_string_lossy().to_string())
            .collect();

        serde_json::to_string(&mount_strings)
            .map_err(|e| zbus::fdo::Error::Failed(format!("Failed to serialize mounts: {e}")))
    }

    /// Probe authorization for enabling Show All Files in the current session.
    ///
    /// Authorization: org.cosmic.ext.storage-service.filesystem-modify (auth_admin_keep)
    #[authorized_interface(action = "org.cosmic.ext.storage-service.filesystem-modify")]
    async fn authorize_usage_show_all_files(
        &self,
        #[zbus(connection)] _connection: &Connection,
        #[zbus(header)] _header: MessageHeader<'_>,
    ) -> zbus::fdo::Result<bool> {
        Ok(true)
    }

    /// Delete selected usage paths.
    ///
    /// Args:
    /// - paths_json: JSON-serialized Vec<String> of absolute paths.
    ///
    /// Returns: JSON UsageDeleteResult
    ///
    /// Authorization: org.cosmic.ext.storage-service.filesystem-mount (allow_active)
    #[authorized_interface(action = "org.cosmic.ext.storage-service.filesystem-mount")]
    async fn delete_usage_files(
        &self,
        #[zbus(connection)] connection: &Connection,
        #[zbus(header)] header: MessageHeader<'_>,
        paths_json: String,
    ) -> zbus::fdo::Result<String> {
        let caller_groups = resolve_caller_groups(caller.uid, caller.username.as_deref());

        let paths: Vec<String> = serde_json::from_str(&paths_json)
            .map_err(|e| zbus::fdo::Error::Failed(format!("Failed to parse paths payload: {e}")))?;

        let mut result = UsageDeleteResult {
            deleted: Vec::new(),
            failed: Vec::new(),
        };

        let requires_admin = paths.iter().any(|path_str| {
            let path = Path::new(path_str);
            path.is_absolute()
                && path != Path::new("/")
                && path.exists()
                && path_requires_admin_delete(path, caller.uid, &caller_groups)
        });

        let mut has_admin_authorization = false;
        if requires_admin {
            let sender = header
                .sender()
                .ok_or_else(|| zbus::fdo::Error::Failed("No sender in message header".to_string()))?
                .as_str()
                .to_string();

            has_admin_authorization = crate::auth::check_authorization(
                connection,
                &sender,
                "org.cosmic.ext.storage-service.filesystem-modify",
            )
            .await
            .map_err(|e| zbus::fdo::Error::Failed(format!("Authorization check failed: {e}")))?;

            if !has_admin_authorization {
                return Err(zbus::fdo::Error::AccessDenied(
                    "Not authorized to delete selected files".to_string(),
                ));
            }
        }

        for path_str in paths {
            let path = Path::new(&path_str);

            if !path.is_absolute() {
                result.failed.push(UsageDeleteFailure {
                    path: path_str,
                    reason: "Path must be absolute".to_string(),
                });
                continue;
            }

            if path == Path::new("/") {
                result.failed.push(UsageDeleteFailure {
                    path: path_str,
                    reason: "Refusing to delete root path".to_string(),
                });
                continue;
            }

            if !path.exists() {
                result.failed.push(UsageDeleteFailure {
                    path: path_str,
                    reason: "Path does not exist".to_string(),
                });
                continue;
            }

            let metadata = match fs::symlink_metadata(path) {
                Ok(metadata) => metadata,
                Err(error) => {
                    result.failed.push(UsageDeleteFailure {
                        path: path_str,
                        reason: error.to_string(),
                    });
                    continue;
                }
            };

            if !metadata.is_file() {
                result.failed.push(UsageDeleteFailure {
                    path: path_str,
                    reason: "Only regular files can be deleted".to_string(),
                });
                continue;
            }

            if !has_admin_authorization {
                match caller_can_unlink(path, caller.uid, &caller_groups) {
                    Ok(true) => {}
                    Ok(false) => {
                        result.failed.push(UsageDeleteFailure {
                            path: path_str,
                            reason:
                                "Permission denied: caller cannot unlink file in parent directory"
                                    .to_string(),
                        });
                        continue;
                    }
                    Err(error) => {
                        result.failed.push(UsageDeleteFailure {
                            path: path_str,
                            reason: error.to_string(),
                        });
                        continue;
                    }
                }

                match is_owned_tree(path, caller.uid) {
                    Ok(true) => {}
                    Ok(false) => {
                        result.failed.push(UsageDeleteFailure {
                            path: path_str,
                            reason: "Permission denied: path is not owned by caller".to_string(),
                        });
                        continue;
                    }
                    Err(error) => {
                        result.failed.push(UsageDeleteFailure {
                            path: path_str,
                            reason: error.to_string(),
                        });
                        continue;
                    }
                }
            }

            let delete_result = fs::remove_file(path);

            match delete_result {
                Ok(()) => result.deleted.push(path_str),
                Err(error) => result.failed.push(UsageDeleteFailure {
                    path: path_str,
                    reason: error.to_string(),
                }),
            }
        }

        serde_json::to_string(&result).map_err(|e| {
            zbus::fdo::Error::Failed(format!("Failed to serialize delete result: {e}"))
        })
    }

    /// Get persistent mount options (fstab configuration) for a device
    ///
    /// Returns: JSON Option<MountOptionsSettings> ("null" if none)
    ///
    /// Authorization: org.cosmic.ext.storage-service.filesystem-read (allow_active)
    #[authorized_interface(action = "org.cosmic.ext.storage-service.filesystem-read")]
    async fn get_mount_options(
        &self,
        #[zbus(connection)] _connection: &Connection,
        #[zbus(header)] _header: MessageHeader<'_>,
        device: String,
    ) -> zbus::fdo::Result<String> {
        tracing::debug!("Getting mount options for {} (UID {})", device, caller.uid);

        match storage_dbus::get_mount_options(&device).await {
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
                serde_json::to_string(&Some(out))
                    .map_err(|e| zbus::fdo::Error::Failed(format!("Serialize: {e}")))
            }
            Ok(None) => Ok("null".to_string()),
            Err(e) => {
                tracing::warn!("get_mount_options failed: {e}");
                Ok("null".to_string())
            }
        }
    }

    /// Clear persistent mount options (remove fstab entry) for a device
    ///
    /// Authorization: org.cosmic.ext.storage-service.filesystem-mount (allow_active)
    #[authorized_interface(action = "org.cosmic.ext.storage-service.filesystem-mount")]
    async fn default_mount_options(
        &self,
        #[zbus(connection)] _connection: &Connection,
        #[zbus(header)] _header: MessageHeader<'_>,
        device: String,
    ) -> zbus::fdo::Result<()> {
        tracing::debug!(
            "Resetting mount options for {} (UID {})",
            device,
            caller.uid
        );

        storage_dbus::reset_mount_options(&device)
            .await
            .map_err(|e| {
                tracing::error!("reset_mount_options failed: {e}");
                zbus::fdo::Error::Failed(format!("Failed to clear mount options: {e}"))
            })
    }

    /// Set persistent mount options (fstab configuration) for a device
    ///
    /// Authorization: org.cosmic.ext.storage-service.filesystem-mount (allow_active)
    #[authorized_interface(action = "org.cosmic.ext.storage-service.filesystem-mount")]
    #[allow(clippy::too_many_arguments)]
    async fn edit_mount_options(
        &self,
        #[zbus(connection)] _connection: &Connection,
        #[zbus(header)] _header: MessageHeader<'_>,
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
        tracing::debug!("Setting mount options for {} (UID {})", device, caller.uid);

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

        storage_dbus::set_mount_options(
            &device,
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
            tracing::error!("set_mount_options failed: {e}");
            zbus::fdo::Error::Failed(format!("Failed to set mount options: {e}"))
        })
    }

    /// Take ownership of a mounted filesystem (e.g. for fstab/crypttab)
    ///
    /// Args:
    /// - device: Device path (e.g. "/dev/sda1" or "/dev/mapper/luks-xxx")
    /// - recursive: Take ownership of child mounts
    ///
    /// Authorization: org.cosmic.ext.storage-service.filesystems-take-ownership
    #[authorized_interface(action = "org.cosmic.ext.storage-service.filesystems-take-ownership")]
    async fn take_ownership(
        &self,
        #[zbus(connection)] _connection: &Connection,
        #[zbus(header)] _header: MessageHeader<'_>,
        device: String,
        recursive: bool,
    ) -> zbus::fdo::Result<()> {
        tracing::info!(
            "Taking ownership of {} (recursive={}) for UID {}",
            device,
            recursive,
            caller.uid
        );

        // Delegate to storage-dbus operation
        storage_dbus::take_filesystem_ownership(&device, recursive)
            .await
            .map_err(|e| {
                tracing::error!("Take ownership failed: {e}");
                zbus::fdo::Error::Failed(format!("Take ownership failed: {e}"))
            })?;

        tracing::info!("Successfully took ownership of {}", device);
        Ok(())
    }
}
