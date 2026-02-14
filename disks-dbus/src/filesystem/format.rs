// SPDX-License-Identifier: GPL-3.0-only

//! Filesystem formatting operations

use crate::error::DiskError;
use std::collections::HashMap;
use storage_models::FormatOptions;
use udisks2::block::BlockProxy;
use zbus::{Connection, zvariant::Value};

/// Format a filesystem
pub async fn format_filesystem(
    device_path: &str,
    fs_type: &str,
    label: &str,
    _options: FormatOptions,
) -> Result<(), DiskError> {
    let connection = Connection::system()
        .await
        .map_err(|e| DiskError::ConnectionFailed(e.to_string()))?;

    let block_path = crate::disk::resolve::block_object_path_for_device(device_path).await?;

    // Format using Block.Format
    let block_proxy = BlockProxy::builder(&connection)
        .path(&block_path)
        .map_err(|e| DiskError::DBusError(e.to_string()))?
        .build()
        .await
        .map_err(|e| DiskError::DBusError(e.to_string()))?;

    let mut format_opts: HashMap<&str, Value<'_>> = HashMap::new();
    if !label.is_empty() {
        format_opts.insert("label", Value::from(label));
    }

    block_proxy
        .format(fs_type, format_opts)
        .await
        .map_err(|e| DiskError::OperationFailed(format!("Format failed: {}", e)))?;

    Ok(())
}
