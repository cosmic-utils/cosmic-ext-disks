mod drive;
mod image;
mod nav;
mod smart;

use super::message::Message;
use super::state::AppModel;
use crate::app::REPOSITORY;
use crate::fl;
use crate::ui::error::{UiErrorContext, log_error_and_show_dialog};
use crate::ui::sidebar::SidebarNodeKey;
use crate::ui::volumes::VolumesControl;
use crate::ui::volumes::helpers as volumes_helpers;
use cosmic::app::Task;
use cosmic::widget::nav_bar;
use disks_dbus::{DriveModel, VolumeNode};

/// Recursively search for a volume child by object_path
fn find_volume_child_recursive<'a>(
    children: &'a [VolumeNode],
    object_path: &str,
) -> Option<&'a VolumeNode> {
    for child in children {
        if child.object_path.as_str() == object_path {
            return Some(child);
        }
        if let Some(found) = find_volume_child_recursive(&child.children, object_path) {
            return Some(found);
        }
    }
    None
}

/// Find the segment index and whether the volume is a child for a given object path
fn find_segment_for_volume(
    volumes_control: &VolumesControl,
    object_path: &str,
) -> Option<(usize, bool)> {
    for (segment_idx, segment) in volumes_control.segments.iter().enumerate() {
        let Some(segment_vol) = &segment.volume else {
            continue;
        };

        // Direct match (partition itself)
        if segment_vol.path.as_str() == object_path {
            return Some((segment_idx, false));
        }

        // Check if volume is a child of this segment's partition
        let Some(segment_node) = volumes_helpers::find_volume_node(
            &volumes_control.model.volumes,
            segment_vol.path.as_str(),
        ) else {
            continue;
        };

        if find_volume_child_recursive(&segment_node.children, object_path).is_some() {
            return Some((segment_idx, true));
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
                    match DriveModel::get_drives().await {
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
                    match DriveModel::get_drives().await {
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
            nav::update_nav(app, drive_models, selected);
        }
        Message::UpdateNavWithChildSelection(drive_models, child_object_path) => {
            // Update drives while preserving child volume selection
            nav::update_nav(app, drive_models, None);
            
            // Restore child selection if provided
            if let Some(object_path) = child_object_path {
                app.sidebar.selected_child =
                    Some(crate::ui::sidebar::SidebarNodeKey::Volume(object_path.clone()));

                if let Some(control) = app.nav.active_data_mut::<VolumesControl>() {
                    if let Some((segment_idx, is_child)) =
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
            }
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
            let drive_for_volume = app.sidebar.drives.iter()
                .find(|d| volumes_helpers::find_volume_node(&d.volumes, &object_path).is_some())
                .cloned();

            // If the volume belongs to a different drive, switch to that drive first
            if let Some(drive) = drive_for_volume {
                let current_drive_block_path = app.sidebar.active_drive_block_path(&app.nav);
                if current_drive_block_path.as_deref() != Some(&drive.block_path) {
                    // Switch to the correct drive
                    if let Some(id) = app.sidebar.drive_entities.get(&drive.block_path).copied() {
                        let switch_task = on_nav_select(app, id);
                        // After switching, we need to select the volume again
                        return switch_task.chain(Task::done(cosmic::Action::App(Message::SidebarSelectChild { object_path })));
                    }
                }
            }

            // Sync with volumes control: select the corresponding volume
            let Some(volumes_control) = app.nav.active_data_mut::<VolumesControl>() else {
                return Task::none();
            };

            let Some(vol_node) =
                volumes_helpers::find_volume_node(&volumes_control.model.volumes, &object_path)
            else {
                return Task::none();
            };

            let Some((segment_idx, is_child)) = find_segment_for_volume(volumes_control, &object_path)
            else {
                return Task::none();
            };

            // Apply the selection change
            volumes_control.selected_segment = segment_idx;
            volumes_control.selected_volume = if is_child {
                Some(vol_node.object_path.to_string())
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
                return drive::eject_drive(drive);
            }
        }
        Message::SidebarVolumeUnmount { drive, object_path } => {
            let Some(drive_model) = app.sidebar.find_drive(&drive) else {
                return Task::none();
            };

            let Some(node) =
                volumes_helpers::find_volume_node(&drive_model.volumes, &object_path).cloned()
            else {
                return Task::none();
            };

            let drive_path = drive_model.path.clone();
            let device = drive_model.block_path.clone();

            return Task::perform(
                async move {
                    node.unmount().await?;
                    DriveModel::get_drives().await
                },
                move |res| match res {
                    Ok(drives) => Message::UpdateNav(drives, None).into(),
                    Err(e) => {
                        let ctx = UiErrorContext {
                            operation: "sidebar_volume_unmount",
                            object_path: Some(object_path.as_str()),
                            device: Some(device.as_str()),
                            drive_path: Some(drive_path.as_str()),
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
