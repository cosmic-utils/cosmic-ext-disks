use cosmic::iced::{Alignment, Length};
use cosmic::widget::{self, icon};
use cosmic::{Element, iced_widget};
use disks_dbus::{DriveModel, VolumeNode};

use crate::app::Message;
use crate::fl;
use crate::ui::volumes::Segment;
use crate::ui::volumes::usage_pie::{self, PieSegmentData};
use crate::ui::volumes::helpers;
use crate::utils::DiskSegmentKind;

/// Renders the disk info header with icon, name/partitioning/serial, and multi-partition pie chart.
pub fn disk_header<'a>(
    drive: &'a DriveModel,
    used: u64,
    segments: &'a [Segment],
    volumes: &'a [VolumeNode],
) -> Element<'a, Message> {
    let partition_type = match &drive.partition_table_type {
        Some(t) => t.to_uppercase(),
        None => fl!("unknown"),
    };

    // Title: Vendor + Model (left-aligned text column)
    let title = if drive.vendor.is_empty() && drive.model.is_empty() {
        drive.name()
    } else if drive.vendor.is_empty() {
        drive.model.to_string()
    } else if drive.model.is_empty() {
        drive.vendor.to_string()
    } else {
        format!("{} {}", drive.vendor, drive.model)
    };
    
    let name_text = widget::text(title)
        .size(14.0)
        .font(cosmic::iced::font::Font {
            weight: cosmic::iced::font::Weight::Semibold,
            ..Default::default()
        });

    let partitioning_text =
        widget::text::caption(format!("{}: {}", fl!("partitioning"), partition_type));

    let serial_text = if drive.is_loop {
        widget::text::caption(format!(
            "{}: {}",
            fl!("backing-file"),
            drive.backing_file.as_deref().unwrap_or("")
        ))
    } else {
        widget::text::caption(format!("{}: {}", fl!("serial"), &drive.serial))
    };

    let text_column = iced_widget::column![name_text, partitioning_text, serial_text]
        .spacing(4)
        .width(Length::Fill);

    // Drive action buttons underneath icon and text (left-aligned, spanning both columns)
    let mut drive_actions = Vec::new();
    
    // Eject (for removable drives)
    if drive.removable {
        drive_actions.push(
            widget::tooltip(
                widget::button::icon(icon::from_name("media-eject-symbolic"))
                    .on_press(Message::Eject),
                widget::text(fl!("eject")),
                widget::tooltip::Position::Bottom,
            )
            .into(),
        );
    }
    
    // Power Off (only if supported)
    if drive.can_power_off {
        drive_actions.push(
            widget::tooltip(
                widget::button::icon(icon::from_name("system-shutdown-symbolic"))
                    .on_press(Message::PowerOff),
                widget::text(fl!("power-off")),
                widget::tooltip::Position::Bottom,
            )
            .into(),
        );
    }
    
    // Format (wipe disk)
    drive_actions.push(
        widget::tooltip(
            widget::button::icon(icon::from_name("edit-clear-all-symbolic"))
                .on_press(Message::Format),
            widget::text(fl!("format-disk")),
            widget::tooltip::Position::Bottom,
        )
        .into(),
    );
    
    // SMART Data (not for loop devices)
    if !drive.is_loop {
        drive_actions.push(
            widget::tooltip(
                widget::button::icon(icon::from_name("emblem-system-symbolic"))
                    .on_press(Message::SmartData),
                widget::text(fl!("smart-data-self-tests")),
                widget::tooltip::Position::Bottom,
            )
            .into(),
        );
    }
    
    // Standby (only for drives that support power management)
    if !drive.is_loop && drive.can_power_off {
        drive_actions.push(
            widget::tooltip(
                widget::button::icon(icon::from_name("media-playback-pause-symbolic"))
                    .on_press(Message::StandbyNow),
                widget::text(fl!("standby-now")),
                widget::tooltip::Position::Bottom,
            )
            .into(),
        );
    }
    
    // Wake Up (only for drives that support power management)
    if !drive.is_loop && drive.can_power_off {
        drive_actions.push(
            widget::tooltip(
                widget::button::icon(icon::from_name("alarm-symbolic"))
                    .on_press(Message::Wakeup),
                widget::text(fl!("wake-up-from-standby")),
                widget::tooltip::Position::Bottom,
            )
            .into(),
        );
    }
    
    // Create Image
    drive_actions.push(
        widget::tooltip(
            widget::button::icon(icon::from_name("document-save-as-symbolic"))
                .on_press(Message::NewDiskImage),
            widget::text(fl!("create-image")),
            widget::tooltip::Position::Bottom,
        )
        .into(),
    );
    
    // Restore Image
    drive_actions.push(
        widget::tooltip(
            widget::button::icon(icon::from_name("document-revert-symbolic"))
                .on_press(Message::AttachDisk),
            widget::text(fl!("restore-image")),
            widget::tooltip::Position::Bottom,
        )
        .into(),
    );

    // Multi-partition pie chart (right-aligned)
    let pie_segments: Vec<PieSegmentData> = segments
        .iter()
        .filter(|s| s.kind == DiskSegmentKind::Partition)
        .map(|s| {
            let used = if let Some(ref volume_model) = s.volume {
                // Look up the corresponding VolumeNode to check if it's a LUKS container
                if let Some(volume_node) = helpers::find_volume_node_for_partition(volumes, volume_model) {
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
            } else {
                0
            };

            PieSegmentData {
                name: s.label.clone(),
                used,
            }
        })
        .collect();
    let pie_chart = usage_pie::disk_usage_pie(&pie_segments, drive.size, used, true);

    // Layout: 
    // Row 1: text_column | pie_chart
    // Row 2: action_buttons | (empty space under pie)
    let top_row = text_column;
    
    let action_row = widget::Row::from_vec(drive_actions)
        .spacing(4)
        .align_y(Alignment::Center);
    
    let left_column = iced_widget::column![top_row, action_row]
        .spacing(8)
        .width(Length::Fill);
    
    // Main row: left_column | pie_chart
    iced_widget::Row::new()
        .push(left_column)
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
