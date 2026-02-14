// SPDX-License-Identifier: GPL-3.0-only

//! Backup and restore operations

use std::collections::HashMap;
use std::os::fd::OwnedFd;
use anyhow::Result;
use zbus::{Connection, Proxy, zvariant::{OwnedFd as ZOwnedFd, OwnedObjectPath, Value}};

fn device_for_display(object_path: &OwnedObjectPath) -> String {
    object_path.to_string()
}

async fn call_udisks_raw<R, B>(
    connection: &Connection,
    path: &OwnedObjectPath,
    interface: &str,
    method: &str,
    args: &B,
) -> Result<R>
where
    R: serde::de::DeserializeOwned + zbus::zvariant::Type,
    B: serde::ser::Serialize + zbus::zvariant::DynamicType,
{
    let proxy = Proxy::new(connection, "org.freedesktop.UDisks2", path, interface).await?;

    match proxy.call_method(method, args).await {
        Ok(reply) => Ok(reply.body().deserialize()?),
        Err(err) => {
            if let zbus::Error::MethodError(name, msg, _info) = &err {
                let device = device_for_display(path);
                let msg = msg.as_deref().unwrap_or("");
                anyhow::bail!(
                    "UDisks2 {interface}.{method} failed for {device}: {}{}{}",
                    name.as_str(),
                    if msg.is_empty() { "" } else { ": " },
                    msg
                );
            }

            Err(err.into())
        }
    }
}

/// Open a block device for backup (read-only access)
pub async fn open_for_backup(block_object_path: OwnedObjectPath) -> Result<OwnedFd> {
    let connection = Connection::system().await?;
    let options_empty: HashMap<&str, Value<'_>> = HashMap::new();

    let fd: ZOwnedFd = call_udisks_raw(
        &connection,
        &block_object_path,
        "org.freedesktop.UDisks2.Block",
        "OpenForBackup",
        &(options_empty),
    )
    .await?;

    Ok(fd.into())
}

/// Open a block device for restore (read-write access)
pub async fn open_for_restore(block_object_path: OwnedObjectPath) -> Result<OwnedFd> {
    let connection = Connection::system().await?;
    let options_empty: HashMap<&str, Value<'_>> = HashMap::new();

    let fd: ZOwnedFd = call_udisks_raw(
        &connection,
        &block_object_path,
        "org.freedesktop.UDisks2.Block",
        "OpenForRestore",
        &(options_empty),
    )
    .await?;

    Ok(fd.into())
}
