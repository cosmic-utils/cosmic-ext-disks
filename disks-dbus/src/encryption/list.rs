// SPDX-License-Identifier: GPL-3.0-only

//! LUKS device listing

use storage_models::LuksInfo;
use crate::error::DiskError;

/// List all LUKS encrypted devices
pub async fn list_luks_devices() -> Result<Vec<LuksInfo>, DiskError> {
    // TODO: Rewrite to traverse volumes tree instead of volumes_flat
    // For now return empty list
    Ok(vec![])
}
