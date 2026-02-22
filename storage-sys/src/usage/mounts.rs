// SPDX-License-Identifier: GPL-3.0-only

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use super::error::UsageScanError;

const EXCLUDED_FS_TYPES: &[&str] = &[
    "autofs",
    "binfmt_misc",
    "bpf",
    "cgroup",
    "cgroup2",
    "configfs",
    "debugfs",
    "devpts",
    "devtmpfs",
    "efivarfs",
    "fusectl",
    "hugetlbfs",
    "mqueue",
    "nfs",
    "nfs4",
    "nsfs",
    "overlay",
    "proc",
    "pstore",
    "ramfs",
    "rpc_pipefs",
    "securityfs",
    "selinuxfs",
    "smb3",
    "smbfs",
    "squashfs",
    "sysfs",
    "tmpfs",
    "tracefs",
    "fuse.sshfs",
    "cifs",
    "9p",
    "ceph",
    "glusterfs",
    "lustre",
    "sshfs",
];

pub fn discover_local_mounts_under(root: &Path) -> Result<Vec<PathBuf>, UsageScanError> {
    let mount_info = fs::read_to_string("/proc/self/mountinfo")?;
    let mut mount_points = parse_local_mounts(&mount_info)?;

    if root == Path::new("/") {
        return Ok(mount_points);
    }

    let root = root.to_path_buf();
    mount_points.retain(|mount| mount.starts_with(&root));
    if mount_points.is_empty() {
        mount_points.push(root);
    }

    Ok(mount_points)
}

pub fn parse_local_mounts(input: &str) -> Result<Vec<PathBuf>, UsageScanError> {
    let mut roots = BTreeSet::new();

    for line in input.lines().filter(|line| !line.trim().is_empty()) {
        let (left, right) = line
            .split_once(" - ")
            .ok_or_else(|| UsageScanError::InvalidMountInfoLine(line.to_string()))?;

        let mut left_fields = left.split_whitespace();
        let mount_point = left_fields
            .nth(4)
            .ok_or_else(|| UsageScanError::InvalidMountInfoLine(line.to_string()))?;

        let fs_type = right
            .split_whitespace()
            .next()
            .ok_or_else(|| UsageScanError::InvalidMountInfoLine(line.to_string()))?;

        if is_non_local_fs_type(fs_type) {
            continue;
        }

        roots.insert(PathBuf::from(unescape_mount_field(mount_point)));
    }

    Ok(roots.into_iter().collect())
}

fn is_non_local_fs_type(fs_type: &str) -> bool {
    if EXCLUDED_FS_TYPES.contains(&fs_type) {
        return true;
    }

    fs_type.starts_with("fuse.") || fs_type.starts_with("nfs")
}

fn unescape_mount_field(value: &str) -> String {
    let mut output = String::with_capacity(value.len());
    let bytes = value.as_bytes();
    let mut index = 0;

    while index < bytes.len() {
        if bytes[index] == b'\\'
            && index + 3 < bytes.len()
            && bytes[index + 1].is_ascii_digit()
            && bytes[index + 2].is_ascii_digit()
            && bytes[index + 3].is_ascii_digit()
        {
            let octal = &value[index + 1..index + 4];
            if let Ok(num) = u8::from_str_radix(octal, 8) {
                output.push(num as char);
                index += 4;
                continue;
            }
        }

        output.push(bytes[index] as char);
        index += 1;
    }

    output
}

#[cfg(test)]
mod tests {
    use super::parse_local_mounts;

    #[test]
    fn parses_mountinfo_and_filters_non_local_types() {
        let sample = "36 25 8:2 / / rw,relatime - ext4 /dev/nvme0n1p2 rw\n37 25 0:5 / /proc rw,nosuid,nodev,noexec,relatime - proc proc rw\n38 25 0:57 / /mnt/nfs rw,relatime - nfs server:/x rw\n";

        let mounts = parse_local_mounts(sample).expect("parse should succeed");
        assert_eq!(mounts, vec![std::path::PathBuf::from("/")]);
    }
}
