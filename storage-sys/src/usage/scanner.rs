// SPDX-License-Identifier: GPL-3.0-only

use std::collections::BTreeMap;
use std::fs;
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};
use std::time::Instant;

use rayon::prelude::*;
use rayon::ThreadPoolBuilder;

use super::classifier::classify_path;
use super::error::UsageScanError;
use super::types::{Category, CategoryTotal, ScanConfig, ScanResult};

#[derive(Default)]
struct LocalStats {
    bytes_by_category: BTreeMap<Category, u64>,
    total_bytes: u64,
    files_scanned: u64,
    dirs_scanned: u64,
    skipped_errors: u64,
}

impl LocalStats {
    fn add_file(&mut self, path: &Path, bytes: u64) {
        self.total_bytes += bytes;
        self.files_scanned += 1;

        let category = classify_path(path);
        *self.bytes_by_category.entry(category).or_insert(0) += bytes;
    }

    fn merge(&mut self, other: LocalStats) {
        self.total_bytes += other.total_bytes;
        self.files_scanned += other.files_scanned;
        self.dirs_scanned += other.dirs_scanned;
        self.skipped_errors += other.skipped_errors;

        for (category, bytes) in other.bytes_by_category {
            *self.bytes_by_category.entry(category).or_insert(0) += bytes;
        }
    }
}

pub fn scan_paths(roots: &[PathBuf], config: &ScanConfig) -> Result<ScanResult, UsageScanError> {
    let started = Instant::now();
    let mounts_scanned = roots.len();

    if roots.is_empty() {
        return Ok(ScanResult {
            categories: Vec::new(),
            total_bytes: 0,
            files_scanned: 0,
            dirs_scanned: 0,
            skipped_errors: 0,
            mounts_scanned: 0,
            elapsed_ms: 0,
        });
    }

    let threads = config
        .threads
        .unwrap_or_else(|| std::thread::available_parallelism().map_or(4, usize::from));

    let pool = ThreadPoolBuilder::new()
        .num_threads(threads.max(1))
        .build()
        .map_err(|error| UsageScanError::ThreadPoolBuild(error.to_string()))?;

    let root_stats: Vec<LocalStats> = pool.install(|| {
        roots
            .par_iter()
            .map(|root| scan_single_root(root.as_path()))
            .collect()
    });

    let mut combined = LocalStats::default();
    for stats in root_stats {
        combined.merge(stats);
    }

    let mut categories: Vec<CategoryTotal> = Category::ALL
        .iter()
        .map(|category| CategoryTotal {
            category: *category,
            bytes: *combined.bytes_by_category.get(category).unwrap_or(&0),
        })
        .collect();

    categories.sort_by(|left, right| {
        right
            .bytes
            .cmp(&left.bytes)
            .then_with(|| left.category.cmp(&right.category))
    });

    Ok(ScanResult {
        categories,
        total_bytes: combined.total_bytes,
        files_scanned: combined.files_scanned,
        dirs_scanned: combined.dirs_scanned,
        skipped_errors: combined.skipped_errors,
        mounts_scanned,
        elapsed_ms: started.elapsed().as_millis(),
    })
}

fn scan_single_root(root: &Path) -> LocalStats {
    let mut stats = LocalStats::default();

    let root_metadata = match fs::symlink_metadata(root) {
        Ok(metadata) => metadata,
        Err(_) => {
            stats.skipped_errors += 1;
            return stats;
        }
    };

    if !root_metadata.is_dir() {
        stats.skipped_errors += 1;
        return stats;
    }

    let root_dev = root_metadata.dev();
    let mut stack = vec![root.to_path_buf()];

    while let Some(directory) = stack.pop() {
        stats.dirs_scanned += 1;

        let entries = match fs::read_dir(&directory) {
            Ok(entries) => entries,
            Err(_) => {
                stats.skipped_errors += 1;
                continue;
            }
        };

        for entry in entries {
            let entry = match entry {
                Ok(entry) => entry,
                Err(_) => {
                    stats.skipped_errors += 1;
                    continue;
                }
            };

            let path = entry.path();
            let metadata = match fs::symlink_metadata(&path) {
                Ok(metadata) => metadata,
                Err(_) => {
                    stats.skipped_errors += 1;
                    continue;
                }
            };

            if metadata.is_file() {
                stats.add_file(&path, metadata.len());
                continue;
            }

            if metadata.is_dir() && metadata.dev() == root_dev {
                stack.push(path);
            }
        }
    }

    stats
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicU64, Ordering};

    use super::*;

    static COUNTER: AtomicU64 = AtomicU64::new(1);

    struct TempDir {
        path: PathBuf,
    }

    impl TempDir {
        fn new() -> Self {
            let unique = COUNTER.fetch_add(1, Ordering::Relaxed);
            let path = std::env::temp_dir().join(format!("storage-sys-usage-scan-{unique}"));
            fs::create_dir_all(&path).expect("create temp dir");
            Self { path }
        }
    }

    impl Drop for TempDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.path);
        }
    }

    #[test]
    fn aggregates_category_bytes_over_tree() {
        let temp = TempDir::new();
        fs::write(temp.path.join("main.rs"), vec![b'a'; 10]).expect("write rs file");
        fs::write(temp.path.join("pic.png"), vec![b'a'; 20]).expect("write image file");
        fs::write(temp.path.join("note.txt"), vec![b'a'; 30]).expect("write document file");

        let result = scan_paths(std::slice::from_ref(&temp.path), &ScanConfig::default())
            .expect("scan should succeed");

        let code = result
            .categories
            .iter()
            .find(|entry| entry.category == Category::Code)
            .map(|entry| entry.bytes)
            .unwrap_or(0);
        let images = result
            .categories
            .iter()
            .find(|entry| entry.category == Category::Images)
            .map(|entry| entry.bytes)
            .unwrap_or(0);
        let documents = result
            .categories
            .iter()
            .find(|entry| entry.category == Category::Documents)
            .map(|entry| entry.bytes)
            .unwrap_or(0);

        assert_eq!(code, 10);
        assert_eq!(images, 20);
        assert_eq!(documents, 30);
        assert_eq!(result.total_bytes, 60);
    }
}
