use cosmic::Task;

use crate::app::Message;
use crate::fl;
use crate::ui::dialogs::message::CreateMessage;
use crate::ui::dialogs::state::ShowDialog;
use crate::ui::error::{UiErrorContext, log_error_and_show_dialog};
use crate::ui::volumes::helpers;
use crate::utils::SizeUnit;
use disks_dbus::{CreatePartitionInfo, DriveModel};

use crate::ui::volumes::VolumesControl;

pub(super) fn create_message(
    control: &mut VolumesControl,
    create_message: CreateMessage,
    dialog: &mut Option<ShowDialog>,
) -> Task<cosmic::Action<Message>> {
    let d = match dialog.as_mut() {
        Some(d) => d,
        None => {
            tracing::warn!("create message received with no active dialog; ignoring");
            return Task::none();
        }
    };

    match d {
        ShowDialog::DeletePartition(_) => {}

        ShowDialog::EditMountOptions(_) | ShowDialog::EditEncryptionOptions(_) => {}

        ShowDialog::AddPartition(state) => match create_message {
            CreateMessage::SizeUpdate(size) => {
                state.info.size = size;
                state.error = None;
            }
            CreateMessage::SizeTextUpdate(text) => {
                state.info.size_text = text.clone();
                // Parse and update size in bytes
                if let Ok(value) = text.trim().parse::<f64>() {
                    let unit = SizeUnit::from_index(state.info.size_unit_index);
                    state.info.size = unit.to_bytes(value).min(state.info.max_size);
                }
                state.error = None;
            }
            CreateMessage::SizeUnitUpdate(unit_index) => {
                let old_unit = SizeUnit::from_index(state.info.size_unit_index);
                let new_unit = SizeUnit::from_index(unit_index);
                
                // Parse current text value
                if let Ok(old_value) = state.info.size_text.trim().parse::<f64>() {
                    // Convert to bytes using old unit, then to new unit
                    let bytes = old_unit.to_bytes(old_value);
                    let new_value = new_unit.from_bytes(bytes);
                    state.info.size_text = format!("{:.2}", new_value);
                }
                
                state.info.size_unit_index = unit_index;
                state.error = None;
            }
            CreateMessage::NameUpdate(name) => {
                state.info.name = name;
                state.error = None;
            }
            CreateMessage::PasswordUpdate(password) => {
                state.info.password = password;
                state.error = None;
            }
            CreateMessage::ConfirmedPasswordUpdate(confirmed_password) => {
                state.info.confirmed_password = confirmed_password;
                state.error = None;
            }
            CreateMessage::PasswordProtectedUpdate(protect) => {
                state.info.password_protected = protect;
                state.error = None;
            }
            CreateMessage::EraseUpdate(erase) => {
                state.info.erase = erase;
                state.error = None;
            }
            CreateMessage::PartitionTypeUpdate(p_type) => {
                state.info.selected_partition_type_index = p_type;
                state.error = None;
            }
            CreateMessage::Cancel => return Task::done(Message::CloseDialog.into()),
            CreateMessage::Partition => {
                if state.running {
                    return Task::none();
                }

                // UI-side validation for encrypted partition creation.
                if state.info.password_protected {
                    if state.info.password.is_empty() {
                        tracing::warn!(operation = "create_partition", "password required");
                        state.error = Some(fl!("password-required").to_string());
                        return Task::none();
                    }
                    if state.info.password != state.info.confirmed_password {
                        tracing::warn!(operation = "create_partition", "password mismatch");
                        state.error = Some(fl!("password-mismatch").to_string());
                        return Task::none();
                    }
                }

                state.running = true;
                state.error = None;

                let mut create_partition_info: CreatePartitionInfo = state.info.clone();
                if create_partition_info.name.is_empty() {
                    create_partition_info.name = fl!("untitled").to_string();
                }

                let model = control.model.clone();
                return Task::perform(
                    async move {
                        model.create_partition(create_partition_info).await?;
                        DriveModel::get_drives().await
                    },
                    |result| match result {
                        Ok(drives) => Message::UpdateNav(drives, None).into(),
                        Err(e) => {
                            let ctx = UiErrorContext::new("create_partition");
                            log_error_and_show_dialog(fl!("create-partition-failed"), e, ctx).into()
                        }
                    },
                );
            }
        },

        ShowDialog::FormatPartition(state) => match create_message {
            CreateMessage::NameUpdate(name) => {
                state.info.name = name;
            }
            CreateMessage::EraseUpdate(erase) => state.info.erase = erase,
            CreateMessage::PartitionTypeUpdate(p_type) => {
                state.info.selected_partition_type_index = p_type
            }
            CreateMessage::Cancel => return Task::done(Message::CloseDialog.into()),
            CreateMessage::Partition => {
                if state.running {
                    return Task::none();
                }
                state.running = true;

                let volume = state.volume.clone();
                let info = state.info.clone();
                return Task::perform(
                    async move {
                        let fs_type = helpers::common_partition_filesystem_type(
                            info.table_type.as_str(),
                            info.selected_partition_type_index,
                        )
                        .ok_or_else(|| anyhow::anyhow!("Invalid filesystem selection"))?;

                        volume
                            .format(info.name.clone(), info.erase, fs_type)
                            .await?;
                        DriveModel::get_drives().await
                    },
                    |result| match result {
                        Ok(drives) => Message::UpdateNav(drives, None).into(),
                        Err(e) => {
                            let ctx = UiErrorContext::new("format_partition");
                            log_error_and_show_dialog(fl!("format-partition").to_string(), e, ctx)
                                .into()
                        }
                    },
                );
            }
            _ => {}
        },

        ShowDialog::UnlockEncrypted(_) => {
            tracing::warn!("create message received while an unlock dialog is open; ignoring");
        }

        ShowDialog::FormatDisk(_) => {
            tracing::warn!("create message received while a format disk dialog is open; ignoring");
        }

        ShowDialog::SmartData(_) => {
            tracing::warn!("create message received while a SMART dialog is open; ignoring");
        }

        ShowDialog::NewDiskImage(_)
        | ShowDialog::AttachDiskImage(_)
        | ShowDialog::ImageOperation(_) => {
            tracing::warn!("create message received while an image dialog is open; ignoring");
        }

        ShowDialog::EditPartition(_)
        | ShowDialog::ResizePartition(_)
        | ShowDialog::EditFilesystemLabel(_)
        | ShowDialog::ConfirmAction(_)
        | ShowDialog::TakeOwnership(_)
        | ShowDialog::ChangePassphrase(_)
        | ShowDialog::UnmountBusy(_) => {
            tracing::warn!("create message received while a different dialog is open; ignoring");
        }

        ShowDialog::Info { .. } => {
            tracing::warn!("create message received while an info dialog is open; ignoring");
        }
    }

    // Preserve behavior: no fallthrough action here.
    Task::none()
}
