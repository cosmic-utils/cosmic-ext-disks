// SPDX-License-Identifier: GPL-3.0-only

//! Native BTRFS filesystem operations using btrfsutil
//!
//! This module provides async wrappers around the btrfsutil library,
//! using a privileged helper binary for operations requiring CAP_SYS_ADMIN.

use anyhow::{Context, Result};
use chrono::{DateTime, Local, TimeZone};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use uuid::Uuid;

/// Extended subvolume information with all metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BtrfsSubvolume {
    // Identity
    pub id: u64,
    pub path: PathBuf,
    pub parent_id: Option<u64>,

    // UUIDs for tracking relationships
    pub uuid: Uuid,
    pub parent_uuid: Option<Uuid>,  // For snapshot source tracking
    pub received_uuid: Option<Uuid>, // For send/receive tracking

    // Timestamps
    pub created: DateTime<Local>,  // otime - original/creation time
    pub modified: DateTime<Local>, // ctime - change time

    // Transaction IDs
    pub generation: u64,
    pub ctransid: u64,
    pub otransid: u64,
    pub stransid: Option<u64>,
    pub rtransid: Option<u64>,

    // Properties
    pub flags: u64,
    pub is_readonly: bool,
    pub is_default: bool,
}

/// BTRFS filesystem operations interface
pub struct BtrfsFilesystem {
    mount_point: PathBuf,
    helper: BtrfsHelper,
}

impl BtrfsFilesystem {
    /// Create a new BTRFS filesystem interface
    pub async fn new(mount_point: PathBuf) -> Result<Self> {
        let helper = BtrfsHelper::new()
            .context("Failed to initialize BTRFS helper")?;

        Ok(Self {
            mount_point,
            helper,
        })
    }

    /// List all subvolumes in the filesystem
    pub async fn list_subvolumes(&self) -> Result<Vec<BtrfsSubvolume>> {
        let operation = Operation::ListSubvolumes {
            mount_point: self.mount_point.clone(),
        };

        let output = self.helper.execute(operation).await
            .context("Failed to list subvolumes")?;

        // Parse the response which now includes both subvolumes and default_id
        let list_output: ListSubvolumesOutput = serde_json::from_value(output)
            .context("Failed to deserialize subvolume list")?;

        let default_id = list_output.default_id;

        // Convert to BtrfsSubvolume
        let mut subvolumes = Vec::new();
        for helper_output in list_output.subvolumes {
            let mut subvol = BtrfsSubvolume::try_from(helper_output)
                .context("Failed to convert subvolume data")?;

            // Mark if this is the default subvolume
            subvol.is_default = subvol.id == default_id;

            subvolumes.push(subvol);
        }

        Ok(subvolumes)
    }

    /// Create a new subvolume
    pub async fn create_subvolume(&self, name: &str) -> Result<BtrfsSubvolume> {
        let operation = Operation::CreateSubvolume {
            mount_point: self.mount_point.clone(),
            name: name.to_string(),
        };

        self.helper.execute(operation).await
            .context("Failed to create subvolume")?;

        // Query the newly created subvolume info
        let subvol_path = self.mount_point.join(name);
        self.get_subvolume_info(&subvol_path).await
    }

    /// Delete a subvolume
    pub async fn delete_subvolume(&self, path: &Path) -> Result<()> {
        let operation = Operation::DeleteSubvolume {
            mount_point: self.mount_point.clone(),
            path: path.to_path_buf(),
            recursive: false,
        };

        self.helper.execute(operation).await
            .context("Failed to delete subvolume")?;

        Ok(())
    }

    /// Create a snapshot of a subvolume
    pub async fn create_snapshot(
        &self,
        source: &Path,
        dest: &Path,
        readonly: bool,
    ) -> Result<BtrfsSubvolume> {
        let operation = Operation::CreateSnapshot {
            mount_point: self.mount_point.clone(),
            source: source.to_path_buf(),
            dest: dest.to_path_buf(),
            readonly,
            recursive: false,
        };

        self.helper.execute(operation).await
            .context("Failed to create snapshot")?;

        // Query the newly created snapshot info
        self.get_subvolume_info(dest).await
    }

    /// Get detailed information about a subvolume
    pub async fn get_subvolume_info(&self, path: &Path) -> Result<BtrfsSubvolume> {
        // List all subvolumes and find the one with matching path
        let subvolumes = self.list_subvolumes().await?;
        
        let canonical_path = path.canonicalize()
            .unwrap_or_else(|_| path.to_path_buf());

        subvolumes
            .into_iter()
            .find(|s| {
                let subvol_canonical = s.path.canonicalize()
                    .unwrap_or_else(|_| s.path.clone());
                subvol_canonical == canonical_path
            })
            .ok_or_else(|| anyhow::anyhow!("Subvolume not found at {}", path.display()))
    }

    /// Set or unset the read-only flag on a subvolume
    pub async fn set_readonly(&self, path: &Path, readonly: bool) -> Result<()> {
        let operation = Operation::SetReadonly {
            mount_point: self.mount_point.clone(),
            path: path.to_path_buf(),
            readonly,
        };

        self.helper.execute(operation).await
            .context("Failed to set readonly flag")?;

        Ok(())
    }

    /// Get the default subvolume
    pub async fn get_default_subvolume(&self) -> Result<BtrfsSubvolume> {
        // List all subvolumes (which includes marking the default)
        let subvolumes = self.list_subvolumes().await?;
        
        subvolumes
            .into_iter()
            .find(|s| s.is_default)
            .ok_or_else(|| anyhow::anyhow!("Default subvolume not found"))
    }

    /// Set a subvolume as the default
    pub async fn set_default_subvolume(&self, path: &Path) -> Result<()> {
        let operation = Operation::SetDefault {
            mount_point: self.mount_point.clone(),
            path: path.to_path_buf(),
        };

        self.helper.execute(operation).await
            .context("Failed to set default subvolume")?;

        Ok(())
    }

    /// List deleted subvolumes pending cleanup
    pub async fn list_deleted_subvolumes(&self) -> Result<Vec<BtrfsSubvolume>> {
        let operation = Operation::ListDeleted {
            mount_point: self.mount_point.clone(),
        };

        let output = self.helper.execute(operation).await
            .context("Failed to list deleted subvolumes")?;

        // Parse array of deleted subvolumes
        let helper_outputs: Vec<SubvolumeHelperOutput> = serde_json::from_value(output)
            .context("Failed to deserialize deleted subvolume list")?;

        // Convert to BtrfsSubvolume
        helper_outputs
            .into_iter()
            .map(|output| BtrfsSubvolume::try_from(output))
            .collect::<Result<Vec<_>>>()
            .context("Failed to convert deleted subvolume data")
    }

    /// Check if a path is a subvolume
    pub async fn is_subvolume(&self, path: &Path) -> Result<bool> {
        // Try to get subvolume info - if it succeeds, it's a subvolume
        Ok(self.get_subvolume_info(path).await.is_ok())
    }
}

/// Helper binary wrapper for privilege escalation
struct BtrfsHelper {
    helper_path: PathBuf,
}

impl BtrfsHelper {
    /// Create a new helper instance
    fn new() -> Result<Self> {
        // Try common installation paths
        let paths = [
            PathBuf::from("/usr/libexec/cosmic-ext-disks-btrfs-helper"),
            PathBuf::from("/usr/local/libexec/cosmic-ext-disks-btrfs-helper"),
            // For development/testing, also check in target directory
            PathBuf::from("target/debug/cosmic-ext-disks-btrfs-helper"),
            PathBuf::from("target/release/cosmic-ext-disks-btrfs-helper"),
        ];

        for path in &paths {
            if path.exists() {
                return Ok(Self {
                    helper_path: path.clone(),
                });
            }
        }

        anyhow::bail!(
            "BTRFS helper binary not found. Expected at: {}",
            paths[0].display()
        )
    }

    /// Execute a privileged operation via the helper binary
    async fn execute(&self, operation: Operation) -> Result<serde_json::Value> {
        // Convert operation to command-line arguments
        let args = operation.to_args();

        // Spawn helper via pkexec for privilege escalation
        let output = tokio::process::Command::new("pkexec")
            .arg(&self.helper_path)
            .args(&args)
            .output()
            .await
            .context("Failed to spawn helper binary via pkexec")?;

        // Check exit status
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!(
                "Helper binary failed with exit code {}: {}",
                output.status.code().unwrap_or(-1),
                stderr
            );
        }

        // Parse JSON output
        let stdout = String::from_utf8(output.stdout)
            .context("Helper output is not valid UTF-8")?;

        serde_json::from_str(&stdout)
            .with_context(|| format!("Failed to parse helper JSON output. Received {} bytes: {:?}", 
                stdout.len(), 
                stdout.chars().take(200).collect::<String>()))
    }
}

impl Operation {
    /// Convert operation to command-line arguments for the helper binary
    fn to_args(&self) -> Vec<String> {
        match self {
            Operation::ListSubvolumes { mount_point } => {
                vec!["list".to_string(), mount_point.to_string_lossy().to_string()]
            }
            Operation::CreateSubvolume { mount_point, name } => {
                vec![
                    "create".to_string(),
                    mount_point.to_string_lossy().to_string(),
                    name.clone(),
                ]
            }
            Operation::DeleteSubvolume {
                mount_point,
                path,
                recursive,
            } => {
                let mut args = vec![
                    "delete".to_string(),
                    mount_point.to_string_lossy().to_string(),
                    path.to_string_lossy().to_string(),
                ];
                if *recursive {
                    args.push("--recursive".to_string());
                }
                args
            }
            Operation::CreateSnapshot {
                mount_point,
                source,
                dest,
                readonly,
                recursive,
            } => {
                let mut args = vec![
                    "snapshot".to_string(),
                    mount_point.to_string_lossy().to_string(),
                    source.to_string_lossy().to_string(),
                    dest.to_string_lossy().to_string(),
                ];
                if *readonly {
                    args.push("--readonly".to_string());
                }
                if *recursive {
                    args.push("--recursive".to_string());
                }
                args
            }
            Operation::SetReadonly {
                mount_point,
                path,
                readonly,
            } => {
                vec![
                    "set-readonly".to_string(),
                    mount_point.to_string_lossy().to_string(),
                    path.to_string_lossy().to_string(),
                    readonly.to_string(),
                ]
            }
            Operation::SetDefault { mount_point, path } => {
                vec![
                    "set-default".to_string(),
                    mount_point.to_string_lossy().to_string(),
                    path.to_string_lossy().to_string(),
                ]
            }
            Operation::GetDefault { mount_point } => {
                vec![
                    "get-default".to_string(),
                    mount_point.to_string_lossy().to_string(),
                ]
            }
            Operation::ListDeleted { mount_point } => {
                vec![
                    "list-deleted".to_string(),
                    mount_point.to_string_lossy().to_string(),
                ]
            }
        }
    }
}

/// Operations that can be performed via the helper binary
#[derive(Debug, Clone)]
enum Operation {
    ListSubvolumes {
        mount_point: PathBuf,
    },
    CreateSubvolume {
        mount_point: PathBuf,
        name: String,
    },
    DeleteSubvolume {
        mount_point: PathBuf,
        path: PathBuf,
        recursive: bool,
    },
    CreateSnapshot {
        mount_point: PathBuf,
        source: PathBuf,
        dest: PathBuf,
        readonly: bool,
        recursive: bool,
    },
    SetReadonly {
        mount_point: PathBuf,
        path: PathBuf,
        readonly: bool,
    },
    SetDefault {
        mount_point: PathBuf,
        path: PathBuf,
    },
    GetDefault {
        mount_point: PathBuf,
    },
    ListDeleted {
        mount_point: PathBuf,
    },
}

/// Response from list command including default subvolume ID
#[derive(Debug, Deserialize)]
struct ListSubvolumesOutput {
    subvolumes: Vec<SubvolumeHelperOutput>,
    default_id: u64,
}

/// Helper output format (matches SubvolumeOutput in helper binary)
#[derive(Debug, Deserialize)]
struct SubvolumeHelperOutput {
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
    ctime: i64,
    otime: i64,
    stime: Option<i64>,
    rtime: Option<i64>,
    flags: u64,
}

/// Convert helper output to BtrfsSubvolume
impl TryFrom<SubvolumeHelperOutput> for BtrfsSubvolume {
    type Error = anyhow::Error;

    fn try_from(output: SubvolumeHelperOutput) -> Result<Self> {
        Ok(BtrfsSubvolume {
            id: output.id,
            path: PathBuf::from(output.path),
            parent_id: output.parent_id,
            uuid: output.uuid.parse()
                .context("Failed to parse UUID")?,
            parent_uuid: output.parent_uuid
                .map(|s| s.parse())
                .transpose()
                .context("Failed to parse parent UUID")?,
            received_uuid: output.received_uuid
                .map(|s| s.parse())
                .transpose()
                .context("Failed to parse received UUID")?,
            created: Local.timestamp_opt(output.otime, 0)
                .single()
                .context("Invalid otime timestamp")?,
            modified: Local.timestamp_opt(output.ctime, 0)
                .single()
                .context("Invalid ctime timestamp")?,
            generation: output.generation,
            ctransid: output.ctransid,
            otransid: output.otransid,
            stransid: output.stransid,
            rtransid: output.rtransid,
            flags: output.flags,
            is_readonly: (output.flags & 1) != 0, // BTRFS_SUBVOL_RDONLY
            is_default: false, // Will be determined separately
        })
    }
}
