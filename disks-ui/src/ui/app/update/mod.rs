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
use disks_dbus::DriveModel;

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
            // TODO: use DeviceManager.apply_change()

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
        Message::SidebarSelectChild { object_path } => {
            app.sidebar.selected_child = Some(SidebarNodeKey::Volume(object_path));
        }
        Message::SidebarToggleExpanded(key) => {
            app.sidebar.close_menu();
            app.sidebar.toggle_expanded(key);
        }
        Message::SidebarOpenMenu(key) => {
            if app.sidebar.open_menu_for.as_ref() == Some(&key) {
                app.sidebar.close_menu();
            } else {
                app.sidebar.open_menu(key);
            }
        }
        Message::SidebarDriveEject(block_path) => {
            app.sidebar.close_menu();
            if let Some(drive) = app.sidebar.find_drive(&block_path) {
                return drive::eject_drive(drive);
            }
        }
        Message::SidebarDriveAction { drive, action } => {
            app.sidebar.close_menu();
            let Some(model) = app.sidebar.find_drive(&drive) else {
                return Task::none();
            };

            return match action {
                crate::ui::app::message::SidebarDriveAction::Eject => drive::eject_drive(model),
                crate::ui::app::message::SidebarDriveAction::PowerOff => {
                    drive::power_off_drive(model)
                }
                crate::ui::app::message::SidebarDriveAction::Format => {
                    drive::format_for(app, model);
                    Task::none()
                }
                crate::ui::app::message::SidebarDriveAction::SmartData => {
                    drive::smart_data_for(app, model)
                }
                crate::ui::app::message::SidebarDriveAction::StandbyNow => {
                    drive::standby_now_drive(model)
                }
                crate::ui::app::message::SidebarDriveAction::Wakeup => drive::wakeup_drive(model),
            };
        }
        Message::SidebarVolumeUnmount { drive, object_path } => {
            app.sidebar.close_menu();

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
