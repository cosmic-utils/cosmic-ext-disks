use anyhow::Result;
use futures::StreamExt;
use futures::stream::Stream;
use futures::task::{Context, Poll};
use std::collections::HashMap;
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::time::sleep;
use tracing::{error, warn};
use zbus::{
    Connection,
    zvariant::{self, Value},
};
use zbus_macros::proxy;

use super::DriveModel;

#[proxy(
    default_service = "org.freedesktop.UDisks2",
    default_path = "/org/freedesktop/UDisks2/Manager",
    interface = "org.freedesktop.UDisks2.Manager"
)]
pub trait UDisks2Manager {
    fn get_block_devices(
        &self,
        options: HashMap<String, Value<'_>>,
    ) -> zbus::Result<Vec<zvariant::OwnedObjectPath>>;
}

#[proxy(
    default_service = "org.freedesktop.UDisks2",
    default_path = "/org/freedesktop/UDisks2",
    interface = "org.freedesktop.DBus.ObjectManager"
)]
pub trait UDisks2ObjectManager {
    #[zbus(signal)]
    fn interfaces_added(
        &self,
        object_path: zvariant::OwnedObjectPath,
        interfaces_and_properties: HashMap<String, HashMap<String, zvariant::OwnedValue>>,
    ) -> zbus::Result<()>;

    #[zbus(signal)]
    fn interfaces_removed(
        &self,
        object_path: zvariant::OwnedObjectPath,
        interfaces: Vec<String>,
    ) -> zbus::Result<()>;
}

pub struct DiskManager {
    connection: Connection,
    proxy: UDisks2ManagerProxy<'static>,
}

#[derive(Debug, PartialEq)]
pub enum DeviceEvent {
    Added(String),
    Removed(String),
}

pub struct DeviceEventStream {
    receiver: mpsc::Receiver<DeviceEvent>,
}

impl DiskManager {
    pub async fn new() -> Result<Self> {
        let connection = Connection::system().await?;
        let proxy = UDisks2ManagerProxy::new(&connection).await?;
        Ok(Self { connection, proxy })
    }

    /// A signal-based event stream for block device add/remove.
    ///
    /// Uses `org.freedesktop.DBus.ObjectManager` on the UDisks2 root object and
    /// filters to events affecting the `org.freedesktop.UDisks2.Block` interface.
    ///
    /// Intended to be used as the primary mechanism for UI updates; callers can
    /// fall back to `device_event_stream` polling if this fails.
    pub async fn device_event_stream_signals(&self) -> Result<DeviceEventStream> {
        const BLOCK_IFACE: &str = "org.freedesktop.UDisks2.Block";

        let (sender, receiver) = mpsc::channel(32);
        let connection = self.connection.clone();

        let object_manager = UDisks2ObjectManagerProxy::new(&connection).await?;
        let mut added_stream = object_manager.receive_interfaces_added().await?;
        let mut removed_stream = object_manager.receive_interfaces_removed().await?;

        tokio::spawn(async move {
            loop {
                tokio::select! {
                    maybe_added = added_stream.next() => {
                        let Some(signal) = maybe_added else {
                            break;
                        };

                        match signal.args() {
                            Ok(args) => {
                                if args.interfaces_and_properties.contains_key(BLOCK_IFACE) {
                                    if let Err(e) = sender.send(DeviceEvent::Added(args.object_path.to_string())).await {
                                        warn!("Device event receiver dropped: {e}");
                                        break;
                                    }
                                }
                            }
                            Err(e) => {
                                warn!("Failed to parse InterfacesAdded signal args: {e}");
                            }
                        }
                    }
                    maybe_removed = removed_stream.next() => {
                        let Some(signal) = maybe_removed else {
                            break;
                        };

                        match signal.args() {
                            Ok(args) => {
                                if args.interfaces.iter().any(|i| i == BLOCK_IFACE) {
                                    if let Err(e) = sender.send(DeviceEvent::Removed(args.object_path.to_string())).await {
                                        warn!("Device event receiver dropped: {e}");
                                        break;
                                    }
                                }
                            }
                            Err(e) => {
                                warn!("Failed to parse InterfacesRemoved signal args: {e}");
                            }
                        }
                    }
                    _ = sender.closed() => {
                        break;
                    }
                }
            }
        });

        Ok(DeviceEventStream { receiver })
    }

    pub fn device_event_stream(&self, interval: Duration) -> DeviceEventStream {
        let (sender, receiver) = mpsc::channel(32); // Channel capacity of 32
        let proxy = self.proxy.clone();

        tokio::spawn(async move {
            let mut previous_devices: Option<Vec<String>> = None;
            loop {
                let current_devices = match proxy.get_block_devices(HashMap::new()).await {
                    Ok(paths) => paths.into_iter().map(|p| p.to_string()).collect(),
                    Err(e) => {
                        error!("Failed to get block devices: {}", e);
                        Vec::new()
                    }
                };

                let mut events = Vec::new();
                if let Some(prev_devices) = &previous_devices {
                    for device in &current_devices {
                        if !prev_devices.contains(device) {
                            events.push(DeviceEvent::Added(device.clone()));
                        }
                    }

                    for device in prev_devices {
                        if !current_devices.contains(device) {
                            events.push(DeviceEvent::Removed(device.clone()));
                        }
                    }
                }

                for event in events {
                    if let Err(e) = sender.send(event).await {
                        error!("Failed to send event: {}", e);
                        break; // Exit loop if sender is closed
                    }
                }

                previous_devices = Some(current_devices);
                sleep(interval).await;
            }
        });

        DeviceEventStream { receiver }
    }

    pub async fn apply_change(
        drives: &mut Vec<DriveModel>,
        added: Option<String>,
        removed: Option<String>,
    ) -> Result<()> {
        match removed {
            Some(removed_str) => {
                // Check for direct match on drive path or block path FIRST
                if let Some(index) = drives
                    .iter()
                    .position(|d| d.path == removed_str || d.block_path == removed_str)
                {
                    drives.remove(index);
                    return Ok(()); // Early return after removing a drive
                }

                // If no direct match, THEN check partitions (using a reference!)
                for drive in drives.iter_mut() {
                    if let Some(index) = drive
                        .partitions
                        .iter()
                        .position(|p| p.path.as_str() == removed_str)
                    {
                        drive.partitions.remove(index);
                    }
                }
            }
            None => {}
        }

        match added {
            Some(_) => {
                let mut new_drives = DriveModel::get_drives().await?;
                drives.retain(|drive| {
                    !new_drives
                        .iter()
                        .any(|new_drive| new_drive.path == drive.path)
                });
                drives.append(&mut new_drives);
            }
            None => {}
        }

        Ok(())
    }
}

impl Stream for DeviceEventStream {
    type Item = DeviceEvent;

    fn poll_next(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Self::Item>> {
        self.receiver.poll_recv(cx)
    }
}

// ... (main function remains the same)
