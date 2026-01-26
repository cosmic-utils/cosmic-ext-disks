use cosmic::{
    Element,
    cosmic_theme::palette::WithAlpha,
    iced::{Alignment, Background, Length, Shadow},
    iced_widget::{self, column},
    widget::{
        self, checkbox, container, icon,
        text::{caption, caption_heading},
    },
};

use crate::app::Message;
use crate::fl;
use crate::ui::dialogs::state::{
    CreatePartitionDialog, DeletePartitionDialog, ShowDialog, UnlockEncryptedDialog,
};
use crate::ui::volumes::helpers;
use crate::utils::DiskSegmentKind;
use disks_dbus::bytes_to_pretty;
use disks_dbus::{VolumeKind, VolumeNode};

use super::{Segment, ToggleState, VolumesControl, VolumesControlMessage};

impl Segment {
    pub fn get_segment_control<'a>(&self) -> Element<'a, Message> {
        if self.kind == DiskSegmentKind::FreeSpace {
            container(
                iced_widget::column![
                    caption_heading(fl!("free-space-caption")).center(),
                    caption(bytes_to_pretty(&self.size, false)).center()
                ]
                .spacing(5)
                .width(Length::Fill)
                .align_x(Alignment::Center),
            )
            .padding(5)
            .align_x(Alignment::Center)
            .align_y(Alignment::Center)
            .into()
        } else if self.kind == DiskSegmentKind::Reserved {
            container(
                iced_widget::column![
                    caption_heading(fl!("reserved-space-caption")).center(),
                    caption(bytes_to_pretty(&self.size, false)).center()
                ]
                .spacing(5)
                .width(Length::Fill)
                .align_x(Alignment::Center),
            )
            .padding(5)
            .align_x(Alignment::Center)
            .align_y(Alignment::Center)
            .into()
        } else {
            container(
                iced_widget::column![
                    caption_heading(self.name.clone()).center(),
                    caption(self.label.clone()).center(),
                    caption(self.partition_type.clone()).center(),
                    caption(bytes_to_pretty(&self.size, false)).center()
                ]
                .spacing(5)
                .align_x(Alignment::Center),
            )
            .padding(5)
            .align_x(Alignment::Center)
            .align_y(Alignment::Center)
            .into()
        }
    }
}

impl VolumesControl {
    pub fn view(&self) -> Element<'_, Message> {
        const SEGMENT_BUTTON_HEIGHT: f32 = 130.0;

        let show_reserved = checkbox(fl!("show-reserved"), self.show_reserved)
            .on_toggle(|v| VolumesControlMessage::ToggleShowReserved(v).into());

        let segment_buttons: Vec<Element<Message>> = self
            .segments
            .iter()
            .enumerate()
            .map(|(index, segment)| {
                // When a child filesystem is selected, visually de-emphasize the container.
                let container_selected = segment.state && self.selected_volume.is_none();
                let active_state = ToggleState::active_or(&container_selected, ToggleState::Normal);
                let hovered_state =
                    ToggleState::active_or(&container_selected, ToggleState::Hovered);

                // For encrypted container partitions, render a split tile:
                // - top half: the container itself (selects the segment)
                // - bottom half: contained volumes (select filesystem/LV)
                let container_volume = segment
                    .volume
                    .as_ref()
                    .and_then(|p| helpers::find_volume_node_for_partition(&self.model.volumes, p))
                    .filter(|v| v.kind == VolumeKind::CryptoContainer);

                if let Some(v) = container_volume {
                    let state_text = if v.locked {
                        fl!("locked")
                    } else {
                        fl!("unlocked")
                    };

                    let top = cosmic::widget::button::custom(
                        container(
                            iced_widget::column![
                                caption_heading(segment.name.clone()).center(),
                                caption(bytes_to_pretty(&segment.size, false)).center(),
                                caption(state_text).center(),
                            ]
                            .spacing(4)
                            .width(Length::Fill)
                            .align_x(Alignment::Center),
                        )
                        .padding(6)
                        .align_x(Alignment::Center)
                        .align_y(Alignment::Center),
                    )
                    .on_press(Message::VolumesMessage(
                        VolumesControlMessage::SegmentSelected(index),
                    ))
                    .class(cosmic::theme::Button::Custom {
                        active: Box::new(move |_b, theme| get_button_style(active_state, theme)),
                        disabled: Box::new(|theme| get_button_style(ToggleState::Disabled, theme)),
                        hovered: Box::new(move |_, theme| get_button_style(hovered_state, theme)),
                        pressed: Box::new(|_, theme| get_button_style(ToggleState::Pressed, theme)),
                    })
                    .height(Length::FillPortion(1));

                    let bottom_content: Element<Message> = if v.locked {
                        // No children while locked.
                        container(
                            iced_widget::column![caption(fl!("locked")).center()]
                                .width(Length::Fill)
                                .height(Length::Fill)
                                .align_x(Alignment::Center),
                        )
                        .padding(6)
                        .into()
                    } else {
                        // Prefer showing the immediate children if any; if the only child is a
                        // container (e.g. cleartext PV), show its children instead.
                        let direct = &v.children;
                        let mut col = iced_widget::column![].spacing(8);
                        col = col.width(Length::Fill).height(Length::Fill);

                        if direct.len() == 1 && !direct[0].children.is_empty() {
                            col = col.push(volume_row_compact(
                                index,
                                &direct[0],
                                &direct[0].children,
                                self.selected_volume.as_deref(),
                            ));
                        } else {
                            col = col.push(volume_row_compact(
                                index,
                                v,
                                direct,
                                self.selected_volume.as_deref(),
                            ));
                        }

                        col.into()
                    };

                    // No extra outer padding here; child tiles should align with container extents.
                    let bottom = container(bottom_content)
                        .padding(0)
                        .height(Length::FillPortion(1))
                        .width(Length::Fill);

                    return container(
                        iced_widget::column![top, bottom]
                            .spacing(6)
                            .height(Length::Fixed(SEGMENT_BUTTON_HEIGHT)),
                    )
                    .width(Length::FillPortion(segment.width))
                    .into();
                }

                cosmic::widget::button::custom(segment.get_segment_control())
                    .on_press(Message::VolumesMessage(
                        VolumesControlMessage::SegmentSelected(index),
                    ))
                    .class(cosmic::theme::Button::Custom {
                        active: Box::new(move |_b, theme| get_button_style(active_state, theme)),
                        disabled: Box::new(|theme| get_button_style(ToggleState::Disabled, theme)),
                        hovered: Box::new(move |_, theme| get_button_style(hovered_state, theme)),
                        pressed: Box::new(|_, theme| get_button_style(ToggleState::Pressed, theme)),
                    })
                    .height(Length::Fixed(SEGMENT_BUTTON_HEIGHT))
                    .width(Length::FillPortion(segment.width))
                    .into()
            })
            .collect();

        let selected = match self.segments.get(self.selected_segment).cloned() {
            Some(segment) => segment,
            None => {
                // Handle the case where selected_segment is out of range
                return container(
                    column![
                        cosmic::widget::Row::from_vec(vec![])
                            .spacing(10)
                            .width(Length::Fill),
                        widget::Row::from_vec(vec![]).width(Length::Fill)
                    ]
                    .spacing(10),
                )
                .width(Length::Fill)
                .padding(10)
                .class(cosmic::style::Container::Card)
                .into();
            }
        };
        let mut action_bar: Vec<Element<Message>> = vec![];

        let selected_volume = selected
            .volume
            .as_ref()
            .and_then(|p| helpers::find_volume_node_for_partition(&self.model.volumes, p));

        let selected_child_volume = self.selected_volume_node();

        match selected.kind {
            DiskSegmentKind::Partition => {
                if let Some(p) = selected.volume.as_ref() {
                    // Container actions are based on the selected partition (segment).
                    if let Some(v) = selected_volume
                        && v.kind == VolumeKind::CryptoContainer
                    {
                        if v.locked {
                            action_bar.push(tooltip_icon_button(
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
                            action_bar.push(tooltip_icon_button(
                                "changes-prevent-symbolic",
                                fl!("lock").to_string(),
                                Some(VolumesControlMessage::LockContainer.into()),
                            ));
                        }
                    }

                    // If a child filesystem/LV is selected, mount/unmount applies to it.
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
                            action_bar.push(tooltip_icon_button(
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
                        action_bar.push(tooltip_icon_button(
                            icon_name,
                            fl!("mount-toggle").to_string(),
                            Some(msg.into()),
                        ));
                    }
                }
            }
            DiskSegmentKind::FreeSpace => {
                action_bar.push(tooltip_icon_button(
                    "list-add-symbolic",
                    fl!("create-partition").to_string(),
                    Some(Message::Dialog(Box::new(ShowDialog::AddPartition(
                        CreatePartitionDialog {
                            info: selected.get_create_info(),
                            running: false,
                        },
                    )))),
                ));
            }
            DiskSegmentKind::Reserved => {}
        }

        if selected.kind == DiskSegmentKind::Partition
            && let Some(p) = selected.volume.as_ref()
        {
            // Command visibility is based on the selected partition (segment), not a selected child.
            // Child selection only affects filesystem-targeted actions (label/check).

            // Always: Format Partition
            action_bar.push(tooltip_icon_button(
                "edit-clear-symbolic",
                fl!("format-partition").to_string(),
                Some(VolumesControlMessage::OpenFormatPartition.into()),
            ));

            // Partition-only: Edit Partition + Resize
            if selected_child_volume.is_none() && p.volume_type == disks_dbus::VolumeType::Partition
            {
                action_bar.push(tooltip_icon_button(
                    "document-edit-symbolic",
                    fl!("edit-partition").to_string(),
                    Some(VolumesControlMessage::OpenEditPartition.into()),
                ));

                let right_free_bytes = self
                    .segments
                    .get(self.selected_segment.saturating_add(1))
                    .filter(|s| s.kind == DiskSegmentKind::FreeSpace)
                    .map(|s| s.size)
                    .unwrap_or(0);
                let max_size = p.size.saturating_add(right_free_bytes);
                let min_size = p.usage.as_ref().map(|u| u.used).unwrap_or(0).min(max_size);

                let resize_enabled = max_size.saturating_sub(min_size) >= 1024;
                action_bar.push(tooltip_icon_button(
                    "transform-scale-symbolic",
                    fl!("resize-partition").to_string(),
                    resize_enabled.then_some(VolumesControlMessage::OpenResizePartition.into()),
                ));
            }

            // Partition + Filesystem: Edit filesystem label + Check filesystem
            let fs_target_available = selected_child_volume
                .map(|n| n.can_mount())
                .unwrap_or_else(|| p.can_mount());
            if fs_target_available {
                action_bar.push(tooltip_icon_button(
                    "document-properties-symbolic",
                    fl!("edit-mount-options").to_string(),
                    Some(VolumesControlMessage::OpenEditMountOptions.into()),
                ));
                action_bar.push(tooltip_icon_button(
                    "tag-symbolic",
                    fl!("edit-filesystem").to_string(),
                    Some(VolumesControlMessage::OpenEditFilesystemLabel.into()),
                ));
                action_bar.push(tooltip_icon_button(
                    "emblem-ok-symbolic",
                    fl!("check-filesystem").to_string(),
                    Some(VolumesControlMessage::OpenCheckFilesystem.into()),
                ));

                // Filesystem operations: Repair + Take Ownership
                action_bar.push(tooltip_icon_button(
                    "tools-symbolic",
                    fl!("repair-filesystem").to_string(),
                    Some(VolumesControlMessage::OpenRepairFilesystem.into()),
                ));
                action_bar.push(tooltip_icon_button(
                    "user-home-symbolic",
                    fl!("take-ownership").to_string(),
                    Some(VolumesControlMessage::OpenTakeOwnership.into()),
                ));
            }

            // Container-only (VolumeType::Container): Change Passphrase
            if selected_volume.is_some_and(|v| v.kind == VolumeKind::CryptoContainer) {
                action_bar.push(tooltip_icon_button(
                    "dialog-password-symbolic",
                    fl!("change-passphrase").to_string(),
                    Some(VolumesControlMessage::OpenChangePassphrase.into()),
                ));

                action_bar.push(tooltip_icon_button(
                    "document-properties-symbolic",
                    fl!("edit-encryption-options").to_string(),
                    Some(VolumesControlMessage::OpenEditEncryptionOptions.into()),
                ));
            }

            // Delete partition
            if selected_child_volume.is_none()
                && p.volume_type != disks_dbus::VolumeType::Filesystem
            {
                action_bar.push(widget::horizontal_space().into());
                action_bar.push(tooltip_icon_button(
                    "edit-delete-symbolic",
                    fl!("delete", name = selected.name.clone()).to_string(),
                    Some(Message::Dialog(Box::new(ShowDialog::DeletePartition(
                        DeletePartitionDialog {
                            name: selected.name.clone(),
                            running: false,
                        },
                    )))),
                ));
            }
        }

        let root = column![
            cosmic::widget::Row::from_vec(vec![show_reserved.into()])
                .spacing(10)
                .width(Length::Fill),
            cosmic::widget::Row::from_vec(segment_buttons)
                .spacing(10)
                .width(Length::Fill),
            widget::Row::from_vec(action_bar).width(Length::Fill)
        ]
        .spacing(10);

        container(root)
            .width(Length::Fill)
            .padding(10)
            .class(cosmic::style::Container::Card)
            .into()
    }
}

fn tooltip_icon_button(
    icon_name: &str,
    tooltip: String,
    msg: Option<Message>,
) -> Element<'_, Message> {
    let mut button = widget::button::custom(icon::from_name(icon_name));
    if let Some(m) = msg {
        button = button.on_press(m);
    }

    widget::tooltip(
        button,
        widget::text::body(tooltip),
        widget::tooltip::Position::Top,
    )
    .into()
}

fn volume_row_compact<'a>(
    segment_index: usize,
    parent: &VolumeNode,
    children: &'a [VolumeNode],
    selected_volume: Option<&str>,
) -> Element<'a, Message> {
    let total = parent.size.max(1);
    let mut buttons: Vec<Element<Message>> = Vec::new();

    for child in children {
        let child_object_path = child.object_path.to_string();
        let denom = total;
        let width = (((child.size as f64 / denom as f64) * 1000.).log10().ceil() as u16).max(1);

        let col = iced_widget::column![
            cosmic::widget::text::caption_heading(child.label.clone()).center(),
        ]
        .spacing(4)
        .width(Length::Fill)
        .align_x(Alignment::Center);

        let is_selected = selected_volume.is_some_and(|p| p == child_object_path);
        let active_state = if is_selected {
            ToggleState::Active
        } else {
            ToggleState::Normal
        };
        let hovered_state = if is_selected {
            ToggleState::Active
        } else {
            ToggleState::Hovered
        };

        let b = cosmic::widget::button::custom(container(col).padding(6))
            .on_press(
                VolumesControlMessage::SelectVolume {
                    segment_index,
                    object_path: child_object_path,
                }
                .into(),
            )
            .class(cosmic::theme::Button::Custom {
                active: Box::new(move |_b, theme| get_button_style(active_state, theme)),
                disabled: Box::new(|theme| get_button_style(ToggleState::Disabled, theme)),
                hovered: Box::new(move |_, theme| get_button_style(hovered_state, theme)),
                pressed: Box::new(|_, theme| get_button_style(ToggleState::Pressed, theme)),
            })
            .height(Length::Fill)
            .width(Length::FillPortion(width));

        buttons.push(b.into());
    }

    cosmic::widget::Row::from_vec(buttons)
        .spacing(10)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

fn get_button_style(
    state: ToggleState,
    theme: &cosmic::theme::Theme,
) -> cosmic::widget::button::Style {
    let mut base = cosmic::widget::button::Style {
        shadow_offset: Shadow::default().offset,
        background: Some(cosmic::iced::Background::Color(
            theme.cosmic().primary.base.into(),
        )), // Some(cosmic::iced::Background::Color(Color::TRANSPARENT)),
        overlay: None,
        border_radius: (theme.cosmic().corner_radii.radius_xs).into(),
        border_width: 0.,
        border_color: theme.cosmic().primary.base.into(),
        outline_width: 2.,
        outline_color: theme.cosmic().primary.base.into(),
        icon_color: None,
        text_color: None,
    };

    match state {
        ToggleState::Normal => {}
        ToggleState::Active => {
            base.border_color = theme.cosmic().accent_color().into();
            base.outline_color = theme.cosmic().accent_color().into();
            base.background = Some(Background::Color(
                theme.cosmic().accent_color().with_alpha(0.2).into(),
            ));
        }
        ToggleState::Disabled => {
            base.border_color = theme.cosmic().primary.base.with_alpha(0.35).into();
            base.outline_color = theme.cosmic().primary.base.with_alpha(0.35).into();
            base.background = Some(Background::Color(
                theme.cosmic().primary.base.with_alpha(0.08).into(),
            ));
        }
        ToggleState::Hovered => {
            base.text_color = Some(theme.cosmic().accent_button.base.into());
            base.background = Some(Background::Color(theme.cosmic().button.hover.into()));
        }
        ToggleState::Pressed => {
            base.border_color = theme.cosmic().accent_color().into();
            base.outline_color = theme.cosmic().accent_color().into();
        }
    }

    base
}
