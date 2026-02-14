//! Common utility types shared across models

use serde::{Deserialize, Serialize};

/// A byte range representing a contiguous region (used for GPT usable space, etc.)
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ByteRange {
    /// Start byte (inclusive)
    pub start: u64,
    
    /// End byte (exclusive)
    pub end: u64,
}

impl ByteRange {
    /// Check if this range is valid for a disk of the given size
    pub fn is_valid_for_disk(&self, disk_size: u64) -> bool {
        self.start < self.end && self.end <= disk_size
    }

    /// Clamp this range to fit within a disk of the given size
    pub fn clamp_to_disk(&self, disk_size: u64) -> Self {
        let start = self.start.min(disk_size);
        let end = self.end.min(disk_size);
        Self { start, end }
    }
    
    /// Get the size of this range in bytes
    pub fn size(&self) -> u64 {
        self.end.saturating_sub(self.start)
    }
}

/// Filesystem usage statistics
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Usage {
    /// Filesystem type (e.g., "ext4", "btrfs", "xfs")
    pub filesystem: String,
    
    /// Total blocks
    pub blocks: u64,
    
    /// Used blocks
    pub used: u64,
    
    /// Available blocks
    pub available: u64,
    
    /// Usage percentage (0-100)
    pub percent: u32,
    
    /// Mount point where this filesystem is mounted
    pub mount_point: String,
}

impl Usage {
    /// Get total size in bytes (assuming standard 4K block size)
    pub fn total_bytes(&self) -> u64 {
        self.blocks * 4096
    }
    
    /// Get used size in bytes (assuming standard 4K block size)
    pub fn used_bytes(&self) -> u64 {
        self.used * 4096
    }
    
    /// Get available size in bytes (assuming standard 4K block size)
    pub fn available_bytes(&self) -> u64 {
        self.available * 4096
    }
}
