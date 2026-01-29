use cosmic::Task;

use crate::app::Message;
use crate::fl;
use crate::ui::dialogs::message::CreateMessage;
use crate::ui::dialogs::state::ShowDialog;
use crate::ui::volumes::helpers;
use disks_dbus::{CreatePartitionInfo, DriveModel};

use super::super::VolumesControl;

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
            CreateMessage::SizeUpdate(size) => state.info.size = size,
            CreateMessage::NameUpdate(name) => {
                state.info.name = name;
            }
            CreateMessage::PasswordUpdate(password) => state.info.password = password,
            CreateMessage::ConfirmedPasswordUpdate(confirmed_password) => {
                state.info.confirmed_password = confirmed_password
            }
            CreateMessage::PasswordProtectedUpdate(protect) => {
                state.info.password_protected = protect
            }
            CreateMessage::EraseUpdate(erase) => state.info.erase = erase,
            CreateMessage::PartitionTypeUpdate(p_type) => {
                state.info.selected_partition_type_index = p_type
            }
            CreateMessage::Continue => {
                tracing::warn!("create message continue is not implemented; ignoring");
            }
            CreateMessage::Cancel => return Task::done(Message::CloseDialog.into()),
            CreateMessage::Partition => {
                if state.running {
                    return Task::none();
                }

                state.running = true;

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
                        Err(e) => Message::Dialog(Box::new(ShowDialog::Info {
                            title: fl!("app-title"),
                            body: format!("{e:#}"),
                        }))
                        .into(),
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
                        .ok_or_else(|| anyhow::anyhow!("Invalid filesystem selection"))?
                        .to_string();

                        volume
                            .format(info.name.clone(), info.erase, fs_type)
                            .await?;
                        DriveModel::get_drives().await
                    },
                    |result| match result {
                        Ok(drives) => Message::UpdateNav(drives, None).into(),
                        Err(e) => Message::Dialog(Box::new(ShowDialog::Info {
                            title: fl!("format-partition").to_string(),
                            body: format!("{e:#}"),
                        }))
                        .into(),
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
        | ShowDialog::ChangePassphrase(_) => {
            tracing::warn!("create message received while a different dialog is open; ignoring");
        }

        ShowDialog::Info { .. } => {
            tracing::warn!("create message received while an info dialog is open; ignoring");
        }
    }

    // Preserve behavior: no fallthrough action here.
    Task::none()
}
