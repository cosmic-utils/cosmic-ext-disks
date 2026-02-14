use super::Message;
use crate::client::ImageClient;
use crate::config::Config;
use crate::ui::dialogs::message::ImageOperationDialogMessage;
use cosmic::Application;
use cosmic::iced::Subscription;
use cosmic::iced::futures::SinkExt;
use std::time::Duration;
// TODO: DisksClient device event subscription — disk hotplug will be wired here once the client exposes the API

use super::state::AppModel;

/// Subscription for image operation progress and completion.
struct ImageOperationSubscription;

/// Register subscriptions for this application.
///
/// Subscriptions are long-running async tasks running in the background which
/// emit messages to the application through a channel.
pub(crate) fn subscription(app: &AppModel) -> Subscription<Message> {
    struct DiskEventSubscription;

    let mut subs: Vec<Subscription<Message>> = vec![
        // Create a subscription which emits updates through a channel.
        Subscription::run_with_id(
            std::any::TypeId::of::<DiskEventSubscription>(),
            cosmic::iced::stream::channel(4, move |_c| async move {
                // Placeholder until DisksClient exposes device event subscription
                std::future::pending::<()>().await
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
