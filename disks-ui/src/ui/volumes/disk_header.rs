use cosmic::iced::{Alignment, Length};
use cosmic::widget::{self, icon};
use cosmic::{Apply, Element, iced_widget};
use disks_dbus::{DriveModel, bytes_to_pretty};

use crate::app::Message;
use crate::fl;

/// Renders the disk info header with icon, name/partitioning/serial, and used/total box.
pub fn disk_header(drive: &DriveModel, used: u64) -> Element<'_, Message> {
    let partition_type = match &drive.partition_table_type {
        Some(t) => t.clone().to_uppercase(),
        None => fl!("unknown"),
    };

    // Large icon for the drive
    let drive_icon = icon::from_name(if drive.is_loop {
        "media-optical-symbolic"
    } else if drive.removable {
        "drive-removable-media-symbolic"
    } else {
        "drive-harddisk-symbolic"
    })
    .size(64)
    .apply(widget::container)
    .padding(10);

    // Name, partitioning, serial (left-aligned text column)
    let name_text = widget::text(drive.name())
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

    // Used / Total box (right-aligned)
    let used_str = bytes_to_pretty(&used, false);
    let total_str = bytes_to_pretty(&drive.size, false);

    let size_box = iced_widget::column![
        widget::text::caption_heading(fl!("disk-usage")),
        widget::text::body(format!("{} / {}", used_str, total_str))
    ]
    .spacing(4)
    .align_x(Alignment::End)
    .apply(widget::container)
    .padding(10)
    .class(cosmic::style::Container::Card);

    // Row layout: icon | text_column | size_box
    iced_widget::Row::new()
        .push(drive_icon)
        .push(text_column)
        .push(size_box)
        .spacing(15)
        .align_y(Alignment::Center)
        .width(Length::Fill)
        .into()
}
