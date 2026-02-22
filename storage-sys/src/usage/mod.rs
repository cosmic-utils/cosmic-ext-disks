// SPDX-License-Identifier: GPL-3.0-only

pub mod classifier;
pub mod error;
pub mod mounts;
pub mod scanner;
pub mod types;

pub use classifier::classify_path;
pub use error::UsageScanError;
pub use scanner::scan_paths;
pub use types::{
    Category, CategoryTopFiles, CategoryTotal, ScanConfig, ScanResult, TopFileEntry,
};

use std::path::Path;

use mounts::discover_local_mounts_under;

pub fn scan_local_mounts(root: &Path, config: &ScanConfig) -> Result<ScanResult, UsageScanError> {
    let roots = discover_local_mounts_under(root)?;
    scan_paths(&roots, config)
}
