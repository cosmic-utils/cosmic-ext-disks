use std::collections::HashMap;
use std::os::fd::OwnedFd;

use anyhow::Result;
use udisks2::filesystem::FilesystemProxy;
use zbus::zvariant::{OwnedFd as ZOwnedFd, Value};
use zbus::{Connection, Proxy, zvariant::OwnedObjectPath};

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

pub async fn loop_setup(image_path: &str) -> Result<OwnedObjectPath> {
    let connection = Connection::system().await?;

    let manager_path: OwnedObjectPath = "/org/freedesktop/UDisks2/Manager".try_into()?;

    // UDisks2 expects a Unix FD handle for LoopSetup: (h a{sv}).
    // Passing a path string will fail with InvalidArgs.
    let file = std::fs::OpenOptions::new().read(true).open(image_path)?;
    let fd: OwnedFd = file.into();
    let fd: ZOwnedFd = fd.into();

    // Attach is used for mounting images (e.g. ISO); default to read-only.
    let mut options: HashMap<&str, Value<'_>> = HashMap::new();
    options.insert("read-only", Value::from(true));

    call_udisks_raw(
        &connection,
        &manager_path,
        "org.freedesktop.UDisks2.Manager",
        "LoopSetup",
        &(fd, options),
    )
    .await
}

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

pub async fn mount_filesystem(block_object_path: OwnedObjectPath) -> Result<()> {
    let connection = Connection::system().await?;
    let proxy = FilesystemProxy::builder(&connection)
        .path(&block_object_path)?
        .build()
        .await?;

    proxy.mount(HashMap::new()).await?;
    Ok(())
}
