use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UsageCategory {
    Documents,
    Images,
    Audio,
    Video,
    Archives,
    Code,
    Binaries,
    Other,
}

impl UsageCategory {
    pub const ALL: [UsageCategory; 8] = [
        UsageCategory::Documents,
        UsageCategory::Images,
        UsageCategory::Audio,
        UsageCategory::Video,
        UsageCategory::Archives,
        UsageCategory::Code,
        UsageCategory::Binaries,
        UsageCategory::Other,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            UsageCategory::Documents => "Documents",
            UsageCategory::Images => "Images",
            UsageCategory::Audio => "Audio",
            UsageCategory::Video => "Video",
            UsageCategory::Archives => "Archives",
            UsageCategory::Code => "Code",
            UsageCategory::Binaries => "Binaries",
            UsageCategory::Other => "Other",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageCategoryTotal {
    pub category: UsageCategory,
    pub bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageTopFileEntry {
    pub path: PathBuf,
    pub bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageCategoryTopFiles {
    pub category: UsageCategory,
    pub files: Vec<UsageTopFileEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageScanResult {
    pub categories: Vec<UsageCategoryTotal>,
    pub top_files_by_category: Vec<UsageCategoryTopFiles>,
    pub total_bytes: u64,
    pub total_free_bytes: u64,
    pub files_scanned: u64,
    pub dirs_scanned: u64,
    pub skipped_errors: u64,
    pub mounts_scanned: usize,
    pub elapsed_ms: u128,
}
