use crate::app::Message;
use crate::fl;
use crate::ui::dialogs::message::{FormatDiskMessage, SmartDialogMessage};
use crate::ui::dialogs::state::{FormatDiskDialog, SmartDataDialog};
use cosmic::{
    Element, iced_widget,
    widget::text::{caption, caption_heading},
    widget::{button, dialog, dropdown},
};

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
        .spacing(6)
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
