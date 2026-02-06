use cosmic::Task;

use crate::app::Message;
use crate::fl;
use crate::ui::dialogs::message::{
    ChangePassphraseMessage, EditEncryptionOptionsMessage, TakeOwnershipMessage, UnlockMessage,
};
use crate::ui::dialogs::state::{
    ChangePassphraseDialog, EditEncryptionOptionsDialog, FilesystemTarget, ShowDialog,
    TakeOwnershipDialog, UnlockEncryptedDialog,
};
use crate::ui::error::{UiErrorContext, log_error_and_show_dialog};
use crate::ui::volumes::helpers;
use disks_dbus::{DriveModel, VolumeKind, VolumeNode};

use super::super::VolumesControl;

pub(super) fn open_take_ownership(
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

    *dialog = Some(ShowDialog::TakeOwnership(TakeOwnershipDialog {
        target,
        recursive: true,
        running: false,
    }));

    Task::none()
}

pub(super) fn open_change_passphrase(
    control: &mut VolumesControl,
    dialog: &mut Option<ShowDialog>,
) -> Task<cosmic::Action<Message>> {
    if dialog.is_some() {
        return Task::none();
    }

    let Some(segment) = control.segments.get(control.selected_segment) else {
        return Task::none();
    };
    let Some(volume) = segment.volume.clone() else {
        return Task::none();
    };

    let is_crypto_container =
        helpers::find_volume_node_for_partition(&control.model.volumes, &volume)
            .is_some_and(|n| n.kind == VolumeKind::CryptoContainer);
    if !is_crypto_container {
        return Task::none();
    }

    *dialog = Some(ShowDialog::ChangePassphrase(ChangePassphraseDialog {
        volume,
        current_passphrase: String::new(),
        new_passphrase: String::new(),
        confirm_passphrase: String::new(),
        error: None,
        running: false,
    }));

    Task::none()
}

pub(super) fn open_edit_encryption_options(
    control: &mut VolumesControl,
    dialog: &mut Option<ShowDialog>,
) -> Task<cosmic::Action<Message>> {
    if dialog.is_some() {
        return Task::none();
    }

    let Some(segment) = control.segments.get(control.selected_segment) else {
        return Task::none();
    };
    let Some(volume) = segment.volume.clone() else {
        return Task::none();
    };

    let is_crypto_container =
        helpers::find_volume_node_for_partition(&control.model.volumes, &volume)
            .is_some_and(|n| n.kind == VolumeKind::CryptoContainer);
    if !is_crypto_container {
        return Task::none();
    }

    let suggested_name = if volume.name.trim().is_empty() {
        volume
            .device_path
            .as_deref()
            .and_then(|p| p.split('/').next_back())
            .unwrap_or("luks")
            .to_string()
    } else {
        volume.name.clone()
    };

    Task::perform(
        async move {
            let loaded = volume.get_encryption_options_settings().await;
            let mut error: Option<String> = None;
            let settings = match loaded {
                Ok(v) => v,
                Err(e) => {
                    tracing::error!(
                        ?e,
                        operation = "get_encryption_options_settings",
                        object_path = %volume.path,
                        device = ?volume.device_path,
                        drive_path = %volume.drive_path,
                        "error surfaced in UI"
                    );
                    error = Some(format!("{e:#}"));
                    None
                }
            };

            let (use_defaults, unlock_at_startup, require_auth, other_options, name) =
                if let Some(s) = settings {
                    (
                        false,
                        s.unlock_at_startup,
                        s.require_auth,
                        s.other_options,
                        if s.name.is_empty() {
                            suggested_name.clone()
                        } else {
                            s.name
                        },
                    )
                } else {
                    (
                        true,
                        true,
                        false,
                        "nofail".to_string(),
                        suggested_name.clone(),
                    )
                };

            ShowDialog::EditEncryptionOptions(EditEncryptionOptionsDialog {
                volume,
                use_defaults,
                unlock_at_startup,
                require_auth,
                other_options,
                name,
                // Never prefill passphrase.
                passphrase: String::new(),
                show_passphrase: false,
                error,
                running: false,
            })
        },
        |dialog_state| Message::Dialog(Box::new(dialog_state)).into(),
    )
}

pub(super) fn unlock_message(
    control: &mut VolumesControl,
    unlock_message: UnlockMessage,
    dialog: &mut Option<ShowDialog>,
) -> Task<cosmic::Action<Message>> {
    let d = match dialog.as_mut() {
        Some(d) => d,
        None => {
            tracing::warn!("unlock message received with no active dialog; ignoring");
            return Task::none();
        }
    };

    let ShowDialog::UnlockEncrypted(state) = d else {
        tracing::warn!("unlock message received while a different dialog is open; ignoring");
        return Task::none();
    };

    match unlock_message {
        UnlockMessage::PassphraseUpdate(p) => {
            state.passphrase = p;
            state.error = None;
            Task::none()
        }
        UnlockMessage::Cancel => Task::done(Message::CloseDialog.into()),
        UnlockMessage::Confirm => {
            if state.running {
                return Task::none();
            }

            state.running = true;

            let partition_path = state.partition_path.clone();
            let partition_name = state.partition_name.clone();
            let passphrase = state.passphrase.clone();
            let passphrase_for_task = passphrase.clone();

            // Look up the partition in the current model.
            let part = control
                .model
                .volumes_flat
                .iter()
                .find(|p| p.path.to_string() == partition_path)
                .cloned();

            let Some(p) = part else {
                tracing::error!(
                    operation = "unlock_encrypted",
                    object_path = %partition_path,
                    partition_name = %partition_name,
                    "unlock missing partition in model"
                );
                return Task::done(
                    Message::Dialog(Box::new(ShowDialog::Info {
                        title: fl!("unlock-failed"),
                        body: fl!("unlock-missing-partition", name = partition_name),
                    }))
                    .into(),
                );
            };

            Task::perform(
                async move {
                    p.unlock(&passphrase_for_task).await?;
                    DriveModel::get_drives().await
                },
                move |result| match result {
                    Ok(drives) => Message::UpdateNav(drives, None).into(),
                    Err(e) => {
                        tracing::error!(
                            ?e,
                            operation = "unlock_encrypted",
                            object_path = %partition_path,
                            "unlock encrypted dialog error"
                        );
                        Message::Dialog(Box::new(ShowDialog::UnlockEncrypted(
                            UnlockEncryptedDialog {
                                partition_path: partition_path.clone(),
                                partition_name: partition_name.clone(),
                                passphrase: passphrase.clone(),
                                error: Some(e.to_string()),
                                running: false,
                            },
                        )))
                        .into()
                    }
                },
            )
        }
    }
}

pub(super) fn take_ownership_message(
    _control: &mut VolumesControl,
    msg: TakeOwnershipMessage,
    dialog: &mut Option<ShowDialog>,
) -> Task<cosmic::Action<Message>> {
    let Some(ShowDialog::TakeOwnership(state)) = dialog.as_mut() else {
        return Task::none();
    };

    match msg {
        TakeOwnershipMessage::RecursiveUpdate(v) => {
            state.recursive = v;
            Task::none()
        }
        TakeOwnershipMessage::Cancel => Task::done(Message::CloseDialog.into()),
        TakeOwnershipMessage::Confirm => {
            if state.running {
                return Task::none();
            }

            state.running = true;
            let target = state.target.clone();
            let recursive = state.recursive;

            Task::perform(
                async move {
                    match target {
                        FilesystemTarget::Volume(v) => v.take_ownership(recursive).await?,
                        FilesystemTarget::Node(n) => n.take_ownership(recursive).await?,
                    }
                    DriveModel::get_drives().await
                },
                |result| match result {
                    Ok(drives) => Message::UpdateNav(drives, None).into(),
                    Err(e) => {
                        let ctx = UiErrorContext::new("take_ownership");
                        log_error_and_show_dialog(fl!("take-ownership").to_string(), e, ctx).into()
                    }
                },
            )
        }
    }
}

pub(super) fn change_passphrase_message(
    _control: &mut VolumesControl,
    msg: ChangePassphraseMessage,
    dialog: &mut Option<ShowDialog>,
) -> Task<cosmic::Action<Message>> {
    let Some(ShowDialog::ChangePassphrase(state)) = dialog.as_mut() else {
        return Task::none();
    };

    match msg {
        ChangePassphraseMessage::CurrentUpdate(v) => {
            state.current_passphrase = v;
            state.error = None;
            Task::none()
        }
        ChangePassphraseMessage::NewUpdate(v) => {
            state.new_passphrase = v;
            state.error = None;
            Task::none()
        }
        ChangePassphraseMessage::ConfirmUpdate(v) => {
            state.confirm_passphrase = v;
            state.error = None;
            Task::none()
        }
        ChangePassphraseMessage::Cancel => Task::done(Message::CloseDialog.into()),
        ChangePassphraseMessage::Confirm => {
            if state.running {
                return Task::none();
            }

            if state.new_passphrase.is_empty() || state.new_passphrase != state.confirm_passphrase {
                tracing::warn!(operation = "change_passphrase", "passphrase mismatch");
                state.error = Some(fl!("passphrase-mismatch").to_string());
                return Task::none();
            }

            state.running = true;
            let volume = state.volume.clone();
            let current = state.current_passphrase.clone();
            let new = state.new_passphrase.clone();

            Task::perform(
                async move {
                    volume.change_passphrase(&current, &new).await?;
                    DriveModel::get_drives().await
                },
                |result| match result {
                    Ok(drives) => Message::UpdateNav(drives, None).into(),
                    Err(e) => {
                        let ctx = UiErrorContext::new("change_passphrase");
                        log_error_and_show_dialog(fl!("change-passphrase").to_string(), e, ctx)
                            .into()
                    }
                },
            )
        }
    }
}

pub(super) fn edit_encryption_options_message(
    _control: &mut VolumesControl,
    msg: EditEncryptionOptionsMessage,
    dialog: &mut Option<ShowDialog>,
) -> Task<cosmic::Action<Message>> {
    let Some(ShowDialog::EditEncryptionOptions(state)) = dialog.as_mut() else {
        return Task::none();
    };

    match msg {
        EditEncryptionOptionsMessage::UseDefaultsUpdate(v) => {
            state.use_defaults = v;
            state.error = None;
            Task::none()
        }
        EditEncryptionOptionsMessage::UnlockAtStartupUpdate(v) => {
            state.unlock_at_startup = v;
            state.error = None;
            Task::none()
        }
        EditEncryptionOptionsMessage::RequireAuthUpdate(v) => {
            state.require_auth = v;
            state.error = None;
            Task::none()
        }
        EditEncryptionOptionsMessage::OtherOptionsUpdate(v) => {
            state.other_options = v;
            state.error = None;
            Task::none()
        }
        EditEncryptionOptionsMessage::NameUpdate(v) => {
            state.name = v;
            state.error = None;
            Task::none()
        }
        EditEncryptionOptionsMessage::PassphraseUpdate(v) => {
            state.passphrase = v;
            state.error = None;
            Task::none()
        }
        EditEncryptionOptionsMessage::ShowPassphraseUpdate(v) => {
            state.show_passphrase = v;
            Task::none()
        }
        EditEncryptionOptionsMessage::Cancel => Task::done(Message::CloseDialog.into()),
        EditEncryptionOptionsMessage::Confirm => {
            if state.running {
                return Task::none();
            }
            state.running = true;

            let volume = state.volume.clone();
            let use_defaults = state.use_defaults;
            let unlock_at_startup = state.unlock_at_startup;
            let require_auth = state.require_auth;
            let other_options = state.other_options.clone();
            let name = state.name.clone();
            let passphrase = state.passphrase.clone();

            Task::perform(
                async move {
                    if use_defaults {
                        volume.default_encryption_options().await?;
                    } else {
                        volume
                            .edit_encryption_options(
                                unlock_at_startup,
                                require_auth,
                                other_options,
                                name,
                                passphrase,
                            )
                            .await?;
                    }
                    DriveModel::get_drives().await
                },
                move |result| match result {
                    Ok(drives) => Message::UpdateNav(drives, None).into(),
                    Err(e) => {
                        let ctx = UiErrorContext::new("edit_encryption_options");
                        log_error_and_show_dialog(fl!("edit-encryption-options"), e, ctx).into()
                    }
                },
            )
        }
    }
}

pub(super) fn lock_container(control: &mut VolumesControl) -> Task<cosmic::Action<Message>> {
    let segment = control.segments.get(control.selected_segment).cloned();
    if let Some(s) = segment
        && let Some(p) = s.volume
    {
        let mounted_children: Vec<VolumeNode> =
            helpers::find_volume_node_for_partition(&control.model.volumes, &p)
                .map(helpers::collect_mounted_descendants_leaf_first)
                .unwrap_or_default();

        return Task::perform(
            async move {
                // UDisks2 typically refuses to lock while the cleartext/child FS is mounted.
                // Unmount any mounted descendants first, then lock the container.
                for v in mounted_children {
                    v.unmount().await?;
                }
                p.lock().await?;
                DriveModel::get_drives().await
            },
            |result| match result {
                Ok(drives) => Message::UpdateNav(drives, None).into(),
                Err(e) => {
                    let ctx = UiErrorContext::new("lock_container");
                    log_error_and_show_dialog(fl!("lock-failed"), e, ctx).into()
                }
            },
        );
    }

    Task::none()
}
