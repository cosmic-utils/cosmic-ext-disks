use disks_dbus::{PartitionTypeInfo, VolumeKind, VolumeModel, VolumeNode};

pub(crate) fn common_partition_filesystem_type(table_type: &str, index: usize) -> Option<String> {
    match table_type {
        "gpt" => disks_dbus::COMMON_GPT_TYPES
            .get(index)
            .map(|p: &PartitionTypeInfo| p.filesystem_type.clone()),
        "dos" => disks_dbus::COMMON_DOS_TYPES
            .get(index)
            .map(|p: &PartitionTypeInfo| p.filesystem_type.clone()),
        _ => None,
    }
}

pub(crate) fn common_partition_type_index_for(table_type: &str, id_type: Option<&str>) -> usize {
    let Some(id_type) = id_type else {
        return 0;
    };

    let list: &[PartitionTypeInfo] = match table_type {
        "gpt" => &disks_dbus::COMMON_GPT_TYPES,
        "dos" => &disks_dbus::COMMON_DOS_TYPES,
        _ => return 0,
    };

    list.iter()
        .position(|p| p.filesystem_type.eq_ignore_ascii_case(id_type))
        .unwrap_or(0)
}

pub(crate) fn collect_mounted_descendants_leaf_first(node: &VolumeNode) -> Vec<VolumeNode> {
    fn visit(node: &VolumeNode, out: &mut Vec<VolumeNode>) {
        for child in &node.children {
            visit(child, out);
        }

        if node.can_mount() && node.is_mounted() {
            out.push(node.clone());
        }
    }

    let mut out = Vec::new();
    visit(node, &mut out);
    out
}

pub(crate) fn find_volume_node<'a>(
    volumes: &'a [VolumeNode],
    object_path: &str,
) -> Option<&'a VolumeNode> {
    for v in volumes {
        if v.object_path.to_string() == object_path {
            return Some(v);
        }
        if let Some(child) = find_volume_node(&v.children, object_path) {
            return Some(child);
        }
    }
    None
}

pub(crate) fn find_volume_node_for_partition<'a>(
    volumes: &'a [VolumeNode],
    partition: &VolumeModel,
) -> Option<&'a VolumeNode> {
    let target = partition.path.to_string();
    find_volume_node(volumes, &target)
}

/// Check if a VolumeNode is or contains (inside a LUKS container) a BTRFS filesystem.
/// Returns the BTRFS mount point if found (None mount point means BTRFS exists but is not mounted).
pub(crate) fn detect_btrfs_in_node(node: &VolumeNode) -> Option<Option<String>> {
    if node.id_type.to_lowercase() == "btrfs" {
        return Some(node.mount_points.first().cloned());
    }

    if node.kind == VolumeKind::CryptoContainer {
        for child in &node.children {
            if let Some(mp) = detect_btrfs_in_node(child) {
                return Some(mp);
            }
        }
    }

    None
}

/// Check if a VolumeModel (from volumes_flat) is or contains a BTRFS filesystem.
/// Looks up the corresponding VolumeNode tree to check through LUKS containers.
/// Returns Some((mount_point_option, block_path)) if BTRFS is found.
pub(crate) fn detect_btrfs_for_volume(
    volumes: &[VolumeNode],
    volume: &VolumeModel,
) -> Option<(Option<String>, String)> {
    if volume.id_type.to_lowercase() == "btrfs" {
        let mount_point = volume.mount_points.first().cloned();
        let block_path = volume.path.to_string();
        return Some((mount_point, block_path));
    }

    if let Some(node) = find_volume_node_for_partition(volumes, volume) {
        if let Some(mp) = detect_btrfs_in_node(node) {
            // Find the BTRFS block device path by looking for the BTRFS child
            if let Some(btrfs_child) = find_btrfs_child(node) {
                return Some((mp, btrfs_child.object_path.to_string()));
            }
        }
    }

    None
}

/// Helper to find BTRFS child node
fn find_btrfs_child(node: &VolumeNode) -> Option<&VolumeNode> {
    for child in &node.children {
        if child.id_type.to_lowercase() == "btrfs" {
            return Some(child);
        }
        if let Some(found) = find_btrfs_child(child) {
            return Some(found);
        }
    }
    None
}
