use super::message::Message;
use super::state::{AppModel, ContextPage};
use crate::fl;
use crate::ui::dialogs::state::{
    CreatePartitionDialog, DeletePartitionDialog, ShowDialog, UnlockEncryptedDialog,
};
use crate::ui::dialogs::view as dialogs;
use crate::ui::sidebar;
use crate::ui::volumes::{VolumesControl, VolumesControlMessage, disk_header};
use crate::utils::DiskSegmentKind;
use crate::views::about::about;
use crate::views::menu::menu_view;
use cosmic::app::context_drawer as cosmic_context_drawer;
use cosmic::iced::Length;
use cosmic::iced::alignment::{Alignment, Horizontal, Vertical};
use cosmic::widget::{self, Space, icon};
use cosmic::{Apply, Element, iced_widget};
use disks_dbus::bytes_to_pretty;
use disks_dbus::{DriveModel, VolumeKind};

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

            // Calculate actual used space on the disk (sum of filesystem usage)
            let used: u64 = volumes_control
                .segments
                .iter()
                .filter_map(|s| s.volume.as_ref())
                .filter_map(|v| v.usage.as_ref())
                .map(|u| u.used)
                .sum();

            // Disk action buttons row
            let disk_actions = build_disk_action_bar(drive);

            // Top section: Disk header + disk actions + volumes control
            let top_section = iced_widget::column![
                disk_header::disk_header(drive, used),
                Space::new(0, 10),
                widget::Row::from_vec(disk_actions).spacing(10),
                Space::new(0, 10),
                volumes_control.view(),
            ]
            .spacing(10)
            .width(Length::Fill);

            // Bottom section: Volume-specific detail view (2/3 of height)
            let bottom_section = volume_detail_view(volumes_control, segment);

            // Split layout: header shrinks to fit, detail view fills remaining space
            iced_widget::column![
                widget::container(top_section)
                    .padding(20)
                    .width(Length::Fill)
                    .height(Length::Shrink),
                widget::container(bottom_section)
                    .padding(20)
                    .width(Length::Fill)
                    .height(Length::Fill)
            ]
            .spacing(0)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
        }
    }
}

/// Renders the volume detail view for the selected volume with action buttons.
fn volume_detail_view<'a>(
    volumes_control: &'a VolumesControl,
    segment: &'a crate::ui::volumes::Segment,
) -> Element<'a, Message> {
    let selected_volume_node = volumes_control.selected_volume_node();
    let selected_volume = segment.volume.as_ref().and_then(|p| {
        crate::ui::volumes::helpers::find_volume_node_for_partition(
            &volumes_control.model.volumes,
            p,
        )
    });

    // Build the info section (mirroring disk header layout)
    let header_section = if let Some(v) = selected_volume_node {
        build_volume_node_info(v)
    } else if let Some(ref p) = segment.volume {
        build_partition_info(p)
    } else {
        build_free_space_info(segment)
    };

    // Build the action bar
    let action_bar = build_action_bar(
        volumes_control,
        segment,
        selected_volume,
        selected_volume_node,
    );

    iced_widget::column![
        header_section,
        Space::new(0, 20),
        widget::Row::from_vec(action_bar).spacing(10)
    ]
    .spacing(10)
    .into()
}

/// Aggregate children's used space for LUKS containers
fn aggregate_children_usage(node: &disks_dbus::VolumeNode) -> u64 {
    node.children
        .iter()
        .filter_map(|child| child.usage.as_ref())
        .map(|u| u.used)
        .sum()
}

/// Build info display for a volume node (child filesystem/LV) - mirrors disk header layout
fn build_volume_node_info(v: &disks_dbus::VolumeNode) -> Element<'_, Message> {
    use crate::ui::volumes::usage_pie;
    
    // Pie chart showing usage (left side, replacing icon)
    // For LUKS containers, aggregate children's usage
    let used = if v.kind == VolumeKind::CryptoContainer && !v.children.is_empty() {
        aggregate_children_usage(v)
    } else {
        v.usage.as_ref().map(|u| u.used).unwrap_or(0)
    };
    let pie_chart = usage_pie::usage_pie(used, v.size);

    // Name, filesystem type, mount point (center text column)
    let name_text = widget::text(v.label.clone())
        .size(14.0)
        .font(cosmic::iced::font::Font {
            weight: cosmic::iced::font::Weight::Semibold,
            ..Default::default()
        });

    let contents = if v.id_type.is_empty() {
        match v.kind {
            VolumeKind::Filesystem => fl!("filesystem"),
            VolumeKind::LvmLogicalVolume => "LVM LV".to_string(),
            VolumeKind::LvmPhysicalVolume => "LVM PV".to_string(),
            VolumeKind::CryptoContainer => "LUKS".to_string(),
            VolumeKind::Partition => "Partition".to_string(),
            VolumeKind::Block => "Device".to_string(),
        }
    } else {
        v.id_type.to_uppercase()
    };

    let type_text = widget::text::caption(format!("{}: {}", fl!("contents"), contents));

    let mount_text = if let Some(mount_point) = v.mount_points.first() {
        widget::text::caption(format!("{}: {}", fl!("mounted-at"), mount_point))
    } else {
        widget::text::caption("Not mounted")
    };

    let text_column = iced_widget::column![name_text, type_text, mount_text]
        .spacing(4)
        .width(Length::Fill);

    // Device info box (right side)
    let device_str = match v.device_path.as_ref() {
        Some(s) => s.clone(),
        None => fl!("unresolved"),
    };
    
    let info_box = iced_widget::column![
        widget::text::caption_heading(fl!("device")),
        widget::text::body(device_str),
    ]
    .spacing(4)
    .align_x(Alignment::End)
    .apply(widget::container)
    .padding(10)
    .class(cosmic::style::Container::Card);

    // Row layout: pie_chart | text_column | info_box
    iced_widget::Row::new()
        .push(pie_chart)
        .push(text_column)
        .push(info_box)
        .spacing(15)
        .align_y(Alignment::Center)
        .width(Length::Fill)
        .into()
}

/// Build info display for a partition - mirrors disk header layout
fn build_partition_info(p: &disks_dbus::VolumeModel) -> Element<'_, Message> {
    use crate::ui::volumes::usage_pie;
    
    // Pie chart showing usage (left side)
    let used = p.usage.as_ref().map(|u| u.used).unwrap_or(0);
    let pie_chart = usage_pie::usage_pie(used, p.size);

    // Name, type, mount point (center text column)
    let mut name = p.name.clone();
    if name.is_empty() {
        name = fl!("partition-number", number = p.number);
    } else {
        name = fl!("partition-number-with-name", number = p.number, name = name);
    }

    let name_text = widget::text(name)
        .size(14.0)
        .font(cosmic::iced::font::Font {
            weight: cosmic::iced::font::Weight::Semibold,
            ..Default::default()
        });

    let mut type_str = p.id_type.clone().to_uppercase();
    type_str = format!("{} - {}", type_str, p.partition_type.clone());
    let type_text = widget::text::caption(format!("{}: {}", fl!("contents"), type_str));

    let mount_text = if let Some(mount_point) = p.mount_points.first() {
        widget::text::caption(format!("{}: {}", fl!("mounted-at"), mount_point))
    } else {
        widget::text::caption("Not mounted")
    };

    let text_column = iced_widget::column![name_text, type_text, mount_text]
        .spacing(4)
        .width(Length::Fill);

    // Device info box (right side)
    let device_str = match &p.device_path {
        Some(s) => s.clone(),
        None => fl!("unresolved"),
    };
    
    let info_box = iced_widget::column![
        widget::text::caption_heading(fl!("device")),
        widget::text::body(device_str),
        widget::text::caption(format!("UUID: {}", &p.uuid)),
    ]
    .spacing(4)
    .align_x(Alignment::End)
    .apply(widget::container)
    .padding(10)
    .class(cosmic::style::Container::Card);

    // Row layout: pie_chart | text_column | info_box
    iced_widget::Row::new()
        .push(pie_chart)
        .push(text_column)
        .push(info_box)
        .spacing(15)
        .align_y(Alignment::Center)
        .width(Length::Fill)
        .into()
}

/// Build info display for free space - mirrors disk header layout
fn build_free_space_info(segment: &crate::ui::volumes::Segment) -> Element<'_, Message> {
    // No pie chart for free space, use a placeholder
    let placeholder = widget::container(
        widget::text::caption(fl!("free-space-segment"))
            .center()
    )
    .padding(4)
    .width(Length::Fixed(72.0))
    .height(Length::Fixed(72.0))
    .center_x(Length::Fixed(72.0))
    .center_y(Length::Fixed(72.0))
    .style(move |theme: &cosmic::Theme| {
        cosmic::iced_widget::container::Style {
            background: Some(cosmic::iced::Background::Color(
                theme.cosmic().background.component.base.into(),
            )),
            border: cosmic::iced::Border {
                color: theme.cosmic().background.component.divider.into(),
                width: 2.0,
                radius: 36.0.into(),
            },
            ..Default::default()
        }
    });

    // Name and size (center text column)
    let name_text = widget::text(fl!("free-space-segment"))
        .size(14.0)
        .font(cosmic::iced::font::Font {
            weight: cosmic::iced::font::Weight::Semibold,
            ..Default::default()
        });

    let size_text = widget::text::caption(format!("{}: {}", fl!("size"), bytes_to_pretty(&segment.size, true)));
    let offset_text = widget::text::caption(format!("Offset: {}", bytes_to_pretty(&segment.offset, false)));

    let text_column = iced_widget::column![name_text, size_text, offset_text]
        .spacing(4)
        .width(Length::Fill);

    // Info box (right side)
    let info_box = iced_widget::column![
        widget::text::caption_heading("Available"),
        widget::text::body("Can create partition"),
    ]
    .spacing(4)
    .align_x(Alignment::End)
    .apply(widget::container)
    .padding(10)
    .class(cosmic::style::Container::Card);

    // Row layout: placeholder | text_column | info_box
    iced_widget::Row::new()
        .push(placeholder)
        .push(text_column)
        .push(info_box)
        .spacing(15)
        .align_y(Alignment::Center)
        .width(Length::Fill)
        .into()
}

/// Build the disk-level action button bar (below disk header)
fn build_disk_action_bar(_drive: &DriveModel) -> Vec<Element<'_, Message>> {
    vec![
        action_button(
            "media-eject-symbolic",
            fl!("eject").to_string(),
            Some(Message::Eject),
        ),
        action_button(
            "system-shutdown-symbolic",
            fl!("power-off").to_string(),
            Some(Message::PowerOff),
        ),
        action_button(
            "edit-clear-symbolic",
            fl!("format-disk").to_string(),
            Some(Message::Format),
        ),
        action_button(
            "speedometer-symbolic",
            fl!("smart-data-self-tests").to_string(),
            Some(Message::SmartData),
        ),
        action_button(
            "media-playback-pause-symbolic",
            fl!("standby-now").to_string(),
            Some(Message::StandbyNow),
        ),
        action_button(
            "system-run-symbolic",
            fl!("wake-up-from-standby").to_string(),
            Some(Message::Wakeup),
        ),
        widget::horizontal_space().into(),
        action_button(
            "media-floppy-symbolic",
            fl!("create-disk-from-drive").to_string(),
            Some(Message::CreateDiskFrom),
        ),
        action_button(
            "document-revert-symbolic",
            fl!("restore-image-to-drive").to_string(),
            Some(Message::RestoreImageTo),
        ),
    ]
}

/// Build the action button bar based on the selected segment and volume
fn build_action_bar<'a>(
    volumes_control: &'a VolumesControl,
    segment: &'a crate::ui::volumes::Segment,
    selected_volume: Option<&disks_dbus::VolumeNode>,
    selected_child_volume: Option<&disks_dbus::VolumeNode>,
) -> Vec<Element<'a, Message>> {
    let mut action_bar: Vec<Element<Message>> = vec![];

    match segment.kind {
        DiskSegmentKind::Partition => {
            if let Some(p) = segment.volume.as_ref() {
                // Container actions (unlock/lock)
                if let Some(v) = selected_volume
                    && v.kind == VolumeKind::CryptoContainer
                {
                    if v.locked {
                        action_bar.push(action_button(
                            "dialog-password-symbolic",
                            fl!("unlock-button").to_string(),
                            Some(Message::Dialog(Box::new(ShowDialog::UnlockEncrypted(
                                UnlockEncryptedDialog {
                                    partition_path: p.path.to_string(),
                                    partition_name: p.name(),
                                    passphrase: String::new(),
                                    error: None,
                                    running: false,
                                },
                            )))),
                        ));
                    } else {
                        action_bar.push(action_button(
                            "changes-prevent-symbolic",
                            fl!("lock").to_string(),
                            Some(VolumesControlMessage::LockContainer.into()),
                        ));
                    }
                }

                // Mount/Unmount actions
                if let Some(v) = selected_child_volume {
                    if v.can_mount() {
                        let msg = if v.is_mounted() {
                            VolumesControlMessage::ChildUnmount(v.object_path.to_string())
                        } else {
                            VolumesControlMessage::ChildMount(v.object_path.to_string())
                        };
                        let icon_name = if v.is_mounted() {
                            "media-playback-stop-symbolic"
                        } else {
                            "media-playback-start-symbolic"
                        };
                        action_bar.push(action_button(
                            icon_name,
                            fl!("mount-toggle").to_string(),
                            Some(msg.into()),
                        ));
                    }
                } else if p.can_mount() {
                    let (icon_name, msg) = if p.is_mounted() {
                        (
                            "media-playback-stop-symbolic",
                            VolumesControlMessage::Unmount,
                        )
                    } else {
                        (
                            "media-playback-start-symbolic",
                            VolumesControlMessage::Mount,
                        )
                    };
                    action_bar.push(action_button(
                        icon_name,
                        fl!("mount-toggle").to_string(),
                        Some(msg.into()),
                    ));
                }

                // Format Partition
                action_bar.push(action_button(
                    "edit-clear-symbolic",
                    fl!("format-partition").to_string(),
                    Some(VolumesControlMessage::OpenFormatPartition.into()),
                ));

                // Partition-only: Edit Partition + Resize
                if selected_child_volume.is_none()
                    && p.volume_type == disks_dbus::VolumeType::Partition
                {
                    action_bar.push(action_button(
                        "document-edit-symbolic",
                        fl!("edit-partition").to_string(),
                        Some(VolumesControlMessage::OpenEditPartition.into()),
                    ));

                    let right_free_bytes = volumes_control
                        .segments
                        .get(volumes_control.selected_segment.saturating_add(1))
                        .filter(|s| s.kind == DiskSegmentKind::FreeSpace)
                        .map(|s| s.size)
                        .unwrap_or(0);
                    let max_size = p.size.saturating_add(right_free_bytes);
                    let min_size = p.usage.as_ref().map(|u| u.used).unwrap_or(0).min(max_size);

                    let resize_enabled = max_size.saturating_sub(min_size) >= 1024;
                    action_bar.push(action_button(
                        "transform-scale-symbolic",
                        fl!("resize-partition").to_string(),
                        resize_enabled.then_some(VolumesControlMessage::OpenResizePartition.into()),
                    ));
                }

                // Filesystem actions
                let fs_target_available = selected_child_volume
                    .map(|n| n.can_mount())
                    .unwrap_or_else(|| p.can_mount());
                if fs_target_available {
                    action_bar.push(action_button(
                        "document-properties-symbolic",
                        fl!("edit-mount-options").to_string(),
                        Some(VolumesControlMessage::OpenEditMountOptions.into()),
                    ));
                    action_bar.push(action_button(
                        "tag-symbolic",
                        fl!("edit-filesystem").to_string(),
                        Some(VolumesControlMessage::OpenEditFilesystemLabel.into()),
                    ));
                    action_bar.push(action_button(
                        "emblem-ok-symbolic",
                        fl!("check-filesystem").to_string(),
                        Some(VolumesControlMessage::OpenCheckFilesystem.into()),
                    ));
                    action_bar.push(action_button(
                        "tools-symbolic",
                        fl!("repair-filesystem").to_string(),
                        Some(VolumesControlMessage::OpenRepairFilesystem.into()),
                    ));
                    action_bar.push(action_button(
                        "user-home-symbolic",
                        fl!("take-ownership").to_string(),
                        Some(VolumesControlMessage::OpenTakeOwnership.into()),
                    ));
                }

                // Container encryption options
                if selected_volume.is_some_and(|v| v.kind == VolumeKind::CryptoContainer) {
                    action_bar.push(action_button(
                        "dialog-password-symbolic",
                        fl!("change-passphrase").to_string(),
                        Some(VolumesControlMessage::OpenChangePassphrase.into()),
                    ));
                    action_bar.push(action_button(
                        "document-properties-symbolic",
                        fl!("edit-encryption-options").to_string(),
                        Some(VolumesControlMessage::OpenEditEncryptionOptions.into()),
                    ));
                }

                // Partition image operations
                action_bar.push(widget::horizontal_space().into());
                action_bar.push(action_button(
                    "media-floppy-symbolic",
                    fl!("create-disk-from-partition").to_string(),
                    Some(Message::CreateDiskFromPartition),
                ));
                action_bar.push(action_button(
                    "document-revert-symbolic",
                    fl!("restore-image-to-partition").to_string(),
                    Some(Message::RestoreImageToPartition),
                ));

                // Delete partition
                if selected_child_volume.is_none()
                    && p.volume_type != disks_dbus::VolumeType::Filesystem
                {
                    action_bar.push(widget::horizontal_space().into());
                    action_bar.push(action_button(
                        "edit-delete-symbolic",
                        fl!("delete", name = segment.name.clone()).to_string(),
                        Some(Message::Dialog(Box::new(ShowDialog::DeletePartition(
                            DeletePartitionDialog {
                                name: segment.name.clone(),
                                running: false,
                            },
                        )))),
                    ));
                }
            }
        }
        DiskSegmentKind::FreeSpace => {
            action_bar.push(action_button(
                "list-add-symbolic",
                fl!("create-partition").to_string(),
                Some(Message::Dialog(Box::new(ShowDialog::AddPartition(
                    CreatePartitionDialog {
                        info: segment.get_create_info(),
                        running: false,
                        error: None,
                    },
                )))),
            ));
        }
        DiskSegmentKind::Reserved => {}
    }

    action_bar
}

/// Helper function to create an action button with icon above text label
fn action_button(
    icon_name: &str,
    label: String,
    msg: Option<Message>,
) -> Element<'_, Message> {
    let content = iced_widget::column![
        icon::from_name(icon_name).size(24),
        widget::text::caption(label)
            .center()
            .width(Length::Fixed(64.0))
    ]
    .spacing(4)
    .align_x(Alignment::Center)
    .width(Length::Fixed(64.0));

    let mut button = widget::button::custom(content)
        .padding(8);
    
    if let Some(m) = msg {
        button = button.on_press(m);
    }

    button.into()
}
