use cosmic::Task;

use crate::app::Message;
use crate::ui::volumes::helpers;
use disks_dbus::DriveModel;

use super::super::VolumesControl;

pub(super) fn mount(control: &mut VolumesControl) -> Task<cosmic::Action<Message>> {
    let segment = control.segments.get(control.selected_segment).cloned();
    if let Some(s) = segment.clone() {
        match s.volume {
            Some(p) => {
                return Task::perform(
                    async move {
                        match p.mount().await {
                            Ok(_) => match DriveModel::get_drives().await {
                                Ok(drives) => Ok(drives),
                                Err(e) => Err(e),
                            },
                            Err(e) => Err(e),
                        }
                    },
                    |result| match result {
                        Ok(drives) => Message::UpdateNav(drives, None).into(),
                        Err(e) => {
                            tracing::error!(?e, "mount failed");
                            Message::None.into()
                        }
                    },
                );
            }
            None => return Task::none(),
        }
    }
    Task::none()
}

pub(super) fn unmount(control: &mut VolumesControl) -> Task<cosmic::Action<Message>> {
    let segment = control.segments.get(control.selected_segment).cloned();
    if let Some(s) = segment.clone() {
        match s.volume {
            Some(p) => {
                return Task::perform(
                    async move {
                        match p.unmount().await {
                            Ok(_) => match DriveModel::get_drives().await {
                                Ok(drives) => Ok(drives),
                                Err(e) => Err(e),
                            },
                            Err(e) => Err(e),
                        }
                    },
                    |result| match result {
                        Ok(drives) => Message::UpdateNav(drives, None).into(),
                        Err(e) => {
                            tracing::error!(%e, "unmount failed");
                            Message::None.into()
                        }
                    },
                );
            }
            None => return Task::none(),
        }
    }
    Task::none()
}

pub(super) fn child_mount(
    control: &mut VolumesControl,
    object_path: String,
) -> Task<cosmic::Action<Message>> {
    let node = helpers::find_volume_node(&control.model.volumes, &object_path).cloned();
    if let Some(v) = node {
        return Task::perform(
            async move {
                v.mount().await?;
                DriveModel::get_drives().await
            },
            |result| match result {
                Ok(drives) => Message::UpdateNav(drives, None).into(),
                Err(e) => {
                    tracing::error!(?e, "child mount failed");
                    Message::None.into()
                }
            },
        );
    }
    Task::none()
}

pub(super) fn child_unmount(
    control: &mut VolumesControl,
    object_path: String,
) -> Task<cosmic::Action<Message>> {
    let node = helpers::find_volume_node(&control.model.volumes, &object_path).cloned();
    if let Some(v) = node {
        return Task::perform(
            async move {
                v.unmount().await?;
                DriveModel::get_drives().await
            },
            |result| match result {
                Ok(drives) => Message::UpdateNav(drives, None).into(),
                Err(e) => {
                    tracing::error!(?e, "child unmount failed");
                    Message::None.into()
                }
            },
        );
    }
    Task::none()
}
