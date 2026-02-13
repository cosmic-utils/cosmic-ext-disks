//! BTRFS command-line interface wrapper
//!
//! Provides safe wrappers around `btrfs` CLI commands for filesystem management.

use anyhow::{Context, Result, bail};
use std::process::Command;

/// Represents a BTRFS subvolume
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Subvolume {
    /// Subvolume ID
    pub id: u64,
    /// Full path relative to mount point
    pub path: String,
    /// Name (last component of path)
    pub name: String,
}

/// BTRFS filesystem usage breakdown
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UsageInfo {
    /// Data used (bytes)
    pub data_used: u64,
    /// Data total allocation (bytes)
    pub data_total: u64,
    /// Metadata used (bytes)
    pub metadata_used: u64,
    /// Metadata total allocation (bytes)
    pub metadata_total: u64,
    /// System used (bytes)
    pub system_used: u64,
    /// System total allocation (bytes)
    pub system_total: u64,
}

/// Check if the `btrfs` command is available
#[allow(dead_code)]
pub fn command_exists() -> bool {
    Command::new("which")
        .arg("btrfs")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// List all subvolumes for a mounted BTRFS filesystem
///
/// # Arguments
/// * `mount_point` - Path to the mounted BTRFS filesystem
///
/// # Returns
/// Vector of subvolumes, or error if command fails
///
/// # Example
/// ```no_run
/// let subvols = list_subvolumes("/mnt/btrfs").await?;
/// ```
#[allow(dead_code)]
pub async fn list_subvolumes(mount_point: &str) -> Result<Vec<Subvolume>> {
    let output = tokio::process::Command::new("btrfs")
        .args(["subvolume", "list", mount_point])
        .output()
        .await
        .context("Failed to execute 'btrfs subvolume list'")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("btrfs subvolume list failed: {}", stderr);
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    parse_subvolume_list(&stdout)
}

/// Parse output from `btrfs subvolume list`
///
/// Expected format:
/// ```text
/// ID 256 gen 123 top level 5 path @
/// ID 257 gen 124 top level 5 path @home
/// ```
fn parse_subvolume_list(output: &str) -> Result<Vec<Subvolume>> {
    let mut subvolumes = Vec::new();

    for line in output.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        // Parse format: ID <id> gen <gen> top level <level> path <path>
        let parts: Vec<&str> = line.split_whitespace().collect();

        // Find "ID" and "path" keywords
        let id_idx = parts.iter().position(|&p| p == "ID");
        let path_idx = parts.iter().position(|&p| p == "path");

        if let (Some(id_idx), Some(path_idx)) = (id_idx, path_idx)
            && id_idx + 1 < parts.len()
            && path_idx + 1 < parts.len()
        {
            let id = parts[id_idx + 1]
                .parse::<u64>()
                .context("Failed to parse subvolume ID")?;

            // Path may contain spaces, so join all remaining parts
            let path = parts[path_idx + 1..].join(" ");

            // Extract name (last component)
            let name = path.rsplit('/').next().unwrap_or(&path).to_string();

            subvolumes.push(Subvolume { id, path, name });
        }
    }

    Ok(subvolumes)
}

/// Create a new BTRFS subvolume
///
/// # Arguments
/// * `mount_point` - Path to the mounted BTRFS filesystem  
/// * `name` - Name of the subvolume to create (must not contain '/')
///
/// # Returns
/// Ok if successful, error otherwise
#[allow(dead_code)]
pub async fn create_subvolume(mount_point: &str, name: &str) -> Result<()> {
    // Validate name
    if name.is_empty() {
        bail!("Subvolume name cannot be empty");
    }
    if name.contains('/') {
        bail!("Subvolume name cannot contain '/' characters");
    }
    if name.len() > 255 {
        bail!("Subvolume name too long (max 255 characters)");
    }

    let path = format!("{}/{}", mount_point.trim_end_matches('/'), name);

    let output = tokio::process::Command::new("btrfs")
        .args(["subvolume", "create", &path])
        .output()
        .await
        .context("Failed to execute 'btrfs subvolume create'")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("btrfs subvolume create failed: {}", stderr);
    }

    Ok(())
}

/// Delete a BTRFS subvolume
///
/// # Arguments
/// * `path` - Full path to the subvolume (e.g., /mnt/btrfs/@snapshots/snapshot1)
///
/// # Returns
/// Ok if successful, error otherwise
#[allow(dead_code)]
pub async fn delete_subvolume(path: &str) -> Result<()> {
    if path.is_empty() {
        bail!("Subvolume path cannot be empty");
    }

    let output = tokio::process::Command::new("btrfs")
        .args(["subvolume", "delete", path])
        .output()
        .await
        .context("Failed to execute 'btrfs subvolume delete'")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("btrfs subvolume delete failed: {}", stderr);
    }

    Ok(())
}

/// Create a snapshot of a subvolume
///
/// # Arguments
/// * `source` - Path to the source subvolume
/// * `dest` - Path for the snapshot
/// * `read_only` - Whether to create a read-only snapshot
///
/// # Returns
/// Ok if successful, error otherwise
#[allow(dead_code)]
pub async fn create_snapshot(source: &str, dest: &str, read_only: bool) -> Result<()> {
    if source.is_empty() || dest.is_empty() {
        bail!("Source and destination paths cannot be empty");
    }

    let mut args = vec!["subvolume", "snapshot"];
    if read_only {
        args.push("-r");
    }
    args.push(source);
    args.push(dest);

    let output = tokio::process::Command::new("btrfs")
        .args(&args)
        .output()
        .await
        .context("Failed to execute 'btrfs subvolume snapshot'")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("btrfs subvolume snapshot failed: {}", stderr);
    }

    Ok(())
}

/// Get filesystem usage information
///
/// # Arguments
/// * `mount_point` - Path to the mounted BTRFS filesystem
///
/// # Returns
/// UsageInfo struct with allocation details, or error if command fails
#[allow(dead_code)]
pub async fn get_filesystem_usage(mount_point: &str) -> Result<UsageInfo> {
    let output = tokio::process::Command::new("btrfs")
        .args(["filesystem", "usage", "-b", mount_point])
        .output()
        .await
        .context("Failed to execute 'btrfs filesystem usage'")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("btrfs filesystem usage failed: {}", stderr);
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    parse_filesystem_usage(&stdout)
}

/// Parse output from `btrfs filesystem usage -b`
///
/// Expected format:
/// ```text
/// Data,single: Size:107374182400, Used:45234569216
/// Metadata,single: Size:5368709120, Used:2147483648
/// System,single: Size:33554432, Used:16777216
/// ```
fn parse_filesystem_usage(output: &str) -> Result<UsageInfo> {
    let mut data_used = 0;
    let mut data_total = 0;
    let mut metadata_used = 0;
    let mut metadata_total = 0;
    let mut system_used = 0;
    let mut system_total = 0;

    for line in output.lines() {
        let line = line.trim();

        if line.starts_with("Data") {
            if let Some((used, total)) = parse_usage_line(line) {
                data_used = used;
                data_total = total;
            }
        } else if line.starts_with("Metadata") {
            if let Some((used, total)) = parse_usage_line(line) {
                metadata_used = used;
                metadata_total = total;
            }
        } else if line.starts_with("System")
            && let Some((used, total)) = parse_usage_line(line)
        {
            system_used = used;
            system_total = total;
        }
    }

    Ok(UsageInfo {
        data_used,
        data_total,
        metadata_used,
        metadata_total,
        system_used,
        system_total,
    })
}

/// Parse a usage line like "Data,single: Size:107374182400, Used:45234569216"
fn parse_usage_line(line: &str) -> Option<(u64, u64)> {
    // Expected format: "Data,single: Size:12345, Used:6789"
    // Split by comma to separate parts
    let parts: Vec<&str> = line.split(',').collect();
    if parts.len() < 2 {
        return None;
    }

    // Find Size: and Used: values
    let mut total = None;
    let mut used = None;

    for part in &parts {
        let part = part.trim();

        // Look for "Size:" anywhere in the part
        if let Some(idx) = part.find("Size:") {
            let size_str = &part[idx + 5..].trim();
            total = size_str.parse::<u64>().ok();
        }

        // Look for "Used:" anywhere in the part
        if let Some(idx) = part.find("Used:") {
            let used_str = &part[idx + 5..].trim();
            used = used_str.parse::<u64>().ok();
        }
    }

    match (used, total) {
        (Some(u), Some(t)) => Some((u, t)),
        _ => None,
    }
}

/// Get compression setting for a BTRFS filesystem or directory
///
/// # Arguments
/// * `path` - Path to query (filesystem or directory)
///
/// # Returns
/// Some(algorithm) if compression is enabled, None if disabled
#[allow(dead_code)]
pub async fn get_compression(path: &str) -> Result<Option<String>> {
    let output = tokio::process::Command::new("btrfs")
        .args(["property", "get", path, "compression"])
        .output()
        .await
        .context("Failed to execute 'btrfs property get'")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("btrfs property get failed: {}", stderr);
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    // Expected output: "compression=zstd\n" or "compression=\n"
    if let Some(line) = stdout.lines().next()
        && let Some(value) = line.split('=').nth(1)
    {
        let value = value.trim();
        if value.is_empty() || value == "none" {
            return Ok(None);
        }
        return Ok(Some(value.to_string()));
    }

    Ok(None)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_subvolume_list() {
        let output = "\
ID 256 gen 123 top level 5 path @
ID 257 gen 124 top level 5 path @home
ID 258 gen 125 top level 256 path @snapshots/test
";
        let result = parse_subvolume_list(output).unwrap();
        assert_eq!(result.len(), 3);
        assert_eq!(result[0].id, 256);
        assert_eq!(result[0].path, "@");
        assert_eq!(result[0].name, "@");
        assert_eq!(result[1].id, 257);
        assert_eq!(result[1].path, "@home");
        assert_eq!(result[2].id, 258);
        assert_eq!(result[2].path, "@snapshots/test");
        assert_eq!(result[2].name, "test");
    }

    #[test]
    fn test_parse_filesystem_usage() {
        let output = "\
Data,single: Size:107374182400, Used:45234569216
Metadata,single: Size:5368709120, Used:2147483648
System,single: Size:33554432, Used:16777216
";
        let result = parse_filesystem_usage(output).unwrap();
        assert_eq!(result.data_total, 107374182400);
        assert_eq!(result.data_used, 45234569216);
        assert_eq!(result.metadata_total, 5368709120);
        assert_eq!(result.metadata_used, 2147483648);
        assert_eq!(result.system_total, 33554432);
        assert_eq!(result.system_used, 16777216);
    }

    #[test]
    fn test_parse_usage_line() {
        let line = "Data,single: Size:107374182400, Used:45234569216";
        let (used, total) = parse_usage_line(line).unwrap();
        assert_eq!(total, 107374182400);
        assert_eq!(used, 45234569216);
    }
}
