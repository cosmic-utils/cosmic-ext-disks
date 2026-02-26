// SPDX-License-Identifier: GPL-3.0-only

use anyhow::Result;

use crate::handlers::disk::DiskHandler;

/// Monitor UDisks2 for disk hotplug events and emit D-Bus signals.
pub(crate) async fn monitor_hotplug_events(
    connection: zbus::Connection,
    object_path: &str,
) -> Result<()> {
    use std::collections::HashMap;
    use zbus::zvariant::{OwnedObjectPath, OwnedValue};

    tracing::info!("Starting disk hotplug monitoring");

    let obj_manager = zbus::Proxy::new(
        &connection,
        "org.freedesktop.UDisks2",
        "/org/freedesktop/UDisks2",
        "org.freedesktop.DBus.ObjectManager",
    )
    .await?;

    let object_server = connection.object_server();
    let iface_ref = object_server
        .interface::<_, DiskHandler>(object_path)
        .await?;

    let mut added_stream = obj_manager.receive_signal("InterfacesAdded").await?;
    let mut removed_stream = obj_manager.receive_signal("InterfacesRemoved").await?;

    let iface_ref_clone = iface_ref.clone();
    tokio::spawn(async move {
        use futures_util::StreamExt;

        while let Some(signal) = added_stream.next().await {
            match signal.body().deserialize::<(
                OwnedObjectPath,
                HashMap<String, HashMap<String, OwnedValue>>,
            )>() {
                Ok((object_path, interfaces)) => {
                    if interfaces.contains_key("org.freedesktop.UDisks2.Drive") {
                        tracing::debug!("Drive added: {}", object_path);

                        match get_disk_info_for_path(&object_path.as_ref()).await {
                            Ok(disk_info) => {
                                let device = disk_info.device.clone();
                                match serde_json::to_string(&disk_info) {
                                    Ok(json) => {
                                        tracing::info!("Disk added: {}", device);

                                        if let Err(e) = DiskHandler::disk_added(
                                            iface_ref_clone.signal_emitter(),
                                            &device,
                                            &json,
                                        )
                                        .await
                                        {
                                            tracing::error!(
                                                "Failed to emit disk_added signal: {}",
                                                e
                                            );
                                        }
                                    }
                                    Err(e) => {
                                        tracing::error!("Failed to serialize disk info: {}", e);
                                    }
                                }
                            }
                            Err(e) => {
                                tracing::error!(
                                    "Failed to get disk info for {}: {}",
                                    object_path,
                                    e
                                );
                            }
                        }
                    }
                }
                Err(e) => {
                    tracing::error!("Failed to parse InterfacesAdded signal: {}", e);
                }
            }
        }
    });

    let iface_ref_clone = iface_ref.clone();
    tokio::spawn(async move {
        use futures_util::StreamExt;

        while let Some(signal) = removed_stream.next().await {
            match signal
                .body()
                .deserialize::<(OwnedObjectPath, Vec<String>)>()
            {
                Ok((object_path, interfaces)) => {
                    if interfaces.contains(&"org.freedesktop.UDisks2.Drive".to_string()) {
                        let device = format!(
                            "/dev/{}",
                            object_path.as_str().rsplit('/').next().unwrap_or("unknown")
                        );

                        tracing::info!("Disk removed: {} ({})", device, object_path);

                        if let Err(e) =
                            DiskHandler::disk_removed(iface_ref_clone.signal_emitter(), &device)
                                .await
                        {
                            tracing::error!("Failed to emit disk_removed signal: {}", e);
                        }
                    }
                }
                Err(e) => {
                    tracing::error!("Failed to parse InterfacesRemoved signal: {}", e);
                }
            }
        }
    });

    tracing::info!("Disk hotplug monitoring started");
    Ok(())
}

async fn get_disk_info_for_path(
    object_path: &zbus::zvariant::ObjectPath<'_>,
) -> Result<storage_types::DiskInfo> {
    let manager = storage_udisks::DiskManager::new().await?;
    storage_udisks::get_disk_info_for_drive_path(&manager, object_path.as_str())
        .await
        .map_err(|e| anyhow::anyhow!("{}", e))
}
