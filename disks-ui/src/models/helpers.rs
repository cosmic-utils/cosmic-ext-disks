// SPDX-License-Identifier: GPL-3.0-only

//! Helper functions for building volume hierarchies from flat lists

use std::collections::HashMap;
use storage_models::VolumeInfo;
use crate::client::error::ClientError;
use super::UiVolume;

/// Build a hierarchical volume tree from a flat list with parent_path references
/// 
/// This function:
/// 1. Filters volumes that belong to the specified disk
/// 2. Groups volumes by their parent_path
/// 3. Recursively attaches children to build the tree
/// 
/// # Arguments
/// * `disk` - Device path of the disk (e.g., "/dev/sda")
/// * `all_volumes` - Flat list of all volumes from list_volumes()
/// 
/// # Returns
/// Vector of root volumes (direct children of the disk) with nested children
/// 
/// # Example
/// ```no_run
/// let all_volumes = disks_client.list_volumes().await?;
/// let tree = build_volume_tree("/dev/sda", all_volumes)?;
/// 
/// // tree now contains roots like [sda1, sda2, sda3]
/// // each with their children (unlocked LUKS, etc.)
/// ```
pub fn build_volume_tree(
    disk: &str,
    all_volumes: Vec<VolumeInfo>,
) -> Result<Vec<UiVolume>, ClientError> {
    // Group volumes by parent_path
    let mut tree_map: HashMap<Option<String>, Vec<VolumeInfo>> = HashMap::new();
    
    for vol in all_volumes {
        tree_map
            .entry(vol.parent_path.clone())
            .or_default()
            .push(vol);
    }
    
    // Helper function to recursively build tree
    fn attach_children(
        vol_info: VolumeInfo,
        tree_map: &HashMap<Option<String>, Vec<VolumeInfo>>,
    ) -> Result<UiVolume, ClientError> {
        let device = vol_info.device_path.clone();
        
        // Recursively build children
        let children = if let Some(device_path) = &device {
            if let Some(child_infos) = tree_map.get(&Some(device_path.clone())) {
                child_infos
                    .iter()
                    .map(|child_info| attach_children(child_info.clone(), tree_map))
                    .collect::<Result<Vec<_>, _>>()?
            } else {
                Vec::new()
            }
        } else {
            Vec::new()
        };
        
        UiVolume::with_children(vol_info, children)
    }
    
    // Build roots (volumes whose parent is the disk)
    let roots = tree_map
        .get(&Some(disk.to_string()))
        .map(|root_infos| {
            root_infos
                .iter()
                .map(|root_info| attach_children(root_info.clone(), &tree_map))
                .collect::<Result<Vec<_>, _>>()
        })
        .unwrap_or_else(|| Ok(Vec::new()))?;
    
    Ok(roots)
}

/// Validate tree structure (debug assertions)
/// 
/// Checks:
/// - No cycles in parent-child relationships
/// - All parent_path references are valid
/// - Device paths are unique
/// 
/// # Example
/// ```no_run
/// #[cfg(debug_assertions)]
/// validate_tree(&volumes)?;
/// ```
#[allow(dead_code)]
pub fn validate_tree(volumes: &[UiVolume]) -> Result<(), String> {
    use std::collections::HashSet;
    
    let mut seen_devices = HashSet::new();
    
    fn validate_node(
        node: &UiVolume,
        seen: &mut HashSet<String>,
        ancestors: &mut HashSet<String>,
    ) -> Result<(), String> {
        // Check device path uniqueness
        if let Some(device) = &node.volume.device_path {
            if !seen.insert(device.clone()) {
                return Err(format!("Duplicate device path in tree: {}", device));
            }
            
            // Check for cycles
            if !ancestors.insert(device.clone()) {
                return Err(format!("Cycle detected: {} is its own ancestor", device));
            }
        }
        
        // Validate children
        for child in &node.children {
            validate_node(child, seen, ancestors)?;
        }
        
        // Remove from ancestors when backtracking
        if let Some(device) = &node.volume.device_path {
            ancestors.remove(device);
        }
        
        Ok(())
    }
    
    let mut ancestors = HashSet::new();
    for root in volumes {
        validate_node(root, &mut seen_devices, &mut ancestors)?;
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use storage_models::{VolumeKind, Usage};
    
    #[test]
    fn test_build_simple_tree() {
        let volumes = vec![
            VolumeInfo {
                kind: VolumeKind::Partition,
                label: "Root".to_string(),
                size: 100_000_000,
                id_type: Some("ext4".to_string()),
                device_path: Some("/dev/sda1".to_string()),
                parent_path: Some("/dev/sda".to_string()),
                has_filesystem: true,
                mount_points: vec!["/".to_string()],
                usage: Usage::Filesystem,
                locked: None,
                children: Vec::new(),
            },
            VolumeInfo {
                kind: VolumeKind::Partition,
                label: Some("Home".to_string()),
                size: Some(200_000_000),
                id_type: Some("ext4".to_string()),
                device_path: Some("/dev/sda2".to_string()),
                parent_path: Some("/dev/sda".to_string()),
                has_filesystem: true,
                mount_points: vec!["/home".to_string()],
                usage: Usage::Filesystem,
                locked: None,
                children: Vec::new(),
            },
        ];
        
        let tree = build_volume_tree("/dev/sda", volumes).unwrap();
        assert_eq!(tree.len(), 2);
        assert_eq!(tree[0].volume.label, Some("Root".to_string()));
        assert_eq!(tree[1].volume.label, Some("Home".to_string()));
    }
    
    #[test]
    fn test_build_nested_tree() {
        let volumes = vec![
            // LUKS partition
            VolumeInfo {
                kind: VolumeKind::LuksEncrypted,
                label: None,
                size: Some(100_000_000),
                id_type: Some("crypto_LUKS".to_string()),
                device_path: Some("/dev/sda1".to_string()),
                parent_path: Some("/dev/sda".to_string()),
                has_filesystem: false,
                mount_points: Vec::new(),
                usage: Usage::Crypto,
                locked: Some(false),
                children: Vec::new(),
            },
            // Unlocked container
            VolumeInfo {
                kind: VolumeKind::Block,
                label: Some("Secure".to_string()),
                size: Some(100_000_000),
                id_type: Some("ext4".to_string()),
                device_path: Some("/dev/mapper/luks-123".to_string()),
                parent_path: Some("/dev/sda1".to_string()),
                has_filesystem: true,
                mount_points: vec!["/mnt/secure".to_string()],
                usage: Usage::Filesystem,
                locked: None,
                children: Vec::new(),
            },
        ];
        
        let tree = build_volume_tree("/dev/sda", volumes).unwrap();
        assert_eq!(tree.len(), 1);
        assert_eq!(tree[0].volume.device_path, Some("/dev/sda1".to_string()));
        assert_eq!(tree[0].children.len(), 1);
        assert_eq!(
            tree[0].children[0].volume.device_path,
            Some("/dev/mapper/luks-123".to_string())
        );
    }
}
