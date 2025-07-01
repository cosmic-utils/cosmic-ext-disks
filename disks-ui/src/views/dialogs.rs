use super::volumes::CreateMessage;
use crate::app::Message;
use crate::utils::labelled_spinner;
use cosmic::{
    Element, iced_widget,
    widget::{button, checkbox, dialog, dropdown, slider, text_input, toggler},
};
use hardware::bytes_to_pretty;
use hardware::{COMMON_PARTITION_NAMES, CreatePartitionInfo};
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
        .primary_action(button::destructive("Ok").on_press(ok_message.into()));

    match cancel_message {
        Some(c) => dialog = dialog.secondary_action(button::standard("Cancel").on_press(c.into())),
        None => {}
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
    let step = hardware::get_step(&create.size);

    println!("step: {}", step);

    let create_clone = create.clone();

    let mut content = iced_widget::column![
        text_input("Volume name", create_clone.name)
            .label("Volume Name")
            .on_input(|t| CreateMessage::NameUpdate(t).into()),
        slider(0.0..=len, size, |v| CreateMessage::SizeUpdate(v as u64)
            .into()),
        labelled_spinner("Partition Size", size_pretty, size, step, 0., len, |v| {
            println!("value: {}", v);
            CreateMessage::SizeUpdate(v as u64).into()
        }),
        labelled_spinner("Free Space", free_pretty, free, step, 0., len, move |v| {
            println!("value: {}", v);

            CreateMessage::SizeUpdate((len - v) as u64).into()
        }),
        toggler(create_clone.erase)
            .label("Erase")
            .on_toggle(|v| CreateMessage::EraseUpdate(v).into()),
        dropdown(
            &*COMMON_PARTITION_NAMES,
            Some(create_clone.selected_partitition_type),
            |v| CreateMessage::PartitionTypeUpdate(v).into()
        ),
        checkbox("Password Protected", create.password_protected)
            .on_toggle(|v| CreateMessage::PasswordProectedUpdate(v).into()),
    ];

    if create.password_protected {
        content = content.push(
            text_input::secure_input("", create_clone.password, None, true)
                .label("Password")
                .on_input(|v| CreateMessage::PasswordUpdate(v).into()),
        );

        content = content.push(
            text_input::secure_input("", create_clone.confirmed_password, None, true)
                .label("Confirm")
                .on_input(|v| CreateMessage::ConfirmedPasswordUpdate(v).into()),
        );
    }

    let mut continue_button = button::destructive("Continue");

    // if create.can_continue
    //{
    continue_button = continue_button.on_press(CreateMessage::Partition(create).into());
    //}

    dialog::dialog()
        .title("Create Partition")
        .control(content.spacing(20.))
        .primary_action(continue_button)
        .secondary_action(button::standard("Cancel").on_press(CreateMessage::Cancel.into()))
        .into()
}
