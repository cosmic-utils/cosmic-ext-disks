// SPDX-License-Identifier: GPL-3.0-only

use anyhow::Result;
use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Privileged helper for BTRFS subvolume operations
#[derive(Parser)]
#[command(name = "cosmic-ext-disks-btrfs-helper")]
#[command(about = "Privileged helper for COSMIC Disks BTRFS operations", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// List all subvolumes in a BTRFS filesystem
    List {
        /// Mount point of the BTRFS filesystem
        mount_point: PathBuf,
    },
    /// Create a new subvolume
    Create {
        /// Mount point of the BTRFS filesystem
        mount_point: PathBuf,
        /// Name of the subvolume to create
        name: String,
    },
    /// Delete a subvolume
    Delete {
        /// Mount point of the BTRFS filesystem
        mount_point: PathBuf,
        /// Path to the subvolume to delete
        path: PathBuf,
        /// Delete recursively
        #[arg(long)]
        recursive: bool,
    },
    /// Create a snapshot of a subvolume
    Snapshot {
        /// Mount point of the BTRFS filesystem
        mount_point: PathBuf,
        /// Source subvolume path
        source: PathBuf,
        /// Destination snapshot path
        dest: PathBuf,
        /// Make the snapshot read-only  
        #[arg(long)]
        readonly: bool,
        /// Create snapshot recursively
        #[arg(long)]
        recursive: bool,
    },
    /// Set or unset the read-only flag on a subvolume
    SetReadonly {
        /// Mount point of the BTRFS filesystem
        mount_point: PathBuf,
        /// Path to the subvolume
        path: PathBuf,
        /// Whether to set read-only (true) or writable (false)
        readonly: bool,
    },
    /// Set a subvolume as the default
    SetDefault {
        /// Mount point of the BTRFS filesystem
        mount_point: PathBuf,
        /// Path to the subvolume
        path: PathBuf,
    },
    /// Get the default subvolume
    GetDefault {
        /// Mount point of the BTRFS filesystem
        mount_point: PathBuf,
    },
    /// List deleted subvolumes pending cleanup
    ListDeleted {
        /// Mount point of the BTRFS filesystem
        mount_point: PathBuf,
    },
    /// Get filesystem usage information
    Usage {
        /// Mount point of the BTRFS filesystem
        mount_point: PathBuf,
    },
}

/// Response for list command including default subvolume ID
#[derive(Debug, Serialize, Deserialize)]
struct ListSubvolumesOutput {
    subvolumes: Vec<SubvolumeOutput>,
    default_id: u64,
}

/// Response for usage command
#[derive(Debug, Serialize, Deserialize)]
struct UsageOutput {
    used_bytes: u64,
}

/// Serializable output format for subvolume info
#[derive(Debug, Serialize, Deserialize)]
struct SubvolumeOutput {
    id: u64,
    path: String,
    parent_id: Option<u64>,
    uuid: String,
    parent_uuid: Option<String>,
    received_uuid: Option<String>,
    generation: u64,
    ctransid: u64,
    otransid: u64,
    stransid: Option<u64>,
    rtransid: Option<u64>,
    ctime: i64,  // Unix timestamp
    otime: i64,  // Unix timestamp
    stime: Option<i64>,  // Unix timestamp
    rtime: Option<i64>,  // Unix timestamp
    flags: u64,
}

fn main() -> Result<()> {
    // Initialize tracing to stderr (so it doesn't interfere with JSON stdout)
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("debug"))
        )
        .init();

    let cli = Cli::parse();

    match cli.command {
        Commands::List { mount_point } => {
            let subvolumes = list_subvolumes(&mount_point).map_err(|e| {
                tracing::error!("Failed to list subvolumes: {}", e);
                e
            })?;
            // Get default ID, fallback to 5 (BTRFS root) if it fails
            let default_id = get_default(&mount_point).unwrap_or_else(|e| {
                tracing::warn!("Failed to get default subvolume, using 5: {}", e);
                5
            });
            let output = ListSubvolumesOutput {
                subvolumes,
                default_id,
            };
            let json = serde_json::to_string(&output)?;
            println!("{}", json);
        }
        Commands::Create { mount_point, name } => {
            create_subvolume(&mount_point, &name)?;
            println!("{{\"success\": true}}");
        }
        Commands::Delete {
            mount_point,
            path,
            recursive,
        } => {
            delete_subvolume(&mount_point, &path, recursive)?;
            println!("{{\"success\": true}}");
        }
        Commands::Snapshot {
            mount_point,
            source,
            dest,
            readonly,
            recursive,
        } => {
            create_snapshot(&mount_point, &source, &dest, readonly, recursive)?;
            println!("{{\"success\": true}}");
        }
        Commands::SetReadonly {
            mount_point,
            path,
            readonly,
        } => {
            set_readonly(&mount_point, &path, readonly)?;
            println!("{{\"success\": true}}");
        }
        Commands::SetDefault { mount_point, path } => {
            set_default(&mount_point, &path)?;
            println!("{{\"success\": true}}");
        }
        Commands::GetDefault { mount_point } => {
            let id = get_default(&mount_point)?;
            println!("{{\"id\": {}}}", id);
        }
        Commands::ListDeleted { mount_point } => {
            let deleted = list_deleted(&mount_point)?;
            let json = serde_json::to_string(&deleted)?;
            println!("{}", json);
        }
        Commands::Usage { mount_point } => {
            let usage = get_usage(&mount_point)?;
            let json = serde_json::to_string(&usage)?;
            println!("{}", json);
        }
    }

    Ok(())
}

/// List all subvolumes in a BTRFS filesystem
fn list_subvolumes(mount_point: &PathBuf) -> Result<Vec<SubvolumeOutput>> {
    // Use btrfs command-line tool instead of btrfsutil iterator
    // The iterator fails with "Could not statfs" when running via pkexec
    use std::process::Command;
    
    let output = Command::new("btrfs")
        .args(&["subvolume", "list", "-a", "-u", "-q", "-R"])
        .arg(mount_point)
        .output()
        .map_err(|e| anyhow::anyhow!("Failed to run btrfs command: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("btrfs command failed: {}", stderr);
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut subvolumes = Vec::new();
    
    // Parse output: ID 256 gen 89534 parent 5 top level 5 parent_uuid - received_uuid - uuid ... path ...
    for line in stdout.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 11 || parts[0] != "ID" {
            continue;
        }

        let id = parts[1].parse::<u64>().ok();
        let generation = parts[3].parse::<u64>().ok();
        
        // Find "path" keyword and take everything after it
        let path_idx = parts.iter().position(|&p| p == "path");
        let mut path = if let Some(idx) = path_idx {
            parts[idx + 1..].join(" ")
        } else {
            continue;
        };

        // Strip "<FS_TREE>/" prefix from paths (comes from -a flag)
        if path.starts_with("<FS_TREE>/") {
            path = path.strip_prefix("<FS_TREE>/").unwrap().to_string();
        }

        // Find UUID field (after "uuid" keyword)
        let uuid_idx = parts.iter().position(|&p| p == "uuid");
        let uuid = if let Some(idx) = uuid_idx {
            parts.get(idx + 1).map(|s| s.to_string()).unwrap_or_else(|| String::from("00000000-0000-0000-0000-000000000000"))
        } else {
            String::from("00000000-0000-0000-0000-000000000000")
        };

        // Find parent_uuid field (after "parent_uuid" keyword)
        let parent_uuid_idx = parts.iter().position(|&p| p == "parent_uuid");
        let parent_uuid = if let Some(idx) = parent_uuid_idx {
            parts.get(idx + 1).and_then(|s| {
                if *s == "-" {
                    None  // "-" means no parent (original subvolume)
                } else {
                    Some(s.to_string())
                }
            })
        } else {
            None
        };

        // Find received_uuid field (after "received_uuid" keyword)
        let received_uuid_idx = parts.iter().position(|&p| p == "received_uuid");
        let received_uuid = if let Some(idx) = received_uuid_idx {
            parts.get(idx + 1).and_then(|s| {
                if *s == "-" {
                    None
                } else {
                    Some(s.to_string())
                }
            })
        } else {
            None
        };

        if let (Some(id), Some(generation)) = (id, generation) {
            subvolumes.push(SubvolumeOutput {
                id,
                path,
                parent_id: None,  // Not critical for UI
                uuid,
                parent_uuid,
                received_uuid,
                generation,
                ctransid: generation,  // Approximate
                otransid: generation,  // Approximate
                stransid: None,
                rtransid: None,
                ctime: 0,  // Not available from list command
                otime: 0,  // Not available from list command
                stime: None,
                rtime: None,
                flags: 0,
            });
        }
    }

    if subvolumes.is_empty() {
        tracing::warn!("No subvolumes found - output may not have been parsed correctly");
    }

    Ok(subvolumes)
}

/// Create a new subvolume
fn create_subvolume(mount_point: &PathBuf, name: &str) -> Result<()> {
    use btrfsutil::subvolume::Subvolume;

    // Build full path for new subvolume
    let subvol_path = mount_point.join(name);

    // Create the subvolume
    Subvolume::create(subvol_path.as_path(), None)
        .map_err(|e| anyhow::anyhow!("Failed to create subvolume '{}': {}", name, e))?;

    Ok(())
}

/// Delete a subvolume
fn delete_subvolume(_mount_point: &PathBuf, path: &PathBuf, recursive: bool) -> Result<()> {
    use btrfsutil::subvolume::{DeleteFlags, Subvolume};

    // Get subvolume handle
    let subvol = Subvolume::try_from(path.as_path())
        .map_err(|e| anyhow::anyhow!("Failed to open subvolume at {}: {}", path.display(), e))?;

    // Set flags
    let flags = if recursive {
        DeleteFlags::RECURSIVE
    } else {
        DeleteFlags::empty()
    };

    // Delete the subvolume
    subvol.delete(flags)
        .map_err(|e| anyhow::anyhow!("Failed to delete subvolume at {}: {}", path.display(), e))?;

    Ok(())
}

/// Create a snapshot of a subvolume
fn create_snapshot(
    _mount_point: &PathBuf,
    source: &PathBuf,
    dest: &PathBuf,
    readonly: bool,
    recursive: bool,
) -> Result<()> {
    use btrfsutil::subvolume::{SnapshotFlags, Subvolume};

    // Get source subvolume handle
    let source_subvol = Subvolume::try_from(source.as_path())
        .map_err(|e| anyhow::anyhow!("Failed to open source subvolume at {}: {}", source.display(), e))?;

    // Set flags
    let mut flags = SnapshotFlags::empty();
    if readonly {
        flags |= SnapshotFlags::READ_ONLY;
    }
    if recursive {
        flags |= SnapshotFlags::RECURSIVE;
    }

    // Create the snapshot
    source_subvol.snapshot(dest.as_path(), flags, None)
        .map_err(|e| anyhow::anyhow!("Failed to create snapshot from {} to {}: {}", 
            source.display(), dest.display(), e))?;

    Ok(())
}

/// Set or unset the read-only flag on a subvolume
fn set_readonly(_mount_point: &PathBuf, path: &PathBuf, readonly: bool) -> Result<()> {
    use btrfsutil::subvolume::Subvolume;

    // Get subvolume handle
    let subvol = Subvolume::try_from(path.as_path())
        .map_err(|e| anyhow::anyhow!("Failed to open subvolume at {}: {}", path.display(), e))?;

    // Set readonly flag
    subvol.set_ro(readonly)
        .map_err(|e| anyhow::anyhow!("Failed to set readonly={} on {}: {}", readonly, path.display(), e))?;

    Ok(())
}

/// Set a subvolume as the default
fn set_default(_mount_point: &PathBuf, path: &PathBuf) -> Result<()> {
    use btrfsutil::subvolume::Subvolume;

    // Get subvolume handle
    let subvol = Subvolume::try_from(path.as_path())
        .map_err(|e| anyhow::anyhow!("Failed to open subvolume at {}: {}", path.display(), e))?;

    // Set as default
    Subvolume::set_default(&subvol)
        .map_err(|e| anyhow::anyhow!("Failed to set default subvolume to {}: {}", path.display(), e))?;

    Ok(())
}

/// Get the default subvolume ID
fn get_default(mount_point: &PathBuf) -> Result<u64> {
    use btrfsutil::subvolume::Subvolume;

    // Get filesystem root
    let root = Subvolume::try_from(mount_point.as_path())
        .map_err(|e| anyhow::anyhow!("Failed to open BTRFS filesystem at {}: {}", mount_point.display(), e))?;

    // Get default subvolume
    let default_subvol = Subvolume::get_default(&root)
        .map_err(|e| anyhow::anyhow!("Failed to get default subvolume: {}", e))?;

    Ok(default_subvol.id())
}

/// List deleted subvolumes pending cleanup
fn list_deleted(mount_point: &PathBuf) -> Result<Vec<SubvolumeOutput>> {
    use btrfsutil::subvolume::Subvolume;

    // Get filesystem root
    let root = Subvolume::try_from(mount_point.as_path())
        .map_err(|e| anyhow::anyhow!("Failed to open BTRFS filesystem at {}: {}", mount_point.display(), e))?;

    // Get deleted subvolumes iterator
    let deleted_iter = Subvolume::deleted(&root)
        .map_err(|e| anyhow::anyhow!("Failed to list deleted subvolumes: {}", e))?;

    let mut deleted = Vec::new();

    for subvol in deleted_iter {
        let info = subvol.info().map_err(|e| anyhow::anyhow!("Failed to get deleted subvolume info: {}", e))?;

        deleted.push(SubvolumeOutput {
            id: info.id,
            path: subvol.path().to_string_lossy().to_string(),
            parent_id: info.parent_id,
            uuid: info.uuid.to_string(),
            parent_uuid: info.parent_uuid.map(|u| u.to_string()),
            received_uuid: info.received_uuid.map(|u| u.to_string()),
            generation: info.generation,
            ctransid: info.ctransid,
            otransid: info.otransid,
            stransid: info.stransid,
            rtransid: info.rtransid,
            ctime: info.ctime.timestamp(),
            otime: info.otime.timestamp(),
            stime: info.stime.map(|t| t.timestamp()),
            rtime: info.rtime.map(|t| t.timestamp()),
            flags: info.flags,
        });
    }

    Ok(deleted)
}

/// Get filesystem usage information
fn get_usage(mount_point: &PathBuf) -> Result<UsageOutput> {
    use std::ffi::CString;
    use std::mem::MaybeUninit;
    
    // Convert path to CString
    let c_path = CString::new(mount_point.to_string_lossy().as_bytes())
        .map_err(|e| anyhow::anyhow!("Invalid mount point path: {}", e))?;
    
    // Call statvfs
    let mut stat: MaybeUninit<libc::statvfs> = MaybeUninit::uninit();
    let result = unsafe {
        libc::statvfs(c_path.as_ptr(), stat.as_mut_ptr())
    };
    
    if result != 0 {
        let err = std::io::Error::last_os_error();
        anyhow::bail!("Failed to get filesystem stats for {}: {}", mount_point.display(), err);
    }
    
    let stat = unsafe { stat.assume_init() };
    
    // Calculate used space
    // f_blocks = total blocks, f_bfree = free blocks
    // f_frsize = fragment size (preferred for calculations)
    let total_bytes = stat.f_blocks * stat.f_frsize;
    let free_bytes = stat.f_bfree * stat.f_frsize;
    let used_bytes = total_bytes - free_bytes;
    
    Ok(UsageOutput { used_bytes })
}
