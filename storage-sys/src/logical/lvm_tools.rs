use std::collections::BTreeMap;
use std::process::Command;

use storage_types::{
    LogicalCapabilities, LogicalEntity, LogicalEntityKind, LogicalMember, LogicalOperation,
};

use crate::{Result, SysError};

#[derive(Debug, Clone)]
struct VgRow {
    name: String,
    size: u64,
    free: u64,
    pv_count: u32,
    lv_count: u32,
}

#[derive(Debug, Clone)]
struct LvRow {
    vg_name: String,
    lv_name: String,
    lv_path: String,
    size: u64,
    active: bool,
}

#[derive(Debug, Clone)]
struct PvRow {
    pv_name: String,
    vg_name: Option<String>,
    size: u64,
    free: u64,
}

fn parse_tabbed_line(line: &str) -> Vec<String> {
    line.split('\t')
        .map(|part| part.trim().to_string())
        .collect()
}

fn parse_vgs(output: &str) -> Vec<VgRow> {
    output
        .lines()
        .filter_map(|line| {
            let line = line.trim();
            if line.is_empty() {
                return None;
            }
            let cols = parse_tabbed_line(line);
            if cols.len() < 5 {
                return None;
            }
            Some(VgRow {
                name: cols[0].clone(),
                size: cols[1].parse().ok()?,
                free: cols[2].parse().ok()?,
                pv_count: cols[3].parse().ok()?,
                lv_count: cols[4].parse().ok()?,
            })
        })
        .collect()
}

fn parse_lvs(output: &str) -> Vec<LvRow> {
    output
        .lines()
        .filter_map(|line| {
            let line = line.trim();
            if line.is_empty() {
                return None;
            }
            let cols = parse_tabbed_line(line);
            if cols.len() < 5 {
                return None;
            }

            Some(LvRow {
                vg_name: cols[0].clone(),
                lv_name: cols[1].clone(),
                lv_path: cols[2].clone(),
                size: cols[3].parse().ok()?,
                active: cols[4].eq_ignore_ascii_case("active") || cols[4] == "y",
            })
        })
        .collect()
}

fn parse_pvs(output: &str) -> Vec<PvRow> {
    output
        .lines()
        .filter_map(|line| {
            let line = line.trim();
            if line.is_empty() {
                return None;
            }
            let cols = parse_tabbed_line(line);
            if cols.len() < 4 {
                return None;
            }
            let vg_name = if cols[1].is_empty() {
                None
            } else {
                Some(cols[1].clone())
            };

            Some(PvRow {
                pv_name: cols[0].clone(),
                vg_name,
                size: cols[2].parse().ok()?,
                free: cols[3].parse().ok()?,
            })
        })
        .collect()
}

fn run_command(command: &str, args: &[&str]) -> Result<String> {
    let output = Command::new(command).args(args).output()?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(SysError::OperationFailed(format!(
            "{command} failed: {stderr}"
        )));
    }
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

/// Discover LVM entities via LVM tooling fallbacks.
pub fn discover_lvm_entities() -> Result<Vec<LogicalEntity>> {
    if !cfg!(feature = "lvm-tools") {
        return Ok(Vec::new());
    }

    if which::which("vgs").is_err() || which::which("lvs").is_err() || which::which("pvs").is_err()
    {
        return Ok(Vec::new());
    }

    let vgs_output = run_command(
        "vgs",
        &[
            "--noheadings",
            "--units",
            "b",
            "--nosuffix",
            "-o",
            "vg_name,vg_size,vg_free,pv_count,lv_count",
            "--separator",
            "\t",
        ],
    )?;
    let lvs_output = run_command(
        "lvs",
        &[
            "--noheadings",
            "--units",
            "b",
            "--nosuffix",
            "-o",
            "vg_name,lv_name,lv_path,lv_size,lv_active",
            "--separator",
            "\t",
        ],
    )?;
    let pvs_output = run_command(
        "pvs",
        &[
            "--noheadings",
            "--units",
            "b",
            "--nosuffix",
            "-o",
            "pv_name,vg_name,pv_size,pv_free",
            "--separator",
            "\t",
        ],
    )?;

    Ok(entities_from_rows(
        parse_vgs(&vgs_output),
        parse_lvs(&lvs_output),
        parse_pvs(&pvs_output),
    ))
}

fn entities_from_rows(vgs: Vec<VgRow>, lvs: Vec<LvRow>, pvs: Vec<PvRow>) -> Vec<LogicalEntity> {
    let mut entities = Vec::new();

    for vg in &vgs {
        let vg_id = format!("lvm-vg:{}", vg.name);
        let vg_lvs: Vec<&LvRow> = lvs.iter().filter(|lv| lv.vg_name == vg.name).collect();
        let vg_pvs: Vec<&PvRow> = pvs
            .iter()
            .filter(|pv| pv.vg_name.as_deref() == Some(vg.name.as_str()))
            .collect();

        let mut members = Vec::new();
        for lv in &vg_lvs {
            members.push(LogicalMember {
                id: format!("lvm-lv:{}", lv.lv_path),
                name: lv.lv_name.clone(),
                device_path: Some(lv.lv_path.clone()),
                role: Some("lv".to_string()),
                state: Some(if lv.active {
                    "active".to_string()
                } else {
                    "inactive".to_string()
                }),
                size_bytes: Some(lv.size),
            });
        }
        for pv in &vg_pvs {
            members.push(LogicalMember {
                id: format!("lvm-pv:{}", pv.pv_name),
                name: pv.pv_name.clone(),
                device_path: Some(pv.pv_name.clone()),
                role: Some("pv".to_string()),
                state: None,
                size_bytes: Some(pv.size),
            });
        }

        let mut metadata = BTreeMap::new();
        metadata.insert("pv_count".to_string(), vg.pv_count.to_string());
        metadata.insert("lv_count".to_string(), vg.lv_count.to_string());

        entities.push(LogicalEntity {
            id: vg_id.clone(),
            kind: LogicalEntityKind::LvmVolumeGroup,
            name: vg.name.clone(),
            uuid: None,
            parent_id: None,
            device_path: None,
            size_bytes: vg.size,
            used_bytes: Some(vg.size.saturating_sub(vg.free)),
            free_bytes: Some(vg.free),
            health_status: None,
            progress_fraction: None,
            members,
            capabilities: LogicalCapabilities {
                supported: vec![
                    LogicalOperation::Create,
                    LogicalOperation::Delete,
                    LogicalOperation::AddMember,
                    LogicalOperation::RemoveMember,
                ],
                blocked: vec![],
            },
            metadata,
        });

        for lv in vg_lvs {
            entities.push(LogicalEntity {
                id: format!("lvm-lv:{}", lv.lv_path),
                kind: LogicalEntityKind::LvmLogicalVolume,
                name: lv.lv_name.clone(),
                uuid: None,
                parent_id: Some(vg_id.clone()),
                device_path: Some(lv.lv_path.clone()),
                size_bytes: lv.size,
                used_bytes: None,
                free_bytes: None,
                health_status: Some(if lv.active {
                    "active".to_string()
                } else {
                    "inactive".to_string()
                }),
                progress_fraction: None,
                members: vec![],
                capabilities: LogicalCapabilities {
                    supported: vec![
                        LogicalOperation::Resize,
                        LogicalOperation::Delete,
                        LogicalOperation::Activate,
                        LogicalOperation::Deactivate,
                    ],
                    blocked: vec![],
                },
                metadata: BTreeMap::new(),
            });
        }

        for pv in vg_pvs {
            entities.push(LogicalEntity {
                id: format!("lvm-pv:{}", pv.pv_name),
                kind: LogicalEntityKind::LvmPhysicalVolume,
                name: pv.pv_name.clone(),
                uuid: None,
                parent_id: Some(vg_id.clone()),
                device_path: Some(pv.pv_name.clone()),
                size_bytes: pv.size,
                used_bytes: Some(pv.size.saturating_sub(pv.free)),
                free_bytes: Some(pv.free),
                health_status: None,
                progress_fraction: None,
                members: vec![],
                capabilities: LogicalCapabilities {
                    supported: vec![LogicalOperation::RemoveMember],
                    blocked: vec![],
                },
                metadata: BTreeMap::new(),
            });
        }
    }

    for pv in pvs.into_iter().filter(|pv| pv.vg_name.is_none()) {
        entities.push(LogicalEntity {
            id: format!("lvm-pv:{}", pv.pv_name),
            kind: LogicalEntityKind::LvmPhysicalVolume,
            name: pv.pv_name.clone(),
            uuid: None,
            parent_id: None,
            device_path: Some(pv.pv_name.clone()),
            size_bytes: pv.size,
            used_bytes: Some(pv.size.saturating_sub(pv.free)),
            free_bytes: Some(pv.free),
            health_status: Some("unassigned".to_string()),
            progress_fraction: None,
            members: vec![],
            capabilities: LogicalCapabilities::default(),
            metadata: BTreeMap::new(),
        });
    }

    entities
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_lvm_outputs() {
        let vgs = parse_vgs("vg0\t100\t25\t1\t2\n");
        let lvs = parse_lvs("vg0\troot\t/dev/vg0/root\t50\tactive\n");
        let pvs = parse_pvs("/dev/sda2\tvg0\t100\t25\n");

        assert_eq!(vgs.len(), 1);
        assert_eq!(lvs.len(), 1);
        assert_eq!(pvs.len(), 1);
        assert_eq!(vgs[0].name, "vg0");
        assert_eq!(lvs[0].lv_name, "root");
        assert_eq!(pvs[0].vg_name.as_deref(), Some("vg0"));
    }

    #[test]
    fn builds_logical_entities_from_rows() {
        let entities = entities_from_rows(
            vec![VgRow {
                name: "vg0".to_string(),
                size: 100,
                free: 25,
                pv_count: 1,
                lv_count: 1,
            }],
            vec![LvRow {
                vg_name: "vg0".to_string(),
                lv_name: "root".to_string(),
                lv_path: "/dev/vg0/root".to_string(),
                size: 75,
                active: true,
            }],
            vec![PvRow {
                pv_name: "/dev/sda2".to_string(),
                vg_name: Some("vg0".to_string()),
                size: 100,
                free: 25,
            }],
        );

        assert!(
            entities
                .iter()
                .any(|entity| entity.kind == LogicalEntityKind::LvmVolumeGroup
                    && entity.name == "vg0")
        );
        assert!(
            entities
                .iter()
                .any(|entity| entity.kind == LogicalEntityKind::LvmLogicalVolume
                    && entity.parent_id.as_deref() == Some("lvm-vg:vg0"))
        );
        assert!(
            entities
                .iter()
                .any(|entity| entity.kind == LogicalEntityKind::LvmPhysicalVolume
                    && entity.parent_id.as_deref() == Some("lvm-vg:vg0"))
        );
    }
}
