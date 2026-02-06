use super::message::Message;
use super::state::{AppModel, ContextPage};
use crate::fl;
use crate::ui::dialogs::view as dialogs;
use crate::ui::sidebar;
use crate::ui::volumes::{VolumesControl, VolumesControlMessage};
use crate::utils::{labelled_info, link_info};
use crate::views::about::about;
use crate::views::menu::menu_view;
use cosmic::app::context_drawer as cosmic_context_drawer;
use cosmic::iced::Length;
use cosmic::iced::alignment::{Horizontal, Vertical};
use cosmic::widget::text::heading;
use cosmic::widget::{self, Space, icon};
use cosmic::{Apply, Element, iced_widget};
use disks_dbus::DriveModel;
use disks_dbus::bytes_to_pretty;

/// Elements to pack at the start of the header bar.
pub(crate) fn header_start(app: &AppModel) -> Vec<Element<'_, Message>> {
    menu_view(&app.core, &app.key_binds)
}

pub(crate) fn dialog(app: &AppModel) -> Option<Element<'_, Message>> {
    match app.dialog {
        Some(ref d) => match d {
            crate::ui::dialogs::state::ShowDialog::DeletePartition(state) => {
                Some(dialogs::confirmation(
                    fl!("delete", name = state.name.clone()),
                    fl!("delete-confirmation", name = state.name.clone()),
                    VolumesControlMessage::Delete.into(),
                    Some(Message::CloseDialog),
                    state.running,
                ))
            }

            crate::ui::dialogs::state::ShowDialog::AddPartition(state) => {
                Some(dialogs::create_partition(state.clone()))
            }

            crate::ui::dialogs::state::ShowDialog::FormatPartition(state) => {
                Some(dialogs::format_partition(state.clone()))
            }

            crate::ui::dialogs::state::ShowDialog::EditPartition(state) => {
                Some(dialogs::edit_partition(state.clone()))
            }

            crate::ui::dialogs::state::ShowDialog::ResizePartition(state) => {
                Some(dialogs::resize_partition(state.clone()))
            }

            crate::ui::dialogs::state::ShowDialog::EditFilesystemLabel(state) => {
                Some(dialogs::edit_filesystem_label(state.clone()))
            }

            crate::ui::dialogs::state::ShowDialog::EditMountOptions(state) => {
                Some(dialogs::edit_mount_options(state.clone()))
            }

            crate::ui::dialogs::state::ShowDialog::ConfirmAction(state) => {
                Some(dialogs::confirmation(
                    state.title.clone(),
                    state.body.clone(),
                    state.ok_message.clone(),
                    Some(Message::CloseDialog),
                    state.running,
                ))
            }

            crate::ui::dialogs::state::ShowDialog::TakeOwnership(state) => {
                Some(dialogs::take_ownership(state.clone()))
            }

            crate::ui::dialogs::state::ShowDialog::ChangePassphrase(state) => {
                Some(dialogs::change_passphrase(state.clone()))
            }

            crate::ui::dialogs::state::ShowDialog::EditEncryptionOptions(state) => {
                Some(dialogs::edit_encryption_options(state.clone()))
            }

            crate::ui::dialogs::state::ShowDialog::UnlockEncrypted(state) => {
                Some(dialogs::unlock_encrypted(state.clone()))
            }

            crate::ui::dialogs::state::ShowDialog::FormatDisk(state) => {
                Some(dialogs::format_disk(state.clone()))
            }

            crate::ui::dialogs::state::ShowDialog::SmartData(state) => {
                Some(dialogs::smart_data(state.clone()))
            }

            crate::ui::dialogs::state::ShowDialog::NewDiskImage(state) => {
                Some(dialogs::new_disk_image(state.as_ref().clone()))
            }

            crate::ui::dialogs::state::ShowDialog::AttachDiskImage(state) => {
                Some(dialogs::attach_disk_image(state.as_ref().clone()))
            }

            crate::ui::dialogs::state::ShowDialog::ImageOperation(state) => {
                Some(dialogs::image_operation(state.as_ref().clone()))
            }

            crate::ui::dialogs::state::ShowDialog::Info { title, body } => Some(dialogs::info(
                title.clone(),
                body.clone(),
                Message::CloseDialog,
            )),
        },
        None => None,
    }
}

/// Allows overriding the default nav bar widget.
pub(crate) fn nav_bar(app: &AppModel) -> Option<Element<'_, cosmic::Action<Message>>> {
    if !app.core.nav_bar_active() {
        return None;
    }

    let controls_enabled = app.dialog.is_none();

    let mut nav = sidebar::view::sidebar(&app.nav, &app.sidebar, controls_enabled)
        .map(Into::into)
        .apply(widget::container)
        .padding(8)
        .class(cosmic::style::Container::Background)
        // XXX both must be shrink to avoid flex layout from ignoring it
        .width(cosmic::iced::Length::Shrink)
        .height(cosmic::iced::Length::Shrink);

    if !app.core.is_condensed() {
        nav = nav.max_width(280);
    }

    Some(Element::from(nav))
}

/// Enables the COSMIC application to create a nav bar with this model.
pub(crate) fn nav_model(app: &AppModel) -> Option<&cosmic::widget::nav_bar::Model> {
    Some(&app.nav)
}

/// Display a context drawer if the context page is requested.
pub(crate) fn context_drawer(
    app: &AppModel,
) -> Option<cosmic_context_drawer::ContextDrawer<'_, Message>> {
    if !app.core.window.show_context {
        return None;
    }

    Some(match app.context_page {
        ContextPage::About => cosmic_context_drawer::context_drawer(
            about(),
            Message::ToggleContextPage(ContextPage::About),
        )
        .title(fl!("about")),
    })
}

/// Describes the interface based on the current state of the application model.
pub(crate) fn view(app: &AppModel) -> Element<'_, Message> {
    match app.nav.active_data::<DriveModel>() {
        None => widget::text::title1(fl!("no-disk-selected"))
            .apply(widget::container)
            .width(Length::Fill)
            .height(Length::Fill)
            .align_x(Horizontal::Center)
            .align_y(Vertical::Center)
            .into(),

        Some(drive) => {
            let Some(volumes_control) = app.nav.active_data::<VolumesControl>() else {
                return widget::text::title1(fl!("working"))
                    .apply(widget::container)
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .align_x(Horizontal::Center)
                    .align_y(Vertical::Center)
                    .into();
            };

            let Some(segment) = volumes_control
                .segments
                .get(volumes_control.selected_segment)
                .or_else(|| volumes_control.segments.first())
            else {
                return widget::text::title1(fl!("no-volumes"))
                    .apply(widget::container)
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .align_x(Horizontal::Center)
                    .align_y(Vertical::Center)
                    .into();
            };

            let info = if let Some(v) = volumes_control.selected_volume_node() {
                let mut col = iced_widget::column![
                    heading(v.label.clone()),
                    Space::new(0, 10),
                    labelled_info(fl!("size"), bytes_to_pretty(&v.size, true)),
                ]
                .spacing(5);

                if let Some(usage) = &v.usage {
                    col = col.push(labelled_info(
                        fl!("usage"),
                        bytes_to_pretty(&usage.used, false),
                    ));
                }

                if let Some(mount_point) = v.mount_points.first() {
                    col = col.push(link_info(
                        fl!("mounted-at"),
                        mount_point,
                        Message::OpenPath(mount_point.clone()),
                    ));
                }

                let contents = if v.id_type.is_empty() {
                    match v.kind {
                        disks_dbus::VolumeKind::Filesystem => fl!("filesystem"),
                        disks_dbus::VolumeKind::LvmLogicalVolume => "LVM LV".to_string(),
                        disks_dbus::VolumeKind::LvmPhysicalVolume => "LVM PV".to_string(),
                        disks_dbus::VolumeKind::CryptoContainer => "LUKS".to_string(),
                        disks_dbus::VolumeKind::Partition => "Partition".to_string(),
                        disks_dbus::VolumeKind::Block => "Device".to_string(),
                    }
                } else {
                    v.id_type.to_uppercase()
                };

                col.push(labelled_info(fl!("contents"), contents))
                    .push(labelled_info(
                        fl!("device"),
                        match v.device_path.as_ref() {
                            Some(s) => s.clone(),
                            None => fl!("unresolved"),
                        },
                    ))
            } else {
                match segment.volume.clone() {
                    Some(p) => {
                        let mut name = p.name.clone();
                        if name.is_empty() {
                            name = fl!("partition-number", number = p.number);
                        } else {
                            name =
                                fl!("partition-number-with-name", number = p.number, name = name);
                        }

                        let mut type_str = p.id_type.clone().to_uppercase();
                        type_str = format!("{} - {}", type_str, p.partition_type.clone());

                        let mut col = iced_widget::column![
                            heading(name),
                            Space::new(0, 10),
                            labelled_info(fl!("size"), bytes_to_pretty(&p.size, true)),
                        ]
                        .spacing(5);

                        if let Some(usage) = &p.usage {
                            col = col.push(labelled_info(
                                fl!("usage"),
                                bytes_to_pretty(&usage.used, false),
                            ));
                        }

                        if let Some(mount_point) = p.mount_points.first() {
                            col = col.push(link_info(
                                fl!("mounted-at"),
                                mount_point,
                                Message::OpenPath(mount_point.clone()),
                            ));
                        }

                        col = col
                            .push(labelled_info(fl!("contents"), &type_str))
                            .push(labelled_info(
                                fl!("device"),
                                match p.device_path {
                                    Some(s) => s,
                                    None => fl!("unresolved"),
                                },
                            ))
                            .push(labelled_info(fl!("uuid"), &p.uuid));

                        col
                    }
                    None => iced_widget::column![
                        heading(&segment.label),
                        labelled_info("Size", bytes_to_pretty(&segment.size, true)),
                    ]
                    .spacing(5),
                }
            };

            let partition_type = match &drive.partition_table_type {
                Some(t) => t.clone().to_uppercase(),
                None => "Unknown".into(),
            };

            let can_remove = drive.is_loop || (drive.removable && drive.can_power_off);

            let mut drive_header = iced_widget::Row::new()
                .push(heading(drive.name()))
                .push(Space::new(Length::Fill, 0))
                .spacing(10)
                .width(Length::Fill);

            if can_remove {
                drive_header = drive_header.push(
                    widget::button::custom(icon::from_name("media-eject-symbolic"))
                        .on_press(Message::Eject),
                );
            }

            let drive_info = if drive.is_loop {
                iced_widget::column![
                    drive_header,
                    Space::new(0, 10),
                    labelled_info("Size", bytes_to_pretty(&drive.size, true)),
                    labelled_info("Backing File", drive.backing_file.as_deref().unwrap_or(""),),
                ]
                .spacing(5)
                .width(Length::Fill)
            } else {
                iced_widget::column![
                    drive_header,
                    Space::new(0, 10),
                    labelled_info("Model", &drive.model),
                    labelled_info("Serial", &drive.serial),
                    labelled_info("Size", bytes_to_pretty(&drive.size, true)),
                    labelled_info("Partitioning", &partition_type),
                ]
                .spacing(5)
                .width(Length::Fill)
            };
            iced_widget::column![
                drive_info,
                iced_widget::column![
                    heading("Volumes"),
                    Space::new(0, 10),
                    volumes_control.view()
                ]
                .spacing(5)
                .width(Length::Fill),
                info
            ]
            .spacing(60)
            .padding(20)
            .width(Length::Fill)
            .into()
        }
    }
}
