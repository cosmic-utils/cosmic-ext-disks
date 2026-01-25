use super::volumes::{CreateMessage, UnlockMessage};
use crate::app::CreatePartitionDialog;
use crate::app::Message;
use crate::app::SmartDataDialog;
use crate::app::SmartDialogMessage;
use crate::app::UnlockEncryptedDialog;
use crate::app::{FormatDiskDialog, FormatDiskMessage};
use crate::fl;
use crate::utils::labelled_spinner;
use cosmic::{
    Element, iced_widget,
    widget::text::{caption, caption_heading},
    widget::{button, checkbox, dialog, dropdown, slider, text_input, toggler},
};
use disks_dbus::{bytes_to_pretty, get_valid_partition_names};
use std::borrow::Cow;

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
