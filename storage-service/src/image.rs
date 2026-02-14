// SPDX-License-Identifier: GPL-3.0-only

//! Image operations - disk backup, restore, and loop device management
//!
//! This module handles long-running disk imaging operations with progress tracking.
//! Operations run in background tasks and emit D-Bus signals for progress updates.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::Mutex;
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;
use zbus::{interface, Connection};
use zbus::object_server::SignalEmitter;
use zbus::zvariant::OwnedObjectPath;

/// Operation type for tracking
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OperationType {
    BackupDrive,
    BackupPartition,
    RestoreDrive,
    RestorePartition,
}

impl std::fmt::Display for OperationType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::BackupDrive => write!(f, "backup_drive"),
            Self::BackupPartition => write!(f, "backup_partition"),
            Self::RestoreDrive => write!(f, "restore_drive"),
            Self::RestorePartition => write!(f, "restore_partition"),
        }
    }
}

/// Progress information for an operation
#[derive(Debug, Clone)]
pub struct ProgressInfo {
    pub bytes_completed: u64,
    pub total_bytes: u64,
    pub speed_bytes_per_sec: u64,
    pub started_at: Instant,
}

/// State of an active operation
struct OperationState {
    id: String,
    kind: OperationType,
    source: String,
    destination: String,
    cancel_token: CancellationToken,
    handle: JoinHandle<Result<(), String>>,
    progress: Arc<Mutex<ProgressInfo>>,
}

/// Image operations handler with operation tracking
pub struct ImageHandler {
    active_operations: Arc<Mutex<HashMap<String, OperationState>>>,
}

impl ImageHandler {
    pub fn new() -> Self {
        Self {
            active_operations: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Generate unique operation ID
    fn generate_operation_id() -> String {
        uuid::Uuid::new_v4().to_string()
    }

    /// Find block device object path from device identifier (drive or partition)
    async fn find_block_object_path(device: &str) -> Result<OwnedObjectPath, String> {
        let drives = disks_dbus::disk::get_disks_with_volumes()
            .await
            .map_err(|e| format!("Failed to enumerate drives: {e}"))?;

        let device_name = device.strip_prefix("/dev/").unwrap_or(device);

        // Try whole-disk match first
        for drive in &drives {
            let disk_info: storage_models::DiskInfo = drive.clone().into();
            if disk_info.device == device
                || disk_info.device.rsplit('/').next() == Some(device_name)
                || disk_info.id == device
                || disk_info.id == device_name
            {
                return drive.block_path.as_str().try_into()
                    .map_err(|e| format!("Invalid block path: {e}"));
            }
        }

        // Try partition/volume match (e.g. /dev/sda1)
        for drive in &drives {
            for volume in &drive.volumes {
                let matches = volume.device_path.as_deref() == Some(device)
                    || volume
                        .device_path
                        .as_deref()
                        .map(|p| p.strip_prefix("/dev/").unwrap_or(p))
                        == Some(device_name);
                if matches {
                    return Ok(volume.object_path.clone());
                }
            }
        }

        Err(format!("Device not found: {device}"))
    }

    /// Background task for backup operation
    async fn backup_task(
        block_path: OwnedObjectPath,
        output_path: String,
        cancel_token: CancellationToken,
        progress: Arc<Mutex<ProgressInfo>>,
    ) -> Result<(), String> {
        // Check for cancellation before starting
        if cancel_token.is_cancelled() {
            return Err("Operation cancelled".to_string());
        }

        // Open source device (privileged) via disks-dbus
        let source_fd = disks_dbus::open_for_backup(block_path)
            .await
            .map_err(|e| format!("Failed to open source device: {e}"))?;

        // Get total size for progress tracking
        let total_size = std::fs::File::from(source_fd.try_clone()
            .map_err(|e| format!("Failed to clone fd: {e}"))?)
            .metadata()
            .map_err(|e| format!("Failed to get device size: {e}"))?
            .len();

        // Initialize progress
        {
            let mut prog = progress.lock().await;
            prog.total_bytes = total_size;
        }

        let output_path_buf = PathBuf::from(output_path);
        let start_time = Instant::now();
        let progress_clone = progress.clone();
        let cancel_clone = cancel_token.clone();

        // Perform the copy in a blocking task (storage_sys uses sync I/O)
        let result = tokio::task::spawn_blocking(move || {
            storage_sys::copy_image_to_file(
                source_fd,
                &output_path_buf,
                Some(|bytes_copied: u64| {
                    // Check cancellation in callback
                    if cancel_clone.is_cancelled() {
                        // Note: can't stop mid-operation gracefully with current API
                        return;
                    }

                    // Update progress (blocking mutex)
                    let mut prog = progress_clone.blocking_lock();
                    let elapsed = start_time.elapsed();
                    let speed = if elapsed.as_secs() > 0 {
                        bytes_copied / elapsed.as_secs()
                    } else {
                        0
                    };
                    prog.bytes_completed = bytes_copied;
                    prog.speed_bytes_per_sec = speed;
                }),
            )
        })
        .await
        .map_err(|e| format!("Task join error: {e}"))?
        .map_err(|e| format!("Copy failed: {e}"))?;

        // Check if operation was cancelled
        if cancel_token.is_cancelled() {
            return Err("Operation cancelled".to_string());
        }

        Ok(())
    }

    /// Background task for restore operation
    async fn restore_task(
        input_path: String,
        block_path: OwnedObjectPath,
        cancel_token: CancellationToken,
        progress: Arc<Mutex<ProgressInfo>>,
    ) -> Result<(), String> {
        // Check for cancellation before starting
        if cancel_token.is_cancelled() {
            return Err("Operation cancelled".to_string());
        }

        // Get source file size for progress tracking
        let source_path = PathBuf::from(&input_path);
        let total_size = std::fs::metadata(&source_path)
            .map_err(|e| format!("Failed to get image file size: {e}"))?
            .len();

        // Initialize progress
        {
            let mut prog = progress.lock().await;
            prog.total_bytes = total_size;
        }

        // Open destination device (privileged) via disks-dbus
        let dest_fd = disks_dbus::open_for_restore(block_path)
            .await
            .map_err(|e| format!("Failed to open destination device: {e}"))?;

        let start_time = Instant::now();
        let progress_clone = progress.clone();
        let cancel_clone = cancel_token.clone();

        // Perform the copy in a blocking task (storage_sys uses sync I/O)
        let result = tokio::task::spawn_blocking(move || {
            storage_sys::copy_file_to_image(
                &source_path,
                dest_fd,
                Some(|bytes_copied: u64| {
                    // Check cancellation in callback
                    if cancel_clone.is_cancelled() {
                        // Note: can't stop mid-operation gracefully with current API
                        return;
                    }

                    // Update progress (blocking mutex)
                    let mut prog = progress_clone.blocking_lock();
                    let elapsed = start_time.elapsed();
                    let speed = if elapsed.as_secs() > 0 {
                        bytes_copied / elapsed.as_secs()
                    } else {
                        0
                    };
                    prog.bytes_completed = bytes_copied;
                    prog.speed_bytes_per_sec = speed;
                }),
            )
        })
        .await
        .map_err(|e| format!("Task join error: {e}"))?
        .map_err(|e| format!("Copy failed: {e}"))?;

        // Check if operation was cancelled
        if cancel_token.is_cancelled() {
            return Err("Operation cancelled".to_string());
        }

        Ok(())
    }
}

#[interface(name = "org.cosmic.ext.StorageService.Image")]
impl ImageHandler {
    /// Backup entire drive to an image file
    ///
    /// Args:
    /// - device: Device identifier (e.g., "/dev/sda", "sda")
    /// - output_path: Path to write image file
    ///
    /// Returns: operation_id for tracking progress
    ///
    /// Authorization: org.cosmic.ext.storage-service.disk-backup
    async fn backup_drive(
        &self,
        #[zbus(connection)] connection: &Connection,
        #[zbus(signal_context)] signal_ctx: SignalEmitter<'_>,
        device: String,
        output_path: String,
    ) -> zbus::fdo::Result<String> {
        // Check authorization
        crate::auth::check_polkit_auth(connection, "org.cosmic.ext.storage-service.disk-backup")
            .await
            .map_err(|e| zbus::fdo::Error::Failed(format!("Authorization failed: {e}")))?;

        tracing::info!("Starting drive backup: {device} → {output_path}");

        // Validate output path
        let output_path_obj = Path::new(&output_path);
        if let Some(parent) = output_path_obj.parent() {
            if !parent.exists() {
                return Err(zbus::fdo::Error::Failed(
                    format!("Output directory does not exist: {}", parent.display())
                ));
            }
        }

        // Find block device
        let block_path = Self::find_block_object_path(&device)
            .await
            .map_err(|e| zbus::fdo::Error::Failed(e))?;

        // Generate operation ID
        let operation_id = Self::generate_operation_id();

        // Create progress tracker
        let progress = Arc::new(Mutex::new(ProgressInfo {
            bytes_completed: 0,
            total_bytes: 0,
            speed_bytes_per_sec: 0,
            started_at: Instant::now(),
        }));

        // Create cancellation token
        let cancel_token = CancellationToken::new();

        // Spawn background task
        let task_cancel = cancel_token.clone();
        let task_progress = progress.clone();
        let task_output_path = output_path.clone();

        let handle = tokio::spawn(async move {
            Self::backup_task(
                block_path,
                task_output_path,
                task_cancel,
                task_progress,
            )
            .await
        });

        // Track operation
        let op_state = OperationState {
            id: operation_id.clone(),
            kind: OperationType::BackupDrive,
            source: device.clone(),
            destination: output_path.clone(),
            cancel_token,
            handle,
            progress,
        };

        self.active_operations
            .lock()
            .await
            .insert(operation_id.clone(), op_state);

        // Emit started signal
        Self::operation_started(&signal_ctx, &operation_id, "backup_drive", &device, &output_path)
            .await?;

        Ok(operation_id)
    }

    /// Backup a single partition to an image file
    ///
    /// Args:
    /// - device: Partition identifier (e.g., "/dev/sda1")
    /// - output_path: Path to write image file
    ///
    /// Returns: operation_id for tracking progress
    ///
    /// Authorization: org.cosmic.ext.storage-service.partition-backup
    async fn backup_partition(
        &self,
        #[zbus(connection)] connection: &Connection,
        #[zbus(signal_context)] signal_ctx: SignalEmitter<'_>,
        device: String,
        output_path: String,
    ) -> zbus::fdo::Result<String> {
        // Check authorization
        crate::auth::check_polkit_auth(connection, "org.cosmic.ext.storage-service.partition-backup")
            .await
            .map_err(|e| zbus::fdo::Error::Failed(format!("Authorization failed: {e}")))?;

        tracing::info!("Starting partition backup: {device} → {output_path}");

        // Validate output path
        let output_path_obj = Path::new(&output_path);
        if let Some(parent) = output_path_obj.parent() {
            if !parent.exists() {
                return Err(zbus::fdo::Error::Failed(
                    format!("Output directory does not exist: {}", parent.display())
                ));
            }
        }

        // Find block device
        let block_path = Self::find_block_object_path(&device)
            .await
            .map_err(|e| zbus::fdo::Error::Failed(e))?;

        let operation_id = Self::generate_operation_id();

        let progress = Arc::new(Mutex::new(ProgressInfo {
            bytes_completed: 0,
            total_bytes: 0,
            speed_bytes_per_sec: 0,
            started_at: Instant::now(),
        }));

        let cancel_token = CancellationToken::new();

        let task_progress = progress.clone();
        let task_cancel = cancel_token.clone();
        let task_output_path = output_path.clone();

        let handle = tokio::spawn(async move {
            Self::backup_task(
                block_path,
                task_output_path,
                task_cancel,
                task_progress,
            )
            .await
        });

        let op_state = OperationState {
            id: operation_id.clone(),
            kind: OperationType::BackupPartition,
            source: device.clone(),
            destination: output_path.clone(),
            cancel_token,
            handle,
            progress,
        };

        self.active_operations
            .lock()
            .await
            .insert(operation_id.clone(), op_state);

        Self::operation_started(&signal_ctx, &operation_id, "backup_partition", &device, &output_path)
            .await?;

        Ok(operation_id)
    }

    /// Restore entire drive from an image file
    ///
    /// Args:
    /// - device: Device identifier (e.g., "/dev/sda")
    /// - image_path: Path to image file
    ///
    /// Returns: operation_id for tracking progress
    ///
    /// Authorization: org.cosmic.ext.storage-service.disk-restore (always prompts)
    async fn restore_drive(
        &self,
        #[zbus(connection)] connection: &Connection,
        #[zbus(signal_context)] signal_ctx: SignalEmitter<'_>,
        device: String,
        image_path: String,
    ) -> zbus::fdo::Result<String> {
        // Check authorization (auth_admin - always prompts)
        crate::auth::check_polkit_auth(connection, "org.cosmic.ext.storage-service.disk-restore")
            .await
            .map_err(|e| zbus::fdo::Error::Failed(format!("Authorization failed: {e}")))?;

        tracing::warn!("Starting DESTRUCTIVE drive restore: {image_path} → {device}");

        // Validate image file exists
        if !Path::new(&image_path).exists() {
            return Err(zbus::fdo::Error::Failed(
                format!("Image file does not exist: {image_path}")
            ));
        }

        // Find block device
        let block_path = Self::find_block_object_path(&device)
            .await
            .map_err(|e| zbus::fdo::Error::Failed(e))?;

        let operation_id = Self::generate_operation_id();

        let progress = Arc::new(Mutex::new(ProgressInfo {
            bytes_completed: 0,
            total_bytes: 0,
            speed_bytes_per_sec: 0,
            started_at: Instant::now(),
        }));

        let cancel_token = CancellationToken::new();

        let task_progress = progress.clone();
        let task_cancel = cancel_token.clone();
        let task_image_path = image_path.clone();

        let handle = tokio::spawn(async move {
            Self::restore_task(
                task_image_path,
                block_path,
                task_cancel,
                task_progress,
            )
            .await
        });

        let op_state = OperationState {
            id: operation_id.clone(),
            kind: OperationType::RestoreDrive,
            source: image_path.clone(),
            destination: device.clone(),
            cancel_token,
            handle,
            progress,
        };

        self.active_operations
            .lock()
            .await
            .insert(operation_id.clone(), op_state);

        Self::operation_started(&signal_ctx, &operation_id, "restore_drive", &image_path, &device)
            .await?;

        Ok(operation_id)
    }

    /// Restore a single partition from an image file
    ///
    /// Args:
    /// - device: Partition identifier (e.g., "/dev/sda1")
    /// - image_path: Path to image file
    ///
    /// Returns: operation_id for tracking progress
    ///
    /// Authorization: org.cosmic.ext.storage-service.partition-restore (always prompts)
    async fn restore_partition(
        &self,
        #[zbus(connection)] connection: &Connection,
        #[zbus(signal_context)] signal_ctx: SignalEmitter<'_>,
        device: String,
        image_path: String,
    ) -> zbus::fdo::Result<String> {
        // Check authorization (auth_admin - always prompts)
        crate::auth::check_polkit_auth(connection, "org.cosmic.ext.storage-service.partition-restore")
            .await
            .map_err(|e| zbus::fdo::Error::Failed(format!("Authorization failed: {e}")))?;

        tracing::warn!("Starting DESTRUCTIVE partition restore: {image_path} → {device}");

        // Validate image file
        if !Path::new(&image_path).exists() {
            return Err(zbus::fdo::Error::Failed(
                format!("Image file does not exist: {image_path}")
            ));
        }

        // Find block device
        let block_path = Self::find_block_object_path(&device)
            .await
            .map_err(|e| zbus::fdo::Error::Failed(e))?;

        let operation_id = Self::generate_operation_id();

        let progress = Arc::new(Mutex::new(ProgressInfo {
            bytes_completed: 0,
            total_bytes: 0,
            speed_bytes_per_sec: 0,
            started_at: Instant::now(),
        }));

        let cancel_token = CancellationToken::new();

        let task_progress = progress.clone();
        let task_cancel = cancel_token.clone();
        let task_image_path = image_path.clone();

        let handle = tokio::spawn(async move {
            Self::restore_task(
                task_image_path,
                block_path,
                task_cancel,
                task_progress,
            )
            .await
        });

        let op_state = OperationState {
            id: operation_id.clone(),
            kind: OperationType::RestorePartition,
            source: image_path.clone(),
            destination: device.clone(),
            cancel_token,
            handle,
            progress,
        };

        self.active_operations
            .lock()
            .await
            .insert(operation_id.clone(), op_state);

        Self::operation_started(&signal_ctx, &operation_id, "restore_partition", &image_path, &device)
            .await?;

        Ok(operation_id)
    }

    /// Mount an image file as a loop device
    ///
    /// Args:
    /// - image_path: Path to image file (ISO, IMG, etc.)
    ///
    /// Returns: Loop device path (e.g., "/dev/loop0")
    ///
    /// Authorization: org.cosmic.ext.storage-service.disk-loop-setup
    async fn loop_setup(
        &self,
        #[zbus(connection)] connection: &Connection,
        image_path: String,
    ) -> zbus::fdo::Result<String> {
        // Check authorization
        crate::auth::check_polkit_auth(connection, "org.cosmic.ext.storage-service.disk-loop-setup")
            .await
            .map_err(|e| zbus::fdo::Error::Failed(format!("Authorization failed: {e}")))?;

        tracing::info!("Setting up loop device for: {image_path}");

        // Validate image file
        if !Path::new(&image_path).exists() {
            return Err(zbus::fdo::Error::Failed(
                format!("Image file does not exist: {image_path}")
            ));
        }

        // Call disks-dbus loop_setup
        let loop_object_path = disks_dbus::loop_setup(&image_path)
            .await
            .map_err(|e| {
                tracing::error!("Loop setup failed: {e}");
                zbus::fdo::Error::Failed(format!("Loop setup failed: {e}"))
            })?;

        // Extract device name from object path
        // /org/freedesktop/UDisks2/block_devices/loop0 → loop0
        let device_name = loop_object_path
            .as_str()
            .rsplit('/')
            .next()
            .unwrap_or("unknown");

        tracing::info!("Loop device created: {device_name}");
        Ok(device_name.to_string())
    }

    /// Cancel a running operation
    ///
    /// Args:
    /// - operation_id: ID returned from backup/restore methods
    async fn cancel_operation(&self, operation_id: String) -> zbus::fdo::Result<()> {
        let ops = self.active_operations.lock().await;

        if let Some(op) = ops.get(&operation_id) {
            tracing::info!("Cancelling operation: {operation_id}");
            op.cancel_token.cancel();
            Ok(())
        } else {
            Err(zbus::fdo::Error::Failed(format!(
                "Operation not found: {operation_id}"
            )))
        }
    }

    /// Get status of an operation
    ///
    /// Args:
    /// - operation_id: ID returned from backup/restore methods
    ///
    /// Returns: JSON with operation status
    async fn get_operation_status(&self, operation_id: String) -> zbus::fdo::Result<String> {
        let ops = self.active_operations.lock().await;

        if let Some(op) = ops.get(&operation_id) {
            let progress = op.progress.lock().await;
            let elapsed = progress.started_at.elapsed().as_secs();

            let status = serde_json::json!({
                "operation_id": operation_id,
                "operation_type": op.kind.to_string(),
                "source": op.source,
                "destination": op.destination,
                "bytes_completed": progress.bytes_completed,
                "total_bytes": progress.total_bytes,
                "speed_bytes_per_sec": progress.speed_bytes_per_sec,
                "elapsed_seconds": elapsed,
                "is_finished": op.handle.is_finished(),
            });

            Ok(status.to_string())
        } else {
            Err(zbus::fdo::Error::Failed(format!(
                "Operation not found: {operation_id}"
            )))
        }
    }

    /// List all active operations
    ///
    /// Returns: JSON array of operation status
    async fn list_active_operations(&self) -> zbus::fdo::Result<String> {
        let ops = self.active_operations.lock().await;

        let mut statuses = Vec::new();
        for (id, op) in ops.iter() {
            let progress = op.progress.lock().await;
            statuses.push(serde_json::json!({
                "operation_id": id,
                "operation_type": op.kind.to_string(),
                "source": op.source,
                "destination": op.destination,
                "bytes_completed": progress.bytes_completed,
                "total_bytes": progress.total_bytes,
                "is_finished": op.handle.is_finished(),
            }));
        }

        Ok(serde_json::to_string(&statuses).unwrap_or_else(|_| "[]".to_string()))
    }

    /// Signal emitted when an operation starts
    #[zbus(signal)]
    async fn operation_started(
        signal_ctx: &SignalEmitter<'_>,
        operation_id: &str,
        operation_type: &str,
        source: &str,
        destination: &str,
    ) -> zbus::Result<()>;

    /// Signal emitted periodically during operation with progress updates
    #[zbus(signal)]
    async fn operation_progress(
        signal_ctx: &SignalEmitter<'_>,
        operation_id: &str,
        bytes_completed: u64,
        total_bytes: u64,
        speed_bytes_per_sec: u64,
    ) -> zbus::Result<()>;

    /// Signal emitted when an operation completes (success or failure)
    #[zbus(signal)]
    async fn operation_completed(
        signal_ctx: &SignalEmitter<'_>,
        operation_id: &str,
        success: bool,
        error_message: &str,
    ) -> zbus::Result<()>;
}
