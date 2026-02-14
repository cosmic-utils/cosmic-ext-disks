// SPDX-License-Identifier: GPL-3.0-only

//! LUKS unlocking operations

use crate::error::DiskError;
use std::collections::HashMap;
use udisks2::encrypted::EncryptedProxy;
use zbus::{Connection, zvariant::Value};

/// Unlock a LUKS container
///
/// Returns the cleartext device path (e.g., "/dev/mapper/luks-...")
pub async fn unlock_luks(device_path: &str, passphrase: &str) -> Result<String, DiskError> {
    let connection = Connection::system()
        .await
        .map_err(|e| DiskError::ConnectionFailed(e.to_string()))?;

    let encrypted_path = crate::disk::resolve::block_object_path_for_device(device_path).await?;

    let encrypted_proxy = EncryptedProxy::builder(&connection)
        .path(&encrypted_path)?
        .build()
        .await
        .map_err(|e| DiskError::DBusError(e.to_string()))?;

    let opts: HashMap<&str, Value<'_>> = HashMap::new();
    let cleartext_path = encrypted_proxy
        .unlock(passphrase, opts)
        .await
        .map_err(|e| DiskError::OperationFailed(format!("Unlock failed: {}", e)))?;

    Ok(cleartext_path.to_string())
}
