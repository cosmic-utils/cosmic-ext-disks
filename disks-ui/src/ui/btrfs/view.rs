use cosmic::widget;
use cosmic::{Element, iced_widget};

use super::BtrfsState;
use crate::fl;
use crate::ui::app::message::Message;

/// Builds the BTRFS management section for a BTRFS volume
pub fn btrfs_management_section<'a>(
    _volume: &'a disks_dbus::VolumeModel,
    state: &'a BtrfsState,
) -> Element<'a, Message> {
    let header = widget::text(fl!("btrfs-management"))
        .size(13.0)
        .font(cosmic::iced::font::Font {
            weight: cosmic::iced::font::Weight::Semibold,
            ..Default::default()
        });

    if state.expanded {
        iced_widget::column![
            header,
            widget::text(fl!("btrfs-placeholder")).size(11.0)
        ]
        .spacing(8)
        .padding(8)
        .into()
    } else {
        iced_widget::column![header]
            .spacing(8)
            .padding(8)
            .into()
    }
}
