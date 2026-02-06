use crate::app::Message;
use crate::fl;
use crate::ui::app::message::ImagePathPickerKind;
use crate::ui::dialogs::message::{
    AttachDiskImageDialogMessage, ImageOperationDialogMessage, NewDiskImageDialogMessage,
};
use crate::ui::dialogs::state::{
    AttachDiskImageDialog, ImageOperationDialog, ImageOperationKind, NewDiskImageDialog,
};
use crate::utils::labelled_spinner;
use cosmic::{
    Element,
    iced::{Alignment, Length},
    iced_widget,
    widget::text::caption,
    widget::{button, dialog},
};
use disks_dbus::bytes_to_pretty;

pub fn new_disk_image<'a>(state: NewDiskImageDialog) -> Element<'a, Message> {
    let size_pretty = bytes_to_pretty(&state.size_bytes, false);
    let step = disks_dbus::get_step(&state.size_bytes);

    let path_label = if state.path.trim().is_empty() {
        fl!("no-file-selected")
    } else {
        state.path.clone()
    };

    let path_row = iced_widget::row![
        caption(path_label).width(Length::Fill),
        button::standard(fl!("choose-path")).on_press(Message::OpenImagePathPicker(
            ImagePathPickerKind::NewDiskImage
        ))
    ]
    .align_y(Alignment::Center)
    .spacing(12);

    let mut content = iced_widget::column![
        caption(fl!("image-destination-path")),
        path_row,
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
    let path_label = if state.path.trim().is_empty() {
        fl!("no-file-selected")
    } else {
        state.path.clone()
    };

    let path_row = iced_widget::row![
        caption(path_label).width(Length::Fill),
        button::standard(fl!("choose-path")).on_press(Message::OpenImagePathPicker(
            ImagePathPickerKind::AttachDiskImage
        ))
    ]
    .align_y(Alignment::Center)
    .spacing(12);

    let mut content = iced_widget::column![caption(fl!("image-file-path")), path_row,].spacing(12);

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

    let path_text = if state.image_path.trim().is_empty() {
        fl!("no-file-selected")
    } else {
        state.image_path.clone()
    };

    let picker_kind = match state.kind {
        ImageOperationKind::CreateFromDrive | ImageOperationKind::CreateFromPartition => {
            ImagePathPickerKind::ImageOperationCreate
        }
        ImageOperationKind::RestoreToDrive | ImageOperationKind::RestoreToPartition => {
            ImagePathPickerKind::ImageOperationRestore
        }
    };

    let path_row = iced_widget::row![
        caption(path_text).width(Length::Fill),
        button::standard(fl!("choose-path")).on_press(Message::OpenImagePathPicker(picker_kind))
    ]
    .align_y(Alignment::Center)
    .spacing(12);

    content = content.push(caption(path_label)).push(path_row);

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
