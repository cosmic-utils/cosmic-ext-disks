use std::collections::{BTreeMap, HashMap};
use std::process::Command;

use storage_types::{
    LogicalCapabilities, LogicalEntity, LogicalEntityKind, LogicalMember, LogicalOperation,
};

use crate::{Result, SysError};

#[derive(Debug, Clone)]
struct MdArrayScan {
    device: String,
    name: Option<String>,
    uuid: Option<String>,
}

#[derive(Debug, Clone, Default)]
struct MdstatState {
    level: Option<String>,
    members: Vec<String>,
    degraded: bool,
}

fn parse_mdadm_scan(output: &str) -> Vec<MdArrayScan> {
    output
        .lines()
        .filter_map(|line| {
            let line = line.trim();
            if line.is_empty() || !line.starts_with("ARRAY ") {
                return None;
            }

            let mut parts = line.split_whitespace();
            let _array = parts.next()?;
            let device = parts.next()?.to_string();

            let mut name = None;
            let mut uuid = None;

            for token in parts {
                if let Some(value) = token.strip_prefix("name=") {
                    name = Some(value.to_string());
                }
                if let Some(value) = token.strip_prefix("UUID=") {
                    uuid = Some(value.to_string());
                }
            }

            Some(MdArrayScan { device, name, uuid })
        })
        .collect()
}

fn parse_proc_mdstat(output: &str) -> HashMap<String, MdstatState> {
    let mut map = HashMap::new();
    let mut current_array: Option<String> = None;

    for line in output.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with("Personalities") || line.starts_with("unused") {
            continue;
        }

        if line.starts_with("md") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 4 {
                let array = parts[0].to_string();
                let level = parts
                    .iter()
                    .find(|part| part.starts_with("raid"))
                    .map(|part| (*part).to_string());
                let members: Vec<String> = parts
                    .iter()
                    .filter(|part| part.contains('[') && part.contains(']'))
                    .map(|part| part.split('[').next().unwrap_or(part).to_string())
                    .collect();

                map.insert(
                    array.clone(),
                    MdstatState {
                        level,
                        members,
                        degraded: false,
                    },
                );
                current_array = Some(array);
            }
            continue;
        }

        if let Some(array) = current_array.as_ref()
            && line.starts_with('[')
            && line.contains('/')
        {
            let degraded = line.contains('_');
            if let Some(state) = map.get_mut(array) {
                state.degraded = degraded;
            }
        }
    }

    map
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

/// Discover mdraid entities using mdadm and /proc/mdstat fallbacks.
pub fn discover_mdraid_entities() -> Result<Vec<LogicalEntity>> {
    let scan_output = if which::which("mdadm").is_ok() {
        run_capture("mdadm", &["--detail", "--scan"]).unwrap_or_default()
    } else {
        String::new()
    };

    let mdstat_output = std::fs::read_to_string("/proc/mdstat").unwrap_or_default();

    Ok(entities_from_parsed(
        parse_mdadm_scan(&scan_output),
        parse_proc_mdstat(&mdstat_output),
    ))
}

fn entities_from_parsed(
    scan: Vec<MdArrayScan>,
    mdstat: HashMap<String, MdstatState>,
) -> Vec<LogicalEntity> {
    let mut entities = Vec::new();

    for array in scan {
        let array_name = array
            .device
            .split('/')
            .next_back()
            .map(ToString::to_string)
            .unwrap_or_else(|| array.device.clone());

        let status = mdstat.get(&array_name).cloned().unwrap_or_default();

        let mut members = Vec::new();
        for member in &status.members {
            members.push(LogicalMember {
                id: format!("mdraid-member:{array_name}:{member}"),
                name: member.clone(),
                device_path: Some(format!("/dev/{member}")),
                role: Some("member".to_string()),
                state: Some(if status.degraded {
                    "degraded".to_string()
                } else {
                    "active".to_string()
                }),
                size_bytes: None,
            });
        }

        let mut metadata = BTreeMap::new();
        if let Some(level) = &status.level {
            metadata.insert("level".to_string(), level.clone());
        }

        entities.push(LogicalEntity {
            id: format!("mdraid:{}", array.device),
            kind: LogicalEntityKind::MdRaidArray,
            name: array.name.unwrap_or(array_name),
            uuid: array.uuid,
            parent_id: None,
            device_path: Some(array.device),
            size_bytes: 0,
            used_bytes: None,
            free_bytes: None,
            health_status: Some(if status.degraded {
                "degraded".to_string()
            } else {
                "ok".to_string()
            }),
            progress_fraction: None,
            members,
            capabilities: LogicalCapabilities {
                supported: vec![
                    LogicalOperation::Create,
                    LogicalOperation::Delete,
                    LogicalOperation::Start,
                    LogicalOperation::Stop,
                    LogicalOperation::AddMember,
                    LogicalOperation::RemoveMember,
                    LogicalOperation::Check,
                    LogicalOperation::Repair,
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
    fn parses_mdadm_scan_rows() {
        let parsed = parse_mdadm_scan(
            "ARRAY /dev/md0 metadata=1.2 name=host:0 UUID=abcd\nARRAY /dev/md1 UUID=efgh\n",
        );

        assert_eq!(parsed.len(), 2);
        assert_eq!(parsed[0].device, "/dev/md0");
        assert_eq!(parsed[0].name.as_deref(), Some("host:0"));
        assert_eq!(parsed[0].uuid.as_deref(), Some("abcd"));
    }

    #[test]
    fn parses_proc_mdstat_state() {
        let parsed = parse_proc_mdstat(
            "Personalities : [raid1]\nmd0 : active raid1 sdb1[1] sda1[0]\n      976630336 blocks [2/2] [UU]\nunused devices: <none>\n",
        );

        let state = parsed.get("md0").expect("md0 state");
        assert_eq!(state.level.as_deref(), Some("raid1"));
        assert!(!state.degraded);
        assert_eq!(state.members.len(), 2);
    }

    #[test]
    fn builds_mdraid_entities() {
        let entities = entities_from_parsed(
            vec![MdArrayScan {
                device: "/dev/md0".to_string(),
                name: Some("host:0".to_string()),
                uuid: Some("abcd".to_string()),
            }],
            HashMap::from([(
                "md0".to_string(),
                MdstatState {
                    level: Some("raid1".to_string()),
                    members: vec!["sda1".to_string(), "sdb1".to_string()],
                    degraded: false,
                },
            )]),
        );

        assert_eq!(entities.len(), 1);
        assert_eq!(entities[0].kind, LogicalEntityKind::MdRaidArray);
        assert_eq!(entities[0].members.len(), 2);
    }
}
