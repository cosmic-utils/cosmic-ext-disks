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

    let mut content_items: Vec<Element<'a, Message>> = vec![header.into()];

    // === Usage Breakdown Section ===
    // Try to get mount point from state first, then from volume
    let mount_point = state.mount_point.as_ref()
        .or_else(|| volume.mount_points.first());
    
    tracing::debug!("btrfs_management_section: state.mount_point={:?}, volume.mount_points={:?}, effective_mount_point={:?}",
        state.mount_point, volume.mount_points, mount_point);
        
    if let Some(_mount_point) = mount_point {
        // Check if we need to load usage
        if state.used_space.is_none() && !state.loading_usage {
            // Trigger load on next render cycle (will be caught by update handler)
            // Just note that we need to load - the actual message is sent elsewhere
        }

        if state.loading_usage {
            content_items.push(
                widget::text("Loading usage information...")
                    .size(11.0)
                    .into(),
            );
        } else if let Some(used_space_result) = &state.used_space {
            match used_space_result {
                Ok(used_bytes) => {
                    // Helper to format bytes to human-readable
                    fn format_bytes(bytes: u64) -> String {
                        const GB: u64 = 1024 * 1024 * 1024;
                        const MB: u64 = 1024 * 1024;
                        const KB: u64 = 1024;
                        if bytes >= GB {
                            format!("{:.2} GB", bytes as f64 / GB as f64)
                        } else if bytes >= MB {
                            format!("{:.2} MB", bytes as f64 / MB as f64)
                        } else if bytes >= KB {
                            format!("{:.2} KB", bytes as f64 / KB as f64)
                        } else {
                            format!("{} bytes", bytes)
                        }
                    }

                    // Display used space
                    content_items.push(
                        widget::text(format!(
                            "{}: {}",
                            fl!("btrfs-used-space"),
                            format_bytes(*used_bytes)
                        ))
                        .size(10.0)
                        .into(),
                    );
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
        // Try to get mount point from state first, then from volume
        let mount_point = state.mount_point.as_ref()
            .or_else(|| volume.mount_points.first());
            
        if let Some(_mp) = mount_point {
            // Note: This will trigger on every render until loaded
            // A better approach would be to trigger once, but this works for now
            content_items.push(widget::text("Loading subvolumes...").size(11.0).into());

            // Send message to load (will be handled in update)
            // For now, just show loading state
        } else {
            // Neither state nor volume has a mount point
            if volume.has_filesystem {
                // Filesystem exists but not detected as mounted
                content_items.push(
                    widget::text("BTRFS filesystem not mounted (try refreshing)")
                        .size(11.0)
                        .into(),
                );
            } else {
                content_items.push(
                    widget::text("BTRFS filesystem not mounted")
                        .size(11.0)
                        .into(),
                );
            }
        }
    } else if state.loading {
        content_items.push(widget::text("Loading subvolumes...").size(11.0).into());
    } else if let Some(result) = &state.subvolumes {
        match result {
            Ok(subvolumes) => {
                // Add Create buttons row
                let button_row = iced_widget::row![
                    widget::button::standard(fl!("btrfs-create-subvolume")).on_press(
                        Message::VolumesMessage(
                            crate::ui::volumes::VolumesControlMessage::OpenBtrfsCreateSubvolume
                        )
                    ),
                    widget::button::standard(fl!("btrfs-create-snapshot")).on_press(
                        Message::VolumesMessage(
                            crate::ui::volumes::VolumesControlMessage::OpenBtrfsCreateSnapshot
                        )
                    ),
                ]
                .spacing(8);

                content_items.push(button_row.into());

                if subvolumes.is_empty() {
                    content_items.push(widget::text("No subvolumes found").size(11.0).into());
                } else {
                    // Show subvolumes list
                    tracing::debug!("Rendering {} subvolumes", subvolumes.len());
                    
                    // Create grid with headers
                    let mut subvol_grid = iced_widget::column![].spacing(4);
                    
                    // Add header row
                    let header_row = iced_widget::row![
                        widget::text::caption_heading(fl!("btrfs-subvolume-id")).width(80),
                        widget::text::caption_heading(fl!("btrfs-subvolume-path")),
                        widget::text::caption_heading(fl!("btrfs-subvolume-actions")).width(60),
                    ]
                    .spacing(12);
                    subvol_grid = subvol_grid.push(header_row);

                    for (idx, subvol) in subvolumes.iter().enumerate() {
                        tracing::debug!(
                            "Rendering subvolume {}/{}: id={}, path={}",
                            idx + 1,
                            subvolumes.len(),
                            subvol.id,
                            subvol.path
                        );
                        
                        let delete_button = if let (Some(bp), Some(mp)) = (&state.block_path, &state.mount_point) {
                            widget::button::icon(widget::icon::from_name("edit-delete-symbolic"))
                                .on_press(Message::BtrfsDeleteSubvolume {
                                    block_path: bp.clone(),
                                    mount_point: mp.clone(),
                                    path: subvol.path.clone(),
                                })
                                .padding(4)
                        } else {
                            widget::button::icon(widget::icon::from_name("edit-delete-symbolic"))
                                .padding(4)
                        };

                        let row = iced_widget::row![
                            widget::text(format!("{}", subvol.id)).width(80),
                            widget::text(&subvol.path),
                            delete_button,
                        ]
                        .spacing(12)
                        .align_y(cosmic::iced::Alignment::Center);

                        subvol_grid = subvol_grid.push(row);
                        tracing::debug!("Added subvolume row to grid");
                    }
                    
                    content_items.push(subvol_grid.into());
                    tracing::debug!("Finished rendering all subvolumes, total content_items: {}", content_items.len());
                }
            }
            Err(error) => {
                content_items.push(widget::text(format!("Error: {}", error)).size(11.0).into());
            }
        }
    }

    tracing::debug!("btrfs_management_section: Building final column with {} items", content_items.len());
    iced_widget::Column::from_vec(content_items)
        .spacing(4)
        .padding(8)
        .into()
}
