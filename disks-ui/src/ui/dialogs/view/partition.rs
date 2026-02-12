use crate::app::Message;
use crate::fl;
use crate::ui::dialogs::message::{
    CreateMessage, EditFilesystemLabelMessage, EditPartitionMessage, ResizePartitionMessage,
};
use crate::ui::dialogs::state::{
    CreatePartitionDialog, EditFilesystemLabelDialog, EditPartitionDialog, FormatPartitionDialog,
    ResizePartitionDialog,
};
use crate::utils::labelled_spinner;
use crate::utils::{get_fs_tool_status, SizeUnit};
use cosmic::{
    Element, iced, iced_widget,
    widget::text::caption,
    widget::{button, checkbox, dialog, divider, dropdown, slider, text, text_input},
};
use disks_dbus::{PartitionTypeInfo, bytes_to_pretty, COMMON_GPT_TYPES, COMMON_DOS_TYPES};

pub fn create_partition<'a>(state: CreatePartitionDialog) -> Element<'a, Message> {
    let running = state.running;
    let error =state.error;
    let create = &state.info;

    // Get partition type details for radio list
    let partition_types: &[PartitionTypeInfo] = match create.table_type.as_str() {
        "gpt" => &COMMON_GPT_TYPES,
        "dos" => &COMMON_DOS_TYPES,
        _ => &[],
    };

    let mut content = iced_widget::column![];
    
    // Only show partition name field for table types that support it (not DOS/MBR)
    if create.table_type != "dos" {
        content = content.push(
            text_input(fl!("volume-name"), create.name.clone())
                .label(fl!("volume-name"))
                .on_input(|t| CreateMessage::NameUpdate(t).into())
        );
        content = content.push(divider::horizontal::default());
    }
    
    // Size controls with slider, spinners, and unit selectors on one line
    let len = create.max_size as f64;
    let size = create.size as f64;
    let free = len - size;
    let free_bytes = free as u64;
    let size_pretty = bytes_to_pretty(&create.size, false);
    let free_pretty = bytes_to_pretty(&free_bytes, false);
    let step = disks_dbus::get_step(&create.size);
    let current_size = create.size; // Capture for closure
    
    // Slider for visual feedback
    content = content.push(slider(0.0..=len, size, |v| CreateMessage::SizeUpdate(v as u64).into()));
    
    // Compact row: Size: -[num]+ [unit]  Free: -[num]+ [unit]
    let size_controls = iced_widget::row![
        text(fl!("partition-size")).width(iced::Length::Shrink),
        button::text("-").on_press(CreateMessage::SizeUpdate((size - step).max(0.) as u64).into()),
        text_input("", size_pretty)
            .width(iced::Length::Fixed(100.))
            .on_input(move |v| {
                match disks_dbus::pretty_to_bytes(&v) {
                    Ok(bytes) => CreateMessage::SizeUpdate((bytes as f64).clamp(0., len) as u64).into(),
                    Err(_) => CreateMessage::SizeUpdate(current_size).into(),
                }
            }),
        button::text("+").on_press(CreateMessage::SizeUpdate((size + step).min(len) as u64).into()),
        dropdown(
            SizeUnit::all_labels(),
            Some(create.size_unit_index),
            |idx| CreateMessage::SizeUnitUpdate(idx).into()
        ).width(iced::Length::Fixed(80.)),
        text(fl!("free-space")).width(iced::Length::Shrink),
        button::text("-").on_press(CreateMessage::SizeUpdate((size + step).min(len) as u64).into()),
        text_input("", free_pretty)
            .width(iced::Length::Fixed(100.))
            .on_input(move |v| {
                match disks_dbus::pretty_to_bytes(&v) {
                    Ok(bytes) => CreateMessage::SizeUpdate((len - (bytes as f64).clamp(0., len)) as u64).into(),
                    Err(_) => CreateMessage::SizeUpdate(current_size).into(),
                }
            }),
        button::text("+").on_press(CreateMessage::SizeUpdate((size - step).max(0.) as u64).into()),
        dropdown(
            SizeUnit::all_labels(),
            Some(create.size_unit_index),
            |idx| CreateMessage::SizeUnitUpdate(idx).into()
        ).width(iced::Length::Fixed(80.)),
    ]
    .spacing(8)
    .align_y(iced::Alignment::Center);
    
    content = content.push(size_controls);
    content = content.push(divider::horizontal::default());
    
    // Get filesystem tool availability status
    let tool_status = get_fs_tool_status();
    
    // Filter partition types to only include those with available tools
    let available_types: Vec<_> = partition_types
        .iter()
        .filter(|p_type| {
            let fs_type = p_type.filesystem_type.as_str();
            tool_status.get(fs_type).copied().unwrap_or(true)
        })
        .collect();
    
    let total_types = partition_types.len();
    let available_count = available_types.len();
    let has_missing_tools = available_count < total_types;
    
    // Build dropdown labels for available types
    let dropdown_labels: Vec<String> = available_types
        .iter()
        .map(|p_type| {
            let fs_type = p_type.filesystem_type.as_str();
            match fs_type {
                "ext4" => format!("{} — {}", fl!("fs-name-ext4"), fl!("fs-desc-ext4")),
                "ext3" => format!("{} — {}", fl!("fs-name-ext3"), fl!("fs-desc-ext3")),
                "xfs" => format!("{} — {}", fl!("fs-name-xfs"), fl!("fs-desc-xfs")),
                "btrfs" => format!("{} — {}", fl!("fs-name-btrfs"), fl!("fs-desc-btrfs")),
                "f2fs" => format!("{} — {}", fl!("fs-name-f2fs"), fl!("fs-desc-f2fs")),
                "udf" => format!("{} — {}", fl!("fs-name-udf"), fl!("fs-desc-udf")),
                "ntfs" => format!("{} — {}", fl!("fs-name-ntfs"), fl!("fs-desc-ntfs")),
                "vfat" => format!("{} — {}", fl!("fs-name-vfat"), fl!("fs-desc-vfat")),
                "exfat" => format!("{} — {}", fl!("fs-name-exfat"), fl!("fs-desc-exfat")),
                "swap" => format!("{} — {}", fl!("fs-name-swap"), fl!("fs-desc-swap")),
                fs => fs.to_string(),
            }
        })
        .collect();
    
    // Map selected index from full list to filtered list
    let selected_in_filtered = available_types
        .iter()
        .position(|p| {
            let original_idx = partition_types.iter().position(|orig| {
                orig.filesystem_type == p.filesystem_type
            });
            original_idx == Some(create.selected_partition_type_index)
        });
    
    // Filesystem type selection (dropdown)
    content = content.push(caption(fl!("filesystem-type")));
    content = content.push(
        dropdown(
            dropdown_labels,
            selected_in_filtered,
            move |selected_idx| {
                // Map back from filtered index to original index
                let original_idx = partition_types
                    .iter()
                    .position(|orig| {
                        orig.filesystem_type == available_types[selected_idx].filesystem_type
                    })
                    .unwrap_or(0);
                CreateMessage::PartitionTypeUpdate(original_idx).into()
            }
        )
    );
    
    content = content.push(divider::horizontal::default());
    
    // Show warning if filesystem types are hidden due to missing tools
    if has_missing_tools {
        content = content.push(
            caption(format!("⚠ {}", fl!("fs-tools-warning")))
        );
    }
    
    content = content.push(divider::horizontal::default());
    
    content = content.push(checkbox(fl!("overwrite-data-slow"), create.erase)
        .on_toggle(|v| CreateMessage::EraseUpdate(v).into()));
    
    content = content.push(checkbox(fl!("password-protected-luks"), create.password_protected)
        .on_toggle(|v| CreateMessage::PasswordProtectedUpdate(v).into()));

    if create.password_protected {
        content = content.push(
            text_input::secure_input("", create.password.clone(), None, true)
                .label(fl!("password"))
                .on_input(|v| CreateMessage::PasswordUpdate(v).into()),
        );

        content = content.push(
            text_input::secure_input("", create.confirmed_password.clone(), None, true)
                .label(fl!("confirm"))
                .on_input(|v| CreateMessage::ConfirmedPasswordUpdate(v).into()),
        );
    }

    if let Some(err) = error.as_ref() {
        content = content.push(caption(err.clone()));
    }

    let mut continue_button = button::destructive(fl!("continue"));

    if !running {
        continue_button = continue_button.on_press(CreateMessage::Partition.into());
    }

    if running {
        content = content.push(caption(fl!("working")));
    }

    dialog::dialog()
        .title(fl!("create-partition"))
        .control(content.spacing(20.))
        .primary_action(continue_button)
        .secondary_action(button::standard(fl!("cancel")).on_press(CreateMessage::Cancel.into()))
        .into()
}

pub fn format_partition<'a>(state: FormatPartitionDialog) -> Element<'a, Message> {
    let FormatPartitionDialog {
        volume: _,
        info: create,
        running,
    } = state;

    let size_pretty = bytes_to_pretty(&create.size, false);
    
    // Get partition type details for radio list
    let partition_types: &[PartitionTypeInfo] = match create.table_type.as_str() {
        "gpt" => &COMMON_GPT_TYPES,
        "dos" => &COMMON_DOS_TYPES,
        _ => &[],
    };

    let mut content = iced_widget::column![
        caption(fl!("format-partition-description", size = size_pretty)),
    ];
    
    // Only show partition name field for table types that support it (not DOS/MBR)
    if create.table_type != "dos" {
        content = content.push(
            text_input(fl!("volume-name"), create.name.clone())
                .label(fl!("volume-name"))
                .on_input(|t| CreateMessage::NameUpdate(t).into())
        );
    }
    
    // Get filesystem tool availability status
    let tool_status = get_fs_tool_status();
    
    // Filter partition types to only include those with available tools
    let available_types: Vec<_> = partition_types
        .iter()
        .filter(|p_type| {
            let fs_type = p_type.filesystem_type.as_str();
            tool_status.get(fs_type).copied().unwrap_or(true)
        })
        .collect();
    
    let total_types = partition_types.len();
    let available_count = available_types.len();
    let has_missing_tools = available_count < total_types;
    
    // Build dropdown labels for available types
    let dropdown_labels: Vec<String> = available_types
        .iter()
        .map(|p_type| {
            let fs_type = p_type.filesystem_type.as_str();
            match fs_type {
                "ext4" => format!("{} — {}", fl!("fs-name-ext4"), fl!("fs-desc-ext4")),
                "ext3" => format!("{} — {}", fl!("fs-name-ext3"), fl!("fs-desc-ext3")),
                "xfs" => format!("{} — {}", fl!("fs-name-xfs"), fl!("fs-desc-xfs")),
                "btrfs" => format!("{} — {}", fl!("fs-name-btrfs"), fl!("fs-desc-btrfs")),
                "f2fs" => format!("{} — {}", fl!("fs-name-f2fs"), fl!("fs-desc-f2fs")),
                "udf" => format!("{} — {}", fl!("fs-name-udf"), fl!("fs-desc-udf")),
                "ntfs" => format!("{} — {}", fl!("fs-name-ntfs"), fl!("fs-desc-ntfs")),
                "vfat" => format!("{} — {}", fl!("fs-name-vfat"), fl!("fs-desc-vfat")),
                "exfat" => format!("{} — {}", fl!("fs-name-exfat"), fl!("fs-desc-exfat")),
                "swap" => format!("{} — {}", fl!("fs-name-swap"), fl!("fs-desc-swap")),
                fs => fs.to_string(),
            }
        })
        .collect();
    
    // Map selected index from full list to filtered list
    let selected_in_filtered = available_types
        .iter()
        .position(|p| {
            let original_idx = partition_types.iter().position(|orig| {
                orig.filesystem_type == p.filesystem_type
            });
            original_idx == Some(create.selected_partition_type_index)
        });
    
    // Filesystem type selection (dropdown)
    content = content.push(caption(fl!("filesystem-type")));
    content = content.push(
        dropdown(
            dropdown_labels,
            selected_in_filtered,
            move |selected_idx| {
                // Map back from filtered index to original index
                let original_idx = partition_types
                    .iter()
                    .position(|orig| {
                        orig.filesystem_type == available_types[selected_idx].filesystem_type
                    })
                    .unwrap_or(0);
                CreateMessage::PartitionTypeUpdate(original_idx).into()
            }
        )
    );
    
    content = content.push(divider::horizontal::default());
    
    // Show warning if filesystem types are hidden due to missing tools
    if has_missing_tools {
        content = content.push(
            caption(format!("⚠ {}", fl!("fs-tools-warning")))
        );
    }
    
    content = content.push(divider::horizontal::default());
    
    content = content.push(checkbox(fl!("overwrite-data-slow"), create.erase)
        .on_toggle(|v| CreateMessage::EraseUpdate(v).into()));
    
    content = content.spacing(12);

    if running {
        content = content.push(caption(fl!("working")));
    }

    let mut confirm = button::destructive(fl!("format-partition"));
    if !running {
        confirm = confirm.on_press(CreateMessage::Partition.into());
    }

    dialog::dialog()
        .title(fl!("format-partition"))
        .control(content)
        .primary_action(confirm)
        .secondary_action(button::standard(fl!("cancel")).on_press(CreateMessage::Cancel.into()))
        .into()
}

pub fn edit_partition<'a>(state: EditPartitionDialog) -> Element<'a, Message> {
    let EditPartitionDialog {
        volume: _,
        partition_types,
        selected_type_index,
        name,
        legacy_bios_bootable,
        system_partition,
        hidden,
        running,
    } = state;

    let opts: Vec<String> = partition_types
        .iter()
        .map(|t: &PartitionTypeInfo| format!("{} - {}", t.name, t.ty))
        .collect();

    let mut content = iced_widget::column![
        dropdown(opts, Some(selected_type_index), |v| {
            EditPartitionMessage::TypeUpdate(v).into()
        }),
        text_input(fl!("partition-name"), name)
            .label(fl!("partition-name"))
            .on_input(|t| EditPartitionMessage::NameUpdate(t).into()),
        checkbox(fl!("flag-legacy-bios-bootable"), legacy_bios_bootable)
            .on_toggle(|v| EditPartitionMessage::LegacyBiosBootableUpdate(v).into()),
        checkbox(fl!("flag-system-partition"), system_partition)
            .on_toggle(|v| EditPartitionMessage::SystemPartitionUpdate(v).into()),
        checkbox(fl!("flag-hide-from-firmware"), hidden)
            .on_toggle(|v| EditPartitionMessage::HiddenUpdate(v).into()),
    ]
    .spacing(12);

    if running {
        content = content.push(caption(fl!("working")));
    }

    let mut apply = button::standard(fl!("apply"));
    if !running {
        apply = apply.on_press(EditPartitionMessage::Confirm.into());
    }

    dialog::dialog()
        .title(fl!("edit-partition"))
        .control(content)
        .primary_action(apply)
        .secondary_action(
            button::standard(fl!("cancel")).on_press(EditPartitionMessage::Cancel.into()),
        )
        .into()
}

pub fn resize_partition<'a>(state: ResizePartitionDialog) -> Element<'a, Message> {
    let ResizePartitionDialog {
        volume: _,
        min_size_bytes,
        max_size_bytes,
        new_size_bytes,
        running,
    } = state;

    let min = min_size_bytes as f64;
    let max = max_size_bytes as f64;
    let value = new_size_bytes as f64;
    let step = disks_dbus::get_step(&new_size_bytes);

    let min_pretty = bytes_to_pretty(&min_size_bytes, false);
    let max_pretty = bytes_to_pretty(&max_size_bytes, false);
    let value_pretty = bytes_to_pretty(&new_size_bytes, false);

    let can_resize = max_size_bytes.saturating_sub(min_size_bytes) >= 1024;

    let mut content = iced_widget::column![
        caption(fl!(
            "resize-partition-range",
            min = min_pretty,
            max = max_pretty
        )),
        slider(min..=max, value, |v| ResizePartitionMessage::SizeUpdate(
            v as u64
        )
        .into()),
        labelled_spinner(fl!("new-size"), value_pretty, value, step, min, max, |v| {
            ResizePartitionMessage::SizeUpdate(v as u64).into()
        }),
    ]
    .spacing(12);

    if running {
        content = content.push(caption(fl!("working")));
    }

    let mut apply = button::standard(fl!("apply"));
    if !running && can_resize {
        apply = apply.on_press(ResizePartitionMessage::Confirm.into());
    }

    dialog::dialog()
        .title(fl!("resize-partition"))
        .control(content)
        .primary_action(apply)
        .secondary_action(
            button::standard(fl!("cancel")).on_press(ResizePartitionMessage::Cancel.into()),
        )
        .into()
}

pub fn edit_filesystem_label<'a>(state: EditFilesystemLabelDialog) -> Element<'a, Message> {
    let EditFilesystemLabelDialog {
        target: _,
        label,
        running,
    } = state;

    let mut content = iced_widget::column![
        text_input(fl!("filesystem-label"), label)
            .label(fl!("filesystem-label"))
            .on_input(|t| EditFilesystemLabelMessage::LabelUpdate(t).into()),
    ]
    .spacing(12);

    if running {
        content = content.push(caption(fl!("working")));
    }

    let mut apply = button::standard(fl!("apply"));
    if !running {
        apply = apply.on_press(EditFilesystemLabelMessage::Confirm.into());
    }

    dialog::dialog()
        .title(fl!("edit-filesystem"))
        .control(content)
        .primary_action(apply)
        .secondary_action(
            button::standard(fl!("cancel")).on_press(EditFilesystemLabelMessage::Cancel.into()),
        )
        .into()
}
