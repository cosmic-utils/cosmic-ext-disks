// SPDX-License-Identifier: GPL-3.0-only

//! Filesystem tool detection module
//!
//! Detects required tools for various filesystem types and provides
//! information about missing dependencies.

use std::collections::HashMap;
use std::sync::LazyLock;

/// Information about a filesystem tool requirement
#[derive(Debug, Clone)]
pub struct FsToolInfo {
    /// Filesystem type (e.g., "ntfs", "exfat", "xfs")
    #[allow(dead_code)]
    pub fs_type: &'static str,
    /// Human-readable filesystem name
    pub fs_name: &'static str,
    /// Command to check for availability
    pub command: &'static str,
    /// Package name hint (for common distros)
    pub package_hint: &'static str,
    /// Whether this tool is currently available
    pub available: bool,
}

/// Static mapping of filesystem types to their tool requirements
static FS_TOOL_REQUIREMENTS: LazyLock<Vec<FsToolInfo>> = LazyLock::new(|| {
    vec![
        FsToolInfo {
            fs_type: "ntfs",
            fs_name: "NTFS",
            command: "mkfs.ntfs",
            package_hint: "ntfs-3g / ntfsprogs",
            available: false,
        },
        FsToolInfo {
            fs_type: "exfat",
            fs_name: "exFAT",
            command: "mkfs.exfat",
            package_hint: "exfatprogs / exfat-utils",
            available: false,
        },
        FsToolInfo {
            fs_type: "xfs",
            fs_name: "XFS",
            command: "mkfs.xfs",
            package_hint: "xfsprogs",
            available: false,
        },
        FsToolInfo {
            fs_type: "btrfs",
            fs_name: "Btrfs",
            command: "mkfs.btrfs",
            package_hint: "btrfs-progs",
            available: false,
        },
        FsToolInfo {
            fs_type: "f2fs",
            fs_name: "F2FS",
            command: "mkfs.f2fs",
            package_hint: "f2fs-tools",
            available: false,
        },
        FsToolInfo {
            fs_type: "udf",
            fs_name: "UDF",
            command: "mkudffs",
            package_hint: "udftools",
            available: false,
        },
        FsToolInfo {
            fs_type: "vfat",
            fs_name: "FAT32",
            command: "mkfs.vfat",
            package_hint: "dosfstools",
            available: false,
        },
    ]
});

/// Check if a command is available in PATH
fn command_exists(cmd: &str) -> bool {
    which::which(cmd).is_ok()
}

/// Detect all filesystem tools and return their availability status
pub fn detect_fs_tools() -> Vec<FsToolInfo> {
    FS_TOOL_REQUIREMENTS
        .iter()
        .map(|info| {
            let available = command_exists(info.command);
            FsToolInfo { available, ..*info }
        })
        .collect()
}

/// Get a list of missing filesystem tools
pub fn get_missing_tools() -> Vec<FsToolInfo> {
    detect_fs_tools()
        .into_iter()
        .filter(|info| !info.available)
        .collect()
}

/// Get a mapping of filesystem types to their tool availability
#[allow(dead_code)]
pub fn get_fs_tool_status() -> HashMap<String, bool> {
    detect_fs_tools()
        .into_iter()
        .map(|info| (info.fs_type.to_string(), info.available))
        .collect()
}

/// Format missing tools as a human-readable string for display
#[allow(dead_code)]
pub fn format_missing_tools_message(missing: &[FsToolInfo]) -> String {
    if missing.is_empty() {
        return String::from("All filesystem tools are available.");
    }

    let mut msg = String::from("Missing filesystem tools:\n\n");
    for tool in missing {
        msg.push_str(&format!(
            "• {} - required for {} support\n",
            tool.package_hint, tool.fs_name
        ));
    }
    msg.push_str("\nInstall the missing packages to enable full filesystem support.");
    msg
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_fs_tools() {
        let tools = detect_fs_tools();
        assert!(!tools.is_empty());
        assert_eq!(tools.len(), 7); // We define 7 filesystem types
    }

    #[test]
    fn test_fs_tool_structure() {
        let tools = detect_fs_tools();
        for tool in &tools {
            assert!(!tool.fs_type.is_empty());
            assert!(!tool.fs_name.is_empty());
            assert!(!tool.command.is_empty());
            assert!(!tool.package_hint.is_empty());
        }
    }
}
