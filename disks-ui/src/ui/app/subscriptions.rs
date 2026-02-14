use super::Message;
use crate::config::Config;
use cosmic::Application;
use cosmic::iced::Subscription;
// TODO: DisksClient needs device event subscription support

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
            cosmic::iced::stream::channel(4, move |_c| async move {
                // TODO: Replace with DisksClient signal subscription
                // let disks_client = DisksClient::new().await.expect("DisksClient");
                // let mut stream = disks_client.subscribe_device_events().await.expect("event stream");
                todo!("Implement device event subscription with DisksClient");
                
                /*
                while let Some(event) = stream.next().await {
                    match event {
                        _ => {}
                    }
                }
                */
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
