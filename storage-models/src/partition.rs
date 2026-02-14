//! Partition information - flat representation
//!
//! PartitionInfo provides detailed metadata about a single partition,
//! suitable for partition management operations.

use serde::{Deserialize, Serialize};

use crate::Usage;

/// Partition table type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PartitionTableType {
    /// GPT (GUID Partition Table)
    Gpt,
    
    /// MBR/DOS (Master Boot Record)
    Mbr,
}

impl PartitionTableType {
    /// Convert to UDisks2 string format
    pub fn as_udisks_str(&self) -> &'static str {
        match self {
            Self::Gpt => "gpt",
            Self::Mbr => "dos",
        }
    }
    
    /// Parse from UDisks2 string format
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "gpt" => Some(Self::Gpt),
            "dos" | "mbr" => Some(Self::Mbr),
            _ => None,
        }
    }
}

/// Partition creation request data (UI model)
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct CreatePartitionInfo {
    pub name: String,
    pub size: u64,
    pub max_size: u64,
    pub offset: u64,
    pub erase: bool,
    pub selected_type: String,
    pub selected_partition_type_index: usize,
    pub password_protected: bool,
    pub password: String,
    pub confirmed_password: String,
    pub can_continue: bool,
    pub filesystem_type: String,
    pub table_type: String,
    pub size_text: String,
    pub size_unit_index: usize,
}

/// Detailed partition information
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PartitionInfo {
    /// Device path (e.g., "/dev/sda1")
    pub device: String,
    
    /// Partition number (1-based)
    pub number: u32,
    
    /// Parent disk device (e.g., "/dev/sda")
    pub parent_device: String,
    
    /// Size in bytes
    pub size: u64,
    
    /// Offset from start of disk in bytes
    pub offset: u64,
    
    /// Partition type identifier (GPT GUID or MBR type code)
    pub type_id: String,
    
    /// Human-readable partition type name
    pub type_name: String,
    
    /// Partition flags (bitfield)
    pub flags: u64,
    
    /// Partition name (GPT only, empty for MBR)
    pub name: String,
    
    /// Partition UUID
    pub uuid: String,
    
    /// Partition table type
    pub table_type: String,
    
    /// Whether this partition has a filesystem
    pub has_filesystem: bool,
    
    /// Filesystem type (if has_filesystem is true)
    pub filesystem_type: Option<String>,
    
    /// Current mount points (empty if not mounted)
    pub mount_points: Vec<String>,
    
    /// Filesystem usage (if mounted)
    pub usage: Option<Usage>,
}

impl PartitionInfo {
    /// Check if this partition is currently mounted
    pub fn is_mounted(&self) -> bool {
        self.has_filesystem && !self.mount_points.is_empty()
    }

    /// Check if this partition can be mounted
    pub fn can_mount(&self) -> bool {
        self.has_filesystem && !self.is_mounted()
    }
    
    /// Get a display name for this partition
    pub fn display_name(&self) -> String {
        if !self.name.is_empty() {
            self.name.clone()
        } else if self.number > 0 {
            format!("Partition {}", self.number)
        } else {
            "Filesystem".to_string()
        }
    }
    
    /// Check if this is a GPT partition
    pub fn is_gpt(&self) -> bool {
        self.table_type == "gpt"
    }
    
    /// Check if this is an MBR partition
    pub fn is_mbr(&self) -> bool {
        self.table_type == "dos" || self.table_type == "mbr"
    }
}

/// Partition table information
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PartitionTableInfo {
    /// Device path of the disk (e.g., "/dev/sda")
    pub device: String,
    
    /// Table type (GPT or MBR)
    pub table_type: PartitionTableType,
    
    /// Maximum number of partitions supported
    pub max_partitions: u32,
}
