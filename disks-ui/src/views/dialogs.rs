use super::volumes::{
    ChangePassphraseMessage, CreateMessage, EditFilesystemLabelMessage, EditPartitionMessage,
    ResizePartitionMessage, TakeOwnershipMessage, UnlockMessage,
};
use crate::app::Message;
use crate::app::SmartDataDialog;
use crate::app::SmartDialogMessage;
use crate::app::UnlockEncryptedDialog;
use crate::app::{
    AttachDiskImageDialog, AttachDiskImageDialogMessage, ImageOperationDialog,
    ImageOperationDialogMessage, ImageOperationKind, NewDiskImageDialog, NewDiskImageDialogMessage,
};
use crate::app::{
    ChangePassphraseDialog, CreatePartitionDialog, EditFilesystemLabelDialog, EditPartitionDialog,
    FormatPartitionDialog, ResizePartitionDialog, TakeOwnershipDialog,
};
use crate::app::{FormatDiskDialog, FormatDiskMessage};
use crate::fl;
use crate::utils::labelled_spinner;
use cosmic::{
    Element, iced_widget,
    widget::text::{caption, caption_heading},
    widget::{button, checkbox, dialog, dropdown, slider, text_input, toggler},
};
use disks_dbus::{PartitionTypeInfo, bytes_to_pretty, get_valid_partition_names};
use std::borrow::Cow;

pub fn new_disk_image<'a>(state: NewDiskImageDialog) -> Element<'a, Message> {
    let size_pretty = bytes_to_pretty(&state.size_bytes, false);
    let step = disks_dbus::get_step(&state.size_bytes);

    let mut content = iced_widget::column![
        text_input(fl!("image-destination-path"), state.path.clone())
            .label(fl!("image-destination-path"))
            .on_input(|v| NewDiskImageDialogMessage::PathUpdate(v).into()),
        labelled_spinner(
            fl!("image-size"),
            size_pretty,
            state.size_bytes as f64,
            step,
            (1024 * 1024) as f64,
            (1024_u64.pow(5)) as f64,
            |v| NewDiskImageDialogMessage::SizeUpdate(v as u64).into(),
        ),
    ]
    .spacing(12);

    if let Some(err) = state.error.as_ref() {
        content = content.push(caption(err.clone()));
    }

    if state.running {
        content = content.push(caption(fl!("working")));
    }

    let mut create_button = button::destructive(fl!("create-image"));
    if !state.running {
        create_button = create_button.on_press(NewDiskImageDialogMessage::Create.into());
    }

    // while running, still allow closing (it will not cancel the file create, which is fast)
    let cancel_msg = NewDiskImageDialogMessage::Cancel;

    dialog::dialog()
        .title(fl!("new-disk-image"))
        .control(content)
        .primary_action(create_button)
        .secondary_action(button::standard(fl!("cancel")).on_press(cancel_msg.into()))
        .into()
}

pub fn attach_disk_image<'a>(state: AttachDiskImageDialog) -> Element<'a, Message> {
    let mut content = iced_widget::column![
        text_input(fl!("image-file-path"), state.path.clone())
            .label(fl!("image-file-path"))
            .on_input(|v| AttachDiskImageDialogMessage::PathUpdate(v).into()),
    ]
    .spacing(12);

    if let Some(err) = state.error.as_ref() {
        content = content.push(caption(err.clone()));
    }

    if state.running {
        content = content.push(caption(fl!("working")));
    }

    let mut attach_button = button::destructive(fl!("attach"));
    if !state.running {
        attach_button = attach_button.on_press(AttachDiskImageDialogMessage::Attach.into());
    }

    dialog::dialog()
        .title(fl!("attach-disk-image"))
        .control(content)
        .primary_action(attach_button)
        .secondary_action(
            button::standard(fl!("cancel")).on_press(AttachDiskImageDialogMessage::Cancel.into()),
        )
        .into()
}

pub fn image_operation<'a>(state: ImageOperationDialog) -> Element<'a, Message> {
    let title = match state.kind {
        ImageOperationKind::CreateFromDrive => fl!("create-disk-from-drive"),
        ImageOperationKind::RestoreToDrive => fl!("restore-image-to-drive"),
        ImageOperationKind::CreateFromPartition => fl!("create-disk-from-partition"),
        ImageOperationKind::RestoreToPartition => fl!("restore-image-to-partition"),
    };

    let path_label = match state.kind {
        ImageOperationKind::CreateFromDrive | ImageOperationKind::CreateFromPartition => {
            fl!("image-destination-path")
        }
        ImageOperationKind::RestoreToDrive | ImageOperationKind::RestoreToPartition => {
            fl!("image-source-path")
        }
    };

    let mut content = iced_widget::column![caption(format!(
        "{}: {}",
        fl!("device"),
        state.drive.name()
    ))]
    .spacing(12);

    if matches!(
        state.kind,
        ImageOperationKind::CreateFromPartition | ImageOperationKind::RestoreToPartition
    ) {
        match state.partition.as_ref() {
            Some(partition) => {
                let partition_name = if partition.name.trim().is_empty() {
                    fl!("untitled")
                } else {
                    partition.name.clone()
                };
                let partition_path = partition
                    .device_path
                    .clone()
                    .unwrap_or_else(|| partition.path.to_string());

                content = content
                    .push(caption(format!("{}: {}", fl!("partition"), partition_name)))
                    .push(caption(format!("{}: {}", fl!("path"), partition_path)));
            }
            None => {
                content =
                    content.push(caption(format!("{}: {}", fl!("partition"), fl!("unknown"))));
            }
        }
    }

    if matches!(
        state.kind,
        ImageOperationKind::RestoreToDrive | ImageOperationKind::RestoreToPartition
    ) {
        content = content.push(caption(fl!("restore-warning")));
    }

    content = content.push(
        text_input(path_label.clone(), state.image_path.clone())
            .label(path_label)
            .on_input(|v| ImageOperationDialogMessage::PathUpdate(v).into()),
    );

    if let Some(err) = state.error.as_ref() {
        content = content.push(caption(err.clone()));
    }

    if state.running {
        content = content.push(caption(fl!("working")));
    }

    let primary_label = match state.kind {
        ImageOperationKind::CreateFromDrive | ImageOperationKind::CreateFromPartition => {
            fl!("create-image")
        }
        ImageOperationKind::RestoreToDrive | ImageOperationKind::RestoreToPartition => {
            fl!("restore-image")
        }
    };

    let mut start_button = button::destructive(primary_label);
    if !state.running {
        start_button = start_button.on_press(ImageOperationDialogMessage::Start.into());
    }

    let cancel_msg = ImageOperationDialogMessage::CancelOperation;

    dialog::dialog()
        .title(title)
        .control(content)
        .primary_action(start_button)
        .secondary_action(button::standard(fl!("cancel")).on_press(cancel_msg.into()))
        .into()
}

pub fn confirmation<'a>(
    title: impl Into<Cow<'a, str>>,
    prompt: impl Into<Cow<'a, str>>,
    ok_message: Message,
    cancel_message: Option<Message>,
    running: bool,
) -> Element<'a, Message> {
    let mut dialog = dialog::dialog().title(title).body(prompt);

    let mut ok_button = button::destructive(fl!("ok"));
    if !running {
        ok_button = ok_button.on_press(ok_message);
    }

    dialog = dialog.primary_action(ok_button);

    if let Some(c) = cancel_message {
        dialog = dialog.secondary_action(button::standard(fl!("cancel")).on_press(c))
    };

    if running {
        dialog = dialog.body(fl!("working"));
    }

    dialog.into()
}

pub fn info<'a>(
    title: impl Into<Cow<'a, str>>,
    body: impl Into<Cow<'a, str>>,
    ok_message: Message,
) -> Element<'a, Message> {
    dialog::dialog()
        .title(title)
        .body(body)
        .primary_action(button::standard(fl!("ok")).on_press(ok_message))
        .into()
}

pub fn create_partition<'a>(state: CreatePartitionDialog) -> Element<'a, Message> {
    let CreatePartitionDialog {
        info: create,
        running,
    } = state;

    let len = create.max_size as f64;

    let size = create.size as f64;
    let free = len - size;
    let free_bytes = free as u64;

    let size_pretty = bytes_to_pretty(&create.size, false);
    let free_pretty = bytes_to_pretty(&free_bytes, false);
    let step = disks_dbus::get_step(&create.size);

    let create_clone = create.clone();

    let valid_partition_types = get_valid_partition_names(create.table_type.clone());

    let mut content = iced_widget::column![
        text_input(fl!("volume-name"), create_clone.name)
            .label(fl!("volume-name"))
            .on_input(|t| CreateMessage::NameUpdate(t).into()),
        slider(0.0..=len, size, |v| CreateMessage::SizeUpdate(v as u64)
            .into()),
        labelled_spinner(
            fl!("partition-size"),
            size_pretty,
            size,
            step,
            0.,
            len,
            |v| { CreateMessage::SizeUpdate(v as u64).into() }
        ),
        labelled_spinner(
            fl!("free-space"),
            free_pretty,
            free,
            step,
            0.,
            len,
            move |v| { CreateMessage::SizeUpdate((len - v) as u64).into() }
        ),
        toggler(create_clone.erase)
            .label(fl!("erase"))
            .on_toggle(|v| CreateMessage::EraseUpdate(v).into()),
        dropdown(
            valid_partition_types,
            Some(create_clone.selected_partitition_type),
            |v| CreateMessage::PartitionTypeUpdate(v).into()
        ),
        checkbox(fl!("password-protected"), create.password_protected)
            .on_toggle(|v| CreateMessage::PasswordProectedUpdate(v).into()),
    ];

    if create.password_protected {
        content = content.push(
            text_input::secure_input("", create_clone.password, None, true)
                .label(fl!("password"))
                .on_input(|v| CreateMessage::PasswordUpdate(v).into()),
        );

        content = content.push(
            text_input::secure_input("", create_clone.confirmed_password, None, true)
                .label(fl!("confirm"))
                .on_input(|v| CreateMessage::ConfirmedPasswordUpdate(v).into()),
        );
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
    let valid_partition_types = get_valid_partition_names(create.table_type.clone());

    let mut content = iced_widget::column![
        caption(fl!("format-partition-description", size = size_pretty)),
        text_input(fl!("volume-name"), create.name.clone())
            .label(fl!("volume-name"))
            .on_input(|t| CreateMessage::NameUpdate(t).into()),
        toggler(create.erase)
            .label(fl!("erase"))
            .on_toggle(|v| CreateMessage::EraseUpdate(v).into()),
        dropdown(
            valid_partition_types,
            Some(create.selected_partitition_type),
            |v| CreateMessage::PartitionTypeUpdate(v).into()
        ),
    ]
    .spacing(12);

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
        slider(min..=max, value, |v| {
            ResizePartitionMessage::SizeUpdate(v as u64).into()
        }),
        labelled_spinner(fl!("new-size"), value_pretty, value, step, min, max, |v| {
            ResizePartitionMessage::SizeUpdate(v as u64).into()
        },),
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

pub fn take_ownership<'a>(state: TakeOwnershipDialog) -> Element<'a, Message> {
    let TakeOwnershipDialog {
        target: _,
        recursive,
        running,
    } = state;

    let mut content = iced_widget::column![
        caption(fl!("take-ownership-warning")),
        checkbox(fl!("take-ownership-recursive"), recursive)
            .on_toggle(|v| TakeOwnershipMessage::RecursiveUpdate(v).into()),
    ]
    .spacing(12);

    if running {
        content = content.push(caption(fl!("working")));
    }

    let mut apply = button::destructive(fl!("take-ownership"));
    if !running {
        apply = apply.on_press(TakeOwnershipMessage::Confirm.into());
    }

    dialog::dialog()
        .title(fl!("take-ownership"))
        .control(content)
        .primary_action(apply)
        .secondary_action(
            button::standard(fl!("cancel")).on_press(TakeOwnershipMessage::Cancel.into()),
        )
        .into()
}

pub fn change_passphrase<'a>(state: ChangePassphraseDialog) -> Element<'a, Message> {
    let ChangePassphraseDialog {
        volume: _,
        current_passphrase,
        new_passphrase,
        confirm_passphrase,
        error,
        running,
    } = state;

    let current_for_input = current_passphrase.clone();
    let new_for_input = new_passphrase.clone();
    let confirm_for_input = confirm_passphrase.clone();

    let mut content = iced_widget::column![
        text_input::secure_input("", current_for_input, None, true)
            .label(fl!("current-passphrase"))
            .on_input(|v| ChangePassphraseMessage::CurrentUpdate(v).into()),
        text_input::secure_input("", new_for_input, None, true)
            .label(fl!("new-passphrase"))
            .on_input(|v| ChangePassphraseMessage::NewUpdate(v).into()),
        text_input::secure_input("", confirm_for_input, None, true)
            .label(fl!("confirm"))
            .on_input(|v| ChangePassphraseMessage::ConfirmUpdate(v).into()),
    ]
    .spacing(12);

    if let Some(err) = error.as_ref() {
        content = content.push(caption(err.clone()));
    }

    if running {
        content = content.push(caption(fl!("working")));
    }

    let mut apply = button::standard(fl!("apply"));
    if !running {
        apply = apply.on_press(ChangePassphraseMessage::Confirm.into());
    }

    dialog::dialog()
        .title(fl!("change-passphrase"))
        .control(content)
        .primary_action(apply)
        .secondary_action(
            button::standard(fl!("cancel")).on_press(ChangePassphraseMessage::Cancel.into()),
        )
        .into()
}

pub fn unlock_encrypted<'a>(state: UnlockEncryptedDialog) -> Element<'a, Message> {
    let mut content = iced_widget::column![
        text_input::secure_input("", state.passphrase.clone(), None, true)
            .label(fl!("passphrase"))
            .on_input(|v| UnlockMessage::PassphraseUpdate(v).into()),
    ]
    .spacing(12);

    if let Some(err) = state.error.as_ref() {
        content = content.push(caption(err.clone()));
    }

    if state.running {
        content = content.push(caption(fl!("working")));
    }

    let mut unlock_button = button::destructive(fl!("unlock-button"));
    if !state.running {
        unlock_button = unlock_button.on_press(UnlockMessage::Confirm.into());
    }

    dialog::dialog()
        .title(fl!("unlock", name = state.partition_name))
        .control(content)
        .primary_action(unlock_button)
        .secondary_action(button::standard(fl!("cancel")).on_press(UnlockMessage::Cancel.into()))
        .into()
}

pub fn format_disk<'a>(state: FormatDiskDialog) -> Element<'a, Message> {
    let erase_options = vec![
        fl!("erase-dont-overwrite-quick").to_string(),
        fl!("erase-overwrite-slow").to_string(),
    ];

    let partitioning_options = vec![
        fl!("partitioning-dos-mbr").to_string(),
        fl!("partitioning-gpt").to_string(),
        fl!("partitioning-none").to_string(),
    ];

    let mut content = iced_widget::column![
        caption_heading(fl!("erase")),
        dropdown(erase_options, Some(state.erase_index), |v| {
            FormatDiskMessage::EraseUpdate(v).into()
        }),
        caption_heading(fl!("partitioning")),
        dropdown(partitioning_options, Some(state.partitioning_index), |v| {
            FormatDiskMessage::PartitioningUpdate(v).into()
        }),
    ]
    .spacing(12);

    if state.running {
        content = content.push(caption(fl!("working")));
    }

    let mut confirm = button::destructive(fl!("format-disk"));
    if !state.running {
        confirm = confirm.on_press(FormatDiskMessage::Confirm.into());
    }

    dialog::dialog()
        .title(fl!("format-disk"))
        .control(content)
        .primary_action(confirm)
        .secondary_action(
            button::standard(fl!("cancel")).on_press(FormatDiskMessage::Cancel.into()),
        )
        .into()
}

pub fn smart_data<'a>(state: SmartDataDialog) -> Element<'a, Message> {
    let mut content = iced_widget::column![]
        .spacing(12)
        .width(cosmic::iced::Length::Fill);

    if let Some(err) = state.error.as_ref() {
        content = content.push(caption(err.clone()));
    }

    if let Some(info) = state.info.as_ref() {
        content = content
            .push(caption_heading(fl!("smart-data-self-tests")))
            .push(caption(format!(
                "{}: {}",
                fl!("smart-type"),
                info.device_type
            )))
            .push(caption(format!(
                "{}: {}",
                fl!("smart-updated"),
                info.updated_at
                    .map(|t| t.to_string())
                    .unwrap_or_else(|| fl!("unknown").to_string())
            )));

        if let Some(temp_c) = info.temperature_c {
            content = content.push(caption(format!(
                "{}: {temp_c} °C",
                fl!("smart-temperature")
            )));
        }

        if let Some(hours) = info.power_on_hours {
            content = content.push(caption(format!("{}: {hours}", fl!("smart-power-on-hours"))));
        }

        if let Some(status) = info.selftest_status.as_ref() {
            content = content.push(caption(format!("{}: {status}", fl!("smart-selftest"))));
        }

        if !info.attributes.is_empty() {
            content = content.push(caption_heading(fl!("details")));
            for (k, v) in &info.attributes {
                content = content.push(caption(format!("{k}: {v}")));
            }
        }
    } else if state.running {
        content = content.push(caption(fl!("working")));
    } else {
        content = content.push(caption(fl!("smart-no-data")));
    }

    let mut refresh = button::standard(fl!("refresh"));
    let mut short = button::standard(fl!("smart-selftest-short"));
    let mut extended = button::standard(fl!("smart-selftest-extended"));
    let mut abort = button::standard(fl!("smart-selftest-abort"));
    let mut close = button::standard(fl!("close"));

    if !state.running {
        refresh = refresh.on_press(SmartDialogMessage::Refresh.into());
        short = short.on_press(SmartDialogMessage::SelfTestShort.into());
        extended = extended.on_press(SmartDialogMessage::SelfTestExtended.into());
        abort = abort.on_press(SmartDialogMessage::AbortSelfTest.into());
        close = close.on_press(SmartDialogMessage::Close.into());
    }

    let controls = iced_widget::column![
        iced_widget::row![refresh, short].spacing(8),
        iced_widget::row![extended, abort].spacing(8),
    ]
    .spacing(8);

    dialog::dialog()
        .title(fl!("smart-data-self-tests"))
        .control(content.push(controls))
        .primary_action(close)
        .into()
}
