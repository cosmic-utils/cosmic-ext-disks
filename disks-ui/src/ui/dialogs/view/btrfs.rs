use cosmic::widget::{button, dialog, text_input, text};
use cosmic::{iced_widget, Element};

use crate::app::Message;
use crate::fl;
use crate::ui::dialogs::message::BtrfsCreateSubvolumeMessage;
use crate::ui::dialogs::state::BtrfsCreateSubvolumeDialog;

pub fn create_subvolume<'a>(state: BtrfsCreateSubvolumeDialog) -> Element<'a, Message> {
    let BtrfsCreateSubvolumeDialog {
        mount_point: _,
        name,
        running,
        error,
    } = state;

    let mut content = iced_widget::column![text_input(fl!("btrfs-subvolume-name"), name)
        .label(fl!("btrfs-subvolume-name"))
        .on_input(|t| BtrfsCreateSubvolumeMessage::NameUpdate(t).into()),]
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

    dialog::dialog()
        .title(fl!("btrfs-create-subvolume"))
        .control(content)
        .primary_action(create_button)
        .secondary_action(
            button::standard(fl!("cancel")).on_press(BtrfsCreateSubvolumeMessage::Cancel.into()),
        )
        .into()
}
