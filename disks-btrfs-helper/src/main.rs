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
}

/// Response for list command including default subvolume ID
#[derive(Debug, Serialize, Deserialize)]
struct ListSubvolumesOutput {
    subvolumes: Vec<SubvolumeOutput>,
    default_id: u64,
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
    let cli = Cli::parse();

    match cli.command {
        Commands::List { mount_point } => {
            let subvolumes = list_subvolumes(&mount_point)?;
            let default_id = get_default(&mount_point)?;
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
    }

    Ok(())
}

/// List all subvolumes in a BTRFS filesystem
fn list_subvolumes(mount_point: &PathBuf) -> Result<Vec<SubvolumeOutput>> {
    use btrfsutil::subvolume::{Subvolume, SubvolumeIterator};

    // Get the filesystem root
    let root = Subvolume::try_from(mount_point.as_path())
        .map_err(|e| anyhow::anyhow!("Failed to open BTRFS filesystem at {}: {}", mount_point.display(), e))?;

    // Create iterator for all subvolumes
    let iter = SubvolumeIterator::try_from(&root)
        .map_err(|e| anyhow::anyhow!("Failed to create subvolume iterator: {}", e))?;

    let mut subvolumes = Vec::new();

    for item in iter {
        let subvol = item.map_err(|e| anyhow::anyhow!("Failed to iterate subvolume: {}", e))?;
        let info = subvol.info().map_err(|e| anyhow::anyhow!("Failed to get subvolume info: {}", e))?;

        subvolumes.push(SubvolumeOutput {
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
