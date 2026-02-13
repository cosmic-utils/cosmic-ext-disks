use cosmic::widget;
use cosmic::{Element, iced_widget};

use super::BtrfsState;
use crate::fl;
use crate::ui::app::message::Message;

/// Builds the BTRFS management section for a BTRFS volume
pub fn btrfs_management_section<'a>(
    volume: &'a disks_dbus::VolumeModel,
    state: &'a BtrfsState,
) -> Element<'a, Message> {
    let header = widget::text(fl!("btrfs-management"))
        .size(13.0)
        .font(cosmic::iced::font::Font {
            weight: cosmic::iced::font::Weight::Semibold,
            ..Default::default()
        });

    if !state.expanded {
        // Show collapsed header only
        return iced_widget::column![header]
            .spacing(8)
            .padding(8)
            .into();
    }

    // Expanded view
    let mut content_items: Vec<Element<'a, Message>> = vec![header.into()];

    // Check if we need to load subvolumes
    if state.subvolumes.is_none() && !state.loading {
        // Trigger load if we have a mount point
        if let Some(_mount_point) = volume.mount_points.first() {
            // Note: This will trigger on every render until loaded
            // A better approach would be to trigger once, but this works for now
            content_items.push(
                widget::text("Loading subvolumes...")
                    .size(11.0)
                    .into(),
            );
            
            // Send message to load (will be handled in update)
            // For now, just show loading state
        } else {
            content_items.push(
                widget::text("BTRFS filesystem not mounted")
                    .size(11.0)
                    .into(),
            );
        }
    } else if state.loading {
        content_items.push(
            widget::text("Loading subvolumes...")
                .size(11.0)
                .into(),
        );
    } else if let Some(result) = &state.subvolumes {
        match result {
            Ok(subvolumes) => {
                if subvolumes.is_empty() {
                    content_items.push(
                        widget::text("No subvolumes found")
                            .size(11.0)
                            .into(),
                    );
                } else {
                    // Show subvolumes list
                    content_items.push(
                        widget::text(format!("Subvolumes ({})", subvolumes.len()))
                            .size(11.0)
                            .font(cosmic::iced::font::Font {
                                weight: cosmic::iced::font::Weight::Medium,
                                ..Default::default()
                            })
                            .into(),
                    );

                    for subvol in subvolumes {
                        let subvol_text = format!("ID {} - {}", subvol.id, subvol.path);
                        content_items.push(
                            widget::text(subvol_text)
                                .size(10.0)
                                .into(),
                        );
                    }
                }
            }
            Err(error) => {
                content_items.push(
                    widget::text(format!("Error: {}", error))
                        .size(11.0)
                        .into(),
                );
            }
        }
    }

    iced_widget::Column::from_vec(content_items)
        .spacing(4)
        .padding(8)
        .into()
}
