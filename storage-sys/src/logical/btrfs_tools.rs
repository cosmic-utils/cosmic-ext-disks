use std::collections::BTreeMap;
use std::process::Command;

use storage_types::{
    LogicalCapabilities, LogicalEntity, LogicalEntityKind, LogicalMember, LogicalOperation,
};

use crate::{Result, SysError};

#[derive(Debug, Clone)]
struct BtrfsFs {
    label: Option<String>,
    uuid: String,
    used_bytes: Option<u64>,
    devices: Vec<(String, u64)>,
}

fn parse_btrfs_show(output: &str) -> Vec<BtrfsFs> {
    let mut filesystems = Vec::new();
    let mut current: Option<BtrfsFs> = None;

    for line in output.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        if line.starts_with("Label:") {
            if let Some(previous) = current.take() {
                filesystems.push(previous);
            }

            let label = line
                .split("Label:")
                .nth(1)
                .and_then(|rest| rest.split("uuid:").next())
                .map(str::trim)
                .map(|value| value.trim_matches('\''))
                .filter(|value| !value.is_empty() && *value != "none")
                .map(ToString::to_string);

            let uuid = line
                .split("uuid:")
                .nth(1)
                .map(str::trim)
                .unwrap_or_default()
                .to_string();

            current = Some(BtrfsFs {
                label,
                uuid,
                used_bytes: None,
                devices: Vec::new(),
            });

            continue;
        }

        if let Some(current_fs) = current.as_mut() {
            if line.starts_with("Total devices")
                && let Some(used_fragment) = line.split("FS bytes used").nth(1)
            {
                current_fs.used_bytes = parse_first_u64(used_fragment);
                continue;
            }

            if line.starts_with("devid") {
                let path = line
                    .split("path")
                    .nth(1)
                    .map(str::trim)
                    .unwrap_or_default()
                    .to_string();

                let size = line
                    .split("size")
                    .nth(1)
                    .and_then(parse_first_u64)
                    .unwrap_or(0);

                if !path.is_empty() {
                    current_fs.devices.push((path, size));
                }
            }
        }
    }

    if let Some(last) = current {
        filesystems.push(last);
    }

    filesystems
}

fn parse_first_u64(input: &str) -> Option<u64> {
    let digits: String = input
        .chars()
        .skip_while(|character| !character.is_ascii_digit())
        .take_while(|character| character.is_ascii_digit())
        .collect();

    if digits.is_empty() {
        None
    } else {
        digits.parse().ok()
    }
}

fn run_capture(command: &str, args: &[&str]) -> Result<String> {
    let output = Command::new(command).args(args).output()?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(SysError::OperationFailed(format!(
            "{command} failed: {stderr}"
        )));
    }

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

/// Discover BTRFS filesystem entities through `btrfs filesystem show --raw` fallback.
pub fn discover_btrfs_entities() -> Result<Vec<LogicalEntity>> {
    if !cfg!(feature = "btrfs-tools") {
        return Ok(Vec::new());
    }

    if which::which("btrfs").is_err() {
        return Ok(Vec::new());
    }

    let output = run_capture("btrfs", &["filesystem", "show", "--raw"])?;
    Ok(entities_from_filesystems(parse_btrfs_show(&output)))
}

fn entities_from_filesystems(filesystems: Vec<BtrfsFs>) -> Vec<LogicalEntity> {
    let mut entities = Vec::new();

    for filesystem in filesystems {
        let id = format!("btrfs-fs:{}", filesystem.uuid);
        let total_size: u64 = filesystem.devices.iter().map(|(_, size)| *size).sum();

        let members: Vec<LogicalMember> = filesystem
            .devices
            .iter()
            .map(|(path, size)| LogicalMember {
                id: format!("btrfs-dev:{path}"),
                name: path.clone(),
                device_path: Some(path.clone()),
                role: Some("device".to_string()),
                state: None,
                size_bytes: Some(*size),
            })
            .collect();

        let mut metadata = BTreeMap::new();
        metadata.insert(
            "device_count".to_string(),
            filesystem.devices.len().to_string(),
        );

        entities.push(LogicalEntity {
            id,
            kind: LogicalEntityKind::BtrfsFilesystem,
            name: filesystem
                .label
                .clone()
                .unwrap_or_else(|| filesystem.uuid.clone()),
            uuid: Some(filesystem.uuid),
            parent_id: None,
            device_path: None,
            size_bytes: total_size,
            used_bytes: filesystem.used_bytes,
            free_bytes: filesystem
                .used_bytes
                .map(|used_bytes| total_size.saturating_sub(used_bytes)),
            health_status: None,
            progress_fraction: None,
            members,
            capabilities: LogicalCapabilities {
                supported: vec![
                    LogicalOperation::AddMember,
                    LogicalOperation::RemoveMember,
                    LogicalOperation::Resize,
                    LogicalOperation::SetLabel,
                    LogicalOperation::SetDefaultSubvolume,
                ],
                blocked: vec![],
            },
            metadata,
        });
    }

    entities
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_btrfs_show_output() {
        let parsed = parse_btrfs_show(
            "Label: 'rootfs'  uuid: 1111-2222\nTotal devices 2 FS bytes used 1024\ndevid    1 size 4096 used 1024 path /dev/sda2\ndevid    2 size 4096 used 0 path /dev/sdb2\n",
        );

        assert_eq!(parsed.len(), 1);
        assert_eq!(parsed[0].label.as_deref(), Some("rootfs"));
        assert_eq!(parsed[0].uuid, "1111-2222");
        assert_eq!(parsed[0].devices.len(), 2);
    }

    #[test]
    fn builds_entities_from_parsed_filesystems() {
        let entities = entities_from_filesystems(vec![BtrfsFs {
            label: Some("rootfs".to_string()),
            uuid: "1111-2222".to_string(),
            used_bytes: Some(1024),
            devices: vec![
                ("/dev/sda2".to_string(), 4096),
                ("/dev/sdb2".to_string(), 4096),
            ],
        }]);

        assert_eq!(entities.len(), 1);
        assert_eq!(entities[0].kind, LogicalEntityKind::BtrfsFilesystem);
        assert_eq!(entities[0].members.len(), 2);
        assert_eq!(entities[0].size_bytes, 8192);
    }
}
