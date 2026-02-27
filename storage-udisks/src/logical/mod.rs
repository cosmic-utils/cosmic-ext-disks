//! UDisks-backed logical topology discovery.

pub mod btrfs_udisks;
pub mod lvm_udisks;
pub mod mdraid_udisks;

use std::collections::BTreeMap;

use storage_types::LogicalEntity;

use crate::{DiskError, DiskManager};

/// Discover logical entities from UDisks-backed data sources only.
pub async fn discover_logical_entities(
    manager: &DiskManager,
) -> Result<Vec<LogicalEntity>, DiskError> {
    let disk_volumes = crate::disk::get_disks_with_volumes(manager)
        .await
        .map_err(|error| {
            DiskError::OperationFailed(format!("failed to enumerate volumes: {error}"))
        })?;

    let mut entities = Vec::new();

    for (_disk, volumes) in &disk_volumes {
        entities.extend(lvm_udisks::entities_from_volumes(volumes));
        entities.extend(btrfs_udisks::entities_from_volumes(volumes));
    }

    entities.extend(mdraid_udisks::discover_mdraid_entities(manager.connection().as_ref()).await?);

    // Deduplicate by id while preserving deterministic ordering.
    let mut unique = BTreeMap::<String, LogicalEntity>::new();
    for entity in entities {
        unique.insert(entity.id.clone(), entity);
    }

    Ok(unique.into_values().collect())
}
