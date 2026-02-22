// SPDX-License-Identifier: GPL-3.0-only

pub type Category = storage_common::UsageCategory;
pub type CategoryTotal = storage_common::UsageCategoryTotal;
pub type TopFileEntry = storage_common::UsageTopFileEntry;
pub type CategoryTopFiles = storage_common::UsageCategoryTopFiles;
pub type ScanResult = storage_common::UsageScanResult;

#[derive(Debug, Clone)]
pub struct ScanConfig {
    pub threads: Option<usize>,
    pub top_files_per_category: usize,
}

impl Default for ScanConfig {
    fn default() -> Self {
        Self {
            threads: None,
            top_files_per_category: 20,
        }
    }
}
