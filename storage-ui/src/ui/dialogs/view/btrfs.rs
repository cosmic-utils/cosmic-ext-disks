use cosmic::widget::text::caption;
use cosmic::widget::{button, checkbox, dialog, dropdown, text, text_input};
use cosmic::{Element, iced_widget};

use crate::app::Message;
use crate::fl;
use crate::ui::dialogs::message::{BtrfsCreateSnapshotMessage, BtrfsCreateSubvolumeMessage};
use crate::ui::dialogs::state::{BtrfsCreateSnapshotDialog, BtrfsCreateSubvolumeDialog};
use crate::ui::wizard::{wizard_action_row, wizard_shell};

pub fn create_subvolume<'a>(state: BtrfsCreateSubvolumeDialog) -> Element<'a, Message> {
    let BtrfsCreateSubvolumeDialog {
        mount_point: _,
        block_path: _,
        name,
        running,
        error,
    } = state;

    let mut content = iced_widget::column![
        text_input(fl!("btrfs-subvolume-name"), name)
            .label(fl!("btrfs-subvolume-name"))
            .on_input(|t| BtrfsCreateSubvolumeMessage::NameUpdate(t).into()),
    ]
    .spacing(12);

    if running {
        content = content.push(text(fl!("working")).size(11));
    }

    if let Some(error_msg) = error {
        content = content.push(text(error_msg).size(11));
    }

    let mut create_button = button::standard(fl!("apply"));
    if !running {
        create_button = create_button.on_press(BtrfsCreateSubvolumeMessage::Create.into());
    }

    let footer = wizard_action_row(
        vec![],
        vec![
            button::standard(fl!("cancel"))
                .on_press(BtrfsCreateSubvolumeMessage::Cancel.into())
                .into(),
            create_button.into(),
        ],
    );

    let shell = wizard_shell(
        caption(fl!("btrfs-create-subvolume")).into(),
        content.into(),
        footer,
    );

    dialog::dialog()
        .title(fl!("btrfs-create-subvolume"))
        .control(shell)
        .into()
}

pub fn create_snapshot<'a>(state: BtrfsCreateSnapshotDialog) -> Element<'a, Message> {
    let BtrfsCreateSnapshotDialog {
        mount_point: _,
        block_path: _,
        subvolumes,
        selected_source_index,
        snapshot_name,
        read_only,
        running,
        error,
    } = state;

    // Create dropdown options from subvolumes
    let options: Vec<String> = subvolumes.iter().map(|s| s.path.clone()).collect();

    let mut content = iced_widget::column![
        caption(fl!("btrfs-source-subvolume")),
        dropdown(options, Some(selected_source_index), |idx| {
            BtrfsCreateSnapshotMessage::SourceIndexUpdate(idx).into()
        })
        .width(cosmic::iced::Length::Fill),
        text_input(fl!("btrfs-snapshot-name"), snapshot_name)
            .label(fl!("btrfs-snapshot-name"))
            .on_input(|t| BtrfsCreateSnapshotMessage::NameUpdate(t).into()),
        checkbox(fl!("btrfs-read-only"), read_only)
            .on_toggle(|v| BtrfsCreateSnapshotMessage::ReadOnlyUpdate(v).into()),
    ]
    .spacing(12);

    if running {
        content = content.push(text(fl!("working")).size(11));
    }

    if let Some(error_msg) = error {
        content = content.push(text(error_msg).size(11));
    }

    let mut create_button = button::standard(fl!("apply"));
    if !running {
        create_button = create_button.on_press(BtrfsCreateSnapshotMessage::Create.into());
    }

    let footer = wizard_action_row(
        vec![],
        vec![
            button::standard(fl!("cancel"))
                .on_press(BtrfsCreateSnapshotMessage::Cancel.into())
                .into(),
            create_button.into(),
        ],
    );

    let shell = wizard_shell(
        caption(fl!("btrfs-create-snapshot")).into(),
        content.into(),
        footer,
    );

    dialog::dialog()
        .title(fl!("btrfs-create-snapshot"))
        .control(shell)
        .into()
}
