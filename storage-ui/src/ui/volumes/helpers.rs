use crate::models::UiVolume;
use storage_common::{PartitionTypeInfo, VolumeInfo, VolumeKind};

pub(crate) fn common_partition_filesystem_type(table_type: &str, index: usize) -> Option<String> {
    match table_type {
        "gpt" => storage_common::COMMON_GPT_TYPES
            .get(index)
            .map(|p: &PartitionTypeInfo| p.filesystem_type.clone()),
        "dos" => storage_common::COMMON_DOS_TYPES
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
        "gpt" => &storage_common::COMMON_GPT_TYPES,
        "dos" => &storage_common::COMMON_DOS_TYPES,
        _ => return 0,
    };

    list.iter()
        .position(|p| p.filesystem_type.eq_ignore_ascii_case(id_type))
        .unwrap_or(0)
}

pub(crate) fn collect_mounted_descendants_leaf_first(node: &UiVolume) -> Vec<String> {
    let mut out = Vec::new();

    fn visit(node: &UiVolume, out: &mut Vec<String>) {
        for child in &node.children {
            visit(child, out);
        }

        if node.volume.can_mount()
            && node.volume.is_mounted()
            && let Some(device) = &node.volume.device_path
        {
            out.push(device.clone());
        }
    }

    visit(node, &mut out);
    out
}

pub(crate) fn find_volume_in_ui_tree<'a>(
    volumes: &'a [UiVolume],
    device_path: &str,
) -> Option<&'a UiVolume> {
    for v in volumes {
        if v.device() == Some(device_path) {
            return Some(v);
        }
        if let Some(child) = find_volume_in_ui_tree(&v.children, device_path) {
            return Some(child);
        }
    }
    None
}

pub(crate) fn find_volume_for_partition<'a>(
    volumes: &'a [UiVolume],
    partition_volume: &VolumeInfo,
) -> Option<&'a UiVolume> {
    let Some(target) = &partition_volume.device_path else {
        return None;
    };
    find_volume_in_ui_tree(volumes, target)
}

/// Check if a UiVolume is or contains (inside a LUKS container) a BTRFS filesystem.
/// Returns the BTRFS mount point if found (None mount point means BTRFS exists but is not mounted).
pub(crate) fn detect_btrfs_in_node(node: &UiVolume) -> Option<Option<String>> {
    if node.volume.id_type.eq_ignore_ascii_case("btrfs") {
        return Some(node.volume.mount_points.first().cloned());
    }

    if node.volume.kind == VolumeKind::CryptoContainer {
        for child in &node.children {
            if let Some(mp) = detect_btrfs_in_node(child) {
                return Some(mp);
            }
        }
    }

    None
}

/// Check if a VolumeInfo is or contains a BTRFS filesystem.
/// Looks up the corresponding UiVolume tree to check through LUKS containers.
/// Returns Some((mount_point_option, device_path)) if BTRFS is found.
pub(crate) fn detect_btrfs_for_volume(
    volumes: &[UiVolume],
    volume: &VolumeInfo,
) -> Option<(Option<String>, String)> {
    if volume.id_type.eq_ignore_ascii_case("btrfs") {
        let mount_point = volume.mount_points.first().cloned();
        let device_path = volume.device_path.clone()?;
        return Some((mount_point, device_path));
    }

    if let Some(node) = find_volume_for_partition(volumes, volume)
        && let Some(mp) = detect_btrfs_in_node(node)
    {
        // Find the BTRFS block device path by looking for the BTRFS child
        if let Some(btrfs_child) = find_btrfs_child(node) {
            let device_path = btrfs_child.device()?.to_string();
            return Some((mp, device_path));
        }
    }

    None
}

/// Helper to find BTRFS child node
fn find_btrfs_child(node: &UiVolume) -> Option<&UiVolume> {
    for child in &node.children {
        if child.volume.id_type.eq_ignore_ascii_case("btrfs") {
            return Some(child);
        }
        if let Some(found) = find_btrfs_child(child) {
            return Some(found);
        }
    }
    None
}
