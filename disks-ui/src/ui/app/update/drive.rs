use crate::fl;
use crate::ui::dialogs::message::FormatDiskMessage;
use crate::ui::dialogs::state::{FormatDiskDialog, ShowDialog, SmartDataDialog};
use cosmic::app::Task;
use disks_dbus::DriveModel;

use super::super::message::Message;
use super::super::state::AppModel;

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
            let selected = drive.block_path.clone();
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
                    Ok(drives) => Message::UpdateNav(drives, Some(selected.clone())).into(),
                    Err(e) => Message::Dialog(Box::new(ShowDialog::Info {
                        title: fl!("app-title"),
                        body: format!("{e:#}"),
                    }))
                    .into(),
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

    Task::perform(
        async move {
            let res = drive.remove().await;
            let drives = DriveModel::get_drives().await.ok();
            (res, drives)
        },
        |(res, drives)| match res {
            Ok(()) => match drives {
                Some(drives) => Message::UpdateNav(drives, None).into(),
                None => Message::None.into(),
            },
            Err(e) => Message::Dialog(Box::new(ShowDialog::Info {
                title: fl!("app-title"),
                body: e.to_string(),
            }))
            .into(),
        },
    )
}

pub(super) fn power_off(app: &mut AppModel) -> Task<Message> {
    let Some(drive) = app.nav.active_data::<DriveModel>().cloned() else {
        return Task::none();
    };

    Task::perform(
        async move {
            let res = drive.power_off().await;
            let drives = DriveModel::get_drives().await.ok();
            (res, drives)
        },
        |(res, drives)| match res {
            Ok(()) => match drives {
                Some(drives) => Message::UpdateNav(drives, None).into(),
                None => Message::None.into(),
            },
            Err(e) => Message::Dialog(Box::new(ShowDialog::Info {
                title: fl!("app-title"),
                body: e.to_string(),
            }))
            .into(),
        },
    )
}

pub(super) fn format(app: &mut AppModel) {
    let Some(drive) = app.nav.active_data::<DriveModel>().cloned() else {
        return;
    };

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

    Task::perform(
        async move { drive.standby_now().await.map_err(|e| e.to_string()) },
        |res| {
            Message::Dialog(Box::new(ShowDialog::Info {
                title: fl!("app-title"),
                body: match res {
                    Ok(()) => "Standby requested.".to_string(),
                    Err(e) => e,
                },
            }))
            .into()
        },
    )
}

pub(super) fn wakeup(app: &mut AppModel) -> Task<Message> {
    let Some(drive) = app.nav.active_data::<DriveModel>().cloned() else {
        return Task::none();
    };

    Task::perform(
        async move { drive.wakeup().await.map_err(|e| e.to_string()) },
        |res| {
            Message::Dialog(Box::new(ShowDialog::Info {
                title: fl!("app-title"),
                body: match res {
                    Ok(()) => "Wake-up requested.".to_string(),
                    Err(e) => e,
                },
            }))
            .into()
        },
    )
}
