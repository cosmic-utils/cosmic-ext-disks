// SPDX-License-Identifier: GPL-3.0-only

use zbus::interface;

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

#[interface(name = "org.cosmic.ext.StorageService")]
impl StorageService {
    /// Get service version
    #[zbus(property)]
    async fn version(&self) -> &str {
        &self.version
    }
    
    /// Get list of supported features
    #[zbus(property)]
    async fn supported_features(&self) -> Vec<String> {
        vec![
            "btrfs".to_string(),
            // Future: "partitions".to_string(),
            // Future: "lvm".to_string(),
            // Future: "smart".to_string(),
        ]
    }
}
