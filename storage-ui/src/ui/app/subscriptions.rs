use super::Message;
use crate::client::{DisksClient, FilesystemsClient, ImageClient, LuksClient};
use crate::config::Config;
use crate::ui::dialogs::message::ImageOperationDialogMessage;
use cosmic::Application;
use cosmic::iced::Subscription;
use cosmic::iced::futures::{SinkExt, StreamExt};
use cosmic::iced::{Event, event, keyboard};
use std::time::Duration;

use super::state::AppModel;

/// Subscription for image operation progress and completion.
struct ImageOperationSubscription;

/// Subscription for storage-service Filesystems and LUKS signals (format, mount, unmount, container created/unlocked/locked).
struct StorageEventsSubscription;

/// Register subscriptions for this application.
///
/// Subscriptions are long-running async tasks running in the background which
/// emit messages to the application through a channel.
pub(crate) fn subscription(app: &AppModel) -> Subscription<Message> {
    struct DiskEventSubscription;

    let mut subs: Vec<Subscription<Message>> = vec![
        event::listen_with(|event, _, _| match event {
            Event::Keyboard(keyboard::Event::ModifiersChanged(modifiers)) => {
                Some(Message::UsageSelectionModifiersChanged(modifiers))
            }
            Event::Keyboard(keyboard::Event::KeyPressed { modifiers, .. }) => {
                Some(Message::UsageSelectionModifiersChanged(modifiers))
            }
            Event::Keyboard(keyboard::Event::KeyReleased { modifiers, .. }) => {
                Some(Message::UsageSelectionModifiersChanged(modifiers))
            }
            _ => None,
        }),
        // Disk hotplug: subscribe to storage-service disk_added/disk_removed and refresh nav.
        Subscription::run_with_id(
            std::any::TypeId::of::<DiskEventSubscription>(),
            cosmic::iced::stream::channel(4, move |mut output| async move {
                let Ok(client) = DisksClient::new().await else {
                    return;
                };
                let Ok(mut disk_added) = client.proxy().receive_disk_added().await else {
                    return;
                };
                let Ok(mut disk_removed) = client.proxy().receive_disk_removed().await else {
                    return;
                };
                enum DiskEvent {
                    Added(String),
                    Removed(String),
                }
                loop {
                    let msg = tokio::select! {
                        item = disk_added.next() => {
                            item.and_then(|a| a.args().ok().map(|i| DiskEvent::Added(i.device.to_string())))
                        }
                        item = disk_removed.next() => {
                            item.and_then(|a| a.args().ok().map(|i| DiskEvent::Removed(i.device.to_string())))
                        }
                    };
                    if let Some(event) = msg {
                        let m = match event {
                            DiskEvent::Added(device) => Message::DriveAdded(device),
                            DiskEvent::Removed(device) => Message::DriveRemoved(device),
                        };
                        _ = output.send(m).await;
                    }
                }
            }),
        ),
        // Storage events: refresh nav when filesystem or LUKS state changes.
        Subscription::run_with_id(
            std::any::TypeId::of::<StorageEventsSubscription>(),
            cosmic::iced::stream::channel(4, move |mut output| async move {
                let Ok(fs_client) = FilesystemsClient::new().await else {
                    return;
                };
                let Ok(luks_client) = LuksClient::new().await else {
                    return;
                };
                let Ok(mut formatted) = fs_client.proxy().receive_formatted().await else {
                    return;
                };
                let Ok(mut mounted) = fs_client.proxy().receive_mounted().await else {
                    return;
                };
                let Ok(mut unmounted) = fs_client.proxy().receive_unmounted().await else {
                    return;
                };
                let Ok(mut usage_scan_progress) =
                    fs_client.proxy().receive_usage_scan_progress().await
                else {
                    return;
                };
                let Ok(mut container_created) =
                    luks_client.proxy().receive_container_created().await
                else {
                    return;
                };
                let Ok(mut container_unlocked) =
                    luks_client.proxy().receive_container_unlocked().await
                else {
                    return;
                };
                let Ok(mut container_locked) = luks_client.proxy().receive_container_locked().await
                else {
                    return;
                };
                loop {
                    tokio::select! {
                        _ = formatted.next() => { _ = output.send(Message::DriveAdded(String::new())).await; }
                        _ = mounted.next() => { _ = output.send(Message::DriveAdded(String::new())).await; }
                        _ = unmounted.next() => { _ = output.send(Message::DriveAdded(String::new())).await; }
                        item = usage_scan_progress.next() => {
                            if let Some(signal) = item
                                && let Ok(args) = signal.args()
                            {
                                _ = output.send(Message::UsageScanProgress {
                                    scan_id: args.scan_id.to_string(),
                                    processed_bytes: args.processed_bytes,
                                    estimated_total_bytes: args.estimated_total_bytes,
                                }).await;
                            }
                        }
                        _ = container_created.next() => { _ = output.send(Message::DriveAdded(String::new())).await; }
                        _ = container_unlocked.next() => { _ = output.send(Message::DriveAdded(String::new())).await; }
                        _ = container_locked.next() => { _ = output.send(Message::DriveAdded(String::new())).await; }
                    }
                }
            }),
        ),
        // Watch for application configuration changes.
        app.core
            .watch_config::<Config>(<AppModel as Application>::APP_ID)
            .map(|update| Message::UpdateConfig(update.config)),
    ];

    // When an image operation is running, poll progress and wait for operation_completed.
    if let Some(ref operation_id) = app.image_op_operation_id {
        let operation_id = operation_id.clone();
        subs.push(Subscription::run_with_id(
            std::any::TypeId::of::<ImageOperationSubscription>(),
            cosmic::iced::stream::channel(32, move |mut output| {
                let operation_id = operation_id.clone();
                async move {
                    let Ok(client) = ImageClient::new().await else {
                        _ = output
                            .send(Message::ImageOperationDialog(
                                ImageOperationDialogMessage::Complete(Err(
                                    "Failed to create image client".to_string(),
                                )),
                            ))
                            .await;
                        return;
                    };
                    loop {
                        tokio::select! {
                            result = client.wait_for_operation_completion(&operation_id) => {
                                let result = result.map_err(|e| e.to_string());
                                _ = output
                                    .send(Message::ImageOperationDialog(
                                        ImageOperationDialogMessage::Complete(result),
                                    ))
                                    .await;
                                return;
                            }
                            _ = tokio::time::sleep(Duration::from_millis(400)) => {
                                if let Ok(status) = client.get_operation_status(&operation_id).await
                                {
                                    _ = output
                                        .send(Message::ImageOperationDialog(
                                            ImageOperationDialogMessage::Progress(
                                                operation_id.clone(),
                                                status.bytes_completed,
                                                status.total_bytes,
                                                status.speed_bytes_per_sec,
                                            ),
                                        ))
                                        .await;
                                }
                            }
                        }
                    }
                }
            }),
        ));
    }

    Subscription::batch(subs)
}
