use super::Message;
use crate::config::Config;
use cosmic::Application;
use cosmic::iced::Subscription;
use disks_dbus::DiskManager;
use futures_util::{SinkExt, StreamExt};

use super::state::AppModel;

/// Register subscriptions for this application.
///
/// Subscriptions are long-running async tasks running in the background which
/// emit messages to the application through a channel.
pub(crate) fn subscription(app: &AppModel) -> Subscription<Message> {
    struct DiskEventSubscription;

    Subscription::batch(vec![
        // Create a subscription which emits updates through a channel.
        Subscription::run_with_id(
            std::any::TypeId::of::<DiskEventSubscription>(),
            cosmic::iced::stream::channel(4, move |mut c| async move {
                let manager = match DiskManager::new().await {
                    Ok(m) => m,
                    Err(e) => {
                        tracing::error!(%e, "error creating DiskManager");
                        return;
                    }
                };
                let mut stream = match manager.device_event_stream_signals().await {
                    Ok(stream) => stream,
                    Err(e) => {
                        tracing::warn!(
                            %e,
                            "device updates unavailable (failed to subscribe to UDisks2 signals)"
                        );
                        return;
                    }
                };

                while let Some(event) = stream.next().await {
                    match event {
                        disks_dbus::DeviceEvent::Added(s) => {
                            let _ = c.send(Message::DriveAdded(s)).await;
                        }
                        disks_dbus::DeviceEvent::Removed(s) => {
                            let _ = c.send(Message::DriveRemoved(s)).await;
                        }
                    }
                }
            }),
        ),
        // Watch for application configuration changes.
        app.core
            .watch_config::<Config>(<AppModel as Application>::APP_ID)
            .map(|update| {
                // for why in update.errors {
                //     tracing::error!(?why, "app config error");
                // }

                Message::UpdateConfig(update.config)
            }),
    ])
}
