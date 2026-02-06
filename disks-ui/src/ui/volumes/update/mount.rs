use cosmic::Task;
use std::future::Future;

use crate::app::Message;
use crate::ui::volumes::helpers;
use disks_dbus::DriveModel;

use crate::ui::volumes::VolumesControl;

/// Generic helper for volume mount/unmount operations
fn perform_volume_operation<F, Fut>(
    operation: F,
    operation_name: &'static str,
) -> Task<cosmic::Action<Message>>
where
    F: FnOnce() -> Fut + Send + 'static,
    Fut: Future<Output = anyhow::Result<()>> + Send,
{
    Task::perform(
        async move {
            operation().await?;
            DriveModel::get_drives().await
        },
        move |result| match result {
            Ok(drives) => Message::UpdateNav(drives, None).into(),
            Err(e) => {
                tracing::error!(?e, "{operation_name} failed");
                Message::None.into()
            }
        },
    )
}

pub(super) fn mount(control: &mut VolumesControl) -> Task<cosmic::Action<Message>> {
    let Some(volume) = control
        .segments
        .get(control.selected_segment)
        .and_then(|s| s.volume.clone())
    else {
        return Task::none();
    };

    perform_volume_operation(|| async move { volume.mount().await }, "mount")
}

pub(super) fn unmount(control: &mut VolumesControl) -> Task<cosmic::Action<Message>> {
    let Some(volume) = control
        .segments
        .get(control.selected_segment)
        .and_then(|s| s.volume.clone())
    else {
        return Task::none();
    };

    perform_volume_operation(|| async move { volume.unmount().await }, "unmount")
}

pub(super) fn child_mount(
    control: &mut VolumesControl,
    object_path: String,
) -> Task<cosmic::Action<Message>> {
    let Some(node) = helpers::find_volume_node(&control.model.volumes, &object_path).cloned()
    else {
        return Task::none();
    };

    perform_volume_operation(|| async move { node.mount().await }, "child mount")
}

pub(super) fn child_unmount(
    control: &mut VolumesControl,
    object_path: String,
) -> Task<cosmic::Action<Message>> {
    let Some(node) = helpers::find_volume_node(&control.model.volumes, &object_path).cloned()
    else {
        return Task::none();
    };

    perform_volume_operation(|| async move { node.unmount().await }, "child unmount")
}
