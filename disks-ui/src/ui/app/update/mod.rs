mod btrfs;
mod drive;
mod image;
mod nav;
mod smart;

use super::APP_ID;
use super::message::{ImagePathPickerKind, Message};
use super::state::AppModel;
use crate::app::REPOSITORY;
use crate::client::FilesystemsClient;
use crate::config::Config;
use crate::fl;
use crate::models::load_all_drives;
use crate::ui::dialogs::state::ShowDialog;
use crate::ui::error::{UiErrorContext, log_error_and_show_dialog};
use crate::ui::sidebar::SidebarNodeKey;
use crate::ui::volumes::VolumesControl;
use crate::ui::volumes::helpers as volumes_helpers;
use cosmic::app::Task;
use cosmic::cosmic_config::CosmicConfigEntry;
use cosmic::dialog::file_chooser;
use cosmic::widget::nav_bar;

/// Recursively search for a volume child by object_path
fn find_volume_child_recursive<'a>(
    children: &'a [crate::models::UiVolume],
    device_path: &str,
) -> Option<&'a crate::models::UiVolume> {
    for child in children {
        if child.device() == Some(device_path) {
            return Some(child);
        }
        if let Some(found) = find_volume_child_recursive(&child.children, device_path) {
            return Some(found);
        }
    }
    None
}

/// Find the segment index and whether the volume is a child for a given device path
fn find_segment_for_volume(
    volumes_control: &VolumesControl,
    device_path: &str,
) -> Option<(usize, bool)> {
    for (segment_idx, segment) in volumes_control.segments.iter().enumerate() {
        let Some(segment_vol) = &segment.volume else {
            continue;
        };

        // Direct match (partition itself)
        if segment_vol.device_path.as_ref().map_or(false, |p| p == device_path) {
            return Some((segment_idx, false));
        }

        // Check if volume is a child of this segment's partition
        let Some(segment_device) = &segment_vol.device_path else {
            continue;
        };
        
        // Search for the segment volume in the tree
        let segment_node = volumes_control.volumes.iter()
            .find(|v| v.device() == Some(segment_device.as_str()));
        
        let Some(segment_node) = segment_node else {
            continue;
        };

        // Recursively check children
        if find_volume_in_tree(&segment_node.children, device_path).is_some() {
            return Some((segment_idx, true));
        }
    }

    None
}

/// Find a volume by device path in the tree
fn find_volume_in_tree<'a>(
    volumes: &'a [crate::models::UiVolume],
    device_path: &str,
) -> Option<&'a crate::models::UiVolume> {
    for vol in volumes {
        if vol.device() == Some(device_path) {
            return Some(vol);
        }
        if let Some(found) = find_volume_in_tree(&vol.children, device_path) {
            return Some(found);
        }
    }
    None
}

/// Handles messages emitted by the application and its widgets.
pub(crate) fn update(app: &mut AppModel, message: Message) -> Task<Message> {
    match message {
        Message::OpenRepositoryUrl => {
            _ = open::that_detached(REPOSITORY);
        }
        Message::OpenPath(path) => {
            _ = open::that_detached(path);
        }
        Message::ToggleContextPage(context_page) => {
            if app.context_page == context_page {
                // Close the context drawer if the toggled context page is the same.
                app.core.window.show_context = !app.core.window.show_context;
            } else {
                // Open the context drawer to display the requested context page.
                app.context_page = context_page;
                app.core.window.show_context = true;
            }
        }
        Message::UpdateConfig(config) => {
            app.config = config;
        }
        Message::ToggleShowReserved(show_reserved) => {
            app.config.show_reserved = show_reserved;

            // Persist config change
            if let Ok(helper) = cosmic::cosmic_config::Config::new(APP_ID, Config::VERSION) {
                let _ = app.config.write_entry(&helper);
            }

            // Update the active volumes control if one is selected
            if let Some(volumes_control) = app.nav.active_data_mut::<VolumesControl>() {
                volumes_control.set_show_reserved(show_reserved);
            }
        }
        Message::OpenImagePathPicker(kind) => {
            let title = match kind {
                ImagePathPickerKind::NewDiskImage | ImagePathPickerKind::ImageOperationCreate => {
                    fl!("image-destination-path")
                }
                ImagePathPickerKind::AttachDiskImage
                | ImagePathPickerKind::ImageOperationRestore => fl!("image-file-path"),
            };

            return Task::perform(
                async move {
                    let result = match kind {
                        ImagePathPickerKind::NewDiskImage
                        | ImagePathPickerKind::ImageOperationCreate => {
                            let dialog = file_chooser::save::Dialog::new().title(title);
                            match dialog.save_file().await {
                                Ok(response) => response
                                    .url()
                                    .and_then(|url| url.to_file_path().ok())
                                    .map(|path| path.to_string_lossy().to_string()),
                                Err(file_chooser::Error::Cancelled) => None,
                                Err(err) => {
                                    tracing::warn!(?err, "save file dialog failed");
                                    None
                                }
                            }
                        }
                        ImagePathPickerKind::AttachDiskImage
                        | ImagePathPickerKind::ImageOperationRestore => {
                            let dialog = file_chooser::open::Dialog::new().title(title);
                            match dialog.open_file().await {
                                Ok(response) => response
                                    .url()
                                    .to_file_path()
                                    .ok()
                                    .map(|path| path.to_string_lossy().to_string()),
                                Err(file_chooser::Error::Cancelled) => None,
                                Err(err) => {
                                    tracing::warn!(?err, "open file dialog failed");
                                    None
                                }
                            }
                        }
                    };

                    Message::ImagePathPicked(kind, result)
                },
                |msg| msg.into(),
            );
        }
        Message::ImagePathPicked(kind, path) => match kind {
            ImagePathPickerKind::NewDiskImage => {
                if let Some(ShowDialog::NewDiskImage(state)) = app.dialog.as_mut()
                    && let Some(path) = path
                {
                    state.path = path;
                }
            }
            ImagePathPickerKind::AttachDiskImage => {
                if let Some(ShowDialog::AttachDiskImage(state)) = app.dialog.as_mut()
                    && let Some(path) = path
                {
                    state.path = path;
                }
            }
            ImagePathPickerKind::ImageOperationCreate
            | ImagePathPickerKind::ImageOperationRestore => {
                if let Some(ShowDialog::ImageOperation(state)) = app.dialog.as_mut()
                    && let Some(path) = path
                {
                    state.image_path = path;
                }
            }
        },
        Message::LaunchUrl(url) => match open::that_detached(&url) {
            Ok(()) => {}
            Err(err) => {
                tracing::warn!(?url, %err, "failed to open url");
            }
        },
        Message::VolumesMessage(message) => {
            let Some(volumes_control) = app.nav.active_data_mut::<VolumesControl>() else {
                tracing::warn!("received volumes message with no active VolumesControl");
                return Task::none();
            };

            return volumes_control.update(message, &mut app.dialog);
        }

        Message::FormatDisk(msg) => {
            return drive::format_disk(app, msg);
        }
        Message::DriveRemoved(_drive_model) => {
            return Task::perform(
                async {
                    match load_all_drives().await {
                        Ok(drives) => Some(drives),
                        Err(e) => {
                            tracing::error!(%e, "failed to refresh drives after drive removal");
                            None
                        }
                    }
                },
                move |drives| match drives {
                    None => Message::None.into(),
                    Some(drives) => Message::UpdateNav(drives, None).into(),
                },
            );
        }
        Message::DriveAdded(_drive_model) => {
            return Task::perform(
                async {
                    match load_all_drives().await {
                        Ok(drives) => Some(drives),
                        Err(e) => {
                            tracing::error!(%e, "failed to refresh drives after drive add");
                            None
                        }
                    }
                },
                move |drives| match drives {
                    None => Message::None.into(),
                    Some(drives) => Message::UpdateNav(drives, None).into(),
                },
            );
        }
        Message::None => {}
        Message::UpdateNav(drive_models, selected) => {
            return nav::update_nav(app, drive_models, selected);
        }

        // BTRFS management
        Message::BtrfsLoadSubvolumes { .. }
        | Message::BtrfsSubvolumesLoaded { .. }
        | Message::BtrfsDeleteSubvolume { .. }
        | Message::BtrfsDeleteSubvolumeConfirm { .. }
        | Message::BtrfsLoadUsage { .. }
        | Message::BtrfsUsageLoaded { .. }
        | Message::BtrfsToggleSubvolumeExpanded { .. }
        | Message::BtrfsLoadDefaultSubvolume { .. }
        | Message::BtrfsDefaultSubvolumeLoaded { .. }
        | Message::BtrfsSetDefaultSubvolume { .. }
        | Message::BtrfsToggleReadonly { .. }
        | Message::BtrfsReadonlyToggled { .. }
        | Message::BtrfsShowProperties { .. }
        | Message::BtrfsCloseProperties { .. }
        | Message::BtrfsLoadDeletedSubvolumes { .. }
        | Message::BtrfsDeletedSubvolumesLoaded { .. }
        | Message::BtrfsToggleShowDeleted { .. }
        | Message::BtrfsRefreshAll { .. } => {
            return btrfs::handle_btrfs_message(app, message);
        }

        Message::UpdateNavWithChildSelection(drive_models, child_object_path) => {
            // Update drives while preserving child volume selection
            let task = nav::update_nav(app, drive_models, None);

            // Restore child selection if provided
            if let Some(object_path) = child_object_path {
                app.sidebar.selected_child = Some(crate::ui::sidebar::SidebarNodeKey::Volume(
                    object_path.clone(),
                ));

                if let Some(control) = app.nav.active_data_mut::<VolumesControl>()
                    && let Some((segment_idx, is_child)) =
                        find_segment_for_volume(control, &object_path)
                {
                    control.selected_volume = if is_child {
                        Some(object_path.clone())
                    } else {
                        None
                    };

                    control.segments.iter_mut().for_each(|s| s.state = false);
                    control.selected_segment = segment_idx;
                    if let Some(segment) = control.segments.get_mut(segment_idx) {
                        segment.state = true;
                    }
                }
            }

            return task;
        }
        Message::Dialog(show_dialog) => app.dialog = Some(*show_dialog),
        Message::CloseDialog => {
            app.dialog = None;
        }
        Message::Eject => {
            return drive::eject(app);
        }
        Message::PowerOff => {
            return drive::power_off(app);
        }
        Message::Format => {
            drive::format(app);
        }
        Message::SmartData => {
            return drive::smart_data(app);
        }
        Message::StandbyNow => {
            return drive::standby_now(app);
        }
        Message::Wakeup => {
            return drive::wakeup(app);
        }

        // Sidebar (custom treeview)
        Message::SidebarSelectDrive(block_path) => {
            app.sidebar.selected_child = None;
            if let Some(id) = app.sidebar.drive_entities.get(&block_path).copied() {
                return on_nav_select(app, id);
            }
        }
        Message::SidebarClearChildSelection => {
            app.sidebar.selected_child = None;
        }
        Message::SidebarSelectChild { object_path } => {
            app.sidebar.selected_child = Some(SidebarNodeKey::Volume(object_path.clone()));

            // Find which drive contains this volume node
            let drive_for_volume = app
                .sidebar
                .drives
                .iter()
                .find(|d| volumes_helpers::find_volume_in_ui_tree(&d.volumes, &object_path).is_some())
                .cloned();

            // If the volume belongs to a different drive, switch to that drive first
            if let Some(drive) = drive_for_volume {
                let current_drive_block_path = app.sidebar.active_drive_block_path(&app.nav);
                if current_drive_block_path.as_deref() != Some(drive.block_path()) {
                    // Switch to the correct drive
                    if let Some(id) = app.sidebar.drive_entities.get(drive.block_path()).copied() {
                        let switch_task = on_nav_select(app, id);
                        // After switching, we need to select the volume again
                        return switch_task.chain(Task::done(cosmic::Action::App(
                            Message::SidebarSelectChild { object_path },
                        )));
                    }
                }
            }

            // Sync with volumes control: select the corresponding volume
            let Some(volumes_control) = app.nav.active_data_mut::<VolumesControl>() else {
                return Task::none();
            };

            let Some(vol_node) =
                volumes_helpers::find_volume_in_ui_tree(&volumes_control.volumes, &object_path)
            else {
                return Task::none();
            };

            let Some((segment_idx, is_child)) =
                find_segment_for_volume(volumes_control, &object_path)
            else {
                return Task::none();
            };

            // Apply the selection change
            volumes_control.selected_segment = segment_idx;
            volumes_control.selected_volume = if is_child {
                vol_node.object_path()
            } else {
                None
            };

            // Update segment state
            volumes_control
                .segments
                .iter_mut()
                .for_each(|s| s.state = false);
            if let Some(segment) = volumes_control.segments.get_mut(segment_idx) {
                segment.state = true;
            }
        }
        Message::SidebarToggleExpanded(key) => {
            app.sidebar.toggle_expanded(key);
        }
        Message::SidebarDriveEject(block_path) => {
            if let Some(drive) = app.sidebar.find_drive(&block_path) {
                return drive::eject_drive(drive.clone());
            }
        }
        Message::SidebarVolumeUnmount { drive, object_path } => {
            let Some(drive_model) = app.sidebar.find_drive(&drive) else {
                return Task::none();
            };

            // object_path is now actually a device_path (migration in progress)
            let device_path = object_path.clone();
            let Some(node) =
                volumes_helpers::find_volume_in_ui_tree(&drive_model.volumes, &device_path)
            else {
                return Task::none();
            };

            let device = drive_model.device().to_string();

            return Task::perform(
                async move {
                    let fs_client = FilesystemsClient::new().await
                        .map_err(|e| anyhow::anyhow!("Failed to create filesystems client: {}", e))?;
                    let device_to_unmount = node.device()
                        .ok_or_else(|| anyhow::anyhow!("Node has no device path"))?;
                    fs_client.unmount(device_to_unmount, false, false).await
                        .map_err(|e| anyhow::anyhow!("Failed to unmount: {}", e))?;
                    load_all_drives().await
                        .map_err(|e| anyhow::anyhow!("Failed to reload drives: {}", e))
                },
                move |res| match res {
                    Ok(drives) => Message::UpdateNav(drives, None).into(),
                    Err(e) => {
                        let ctx = UiErrorContext {
                            operation: "sidebar_volume_unmount",
                            object_path: Some(object_path.as_str()),
                            device: Some(device.as_str()),
                            drive_path: None,
                        };
                        log_error_and_show_dialog(fl!("unmount-failed"), e, ctx).into()
                    }
                },
            );
        }
        Message::SmartDialog(msg) => {
            return smart::smart_dialog(app, msg);
        }
        Message::NewDiskImage => {
            image::new_disk_image(app);
        }
        Message::AttachDisk => {
            image::attach_disk(app);
        }
        Message::CreateDiskFrom => {
            return image::create_disk_from(app);
        }
        Message::RestoreImageTo => {
            return image::restore_image_to(app);
        }
        Message::CreateDiskFromPartition => {
            return image::create_disk_from_partition(app);
        }
        Message::RestoreImageToPartition => {
            return image::restore_image_to_partition(app);
        }
        Message::NewDiskImageDialog(msg) => {
            return image::new_disk_image_dialog(app, msg);
        }
        Message::AttachDiskImageDialog(msg) => {
            return image::attach_disk_image_dialog(app, msg);
        }
        Message::ImageOperationDialog(msg) => {
            return image::image_operation_dialog(app, msg);
        }
        Message::UnmountBusy(msg) => {
            use crate::ui::dialogs::message::UnmountBusyMessage;

            // Extract dialog data before consuming it
            let dialog_data = if let Some(ShowDialog::UnmountBusy(ref dialog)) = app.dialog {
                Some((
                    dialog.device_path.clone(),
                    dialog.mount_point.clone(),
                    dialog.processes.iter().map(|p| p.pid).collect::<Vec<_>>(),
                ))
            } else {
                None
            };

            match msg {
                UnmountBusyMessage::Cancel => {
                    tracing::debug!("User cancelled unmount busy dialog");
                    app.dialog = None;
                }
                UnmountBusyMessage::Retry => {
                    tracing::info!(
                        object_path = dialog_data
                            .as_ref()
                            .map(|(op, _, _)| op.as_str())
                            .unwrap_or("unknown"),
                        "User requested unmount retry"
                    );
                    app.dialog = None;

                    if let Some((object_path, _, _)) = dialog_data {
                        // Retry the unmount operation
                        if let Some(volumes) = app.nav.active_data::<VolumesControl>() {
                            return retry_unmount(volumes, object_path);
                        }
                    }
                }
                UnmountBusyMessage::KillAndRetry => {
                    let process_count = dialog_data
                        .as_ref()
                        .map(|(_, _, pids)| pids.len())
                        .unwrap_or(0);
                    tracing::info!(
                        object_path = dialog_data
                            .as_ref()
                            .map(|(op, _, _)| op.as_str())
                            .unwrap_or("unknown"),
                        process_count = process_count,
                        "User requested kill processes and retry unmount"
                    );

                    if let Some((object_path, mount_point, pids)) = dialog_data {
                        app.dialog = None;

                        // Kill processes and then retry unmount
                        return Task::perform(
                            async move {
                                tracing::debug!(
                                    mount_point = %mount_point,
                                    process_count = pids.len(),
                                    "Killing processes holding mount"
                                );

                                // Kill the processes
                                let kill_results = disks_dbus::kill_processes(&pids);
                                let failed = kill_results.iter().filter(|r| !r.success).count();

                                if failed > 0 {
                                    tracing::warn!(
                                        failed_count = failed,
                                        total_count = pids.len(),
                                        "Failed to kill some processes"
                                    );
                                    for result in kill_results.iter().filter(|r| !r.success) {
                                        tracing::debug!(
                                            pid = result.pid,
                                            error = result.error.as_deref().unwrap_or("unknown"),
                                            "Process kill failed"
                                        );
                                    }
                                } else {
                                    tracing::info!(
                                        total_count = pids.len(),
                                        "Successfully killed all processes"
                                    );
                                }

                                // Small delay to let processes terminate
                                tokio::time::sleep(std::time::Duration::from_millis(100)).await;

                                (object_path, mount_point)
                            },
                            move |(object_path, _mount_point)| {
                                // After killing, retry the unmount
                                Message::RetryUnmountAfterKill(object_path).into()
                            },
                        );
                    } else {
                        app.dialog = None;
                    }
                }
            }
        }
        Message::RetryUnmountAfterKill(object_path) => {
            tracing::debug!("Retrying unmount after killing processes");
            if let Some(volumes) = app.nav.active_data::<VolumesControl>() {
                return retry_unmount(volumes, object_path);
            }
        }
        Message::Surface(action) => {
            return cosmic::task::message(cosmic::Action::Cosmic(cosmic::app::Action::Surface(
                action,
            )));
        }
    }
    Task::none()
}

/// Called when a nav item is selected.
pub(crate) fn on_nav_select(app: &mut AppModel, id: nav_bar::Id) -> Task<Message> {
    // Activate the page in the model.
    if app.dialog.is_none() {
        let previous_show_reserved = app
            .nav
            .active_data::<VolumesControl>()
            .map(|v| v.show_reserved);

        app.nav.activate(id);

        if let Some(show_reserved) = previous_show_reserved
            && let Some(volumes_control) = app.nav.active_data_mut::<VolumesControl>()
        {
            volumes_control.set_show_reserved(show_reserved);
        }

        app.update_title()
    } else {
        Task::none()
    }
}

/// Helper function to retry unmount operation on a volume by device path
fn retry_unmount(volumes: &VolumesControl, device_path: String) -> Task<Message> {
    // Find the volume node
    let node = volumes_helpers::find_volume_in_ui_tree(&volumes.volumes, &device_path).cloned();

    if let Some(node) = node {
        let device = node
            .volume
            .device_path
            .clone()
            .unwrap_or_else(|| device_path.clone());
        let mount_point = node.volume.mount_points.first().cloned();
        let device_path_for_retry = device_path.clone();
        let device_path_for_selection = device_path.clone();

        Task::perform(
            async move {
                let fs_client = match FilesystemsClient::new().await {
                    Ok(c) => c,
                    Err(e) => {
                        tracing::error!(?e, "Failed to create filesystems client");
                        return Err(None);
                    }
                };
                
                let unmount_result = match fs_client.unmount(&device, false, false).await {
                    Ok(r) => r,
                    Err(e) => {
                        tracing::error!(?e, "Failed to unmount");
                        return Err(None);
                    }
                };
                
                if unmount_result.success {
                    // Success - reload drives
                    match load_all_drives().await {
                        Ok(drives) => Ok(drives),
                        Err(e) => {
                            tracing::error!(?e, "Failed to reload drives after unmount");
                            Err(None)
                        }
                    }
                } else if !unmount_result.blocking_processes.is_empty() {
                    // Device is busy with processes
                    let mp = mount_point.unwrap_or_default();
                    tracing::warn!(
                        mount_point = %mp,
                        process_count = unmount_result.blocking_processes.len(),
                        "Unmount still busy after retry"
                    );
                    Err(Some((
                        device,
                        mp,
                        unmount_result.blocking_processes,
                        device_path_for_retry,
                    )))
                } else {
                    // Generic error
                    if let Some(err) = unmount_result.error {
                        tracing::error!("unmount retry failed: {}", err);
                    } else {
                        tracing::error!("unmount retry failed with unknown error");
                    }
                    Err(None)
                }
            },
            move |result| match result {
                Ok(drives) => Message::UpdateNavWithChildSelection(
                    drives,
                    Some(device_path_for_selection.clone()),
                )
                .into(),
                Err(Some((device, mount_point, processes, device_path))) => {
                    // Still busy - show dialog again
                    Message::Dialog(Box::new(ShowDialog::UnmountBusy(
                        crate::ui::dialogs::state::UnmountBusyDialog {
                            device,
                            mount_point,
                            processes,
                            device_path,
                        },
                    )))
                    .into()
                }
                Err(None) => {
                    // Generic error already logged
                    Message::None.into()
                }
            },
        )
    } else {
        tracing::warn!("Volume not found for retry: {}", device_path);
        Task::none()
    }
}
