// SPDX-License-Identifier: GPL-3.0-only

use storage_common::FilesystemToolInfo;

pub trait FilesystemsDomain: Send + Sync {
    fn detect_all_filesystem_tools(&self) -> Vec<FilesystemToolInfo>;
    fn require_filesystem_format_support(
        &self,
        fs_type: &str,
        supported_tools: &[String],
    ) -> zbus::fdo::Result<()>;
    fn require_filesystem_check_support(&self) -> zbus::fdo::Result<()>;
}

pub struct DefaultFilesystemsDomain;

impl DefaultFilesystemsDomain {
    #[allow(clippy::match_like_matches_macro)]
    fn feature_enabled_for_fs(fs_type: &str) -> bool {
        match fs_type.to_ascii_lowercase().as_str() {
            "ext4" => cfg!(feature = "fs-ext4"),
            "xfs" => cfg!(feature = "fs-xfs"),
            "btrfs" => cfg!(feature = "fs-btrfs"),
            "vfat" => cfg!(feature = "fs-vfat"),
            "ntfs" => cfg!(feature = "fs-ntfs"),
            "exfat" => cfg!(feature = "fs-exfat"),
            _ => false,
        }
    }

    fn any_filesystem_feature_enabled() -> bool {
        cfg!(feature = "fs-ext4")
            || cfg!(feature = "fs-xfs")
            || cfg!(feature = "fs-btrfs")
            || cfg!(feature = "fs-vfat")
            || cfg!(feature = "fs-ntfs")
            || cfg!(feature = "fs-exfat")
    }
}

impl FilesystemsDomain for DefaultFilesystemsDomain {
    fn detect_all_filesystem_tools(&self) -> Vec<FilesystemToolInfo> {
        let tools = vec![
            ("ext4", "EXT4", "mkfs.ext4", "e2fsprogs"),
            ("xfs", "XFS", "mkfs.xfs", "xfsprogs"),
            ("btrfs", "Btrfs", "mkfs.btrfs", "btrfs-progs"),
            ("vfat", "FAT32", "mkfs.vfat", "dosfstools"),
            ("ntfs", "NTFS", "mkfs.ntfs", "ntfs-3g"),
            ("exfat", "exFAT", "mkfs.exfat", "exfat-utils"),
        ];

        tools
            .into_iter()
            .map(|(fs_type, fs_name, command, package_hint)| {
                let feature_enabled = Self::feature_enabled_for_fs(fs_type);
                let available = feature_enabled && which::which(command).is_ok();
                FilesystemToolInfo {
                    fs_type: fs_type.to_string(),
                    fs_name: fs_name.to_string(),
                    command: command.to_string(),
                    package_hint: package_hint.to_string(),
                    available,
                }
            })
            .collect()
    }

    fn require_filesystem_format_support(
        &self,
        fs_type: &str,
        supported_tools: &[String],
    ) -> zbus::fdo::Result<()> {
        if !Self::feature_enabled_for_fs(fs_type) {
            return Err(zbus::fdo::Error::Failed(format!(
                "Filesystem type '{}' unavailable: compile-time feature disabled",
                fs_type
            )));
        }

        if !supported_tools.contains(&fs_type.to_string()) {
            return Err(zbus::fdo::Error::Failed(format!(
                "Filesystem type '{}' is not supported or tools not installed",
                fs_type
            )));
        }

        Ok(())
    }

    fn require_filesystem_check_support(&self) -> zbus::fdo::Result<()> {
        if !Self::any_filesystem_feature_enabled() {
            return Err(zbus::fdo::Error::Failed(
                "Filesystem check unavailable: all filesystem features disabled".to_string(),
            ));
        }

        Ok(())
    }
}
