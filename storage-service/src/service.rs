// SPDX-License-Identifier: GPL-3.0-only

use zbus::interface;

pub mod domain;

/// Main storage service interface
pub struct StorageService {
    version: String,
}

impl StorageService {
    pub fn new() -> Self {
        Self {
            version: env!("CARGO_PKG_VERSION").to_string(),
        }
    }
}

#[interface(name = "org.cosmic.ext.Storage.Service")]
impl StorageService {
    /// Get service version
    #[zbus(property)]
    async fn version(&self) -> &str {
        &self.version
    }

    /// Get list of supported features
    #[zbus(property)]
    async fn supported_features(&self) -> Vec<String> {
        let mut features = vec![
            "disks".to_string(),
            "partitions".to_string(),
            "filesystems".to_string(),
            "luks".to_string(),
            "image".to_string(),
        ];

        if cfg!(feature = "btrfs-tools") {
            features.push("btrfs".to_string());
        }

        if cfg!(feature = "rclone-tools") {
            features.push("rclone".to_string());
        }

        if cfg!(feature = "lvm-tools") {
            features.push("lvm".to_string());
        }

        features
    }
}
