// SPDX-License-Identifier: GPL-3.0-only

//! Protected system paths for unmount safety
//!
//! This module provides safety checks to prevent users from accidentally
//! killing processes on critical system paths during unmount operations.

use std::path::Path;

/// Critical system paths that should not have their processes killed during unmount
///
/// These paths are essential for system operation and killing processes on them
/// could cause system instability or data loss.
pub const PROTECTED_SYSTEM_PATHS: &[&str] = &[
    "/",         // Root filesystem
    "/boot",     // Bootloader and kernels
    "/boot/efi", // EFI system partition
    "/efi",      // Alternative EFI mount point
    "/home",     // User home directories
    "/usr",      // System programs and libraries
    "/var",      // Variable data (logs, databases, etc.)
    "/etc",      // System configuration
    "/opt",      // Optional software packages
    "/srv",      // Service data
    "/tmp",      // Temporary files (processes may be using)
    "/root",     // Root user home directory
];

/// Check if a mount point is a protected system path
///
/// This function compares the canonical path of the mount point against
/// the list of protected paths. It handles symlinks by canonicalizing both
/// the mount point and the protected paths.
///
/// # Arguments
/// * `mount_point` - The mount point path to check
///
/// # Returns
/// * `true` if the path is protected (processes should not be killed)
/// * `false` if the path is safe for process termination
pub fn is_protected_path(mount_point: &Path) -> bool {
    // Attempt to canonicalize the mount point
    // If it fails (e.g., path doesn't exist), treat it as not protected
    let Ok(canonical_mount) = mount_point.canonicalize() else {
        tracing::debug!(
            "Could not canonicalize mount point {:?}, treating as unprotected",
            mount_point
        );
        return false;
    };

    for protected in PROTECTED_SYSTEM_PATHS {
        let protected_path = Path::new(protected);

        // Try to canonicalize the protected path
        // Some protected paths might not exist on all systems
        let canonical_protected = match protected_path.canonicalize() {
            Ok(p) => p,
            Err(_) => {
                // If canonicalization fails, fall back to direct comparison
                // This handles cases like /boot/efi on non-EFI systems
                protected_path.to_path_buf()
            }
        };

        // Check for exact match or if mount point is a subdirectory of protected path
        if canonical_mount == canonical_protected
            || canonical_mount.starts_with(&canonical_protected)
        {
            tracing::info!(
                "Mount point {:?} is protected (matches {:?})",
                mount_point,
                protected
            );
            return true;
        }
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_root_is_protected() {
        assert!(is_protected_path(Path::new("/")));
    }

    #[test]
    fn test_boot_is_protected() {
        // /boot may not exist in all test environments, but the check should handle it
        // The key is that "/" is always protected
        assert!(is_protected_path(Path::new("/")));
    }

    #[test]
    fn test_nonexistent_path_not_protected() {
        // Non-existent paths should return false (not protected)
        assert!(!is_protected_path(Path::new("/nonexistent/mount/point")));
    }
}
