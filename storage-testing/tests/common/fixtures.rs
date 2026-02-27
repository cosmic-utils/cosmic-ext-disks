use storage_contracts::client::{
    BtrfsClient, DisksClient, FilesystemsClient, ImageClient, LogicalClient, LuksClient,
    PartitionsClient, RcloneClient,
};

use super::assertions;

pub fn destructive_enabled() -> bool {
    std::env::var("STORAGE_TESTING_ENABLE_DESTRUCTIVE")
        .ok()
        .as_deref()
        == Some("1")
}

pub fn env(name: &str) -> Option<String> {
    std::env::var(name)
        .ok()
        .filter(|value| !value.trim().is_empty())
}

pub async fn disks_client() -> Option<DisksClient> {
    assertions::skip_or_panic(DisksClient::new().await, "create disks client")
}

pub async fn filesystems_client() -> Option<FilesystemsClient> {
    assertions::skip_or_panic(FilesystemsClient::new().await, "create filesystems client")
}

pub async fn partitions_client() -> Option<PartitionsClient> {
    assertions::skip_or_panic(PartitionsClient::new().await, "create partitions client")
}

pub async fn luks_client() -> Option<LuksClient> {
    assertions::skip_or_panic(LuksClient::new().await, "create luks client")
}

pub async fn btrfs_client() -> Option<BtrfsClient> {
    assertions::skip_or_panic(BtrfsClient::new().await, "create btrfs client")
}

pub async fn logical_client() -> Option<LogicalClient> {
    assertions::skip_or_panic(LogicalClient::new().await, "create logical client")
}

pub async fn image_client() -> Option<ImageClient> {
    assertions::skip_or_panic(ImageClient::new().await, "create image client")
}

pub async fn rclone_client() -> Option<RcloneClient> {
    assertions::skip_or_panic(RcloneClient::new().await, "create rclone client")
}
