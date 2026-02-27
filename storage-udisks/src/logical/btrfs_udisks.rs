use std::collections::BTreeMap;

use storage_types::{
    LogicalCapabilities, LogicalEntity, LogicalEntityKind, LogicalMember, LogicalOperation,
    VolumeInfo,
};

fn walk_collect<'a>(volume: &'a VolumeInfo, out: &mut Vec<&'a VolumeInfo>) {
    out.push(volume);
    for child in &volume.children {
        walk_collect(child, out);
    }
}

fn looks_like_btrfs(volume: &VolumeInfo) -> bool {
    volume.id_type.eq_ignore_ascii_case("btrfs")
}

/// Build BTRFS logical entities from UDisks-discovered volume trees.
pub fn entities_from_volumes(volumes: &[VolumeInfo]) -> Vec<LogicalEntity> {
    let mut flattened = Vec::new();
    for volume in volumes {
        walk_collect(volume, &mut flattened);
    }

    let mut entities = Vec::new();

    for volume in flattened {
        if !looks_like_btrfs(volume) {
            continue;
        }

        let id = format!(
            "btrfs-fs:{}",
            volume.device_path.clone().unwrap_or_else(|| volume.name())
        );

        let mut members = Vec::new();
        if let Some(device_path) = &volume.device_path {
            members.push(LogicalMember {
                id: format!("btrfs-dev:{device_path}"),
                name: volume.name(),
                device_path: Some(device_path.clone()),
                role: Some("device".to_string()),
                state: None,
                size_bytes: Some(volume.size),
            });
        }

        entities.push(LogicalEntity {
            id,
            kind: LogicalEntityKind::BtrfsFilesystem,
            name: volume.name(),
            uuid: None,
            parent_id: None,
            device_path: volume.device_path.clone(),
            size_bytes: volume.size,
            used_bytes: volume.usage.as_ref().map(|usage| usage.used),
            free_bytes: volume.usage.as_ref().map(|usage| usage.available),
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
            metadata: BTreeMap::new(),
        });
    }

    entities
}

#[cfg(test)]
mod tests {
    use super::*;
    use storage_types::VolumeKind;

    #[test]
    fn maps_btrfs_volume_to_filesystem_entity() {
        let volume = VolumeInfo {
            kind: VolumeKind::Filesystem,
            label: "rootfs".to_string(),
            size: 1024,
            offset: 0,
            partition_number: 0,
            id_type: "btrfs".to_string(),
            device_path: Some("/dev/nvme0n1p2".to_string()),
            parent_path: None,
            has_filesystem: true,
            mount_points: vec![],
            usage: None,
            locked: false,
            children: vec![],
        };

        let entities = entities_from_volumes(&[volume]);
        assert_eq!(entities.len(), 1);
        assert_eq!(entities[0].kind, LogicalEntityKind::BtrfsFilesystem);
        assert_eq!(entities[0].members.len(), 1);
    }
}
