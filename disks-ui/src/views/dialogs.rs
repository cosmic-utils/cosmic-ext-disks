use super::volumes::CreateMessage;
use crate::app::Message;
use crate::fl;
use crate::utils::labelled_spinner;
use cosmic::{
    Element, iced_widget,
    widget::{button, checkbox, dialog, dropdown, slider, text_input, toggler},
};
use disks_dbus::{bytes_to_pretty, get_valid_partition_names};
use disks_dbus::{ CreatePartitionInfo};
use std::borrow::Cow;

pub fn confirmation<'a>(
    title: impl Into<Cow<'a, str>>,
    prompt: impl Into<Cow<'a, str>>,
    ok_message: Message,
    cancel_message: Option<Message>,
) -> Element<'a, Message> {
    let mut dialog = dialog::dialog()
        .title(title)
        .body(prompt)
        .primary_action(button::destructive(fl!("ok")).on_press(ok_message));

    if let Some(c) = cancel_message {
        dialog = dialog.secondary_action(button::standard(fl!("cancel")).on_press(c))
    };

    dialog.into()
}

pub fn create_partition<'a>(create: CreatePartitionInfo) -> Element<'a, Message> {
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
            |v| {
                CreateMessage::SizeUpdate(v as u64).into()
            }
        ),
        labelled_spinner(
            fl!("free-space"),
            free_pretty,
            free,
            step,
            0.,
            len,
            move |v| {

                CreateMessage::SizeUpdate((len - v) as u64).into()
            }
        ),
        toggler(create_clone.erase)
            .label(fl!("erase"))
            .on_toggle(|v| CreateMessage::EraseUpdate(v).into()),
        dropdown(valid_partition_types,
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

    // if create.can_continue
    //{
    continue_button = continue_button.on_press(CreateMessage::Partition(create).into());
    //}

    dialog::dialog()
        .title(fl!("create-partition"))
        .control(content.spacing(20.))
        .primary_action(continue_button)
        .secondary_action(button::standard(fl!("cancel")).on_press(CreateMessage::Cancel.into()))
        .into()
}
