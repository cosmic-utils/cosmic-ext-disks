use cosmic::Task;
use std::future::Future;

use crate::app::Message;
use crate::ui::dialogs::state::{ShowDialog, UnmountBusyDialog};
use crate::ui::volumes::helpers;
use disks_dbus::{DiskError, DriveModel};

use crate::ui::volumes::VolumesControl;

/// Generic helper for volume mount/unmount operations
fn perform_volume_operation<F, Fut>(
    operation: F,
    operation_name: &'static str,
    preserve_selection: Option<String>,
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
            Ok(drives) => {
                // Pass the selected volume to preserve selection after reload
                Message::UpdateNavWithChildSelection(drives, preserve_selection.clone()).into()
            }
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

    let object_path = volume.path.to_string();
    perform_volume_operation(
        || async move { volume.mount().await },
        "mount",
        Some(object_path),
    )
}

// Helper enum to distinguish busy errors from generic errors
#[derive(Debug)]
enum UnmountResult {
    Success(Vec<disks_dbus::DriveModel>),
    Busy {
        device: String,
        mount_point: String,
        processes: Vec<disks_dbus::ProcessInfo>,
        object_path: String,
    },
    GenericError,
}

pub(super) fn unmount(control: &mut VolumesControl) -> Task<cosmic::Action<Message>> {
    let Some(volume) = control
        .segments
        .get(control.selected_segment)
        .and_then(|s| s.volume.clone())
    else {
        return Task::none();
    };

    let device = volume
        .device_path
        .clone()
        .unwrap_or_else(|| volume.path.to_string());
    let mount_point = volume.mount_points.first().cloned();
    let object_path = volume.path.to_string();
    let object_path_for_retry = object_path.clone();

    Task::perform(
        async move {
            match volume.unmount().await {
                Ok(()) => {
                    // Success - reload drives
                    match DriveModel::get_drives().await {
                        Ok(drives) => UnmountResult::Success(drives),
                        Err(e) => {
                            tracing::error!(?e, "Failed to reload drives");
                            UnmountResult::GenericError
                        }
                    }
                }
                Err(e) => {
                    // Check if it's a ResourceBusy error
                    if let Some(disk_err) = e.downcast_ref::<DiskError>()
                        && matches!(disk_err, DiskError::ResourceBusy { .. })
                    {
                        // Check if we have a mount point - can't find processes without it
                        if let Some(mp) = mount_point {
                            if !mp.trim().is_empty() {
                                // Find processes using the mount point
                                match disks_dbus::find_processes_using_mount(&mp).await {
                                    Ok(processes) => {
                                        return UnmountResult::Busy {
                                            device,
                                            mount_point: mp,
                                            processes,
                                            object_path: object_path_for_retry,
                                        };
                                    }
                                    Err(find_err) => {
                                        tracing::warn!(
                                            ?find_err,
                                            "Failed to find processes using mount"
                                        );
                                    }
                                }
                            } else {
                                tracing::warn!("Mount point is empty, cannot find processes");
                            }
                        } else {
                            tracing::warn!("No mount point available, cannot find processes");
                        }
                    }
                    // Generic error - log and continue
                    tracing::error!(?e, "unmount failed");
                    UnmountResult::GenericError
                }
            }
        },
        move |result| match result {
            UnmountResult::Success(drives) => {
                Message::UpdateNavWithChildSelection(drives, Some(object_path.clone())).into()
            }
            UnmountResult::Busy {
                device,
                mount_point,
                processes,
                object_path,
            } => {
                // Show busy dialog
                Message::Dialog(Box::new(ShowDialog::UnmountBusy(UnmountBusyDialog {
                    device,
                    mount_point,
                    processes,
                    object_path,
                })))
                .into()
            }
            UnmountResult::GenericError => {
                // Generic error already logged
                Message::None.into()
            }
        },
    )
}

pub(super) fn child_mount(
    control: &mut VolumesControl,
    object_path: String,
) -> Task<cosmic::Action<Message>> {
    let Some(node) = helpers::find_volume_node(&control.model.volumes, &object_path).cloned()
    else {
        return Task::none();
    };

    let object_path_for_selection = object_path.clone();
    perform_volume_operation(
        || async move { node.mount().await },
        "child mount",
        Some(object_path_for_selection),
    )
}

pub(super) fn child_unmount(
    control: &mut VolumesControl,
    object_path: String,
) -> Task<cosmic::Action<Message>> {
    let Some(node) = helpers::find_volume_node(&control.model.volumes, &object_path).cloned()
    else {
        return Task::none();
    };

    let device = node
        .device_path
        .clone()
        .unwrap_or_else(|| node.object_path.to_string());
    let mount_point = node.mount_points.first().cloned();
    let object_path_for_selection = object_path.clone();
    let object_path_for_retry = object_path.clone();

    Task::perform(
        async move {
            match node.unmount().await {
                Ok(()) => {
                    // Success - reload drives
                    match DriveModel::get_drives().await {
                        Ok(drives) => UnmountResult::Success(drives),
                        Err(e) => {
                            tracing::error!(?e, "Failed to reload drives");
                            UnmountResult::GenericError
                        }
                    }
                }
                Err(e) => {
                    // Check if it's a ResourceBusy error
                    if let Some(disk_err) = e.downcast_ref::<DiskError>()
                        && matches!(disk_err, DiskError::ResourceBusy { .. })
                    {
                        // Check if we have a mount point - can't find processes without it
                        if let Some(mp) = mount_point {
                            if !mp.trim().is_empty() {
                                // Find processes using the mount point
                                match disks_dbus::find_processes_using_mount(&mp).await {
                                    Ok(processes) => {
                                        return UnmountResult::Busy {
                                            device,
                                            mount_point: mp,
                                            processes,
                                            object_path: object_path_for_retry,
                                        };
                                    }
                                    Err(find_err) => {
                                        tracing::warn!(
                                            ?find_err,
                                            "Failed to find processes using mount"
                                        );
                                    }
                                }
                            } else {
                                tracing::warn!("Mount point is empty, cannot find processes");
                            }
                        } else {
                            tracing::warn!("No mount point available, cannot find processes");
                        }
                    }
                    // Generic error - log and continue
                    tracing::error!(?e, "child unmount failed");
                    UnmountResult::GenericError
                }
            }
        },
        move |result| match result {
            UnmountResult::Success(drives) => Message::UpdateNavWithChildSelection(
                drives,
                Some(object_path_for_selection.clone()),
            )
            .into(),
            UnmountResult::Busy {
                device,
                mount_point,
                processes,
                object_path,
            } => {
                // Show busy dialog
                Message::Dialog(Box::new(ShowDialog::UnmountBusy(UnmountBusyDialog {
                    device,
                    mount_point,
                    processes,
                    object_path,
                })))
                .into()
            }
            UnmountResult::GenericError => {
                // Generic error already logged
                Message::None.into()
            }
        },
    )
}
