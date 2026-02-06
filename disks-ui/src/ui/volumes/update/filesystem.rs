use cosmic::Task;

use crate::app::Message;
use crate::fl;
use crate::ui::dialogs::message::EditFilesystemLabelMessage;
use crate::ui::dialogs::state::{
    ConfirmActionDialog, EditFilesystemLabelDialog, FilesystemTarget, ShowDialog,
};
use crate::ui::error::{UiErrorContext, log_error_and_show_dialog};
use disks_dbus::DriveModel;

use super::super::{VolumesControl, VolumesControlMessage};

pub(super) fn open_edit_filesystem_label(
    control: &mut VolumesControl,
    dialog: &mut Option<ShowDialog>,
) -> Task<cosmic::Action<Message>> {
    if dialog.is_some() {
        return Task::none();
    }

    let target = if let Some(node) = control.selected_volume_node() {
        if !node.can_mount() {
            return Task::none();
        }

        FilesystemTarget::Node(node.clone())
    } else {
        let Some(segment) = control.segments.get(control.selected_segment) else {
            return Task::none();
        };
        let Some(volume) = segment.volume.clone() else {
            return Task::none();
        };

        if !volume.can_mount() {
            return Task::none();
        }

        FilesystemTarget::Volume(volume)
    };

    *dialog = Some(ShowDialog::EditFilesystemLabel(EditFilesystemLabelDialog {
        target,
        label: String::new(),
        running: false,
    }));

    Task::none()
}

pub(super) fn edit_filesystem_label_message(
    _control: &mut VolumesControl,
    msg: EditFilesystemLabelMessage,
    dialog: &mut Option<ShowDialog>,
) -> Task<cosmic::Action<Message>> {
    let Some(ShowDialog::EditFilesystemLabel(state)) = dialog.as_mut() else {
        return Task::none();
    };

    match msg {
        EditFilesystemLabelMessage::LabelUpdate(label) => state.label = label,
        EditFilesystemLabelMessage::Cancel => {
            return Task::done(Message::CloseDialog.into());
        }
        EditFilesystemLabelMessage::Confirm => {
            if state.running {
                return Task::none();
            }

            state.running = true;
            let target = state.target.clone();
            let label = state.label.clone();

            return Task::perform(
                async move {
                    match target {
                        FilesystemTarget::Volume(v) => v.edit_filesystem_label(label).await?,
                        FilesystemTarget::Node(n) => n.edit_filesystem_label(&label).await?,
                    }
                    DriveModel::get_drives().await
                },
                |result| match result {
                    Ok(drives) => Message::UpdateNav(drives, None).into(),
                    Err(e) => {
                        let ctx = UiErrorContext::new("edit_filesystem_label");
                        log_error_and_show_dialog(fl!("edit-filesystem").to_string(), e, ctx).into()
                    }
                },
            );
        }
    }

    Task::none()
}

pub(super) fn open_check_filesystem(
    control: &mut VolumesControl,
    dialog: &mut Option<ShowDialog>,
) -> Task<cosmic::Action<Message>> {
    if dialog.is_some() {
        return Task::none();
    }

    let target = if let Some(node) = control.selected_volume_node() {
        if !node.can_mount() {
            return Task::none();
        }
        FilesystemTarget::Node(node.clone())
    } else {
        let Some(segment) = control.segments.get(control.selected_segment) else {
            return Task::none();
        };
        let Some(volume) = segment.volume.clone() else {
            return Task::none();
        };
        if !volume.can_mount() {
            return Task::none();
        }
        FilesystemTarget::Volume(volume)
    };

    *dialog = Some(ShowDialog::ConfirmAction(ConfirmActionDialog {
        title: fl!("check-filesystem").to_string(),
        body: fl!("check-filesystem-warning").to_string(),
        target,
        ok_message: VolumesControlMessage::CheckFilesystemConfirm.into(),
        running: false,
    }));

    Task::none()
}

pub(super) fn check_filesystem_confirm(
    _control: &mut VolumesControl,
    dialog: &mut Option<ShowDialog>,
) -> Task<cosmic::Action<Message>> {
    let Some(ShowDialog::ConfirmAction(state)) = dialog.as_mut() else {
        return Task::none();
    };

    if state.running {
        return Task::none();
    }
    state.running = true;

    let target = state.target.clone();
    Task::perform(
        async move {
            match target {
                FilesystemTarget::Volume(v) => v.check_filesystem().await?,
                FilesystemTarget::Node(n) => n.check_filesystem().await?,
            }
            DriveModel::get_drives().await
        },
        |result| match result {
            Ok(drives) => Message::UpdateNav(drives, None).into(),
            Err(e) => {
                let ctx = UiErrorContext::new("check_filesystem");
                log_error_and_show_dialog(fl!("check-filesystem").to_string(), e, ctx).into()
            }
        },
    )
}

pub(super) fn open_repair_filesystem(
    control: &mut VolumesControl,
    dialog: &mut Option<ShowDialog>,
) -> Task<cosmic::Action<Message>> {
    if dialog.is_some() {
        return Task::none();
    }

    let target = if let Some(node) = control.selected_volume_node() {
        if !node.can_mount() {
            return Task::none();
        }
        FilesystemTarget::Node(node.clone())
    } else {
        let Some(segment) = control.segments.get(control.selected_segment) else {
            return Task::none();
        };
        let Some(volume) = segment.volume.clone() else {
            return Task::none();
        };

        if !volume.can_mount() {
            return Task::none();
        }

        FilesystemTarget::Volume(volume)
    };

    *dialog = Some(ShowDialog::ConfirmAction(ConfirmActionDialog {
        title: fl!("repair-filesystem").to_string(),
        body: fl!("repair-filesystem-warning").to_string(),
        target,
        ok_message: VolumesControlMessage::RepairFilesystemConfirm.into(),
        running: false,
    }));

    Task::none()
}

pub(super) fn repair_filesystem_confirm(
    _control: &mut VolumesControl,
    dialog: &mut Option<ShowDialog>,
) -> Task<cosmic::Action<Message>> {
    let Some(ShowDialog::ConfirmAction(state)) = dialog.as_mut() else {
        return Task::none();
    };

    if state.running {
        return Task::none();
    }
    state.running = true;

    let target = state.target.clone();
    Task::perform(
        async move {
            match target {
                FilesystemTarget::Volume(v) => v.repair_filesystem().await?,
                FilesystemTarget::Node(n) => n.repair_filesystem().await?,
            }
            DriveModel::get_drives().await
        },
        |result| match result {
            Ok(drives) => Message::UpdateNav(drives, None).into(),
            Err(e) => {
                let ctx = UiErrorContext::new("repair_filesystem");
                log_error_and_show_dialog(fl!("repair-filesystem").to_string(), e, ctx).into()
            }
        },
    )
}
