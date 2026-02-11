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
    let mount_point = volume.mount_points.first().cloned().unwrap_or_default();
    let object_path = volume.path.to_string();
    let object_path_for_retry = object_path.clone();

    Task::perform(
        async move {
            match volume.unmount().await {
                Ok(()) => {
                    // Success - reload drives
                    match DriveModel::get_drives().await {
                        Ok(drives) => Ok(drives),
                        Err(e) => {
                            tracing::error!(?e, "Failed to reload drives");
                            Err(UnmountBusyError {
                                device: String::new(),
                                mount_point: String::new(),
                                processes: Vec::new(),
                                object_path: String::new(),
                            })
                        }
                    }
                }
                Err(e) => {
                    // Check if it's a ResourceBusy error
                    if let Some(disk_err) = e.downcast_ref::<DiskError>()
                        && matches!(disk_err, DiskError::ResourceBusy { .. })
                    {
                        // Find processes using the mount point
                        match disks_dbus::find_processes_using_mount(&mount_point).await {
                            Ok(processes) => {
                                return Err(UnmountBusyError {
                                    device,
                                    mount_point,
                                    processes,
                                    object_path: object_path_for_retry,
                                });
                            }
                            Err(find_err) => {
                                tracing::warn!(?find_err, "Failed to find processes using mount");
                            }
                        }
                    }
                    // Generic error - log and continue
                    tracing::error!(?e, "unmount failed");
                    Err(UnmountBusyError {
                        device: String::new(),
                        mount_point: String::new(),
                        processes: Vec::new(),
                        object_path: String::new(),
                    })
                }
            }
        },
        move |result| match result {
            Ok(drives) => {
                Message::UpdateNavWithChildSelection(drives, Some(object_path.clone())).into()
            }
            Err(busy_err) if !busy_err.device.is_empty() => {
                // Show busy dialog
                Message::Dialog(Box::new(ShowDialog::UnmountBusy(UnmountBusyDialog {
                    device: busy_err.device,
                    mount_point: busy_err.mount_point,
                    processes: busy_err.processes,
                    object_path: busy_err.object_path,
                })))
                .into()
            }
            Err(_) => {
                // Generic error already logged
                Message::None.into()
            }
        },
    )
}

// Helper struct to pass busy error data through the async boundary
#[derive(Debug)]
struct UnmountBusyError {
    device: String,
    mount_point: String,
    processes: Vec<disks_dbus::ProcessInfo>,
    object_path: String,
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
    let mount_point = node.mount_points.first().cloned().unwrap_or_default();
    let object_path_for_selection = object_path.clone();
    let object_path_for_retry = object_path.clone();

    Task::perform(
        async move {
            match node.unmount().await {
                Ok(()) => {
                    // Success - reload drives
                    match DriveModel::get_drives().await {
                        Ok(drives) => Ok(drives),
                        Err(e) => {
                            tracing::error!(?e, "Failed to reload drives");
                            Err(UnmountBusyError {
                                device: String::new(),
                                mount_point: String::new(),
                                processes: Vec::new(),
                                object_path: String::new(),
                            })
                        }
                    }
                }
                Err(e) => {
                    // Check if it's a ResourceBusy error
                    if let Some(disk_err) = e.downcast_ref::<DiskError>()
                        && matches!(disk_err, DiskError::ResourceBusy { .. })
                    {
                        // Find processes using the mount point
                        match disks_dbus::find_processes_using_mount(&mount_point).await {
                            Ok(processes) => {
                                return Err(UnmountBusyError {
                                    device,
                                    mount_point,
                                    processes,
                                    object_path: object_path_for_retry,
                                });
                            }
                            Err(find_err) => {
                                tracing::warn!(?find_err, "Failed to find processes using mount");
                            }
                        }
                    }
                    // Generic error - log and continue
                    tracing::error!(?e, "child unmount failed");
                    Err(UnmountBusyError {
                        device: String::new(),
                        mount_point: String::new(),
                        processes: Vec::new(),
                        object_path: String::new(),
                    })
                }
            }
        },
        move |result| match result {
            Ok(drives) => Message::UpdateNavWithChildSelection(
                drives,
                Some(object_path_for_selection.clone()),
            )
            .into(),
            Err(busy_err) if !busy_err.device.is_empty() => {
                // Show busy dialog
                Message::Dialog(Box::new(ShowDialog::UnmountBusy(UnmountBusyDialog {
                    device: busy_err.device,
                    mount_point: busy_err.mount_point,
                    processes: busy_err.processes,
                    object_path: busy_err.object_path,
                })))
                .into()
            }
            Err(_) => {
                // Generic error already logged
                Message::None.into()
            }
        },
    )
}
