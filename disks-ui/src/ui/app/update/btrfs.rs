use super::super::state::AppModel;
use super::Message;
use crate::ui::volumes::VolumesControl;
use crate::utils::btrfs;
use cosmic::app::Task;

/// Handle BTRFS management messages
pub(super) fn handle_btrfs_message(app: &mut AppModel, message: Message) -> Task<Message> {
    match message {
        Message::BtrfsLoadSubvolumes { mount_point } => {
            // Set loading state
            if let Some(volumes_control) = app.nav.active_data_mut::<VolumesControl>()
                && let Some(btrfs_state) = &mut volumes_control.btrfs_state
            {
                btrfs_state.loading = true;
            }

            // Spawn async task to load subvolumes
            let mount_point_for_async = mount_point.clone();
            Task::perform(
                async move {
                    match btrfs::list_subvolumes(&mount_point_for_async).await {
                        Ok(subvolumes) => Ok(subvolumes),
                        Err(e) => Err(format!("{:#}", e)),
                    }
                },
                move |result| {
                    Message::BtrfsSubvolumesLoaded {
                        mount_point: mount_point.clone(),
                        result,
                    }
                    .into()
                },
            )
        }

        Message::BtrfsSubvolumesLoaded {
            mount_point,
            result,
        } => {
            // Update state with loaded subvolumes
            if let Some(volumes_control) = app.nav.active_data_mut::<VolumesControl>()
                && let Some(btrfs_state) = &mut volumes_control.btrfs_state
                && btrfs_state.mount_point.as_deref() == Some(&mount_point)
            {
                btrfs_state.loading = false;
                btrfs_state.subvolumes = Some(result);
            }
            Task::none()
        }

        _ => Task::none(),
    }
}
