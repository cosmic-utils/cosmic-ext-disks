pub mod btrfs_tools;
pub mod lvm_tools;
pub mod mdadm_tools;

use std::collections::BTreeMap;

use storage_types::LogicalEntity;

use crate::Result;

/// Discover logical entities using non-UDisks fallback tooling.
pub fn discover_logical_entities_fallback() -> Result<Vec<LogicalEntity>> {
    let mut entities = Vec::new();
    entities.extend(lvm_tools::discover_lvm_entities()?);
    entities.extend(mdadm_tools::discover_mdraid_entities()?);
    entities.extend(btrfs_tools::discover_btrfs_entities()?);

    let mut unique = BTreeMap::<String, LogicalEntity>::new();
    for entity in entities {
        unique.insert(entity.id.clone(), entity);
    }

    Ok(unique.into_values().collect())
}
