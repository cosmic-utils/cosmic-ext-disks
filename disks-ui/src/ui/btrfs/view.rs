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

    // === Usage Breakdown Section ===
    if let Some(_mount_point) = volume.mount_points.first() {
        // Check if we need to load usage
        if state.usage_info.is_none() && !state.loading_usage {
            // Trigger load on next render cycle (will be caught by update handler)
            // Just note that we need to load - the actual message is sent elsewhere
        }

        if state.loading_usage {
            content_items.push(
                widget::text("Loading usage information...")
                    .size(11.0)
                    .into(),
            );
        } else if let Some(usage_result) = &state.usage_info {
            match usage_result {
                Ok(usage) => {
                    // Add usage breakdown header
                    content_items.push(
                        widget::text(fl!("btrfs-usage"))
                            .size(11.0)
                            .font(cosmic::iced::font::Font {
                                weight: cosmic::iced::font::Weight::Medium,
                                ..Default::default()
                            })
                            .into(),
                    );

                    // Helper to format bytes to human-readable
                    fn format_bytes(bytes: u64) -> String {
                        const GB: u64 = 1024 * 1024 * 1024;
                        const MB: u64 = 1024 * 1024;
                        if bytes >= GB {
                            format!("{:.2} GB", bytes as f64 / GB as f64)
                        } else if bytes >= MB {
                            format!("{:.2} MB", bytes as f64 / MB as f64)
                        } else {
                            format!("{} bytes", bytes)
                        }
                    }

                    // Helper to calculate percentage
                    fn percentage(used: u64, total: u64) -> f32 {
                        if total == 0 {
                            0.0
                        } else {
                            (used as f64 / total as f64 * 100.0) as f32
                        }
                    }

                    // Data usage
                    let data_pct = percentage(usage.data_used, usage.data_total);
                    content_items.push(
                        widget::text(format!(
                            "{}: {} / {} ({:.1}%)",
                            fl!("btrfs-data"),
                            format_bytes(usage.data_used),
                            format_bytes(usage.data_total),
                            data_pct
                        ))
                        .size(10.0)
                        .into(),
                    );

                    // Metadata usage
                    let metadata_pct = percentage(usage.metadata_used, usage.metadata_total);
                    content_items.push(
                        widget::text(format!(
                            "{}: {} / {} ({:.1}%)",
                            fl!("btrfs-metadata"),
                            format_bytes(usage.metadata_used),
                            format_bytes(usage.metadata_total),
                            metadata_pct
                        ))
                        .size(10.0)
                        .into(),
                    );

                    // System usage
                    let system_pct = percentage(usage.system_used, usage.system_total);
                    content_items.push(
                        widget::text(format!(
                            "{}: {} / {} ({:.1}%)",
                            fl!("btrfs-system"),
                            format_bytes(usage.system_used),
                            format_bytes(usage.system_total),
                            system_pct
                        ))
                        .size(10.0)
                        .into(),
                    );

                    // Compression info
                    if let Some(compression) = &state.compression {
                        if let Some(algo) = compression {
                            content_items.push(
                                widget::text(format!("{}: {}", fl!("btrfs-compression"), algo))
                                    .size(10.0)
                                    .into(),
                            );
                        } else {
                            content_items.push(
                                widget::text(format!(
                                    "{}: {}",
                                    fl!("btrfs-compression"),
                                    fl!("btrfs-compression-disabled")
                                ))
                                .size(10.0)
                                .into(),
                            );
                        }
                    }
                }
                Err(error) => {
                    content_items.push(
                        widget::text(format!("Usage error: {}", error))
                            .size(11.0)
                            .into(),
                    );
                }
            }
        }

        // Spacing after usage section
        content_items.push(widget::vertical_space().height(8).into());
    }

    // === Subvolumes Section ===
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
                // Add Create buttons row
                let button_row = iced_widget::row![
                    widget::button::standard(fl!("btrfs-create-subvolume"))
                        .on_press(Message::VolumesMessage(
                            crate::ui::volumes::VolumesControlMessage::OpenBtrfsCreateSubvolume
                        )),
                    widget::button::standard(fl!("btrfs-create-snapshot"))
                        .on_press(Message::VolumesMessage(
                            crate::ui::volumes::VolumesControlMessage::OpenBtrfsCreateSnapshot
                        )),
                ]
                .spacing(8);

                content_items.push(button_row.into());

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
                        // Create row with subvolume info and delete button
                        let subvol_text = widget::text(format!("ID {} - {}", subvol.id, subvol.path))
                            .size(10.0);
                        
                        let delete_button = widget::button::icon(widget::icon::from_name("user-trash-symbolic"))
                            .on_press(Message::BtrfsDeleteSubvolume {
                                path: subvol.path.clone(),
                            })
                            .padding(4);

                        let row = iced_widget::row![
                            subvol_text,
                            widget::horizontal_space(),
                            delete_button,
                        ]
                        .spacing(8)
                        .align_y(cosmic::iced::Alignment::Center);

                        content_items.push(row.into());
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
