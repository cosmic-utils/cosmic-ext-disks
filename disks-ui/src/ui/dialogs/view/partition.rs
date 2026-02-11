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
use cosmic::{
    Element, iced_widget,
    widget::text::caption,
    widget::{button, checkbox, dialog, dropdown, slider, text_input},
};
use disks_dbus::{PartitionTypeInfo, bytes_to_pretty, get_valid_partition_names};

pub fn create_partition<'a>(state: CreatePartitionDialog) -> Element<'a, Message> {
    let CreatePartitionDialog {
        info: create,
        running,
        error,
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
            |v| { CreateMessage::SizeUpdate(v as u64).into() },
        ),
        labelled_spinner(
            fl!("free-space"),
            free_pretty,
            free,
            step,
            0.,
            len,
            move |v| { CreateMessage::SizeUpdate((len - v) as u64).into() },
        ),
        checkbox(fl!("overwrite-data-slow"), create_clone.erase)
            .on_toggle(|v| CreateMessage::EraseUpdate(v).into()),
        dropdown(
            valid_partition_types,
            Some(create_clone.selected_partition_type_index),
            |v| CreateMessage::PartitionTypeUpdate(v).into(),
        ),
        checkbox(fl!("password-protected"), create.password_protected)
            .on_toggle(|v| CreateMessage::PasswordProtectedUpdate(v).into()),
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
    let valid_partition_types = get_valid_partition_names(create.table_type.clone());

    let mut content = iced_widget::column![
        caption(fl!("format-partition-description", size = size_pretty)),
        text_input(fl!("volume-name"), create.name.clone())
            .label(fl!("volume-name"))
            .on_input(|t| CreateMessage::NameUpdate(t).into()),
        checkbox(fl!("overwrite-data-slow"), create.erase)
            .on_toggle(|v| CreateMessage::EraseUpdate(v).into()),
        dropdown(
            valid_partition_types,
            Some(create.selected_partition_type_index),
            |v| CreateMessage::PartitionTypeUpdate(v).into(),
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
