use super::message::Message;
use super::state::{AppModel, ContextPage};
use crate::fl;
use crate::ui::dialogs::state::{
    DeletePartitionDialog, ShowDialog,
};
use crate::ui::dialogs::view as dialogs;
use crate::ui::sidebar;
use crate::ui::volumes::{VolumesControl, VolumesControlMessage, disk_header};
use crate::utils::DiskSegmentKind;
use crate::views::about::about;
use cosmic::app::context_drawer as cosmic_context_drawer;
use cosmic::iced::Length;
use cosmic::iced::alignment::{Alignment, Horizontal, Vertical};
use cosmic::widget::{self, Space, icon};
use cosmic::{Apply, Element, iced_widget};
use disks_dbus::bytes_to_pretty;
use disks_dbus::{DriveModel, VolumeKind};

/// Elements to pack at the start of the header bar.
pub(crate) fn header_start(_app: &AppModel) -> Vec<Element<'_, Message>> {
    vec![
        widget::button::icon(icon::from_name("help-about-symbolic"))
            .on_press(Message::ToggleContextPage(ContextPage::About))
            .into(),
    ]
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
                    &state.title,
                    &state.body,
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

            crate::ui::dialogs::state::ShowDialog::Info { title, body } => {
                Some(dialogs::info(title, body, Message::CloseDialog))
            }
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
        // Both width and height must be Shrink for flex layout to respect the max_width constraint
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
            // For LUKS containers, aggregate children's usage instead of container's (which is 0)
            let used: u64 = volumes_control
                .segments
                .iter()
                .filter_map(|s| s.volume.as_ref())
                .map(|volume_model| {
                    // Look up the corresponding VolumeNode to check if it's a LUKS container
                    if let Some(volume_node) = crate::ui::volumes::helpers::find_volume_node_for_partition(
                        &volumes_control.model.volumes,
                        volume_model,
                    ) {
                        if volume_node.kind == disks_dbus::VolumeKind::CryptoContainer && !volume_node.children.is_empty() {
                            // Aggregate children's usage for LUKS containers
                            volume_node.children
                                .iter()
                                .filter_map(|child| child.usage.as_ref())
                                .map(|u| u.used)
                                .sum()
                        } else {
                            // Use volume's own usage
                            volume_model.usage.as_ref().map(|u| u.used).unwrap_or(0)
                        }
                    } else {
                        // Fallback to volume model's usage
                        volume_model.usage.as_ref().map(|u| u.used).unwrap_or(0)
                    }
                })
                .sum();

            // Top section: Disk header + volumes control
            let top_section = iced_widget::column![
                disk_header::disk_header(drive, used, &volumes_control.segments, &volumes_control.model.volumes),
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
        build_volume_node_info(v, volumes_control, segment, selected_volume)
    } else if let Some(ref p) = segment.volume {
        build_partition_info(p, selected_volume, volumes_control, segment)
    } else {
        build_free_space_info(segment)
    };

    header_section
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
fn build_volume_node_info<'a>(
    v: &'a disks_dbus::VolumeNode,
    _volumes_control: &'a VolumesControl,
    _segment: &'a crate::ui::volumes::Segment,
    _selected_volume: Option<&'a disks_dbus::VolumeNode>,
) -> Element<'a, Message> {
    use crate::ui::volumes::usage_pie;

    // Pie chart showing usage (right side, matching disk header layout)
    // For LUKS containers, aggregate children's usage
    let used = if v.kind == VolumeKind::CryptoContainer {
        if !v.children.is_empty() {
            aggregate_children_usage(v)
        } else {
            // Unlocked LUKS with no children or locked LUKS - show 0
            0
        }
    } else {
        v.usage.as_ref().map(|u| u.used).unwrap_or(0)
    };
    
    // Create a single-segment pie for this volume
    let pie_segment = usage_pie::PieSegmentData {
        name: v.label.clone(),
        used,
    };
    let pie_chart = usage_pie::disk_usage_pie(&[pie_segment], v.size, used, false);

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

    let device_str = match v.device_path.as_ref() {
        Some(s) => s.clone(),
        None => fl!("unresolved"),
    };
    let device_text = widget::text::caption(format!("{}: {}", fl!("device"), device_str));

    // Only show mount info if it's not a LUKS container (containers don't mount, their children do)
    let text_column = if v.kind == VolumeKind::CryptoContainer {
        iced_widget::column![name_text, type_text, device_text]
            .spacing(4)
            .width(Length::Fill)
    } else {
        let mount_text: Element<Message> = if let Some(mount_point) = v.mount_points.first() {
            iced_widget::row![
                widget::text::caption(format!("{}: ", fl!("mounted-at"))),
                cosmic::widget::button::link(mount_point.clone())
                    .padding(0)
                    .on_press(Message::OpenPath(mount_point.clone()))
            ]
            .align_y(Alignment::Center)
            .into()
        } else {
            widget::text::caption("Not mounted").into()
        };

        iced_widget::column![name_text, type_text, device_text, mount_text]
            .spacing(4)
            .width(Length::Fill)
    };

    // Action buttons underneath
    let mut action_buttons = Vec::new();
    
    // Mount/Unmount
    if v.can_mount() {
        if v.is_mounted() {
            action_buttons.push(
                widget::tooltip(
                    widget::button::icon(icon::from_name("media-playback-stop-symbolic"))
                        .on_press(Message::VolumesMessage(VolumesControlMessage::ChildUnmount(v.object_path.to_string()))),
                    widget::text(fl!("unmount")),
                    widget::tooltip::Position::Bottom,
                )
                .into(),
            );
        } else {
            action_buttons.push(
                widget::tooltip(
                    widget::button::icon(icon::from_name("media-playback-start-symbolic"))
                        .on_press(Message::VolumesMessage(VolumesControlMessage::ChildMount(v.object_path.to_string()))),
                    widget::text(fl!("mount")),
                    widget::tooltip::Position::Bottom,
                )
                .into(),
            );
        }
    }

    let info_and_actions = iced_widget::column![
        text_column,
        widget::Row::from_vec(action_buttons).spacing(4)
    ]
    .spacing(8);

    // Row layout: info_and_actions | pie_chart (aligned right, shrink to fit)
    iced_widget::Row::new()
        .push(info_and_actions)
        .push(
            widget::container(pie_chart)
                .width(Length::Shrink)
                .align_x(cosmic::iced::alignment::Horizontal::Right)
        )
        .spacing(15)
        .align_y(Alignment::Start)
        .width(Length::Fill)
        .into()
}

/// Build info display for a partition - mirrors disk header layout
fn build_partition_info<'a>(
    p: &'a disks_dbus::VolumeModel,
    volume_node: Option<&'a disks_dbus::VolumeNode>,
    volumes_control: &'a VolumesControl,
    segment: &'a crate::ui::volumes::Segment,
) -> Element<'a, Message> {
    use crate::ui::volumes::usage_pie;

    // Pie chart showing usage (right side, matching disk header layout)
    // For LUKS containers, aggregate children's usage
    let used = if let Some(v) = volume_node {
        if v.kind == VolumeKind::CryptoContainer && !v.children.is_empty() {
            aggregate_children_usage(v)
        } else {
            p.usage.as_ref().map(|u| u.used).unwrap_or(0)
        }
    } else {
        p.usage.as_ref().map(|u| u.used).unwrap_or(0)
    };
    
    // Create a single-segment pie for this partition
    let partition_name = if p.name.is_empty() {
        fl!("partition-number", number = p.number)
    } else {
        fl!("partition-number-with-name", number = p.number, name = p.name.clone())
    };
    let pie_segment = usage_pie::PieSegmentData {
        name: partition_name.clone(),
        used,
    };
    let pie_chart = usage_pie::disk_usage_pie(&[pie_segment], p.size, used, false);

    // Name, type, mount point (center text column)
    let name_text = widget::text(partition_name.clone())
        .size(14.0)
        .font(cosmic::iced::font::Font {
            weight: cosmic::iced::font::Weight::Semibold,
            ..Default::default()
        });

    let mut type_str = p.id_type.clone().to_uppercase();
    type_str = format!("{} - {}", type_str, p.partition_type.clone());
    let type_text = widget::text::caption(format!("{}: {}", fl!("contents"), type_str));

    let device_str = match &p.device_path {
        Some(s) => s.clone(),
        None => fl!("unresolved"),
    };
    let device_text = widget::text::caption(format!("{}: {}", fl!("device"), device_str));

    let uuid_text = widget::text::caption(format!("UUID: {}", &p.uuid));

    // Only show mount info if it's not a LUKS container (containers don't mount, their children do)
    let text_column = if let Some(v) = volume_node {
        if v.kind == VolumeKind::CryptoContainer {
            iced_widget::column![name_text, type_text, device_text, uuid_text]
                .spacing(4)
                .width(Length::Fill)
        } else {
            let mount_text: Element<Message> = if let Some(mount_point) = p.mount_points.first() {
                iced_widget::row![
                    widget::text::caption(format!("{}: ", fl!("mounted-at"))),
                    cosmic::widget::button::link(mount_point.clone())
                        .padding(0)
                        .on_press(Message::OpenPath(mount_point.clone()))
                ]
                .align_y(Alignment::Center)
                .into()
            } else {
                widget::text::caption("Not mounted").into()
            };

            iced_widget::column![name_text, type_text, device_text, uuid_text, mount_text]
                .spacing(4)
                .width(Length::Fill)
        }
    } else {
        let mount_text: Element<Message> = if let Some(mount_point) = p.mount_points.first() {
            iced_widget::row![
                widget::text::caption(format!("{}: ", fl!("mounted-at"))),
                cosmic::widget::button::link(mount_point.clone())
                    .padding(0)
                    .on_press(Message::OpenPath(mount_point.clone()))
            ]
            .align_y(Alignment::Center)
            .into()
        } else {
            widget::text::caption("Not mounted").into()
        };

        iced_widget::column![name_text, type_text, device_text, uuid_text, mount_text]
            .spacing(4)
            .width(Length::Fill)
    };

    // Action buttons underneath
    let mut action_buttons = Vec::new();
    
    // Lock/Unlock for LUKS containers
    if let Some(v) = volume_node {
        if v.kind == VolumeKind::CryptoContainer {
            if v.locked {
                action_buttons.push(
                    widget::tooltip(
                        widget::button::icon(icon::from_name("changes-allow-symbolic"))
                            .on_press(Message::Dialog(Box::new(ShowDialog::UnlockEncrypted(
                                crate::ui::dialogs::state::UnlockEncryptedDialog {
                                    partition_path: p.path.to_string(),
                                    partition_name: partition_name.clone(),
                                    passphrase: String::new(),
                                    error: None,
                                    running: false,
                                },
                            )))),
                        widget::text(fl!("unlock-button")),
                        widget::tooltip::Position::Bottom,
                    )
                    .into(),
                );
            } else {
                action_buttons.push(
                    widget::tooltip(
                        widget::button::icon(icon::from_name("changes-prevent-symbolic"))
                            .on_press(Message::VolumesMessage(VolumesControlMessage::LockContainer)),
                        widget::text(fl!("lock")),
                        widget::tooltip::Position::Bottom,
                    )
                    .into(),
                );
            }
            
            // Change Passphrase (only for unlocked containers)
            if !v.locked {
                action_buttons.push(
                    widget::tooltip(
                        widget::button::icon(icon::from_name("document-properties-symbolic"))
                            .on_press(Message::VolumesMessage(VolumesControlMessage::OpenChangePassphrase)),
                        widget::text(fl!("change-passphrase")),
                        widget::tooltip::Position::Bottom,
                    )
                    .into(),
                );
            }
            
            // Edit Encryption Options
            action_buttons.push(
                widget::tooltip(
                    widget::button::icon(icon::from_name("preferences-system-symbolic"))
                        .on_press(Message::VolumesMessage(VolumesControlMessage::OpenEditEncryptionOptions)),
                    widget::text(fl!("edit-encryption-options")),
                    widget::tooltip::Position::Bottom,
                )
                .into(),
            );
        }
    }
    
    // Mount/Unmount
    if p.can_mount() {
        if p.is_mounted() {
            action_buttons.push(
                widget::tooltip(
                    widget::button::icon(icon::from_name("media-playback-stop-symbolic"))
                        .on_press(Message::VolumesMessage(VolumesControlMessage::Unmount)),
                    widget::text(fl!("unmount")),
                    widget::tooltip::Position::Bottom,
                )
                .into(),
            );
        } else {
            action_buttons.push(
                widget::tooltip(
                    widget::button::icon(icon::from_name("media-playback-start-symbolic"))
                        .on_press(Message::VolumesMessage(VolumesControlMessage::Mount)),
                    widget::text(fl!("mount")),
                    widget::tooltip::Position::Bottom,
                )
                .into(),
            );
        }
    }

    // Format
    action_buttons.push(
        widget::tooltip(
            widget::button::icon(icon::from_name("edit-clear-symbolic"))
                .on_press(Message::VolumesMessage(VolumesControlMessage::OpenFormatPartition)),
            widget::text(fl!("format")),
            widget::tooltip::Position::Bottom,
        )
        .into(),
    );

    // Edit and Resize (only for partitions)
    if p.volume_type == disks_dbus::VolumeType::Partition {
        action_buttons.push(
            widget::tooltip(
                widget::button::icon(icon::from_name("document-edit-symbolic"))
                    .on_press(Message::VolumesMessage(VolumesControlMessage::OpenEditPartition)),
                widget::text(fl!("edit")),
                widget::tooltip::Position::Bottom,
            )
            .into(),
        );

        // Resize (check if there's space)
        let right_free_bytes = volumes_control
            .segments
            .get(volumes_control.selected_segment.saturating_add(1))
            .filter(|s| s.kind == DiskSegmentKind::FreeSpace)
            .map(|s| s.size)
            .unwrap_or(0);
        let max_size = p.size.saturating_add(right_free_bytes);
        let min_size = p.usage.as_ref().map(|u| u.used).unwrap_or(0).min(max_size);
        let resize_enabled = max_size.saturating_sub(min_size) >= 1024;

        if resize_enabled {
            action_buttons.push(
                widget::tooltip(
                    widget::button::icon(icon::from_name("transform-scale-symbolic"))
                        .on_press(Message::VolumesMessage(VolumesControlMessage::OpenResizePartition)),
                    widget::text(fl!("resize")),
                    widget::tooltip::Position::Bottom,
                )
                .into(),
            );
        }
        
        // Label
        action_buttons.push(
            widget::tooltip(
                widget::button::icon(icon::from_name("tag-symbolic"))
                    .on_press(Message::VolumesMessage(VolumesControlMessage::OpenEditFilesystemLabel)),
                widget::text(fl!("label")),
                widget::tooltip::Position::Bottom,
            )
            .into(),
        );
    }
    
    // Check Filesystem (if mounted)
    if p.can_mount() && p.is_mounted() {
        action_buttons.push(
            widget::tooltip(
                widget::button::icon(icon::from_name("dialog-question-symbolic"))
                    .on_press(Message::VolumesMessage(VolumesControlMessage::OpenCheckFilesystem)),
                widget::text(fl!("check-filesystem")),
                widget::tooltip::Position::Bottom,
            )
            .into(),
        );
    }
    
    // Repair Filesystem (if filesystem type)
    if p.has_filesystem {
        action_buttons.push(
            widget::tooltip(
                widget::button::icon(icon::from_name("emblem-system-symbolic"))
                    .on_press(Message::VolumesMessage(VolumesControlMessage::OpenRepairFilesystem)),
                widget::text(fl!("repair")),
                widget::tooltip::Position::Bottom,
            )
            .into(),
        );
    }
    
    // Take Ownership (if mounted)
    if p.can_mount() && p.is_mounted() {
        action_buttons.push(
            widget::tooltip(
                widget::button::icon(icon::from_name("system-users-symbolic"))
                    .on_press(Message::VolumesMessage(VolumesControlMessage::OpenTakeOwnership)),
                widget::text(fl!("take-ownership")),
                widget::tooltip::Position::Bottom,
            )
            .into(),
        );
    }
    
    // Edit Mount Options (if filesystem)
    if p.has_filesystem {
        action_buttons.push(
            widget::tooltip(
                widget::button::icon(icon::from_name("emblem-documents-symbolic"))
                    .on_press(Message::VolumesMessage(VolumesControlMessage::OpenEditMountOptions)),
                widget::text(fl!("edit-mount-options")),
                widget::tooltip::Position::Bottom,
            )
            .into(),
        );
    }

    // Delete (only for actual partitions, not filesystems)
    if p.volume_type != disks_dbus::VolumeType::Filesystem {
        action_buttons.push(
            widget::tooltip(
                widget::button::icon(icon::from_name("edit-delete-symbolic"))
                    .on_press(Message::Dialog(Box::new(ShowDialog::DeletePartition(
                        DeletePartitionDialog {
                            name: segment.name.clone(),
                            running: false,
                        },
                    )))),
                widget::text(fl!("delete-partition")),
                widget::tooltip::Position::Bottom,
            )
            .into(),
        );
    }

    let info_and_actions = iced_widget::column![
        text_column,
        widget::Row::from_vec(action_buttons).spacing(4)
    ]
    .spacing(8);

    // Row layout: info_and_actions | pie_chart (aligned right, shrink to fit)
    iced_widget::Row::new()
        .push(info_and_actions)
        .push(
            widget::container(pie_chart)
                .width(Length::Shrink)
                .align_x(cosmic::iced::alignment::Horizontal::Right)
        )
        .spacing(15)
        .align_y(Alignment::Start)
        .width(Length::Fill)
        .into()
}

/// Build info display for free space - mirrors disk header layout
fn build_free_space_info(segment: &crate::ui::volumes::Segment) -> Element<'_, Message> {
    use crate::ui::volumes::usage_pie;
    
    // Empty pie chart for free space (0% used)
    let pie_segment = usage_pie::PieSegmentData {
        name: fl!("free-space-segment"),
        used: 0,
    };
    let pie_chart = usage_pie::disk_usage_pie(&[pie_segment], segment.size, 0, false);

    // Name and size (left text column)
    let name_text =
        widget::text(fl!("free-space-segment"))
            .size(14.0)
            .font(cosmic::iced::font::Font {
                weight: cosmic::iced::font::Weight::Semibold,
                ..Default::default()
            });

    let size_text = widget::text::caption(format!(
        "{}: {}",
        fl!("size"),
        bytes_to_pretty(&segment.size, true)
    ));
    let offset_text = widget::text::caption(format!(
        "Offset: {}",
        bytes_to_pretty(&segment.offset, false)
    ));
    
    let available_text = widget::text::caption("Can create partition");

    let text_column = iced_widget::column![name_text, size_text, offset_text, available_text]
        .spacing(4)
        .width(Length::Fill);

    // Row layout: text_column | pie_chart (aligned right, shrink to fit)
    iced_widget::Row::new()
        .push(text_column)
        .push(
            widget::container(pie_chart)
                .width(Length::Shrink)
                .align_x(cosmic::iced::alignment::Horizontal::Right)
        )
        .spacing(15)
        .align_y(Alignment::Start)
        .width(Length::Fill)
        .into()
}

