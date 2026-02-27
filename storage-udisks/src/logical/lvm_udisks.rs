use std::collections::{BTreeMap, BTreeSet};

use storage_types::{
    LogicalCapabilities, LogicalEntity, LogicalEntityKind, LogicalMember, LogicalOperation,
    VolumeInfo, VolumeKind,
};

fn vg_name_from_lv_path(path: &str) -> Option<String> {
    // /dev/<vg>/<lv>
    let stripped = path.strip_prefix("/dev/")?;
    let (vg_name, _lv_name) = stripped.split_once('/')?;
    if vg_name.is_empty() {
        None
    } else {
        Some(vg_name.to_string())
    }
}

fn walk_collect<'a>(volume: &'a VolumeInfo, out: &mut Vec<&'a VolumeInfo>) {
    out.push(volume);
    for child in &volume.children {
        walk_collect(child, out);
    }
}

/// Build LVM entities from UDisks-provided volume trees.
pub fn entities_from_volumes(volumes: &[VolumeInfo]) -> Vec<LogicalEntity> {
    let mut flattened = Vec::new();
    for volume in volumes {
        walk_collect(volume, &mut flattened);
    }

    let mut vg_to_lvs: BTreeMap<String, Vec<&VolumeInfo>> = BTreeMap::new();
    let mut vg_to_pvs: BTreeMap<String, Vec<&VolumeInfo>> = BTreeMap::new();
    let mut pv_without_vg: Vec<&VolumeInfo> = Vec::new();

    for volume in &flattened {
        match volume.kind {
            VolumeKind::LvmLogicalVolume => {
                if let Some(path) = volume.device_path.as_deref()
                    && let Some(vg_name) = vg_name_from_lv_path(path)
                {
                    vg_to_lvs.entry(vg_name).or_default().push(volume);
                }
            }
            VolumeKind::LvmPhysicalVolume => {
                // Infer VG by checking known LV parent path relation if available, otherwise unknown bucket.
                let maybe_vg = volume
                    .children
                    .iter()
                    .find_map(|child| child.device_path.as_deref())
                    .and_then(vg_name_from_lv_path);

                if let Some(vg_name) = maybe_vg {
                    vg_to_pvs.entry(vg_name).or_default().push(volume);
                } else {
                    pv_without_vg.push(volume);
                }
            }
            _ => {}
        }
    }

    let mut entities = Vec::new();
    let mut all_vgs = BTreeSet::new();
    all_vgs.extend(vg_to_lvs.keys().cloned());
    all_vgs.extend(vg_to_pvs.keys().cloned());

    for vg_name in all_vgs {
        let vg_id = format!("lvm-vg:{vg_name}");
        let lvs = vg_to_lvs.get(&vg_name).cloned().unwrap_or_default();
        let pvs = vg_to_pvs.get(&vg_name).cloned().unwrap_or_default();

        let mut members: Vec<LogicalMember> = Vec::new();
        for lv in &lvs {
            members.push(LogicalMember {
                id: format!(
                    "lvm-lv:{}",
                    lv.device_path.clone().unwrap_or_else(|| lv.name())
                ),
                name: lv.name(),
                device_path: lv.device_path.clone(),
                role: Some("lv".to_string()),
                state: None,
                size_bytes: Some(lv.size),
            });
        }
        for pv in &pvs {
            members.push(LogicalMember {
                id: format!(
                    "lvm-pv:{}",
                    pv.device_path.clone().unwrap_or_else(|| pv.name())
                ),
                name: pv.name(),
                device_path: pv.device_path.clone(),
                role: Some("pv".to_string()),
                state: None,
                size_bytes: Some(pv.size),
            });
        }

        entities.push(LogicalEntity {
            id: vg_id.clone(),
            kind: LogicalEntityKind::LvmVolumeGroup,
            name: vg_name.clone(),
            uuid: None,
            parent_id: None,
            device_path: None,
            size_bytes: lvs.iter().map(|volume| volume.size).sum(),
            used_bytes: None,
            free_bytes: None,
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
            metadata: BTreeMap::new(),
        });

        for lv in lvs {
            entities.push(LogicalEntity {
                id: format!(
                    "lvm-lv:{}",
                    lv.device_path.clone().unwrap_or_else(|| lv.name())
                ),
                kind: LogicalEntityKind::LvmLogicalVolume,
                name: lv.name(),
                uuid: None,
                parent_id: Some(vg_id.clone()),
                device_path: lv.device_path.clone(),
                size_bytes: lv.size,
                used_bytes: lv.usage.as_ref().map(|usage| usage.used),
                free_bytes: lv.usage.as_ref().map(|usage| usage.available),
                health_status: None,
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

        for pv in pvs {
            entities.push(LogicalEntity {
                id: format!(
                    "lvm-pv:{}",
                    pv.device_path.clone().unwrap_or_else(|| pv.name())
                ),
                kind: LogicalEntityKind::LvmPhysicalVolume,
                name: pv.name(),
                uuid: None,
                parent_id: Some(vg_id.clone()),
                device_path: pv.device_path.clone(),
                size_bytes: pv.size,
                used_bytes: pv.usage.as_ref().map(|usage| usage.used),
                free_bytes: pv.usage.as_ref().map(|usage| usage.available),
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

    // Orphan PVs still become visible entities.
    for pv in pv_without_vg {
        entities.push(LogicalEntity {
            id: format!(
                "lvm-pv:{}",
                pv.device_path.clone().unwrap_or_else(|| pv.name())
            ),
            kind: LogicalEntityKind::LvmPhysicalVolume,
            name: pv.name(),
            uuid: None,
            parent_id: None,
            device_path: pv.device_path.clone(),
            size_bytes: pv.size,
            used_bytes: pv.usage.as_ref().map(|usage| usage.used),
            free_bytes: pv.usage.as_ref().map(|usage| usage.available),
            health_status: Some("unassigned".to_string()),
            progress_fraction: None,
            members: vec![],
            capabilities: LogicalCapabilities {
                supported: vec![],
                blocked: vec![],
            },
            metadata: BTreeMap::new(),
        });
    }

    entities
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lv(path: &str, size: u64) -> VolumeInfo {
        VolumeInfo {
            kind: VolumeKind::LvmLogicalVolume,
            label: "".to_string(),
            size,
            offset: 0,
            partition_number: 0,
            id_type: "".to_string(),
            device_path: Some(path.to_string()),
            parent_path: None,
            has_filesystem: false,
            mount_points: vec![],
            usage: None,
            locked: false,
            children: vec![],
        }
    }

    #[test]
    fn creates_vg_and_lv_entities_from_lv_paths() {
        let entities = entities_from_volumes(&[lv("/dev/vg0/root", 100), lv("/dev/vg0/home", 200)]);

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
    }
}
