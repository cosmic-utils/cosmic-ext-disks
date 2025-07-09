use anyhow::Result;
use futures::stream::Stream;
use futures::task::{Context, Poll};
use std::collections::HashMap;
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::time::sleep;
use tracing::error;
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

pub struct DiskManager {
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
        Ok(Self { proxy })
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
