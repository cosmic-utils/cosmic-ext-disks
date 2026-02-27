use storage_contracts::client::error::ClientError;
use storage_contracts::client::{
    BtrfsClient, DisksClient, FilesystemsClient, ImageClient, LogicalClient, LuksClient,
    PartitionsClient, RcloneClient,
};

use crate::errors::{Result, TestingError};

pub fn should_skip(error: &ClientError) -> bool {
    matches!(
        error,
        ClientError::ServiceNotAvailable | ClientError::Connection(_)
    )
}

pub fn skip<T>(reason: impl Into<String>) -> Result<T> {
    Err(TestingError::TestSkipped {
        reason: reason.into(),
    })
}

pub fn failure<T>(reason: impl Into<String>) -> Result<T> {
    Err(TestingError::TestFailed {
        reason: reason.into(),
    })
}

pub fn env(name: &str) -> Option<String> {
    std::env::var(name)
        .ok()
        .filter(|value| !value.trim().is_empty())
}

pub fn require_env(name: &str) -> Result<String> {
    env(name).ok_or_else(|| TestingError::TestSkipped {
        reason: format!("set {name}"),
    })
}

pub fn destructive_enabled() -> bool {
    std::env::var("STORAGE_TESTING_ENABLE_DESTRUCTIVE")
        .ok()
        .as_deref()
        == Some("1")
}

pub fn require_destructive(label: &str) -> Result<()> {
    if destructive_enabled() {
        Ok(())
    } else {
        skip(format!("destructive test disabled for {label}"))
    }
}

fn map_client_result<T>(result: std::result::Result<T, ClientError>, context: &str) -> Result<T> {
    match result {
        Ok(value) => Ok(value),
        Err(error) if should_skip(&error) => skip(format!("{context}: {error}")),
        Err(error) => failure(format!("{context}: {error}")),
    }
}

pub fn client_result<T>(result: std::result::Result<T, ClientError>, context: &str) -> Result<T> {
    map_client_result(result, context)
}

pub async fn disks_client() -> Result<DisksClient> {
    map_client_result(DisksClient::new().await, "create disks client")
}

pub async fn filesystems_client() -> Result<FilesystemsClient> {
    map_client_result(FilesystemsClient::new().await, "create filesystems client")
}

pub async fn partitions_client() -> Result<PartitionsClient> {
    map_client_result(PartitionsClient::new().await, "create partitions client")
}

pub async fn luks_client() -> Result<LuksClient> {
    map_client_result(LuksClient::new().await, "create luks client")
}

pub async fn btrfs_client() -> Result<BtrfsClient> {
    map_client_result(BtrfsClient::new().await, "create btrfs client")
}

pub async fn logical_client() -> Result<LogicalClient> {
    map_client_result(LogicalClient::new().await, "create logical client")
}

pub async fn image_client() -> Result<ImageClient> {
    map_client_result(ImageClient::new().await, "create image client")
}

pub async fn rclone_client() -> Result<RcloneClient> {
    map_client_result(RcloneClient::new().await, "create rclone client")
}
