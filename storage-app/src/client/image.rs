// SPDX-License-Identifier: GPL-3.0-only

use crate::client::error::ClientError;
use futures_util::StreamExt;
use zbus::{Connection, proxy};

/// D-Bus proxy interface for disk imaging operations
#[proxy(
    interface = "org.cosmic.ext.Storage.Service.Image",
    default_service = "org.cosmic.ext.Storage.Service",
    default_path = "/org/cosmic/ext/Storage/Service/image"
)]
trait ImageInterface {
    /// Backup entire drive to an image file
    async fn backup_drive(&self, device: &str, output_path: &str) -> zbus::Result<String>;

    /// Backup a single partition to an image file
    async fn backup_partition(&self, device: &str, output_path: &str) -> zbus::Result<String>;

    /// Restore entire drive from an image file
    async fn restore_drive(&self, device: &str, image_path: &str) -> zbus::Result<String>;

    /// Restore a single partition from an image file
    async fn restore_partition(&self, device: &str, image_path: &str) -> zbus::Result<String>;

    /// Mount an image file as a loop device
    async fn loop_setup(&self, image_path: &str) -> zbus::Result<String>;

    /// Cancel a running operation
    async fn cancel_operation(&self, operation_id: &str) -> zbus::Result<()>;

    /// Get status of an operation
    async fn get_operation_status(&self, operation_id: &str) -> zbus::Result<String>;

    /// List all active operations
    async fn list_active_operations(&self) -> zbus::Result<String>;

    /// Signal emitted when an operation starts
    #[zbus(signal)]
    async fn operation_started(
        &self,
        operation_id: &str,
        operation_type: &str,
        source: &str,
        destination: &str,
    ) -> zbus::Result<()>;

    /// Signal emitted periodically during operation with progress updates
    #[zbus(signal)]
    async fn operation_progress(
        &self,
        operation_id: &str,
        bytes_completed: u64,
        total_bytes: u64,
        speed_bytes_per_sec: u64,
    ) -> zbus::Result<()>;

    /// Signal emitted when an operation completes
    #[zbus(signal)]
    async fn operation_completed(
        &self,
        operation_id: &str,
        success: bool,
        error_message: &str,
    ) -> zbus::Result<()>;
}

/// Operation status information
#[derive(Debug, Clone, serde::Deserialize)]
pub struct OperationStatus {
    pub bytes_completed: u64,
    pub total_bytes: u64,
    pub speed_bytes_per_sec: u64,
}

/// Client for disk imaging operations
pub struct ImageClient {
    proxy: ImageInterfaceProxy<'static>,
}

impl ImageClient {
    /// Create a new image client connected to the storage service
    pub async fn new() -> Result<Self, ClientError> {
        let conn = Connection::system().await.map_err(|e| {
            ClientError::Connection(format!("Failed to connect to system bus: {}", e))
        })?;

        let proxy = ImageInterfaceProxy::new(&conn)
            .await
            .map_err(|e| ClientError::Connection(format!("Failed to create image proxy: {}", e)))?;

        Ok(Self { proxy })
    }

    /// Backup entire drive to an image file
    ///
    /// Returns an operation ID for tracking progress via signals.
    ///
    /// Requires administrator authentication (cached for session).
    pub async fn backup_drive(
        &self,
        device: &str,
        output_path: &str,
    ) -> Result<String, ClientError> {
        Ok(self.proxy.backup_drive(device, output_path).await?)
    }

    /// Backup a single partition to an image file
    ///
    /// Returns an operation ID for tracking progress via signals.
    ///
    /// Requires administrator authentication (cached for session).
    pub async fn backup_partition(
        &self,
        device: &str,
        output_path: &str,
    ) -> Result<String, ClientError> {
        Ok(self.proxy.backup_partition(device, output_path).await?)
    }

    /// Restore entire drive from an image file
    ///
    /// **WARNING: This will DESTROY ALL DATA on the target drive!**
    ///
    /// Returns an operation ID for tracking progress via signals.
    ///
    /// Requires administrator authentication (always prompts, never cached).
    pub async fn restore_drive(
        &self,
        device: &str,
        image_path: &str,
    ) -> Result<String, ClientError> {
        Ok(self.proxy.restore_drive(device, image_path).await?)
    }

    /// Restore a single partition from an image file
    ///
    /// **WARNING: This will DESTROY ALL DATA on the target partition!**
    ///
    /// Returns an operation ID for tracking progress via signals.
    ///
    /// Requires administrator authentication (always prompts, never cached).
    pub async fn restore_partition(
        &self,
        device: &str,
        image_path: &str,
    ) -> Result<String, ClientError> {
        Ok(self.proxy.restore_partition(device, image_path).await?)
    }

    /// Mount an image file (ISO, IMG, etc.) as a loop device
    ///
    /// Returns the loop device name (e.g., "loop0").
    ///
    /// Requires no authentication for active sessions.
    pub async fn loop_setup(&self, image_path: &str) -> Result<String, ClientError> {
        Ok(self.proxy.loop_setup(image_path).await?)
    }

    /// Cancel a running backup or restore operation
    pub async fn cancel_operation(&self, operation_id: &str) -> Result<(), ClientError> {
        Ok(self.proxy.cancel_operation(operation_id).await?)
    }

    /// Get the current status of an operation
    ///
    /// Returns detailed progress information including bytes completed, speed, etc.
    pub async fn get_operation_status(
        &self,
        operation_id: &str,
    ) -> Result<OperationStatus, ClientError> {
        let json = self.proxy.get_operation_status(operation_id).await?;
        let status: OperationStatus = serde_json::from_str(&json).map_err(|e| {
            ClientError::ParseError(format!("Failed to parse operation status: {}", e))
        })?;
        Ok(status)
    }

    /// Wait for an operation to complete (success or failure).
    /// Subscribes to the operation_completed signal and returns when the given operation_id is seen.
    pub async fn wait_for_operation_completion(
        &self,
        operation_id: &str,
    ) -> Result<(), ClientError> {
        let mut stream = self.proxy.receive_operation_completed().await?;
        while let Some(signal) = stream.next().await {
            let args = signal
                .args()
                .map_err(|e| ClientError::Connection(e.to_string()))?;
            if args.operation_id == operation_id {
                return if args.success {
                    Ok(())
                } else {
                    Err(ClientError::OperationFailed(args.error_message.to_string()))
                };
            }
        }
        Ok(())
    }
}
