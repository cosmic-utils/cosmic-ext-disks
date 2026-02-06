use disks_dbus::{PartitionTypeInfo, VolumeModel, VolumeNode};

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
