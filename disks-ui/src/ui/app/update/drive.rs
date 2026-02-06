use crate::fl;
use crate::ui::dialogs::message::FormatDiskMessage;
use crate::ui::dialogs::state::{FormatDiskDialog, ShowDialog, SmartDataDialog};
use crate::ui::error::{UiErrorContext, log_error_and_show_dialog};
use cosmic::app::Task;
use disks_dbus::DriveModel;

use crate::ui::app::message::Message;
use crate::ui::app::state::AppModel;

pub(super) fn format_disk(app: &mut AppModel, msg: FormatDiskMessage) -> Task<Message> {
    let Some(ShowDialog::FormatDisk(state)) = app.dialog.as_mut() else {
        return Task::none();
    };

    match msg {
        FormatDiskMessage::EraseUpdate(v) => state.erase_index = v,
        FormatDiskMessage::PartitioningUpdate(v) => state.partitioning_index = v,
        FormatDiskMessage::Cancel => {
            app.dialog = None;
        }
        FormatDiskMessage::Confirm => {
            if state.running {
                return Task::none();
            }

            state.running = true;

            let drive = state.drive.clone();
            let block_path = drive.block_path.clone();
            let drive_path = drive.path.clone();
            let erase = state.erase_index == 1;
            let format_type = match state.partitioning_index {
                0 => "dos",
                1 => "gpt",
                _ => "empty",
            };

            return Task::perform(
                async move {
                    drive.format_disk(format_type, erase).await?;
                    DriveModel::get_drives().await
                },
                move |res| match res {
                    Ok(drives) => Message::UpdateNav(drives, Some(block_path.clone())).into(),
                    Err(e) => {
                        let ctx = UiErrorContext {
                            operation: "format_disk",
                            object_path: Some(drive_path.as_str()),
                            device: Some(block_path.as_str()),
                            drive_path: Some(drive_path.as_str()),
                        };
                        log_error_and_show_dialog(fl!("format-disk-failed"), e, ctx).into()
                    }
                },
            );
        }
    };

    Task::none()
}

pub(super) fn eject(app: &mut AppModel) -> Task<Message> {
    let Some(drive) = app.nav.active_data::<DriveModel>().cloned() else {
        return Task::none();
    };

    eject_drive(drive)
}

pub(super) fn eject_drive(drive: DriveModel) -> Task<Message> {
    let drive_path = drive.path.clone();
    let block_path = drive.block_path.clone();

    Task::perform(
        async move {
            let res = drive.remove().await;
            let drives = DriveModel::get_drives().await.ok();
            (res, drives)
        },
        move |(res, drives)| match res {
            Ok(()) => match drives {
                Some(drives) => Message::UpdateNav(drives, None).into(),
                None => Message::None.into(),
            },
            Err(e) => {
                let ctx = UiErrorContext {
                    operation: "eject_or_remove",
                    object_path: Some(drive_path.as_str()),
                    device: Some(block_path.as_str()),
                    drive_path: Some(drive_path.as_str()),
                };
                log_error_and_show_dialog(fl!("eject-failed"), e, ctx).into()
            }
        },
    )
}

pub(super) fn power_off(app: &mut AppModel) -> Task<Message> {
    let Some(drive) = app.nav.active_data::<DriveModel>().cloned() else {
        return Task::none();
    };

    power_off_drive(drive)
}

pub(super) fn power_off_drive(drive: DriveModel) -> Task<Message> {
    let drive_path = drive.path.clone();
    let block_path = drive.block_path.clone();

    Task::perform(
        async move {
            let res = drive.power_off().await;
            let drives = DriveModel::get_drives().await.ok();
            (res, drives)
        },
        move |(res, drives)| match res {
            Ok(()) => match drives {
                Some(drives) => Message::UpdateNav(drives, None).into(),
                None => Message::None.into(),
            },
            Err(e) => {
                let ctx = UiErrorContext {
                    operation: "power_off",
                    object_path: Some(drive_path.as_str()),
                    device: Some(block_path.as_str()),
                    drive_path: Some(drive_path.as_str()),
                };
                log_error_and_show_dialog(fl!("power-off-failed"), e, ctx).into()
            }
        },
    )
}

pub(super) fn format(app: &mut AppModel) {
    let Some(drive) = app.nav.active_data::<DriveModel>().cloned() else {
        return;
    };

    format_for(app, drive)
}

pub(super) fn format_for(app: &mut AppModel, drive: DriveModel) {
    let partitioning_index = match drive.partition_table_type.as_deref() {
        Some("dos") => 0,
        Some("gpt") => 1,
        _ => 2,
    };

    app.dialog = Some(ShowDialog::FormatDisk(FormatDiskDialog {
        drive,
        erase_index: 0,
        partitioning_index,
        running: false,
    }));
}

pub(super) fn smart_data(app: &mut AppModel) -> Task<Message> {
    let Some(drive) = app.nav.active_data::<DriveModel>().cloned() else {
        return Task::none();
    };

    smart_data_for(app, drive)
}

pub(super) fn smart_data_for(app: &mut AppModel, drive: DriveModel) -> Task<Message> {
    app.dialog = Some(ShowDialog::SmartData(SmartDataDialog {
        drive: drive.clone(),
        running: true,
        info: None,
        error: None,
    }));

    Task::perform(
        async move { drive.smart_info().await.map_err(|e| e.to_string()) },
        |res| {
            Message::SmartDialog(crate::ui::dialogs::message::SmartDialogMessage::Loaded(res))
                .into()
        },
    )
}

pub(super) fn standby_now(app: &mut AppModel) -> Task<Message> {
    let Some(drive) = app.nav.active_data::<DriveModel>().cloned() else {
        return Task::none();
    };

    standby_now_drive(drive)
}

pub(super) fn standby_now_drive(drive: DriveModel) -> Task<Message> {
    let drive_path = drive.path.clone();
    let device = drive.block_path.clone();

    Task::perform(
        async move { drive.standby_now().await },
        move |res| match res {
            Ok(()) => Message::Dialog(Box::new(ShowDialog::Info {
                title: fl!("app-title"),
                body: "Standby requested.".to_string(),
            }))
            .into(),
            Err(e) => {
                let ctx = UiErrorContext {
                    operation: "standby_now",
                    object_path: Some(drive_path.as_str()),
                    device: Some(device.as_str()),
                    drive_path: Some(drive_path.as_str()),
                };
                log_error_and_show_dialog(fl!("standby-failed"), e, ctx).into()
            }
        },
    )
}

pub(super) fn wakeup(app: &mut AppModel) -> Task<Message> {
    let Some(drive) = app.nav.active_data::<DriveModel>().cloned() else {
        return Task::none();
    };

    wakeup_drive(drive)
}

pub(super) fn wakeup_drive(drive: DriveModel) -> Task<Message> {
    let drive_path = drive.path.clone();
    let device = drive.block_path.clone();

    Task::perform(async move { drive.wakeup().await }, move |res| match res {
        Ok(()) => Message::Dialog(Box::new(ShowDialog::Info {
            title: fl!("app-title"),
            body: "Wake-up requested.".to_string(),
        }))
        .into(),
        Err(e) => {
            let ctx = UiErrorContext {
                operation: "wakeup",
                object_path: Some(drive_path.as_str()),
                device: Some(device.as_str()),
                drive_path: Some(drive_path.as_str()),
            };
            log_error_and_show_dialog(fl!("wake-up-failed"), e, ctx).into()
        }
    })
}
