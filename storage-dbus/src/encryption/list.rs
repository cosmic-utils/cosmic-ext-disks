// SPDX-License-Identifier: GPL-3.0-only

//! LUKS device listing

use crate::error::DiskError;
use storage_common::LuksInfo;

/// List all LUKS encrypted devices
pub async fn list_luks_devices() -> Result<Vec<LuksInfo>, DiskError> {
    // TODO: Rewrite to traverse volumes tree instead of volumes_flat
    // For now return empty list
    Ok(vec![])
}
