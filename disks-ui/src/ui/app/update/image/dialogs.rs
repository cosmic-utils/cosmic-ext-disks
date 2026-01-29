use crate::fl;
use crate::ui::dialogs::message::{
    AttachDiskImageDialogMessage, AttachDiskResult, ImageOperationDialogMessage,
    NewDiskImageDialogMessage,
};
use crate::ui::dialogs::state::{AttachDiskImageDialog, NewDiskImageDialog, ShowDialog};
use cosmic::app::Task;
use disks_dbus::DriveModel;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use tokio::fs::OpenOptions;

use super::super::super::message::Message;
use super::super::super::state::AppModel;
use super::ops::run_image_operation;

pub(super) fn new_disk_image(app: &mut AppModel) {
    app.dialog = Some(ShowDialog::NewDiskImage(Box::new(NewDiskImageDialog {
        path: String::new(),
        size_bytes: 16 * 1024 * 1024,
        running: false,
        error: None,
    })));
}

pub(super) fn attach_disk(app: &mut AppModel) {
    app.dialog = Some(ShowDialog::AttachDiskImage(Box::new(
        AttachDiskImageDialog {
            path: String::new(),
            running: false,
            error: None,
        },
    )));
}

pub(super) fn new_disk_image_dialog(
    app: &mut AppModel,
    msg: NewDiskImageDialogMessage,
) -> Task<Message> {
    let Some(ShowDialog::NewDiskImage(state)) = app.dialog.as_mut() else {
        return Task::none();
    };

    match msg {
        NewDiskImageDialogMessage::PathUpdate(v) => state.path = v,
        NewDiskImageDialogMessage::SizeUpdate(v) => state.size_bytes = v,
        NewDiskImageDialogMessage::Cancel => {
            if !state.running {
                app.dialog = None;
            }
        }
        NewDiskImageDialogMessage::Create => {
            if state.running {
                return Task::none();
            }

            let path = state.path.clone();
            let size_bytes = state.size_bytes;

            state.running = true;
            state.error = None;

            return Task::perform(
                async move {
                    if path.trim().is_empty() {
                        anyhow::bail!("Destination path is required");
                    }

                    let file = OpenOptions::new()
                        .write(true)
                        .create_new(true)
                        .open(&path)
                        .await?;
                    file.set_len(size_bytes).await?;
                    Ok(())
                },
                |res: anyhow::Result<()>| {
                    Message::NewDiskImageDialog(NewDiskImageDialogMessage::Complete(
                        res.map_err(|e| e.to_string()),
                    ))
                    .into()
                },
            );
        }
        NewDiskImageDialogMessage::Complete(res) => {
            state.running = false;
            match res {
                Ok(()) => {
                    app.dialog = Some(ShowDialog::Info {
                        title: fl!("app-title"),
                        body: "Disk image created.".to_string(),
                    });
                }
                Err(e) => {
                    tracing::error!(%e, "new disk image dialog error");
                    state.error = Some(e);
                }
            }
        }
    }

    Task::none()
}

pub(super) fn attach_disk_image_dialog(
    app: &mut AppModel,
    msg: AttachDiskImageDialogMessage,
) -> Task<Message> {
    let Some(ShowDialog::AttachDiskImage(state)) = app.dialog.as_mut() else {
        return Task::none();
    };

    match msg {
        AttachDiskImageDialogMessage::PathUpdate(v) => state.path = v,
        AttachDiskImageDialogMessage::Cancel => {
            if !state.running {
                app.dialog = None;
            }
        }
        AttachDiskImageDialogMessage::Attach => {
            if state.running {
                return Task::none();
            }

            let path = state.path.clone();
            state.running = true;
            state.error = None;

            return Task::perform(
                async move {
                    if path.trim().is_empty() {
                        anyhow::bail!("Image file path is required");
                    }

                    let block_object_path = disks_dbus::loop_setup(&path).await?;

                    match disks_dbus::mount_filesystem(block_object_path.clone()).await {
                        Ok(()) => Ok(AttachDiskResult {
                            mounted: true,
                            message: "Attached and mounted image.".to_string(),
                        }),
                        Err(e) => {
                            tracing::warn!(%e, "attach image: mount attempt failed");
                            Ok(AttachDiskResult {
                                mounted: false,
                                message: "Attached image. If it contains partitions, select and mount them from the main view.".to_string(),
                            })
                        }
                    }
                },
                |res: anyhow::Result<AttachDiskResult>| {
                    Message::AttachDiskImageDialog(AttachDiskImageDialogMessage::Complete(
                        res.map_err(|e| e.to_string()),
                    ))
                    .into()
                },
            );
        }
        AttachDiskImageDialogMessage::Complete(res) => {
            state.running = false;
            match res {
                Ok(r) => {
                    app.dialog = Some(ShowDialog::Info {
                        title: fl!("app-title"),
                        body: r.message,
                    });

                    return Task::perform(
                        async { DriveModel::get_drives().await.ok() },
                        |drives| match drives {
                            None => Message::None.into(),
                            Some(drives) => Message::UpdateNav(drives, None).into(),
                        },
                    );
                }
                Err(e) => {
                    tracing::error!(%e, "attach disk image dialog error");
                    state.error = Some(e);
                }
            }
        }
    }

    Task::none()
}

pub(super) fn image_operation_dialog(
    app: &mut AppModel,
    msg: ImageOperationDialogMessage,
) -> Task<Message> {
    let Some(ShowDialog::ImageOperation(state)) = app.dialog.as_mut() else {
        return Task::none();
    };

    match msg {
        ImageOperationDialogMessage::PathUpdate(v) => state.image_path = v,
        ImageOperationDialogMessage::CancelOperation => {
            if state.running {
                if let Some(flag) = app.image_op_cancel.as_ref() {
                    flag.store(true, Ordering::SeqCst);
                }
            } else {
                app.dialog = None;
            }
        }
        ImageOperationDialogMessage::Start => {
            if state.running {
                return Task::none();
            }

            let image_path = state.image_path.clone();
            if image_path.trim().is_empty() {
                let e = "Image path is required".to_string();
                tracing::warn!(%e, "image operation dialog validation error");
                state.error = Some(e);
                return Task::none();
            }

            let kind = state.kind;
            let drive = state.drive.clone();
            let partition = state.partition.clone();

            let cancel = Arc::new(AtomicBool::new(false));
            app.image_op_cancel = Some(cancel.clone());

            state.running = true;
            state.error = None;

            return Task::perform(
                async move { run_image_operation(kind, drive, partition, image_path, cancel).await },
                |res: anyhow::Result<()>| {
                    Message::ImageOperationDialog(ImageOperationDialogMessage::Complete(
                        res.map_err(|e| e.to_string()),
                    ))
                    .into()
                },
            );
        }
        ImageOperationDialogMessage::Complete(res) => {
            state.running = false;
            app.image_op_cancel = None;

            match res {
                Ok(()) => {
                    app.dialog = Some(ShowDialog::Info {
                        title: fl!("app-title"),
                        body: fl!("ok"),
                    });

                    return Task::perform(
                        async { DriveModel::get_drives().await.ok() },
                        |drives| match drives {
                            None => Message::None.into(),
                            Some(drives) => Message::UpdateNav(drives, None).into(),
                        },
                    );
                }
                Err(e) => {
                    tracing::error!(%e, "image operation dialog error");
                    state.error = Some(e);
                }
            }
        }
    }

    Task::none()
}
