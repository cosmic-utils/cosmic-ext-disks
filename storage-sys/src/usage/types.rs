// SPDX-License-Identifier: GPL-3.0-only

use std::path::PathBuf;

use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Category {
    Documents,
    Images,
    Audio,
    Video,
    Archives,
    Code,
    Binaries,
    Other,
}

impl Category {
    pub const ALL: [Category; 8] = [
        Category::Documents,
        Category::Images,
        Category::Audio,
        Category::Video,
        Category::Archives,
        Category::Code,
        Category::Binaries,
        Category::Other,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Category::Documents => "Documents",
            Category::Images => "Images",
            Category::Audio => "Audio",
            Category::Video => "Video",
            Category::Archives => "Archives",
            Category::Code => "Code",
            Category::Binaries => "Binaries",
            Category::Other => "Other",
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct CategoryTotal {
    pub category: Category,
    pub bytes: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct TopFileEntry {
    pub path: PathBuf,
    pub bytes: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct CategoryTopFiles {
    pub category: Category,
    pub files: Vec<TopFileEntry>,
}

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

#[derive(Debug, Clone, Serialize)]
pub struct ScanResult {
    pub categories: Vec<CategoryTotal>,
    pub top_files_by_category: Vec<CategoryTopFiles>,
    pub total_bytes: u64,
    pub files_scanned: u64,
    pub dirs_scanned: u64,
    pub skipped_errors: u64,
    pub mounts_scanned: usize,
    pub elapsed_ms: u128,
}
