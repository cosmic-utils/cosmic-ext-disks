use cosmic::Task;

use crate::app::Message;
use crate::fl;
use crate::ui::dialogs::message::{EditPartitionMessage, ResizePartitionMessage};
use crate::ui::dialogs::state::{
    EditPartitionDialog, FormatPartitionDialog, ResizePartitionDialog, ShowDialog,
};
use crate::ui::volumes::helpers;
use crate::utils::DiskSegmentKind;
use disks_dbus::{CreatePartitionInfo, DriveModel, VolumeKind, VolumeModel, VolumeNode};

use super::super::VolumesControl;

pub(super) fn delete(
    control: &mut VolumesControl,
    dialog: &mut Option<ShowDialog>,
) -> Task<cosmic::Action<Message>> {
    let d = match dialog.as_mut() {
        Some(d) => d,
        None => {
            tracing::warn!("delete received with no active dialog; ignoring");
            return Task::none();
        }
    };

    let ShowDialog::DeletePartition(delete_state) = d else {
        tracing::warn!("delete received while a different dialog is open; ignoring");
        return Task::none();
    };

    if delete_state.running {
        return Task::none();
    }

    delete_state.running = true;

    let segment = control.segments.get(control.selected_segment).cloned();
    match segment.clone() {
        Some(s) => match s.volume {
            Some(p) => {
                let volume_node =
                    helpers::find_volume_node_for_partition(&control.model.volumes, &p).cloned();
                let is_unlocked_crypto = matches!(
                    volume_node.as_ref(),
                    Some(v) if v.kind == VolumeKind::CryptoContainer && !v.locked
                );
                let mounted_children: Vec<VolumeNode> = if is_unlocked_crypto {
                    volume_node
                        .as_ref()
                        .map(helpers::collect_mounted_descendants_leaf_first)
                        .unwrap_or_default()
                } else {
                    Vec::new()
                };

                Task::perform(
                    async move {
                        if is_unlocked_crypto {
                            // UDisks2 typically refuses to lock while the cleartext/child FS is mounted.
                            // Unmount any mounted descendants first, then lock the container.
                            for v in mounted_children {
                                v.unmount().await?;
                            }
                            p.lock().await?;
                        }

                        p.delete().await?;
                        DriveModel::get_drives().await
                    },
                    |result| match result {
                        Ok(drives) => Message::UpdateNav(drives, None).into(),
                        Err(e) => {
                            tracing::error!(?e, "delete failed");
                            Message::Dialog(Box::new(ShowDialog::Info {
                                title: fl!("delete-failed"),
                                body: format!("{e:#}"),
                            }))
                            .into()
                        }
                    },
                )
            }
            None => Task::none(),
        },
        None => Task::none(),
    }
}

pub(super) fn open_format_partition(
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

    let table_type = if volume.table_type.trim().is_empty() {
        "gpt".to_string()
    } else {
        volume.table_type.clone()
    };

    let selected_partition_type_index = helpers::common_partition_type_index_for(
        &table_type,
        if volume.id_type.trim().is_empty() {
            None
        } else {
            Some(volume.id_type.as_str())
        },
    );

    let info = CreatePartitionInfo {
        name: volume.name.clone(),
        size: volume.size,
        max_size: volume.size,
        offset: volume.offset,
        erase: false,
        selected_partition_type_index,
        table_type,
        ..Default::default()
    };

    *dialog = Some(ShowDialog::FormatPartition(FormatPartitionDialog {
        volume,
        info,
        running: false,
    }));

    Task::none()
}

pub(super) fn open_edit_partition(
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

    if volume.volume_type != disks_dbus::VolumeType::Partition {
        return Task::none();
    }

    let partition_types = disks_dbus::get_all_partition_type_infos(volume.table_type.as_str());
    if partition_types.is_empty() {
        return Task::done(
            Message::Dialog(Box::new(ShowDialog::Info {
                title: fl!("app-title"),
                body: fl!("edit-partition-no-types"),
            }))
            .into(),
        );
    }

    let selected_type_index = partition_types
        .iter()
        .position(|t| t.ty == volume.partition_type_id)
        .unwrap_or(0);

    let legacy_bios_bootable = volume.is_legacy_bios_bootable();
    let system_partition = volume.is_system_partition();
    let hidden = volume.is_hidden();
    let name = volume.name.clone();

    *dialog = Some(ShowDialog::EditPartition(EditPartitionDialog {
        volume,
        partition_types,
        selected_type_index,
        name,
        legacy_bios_bootable,
        system_partition,
        hidden,
        running: false,
    }));

    Task::none()
}

pub(super) fn open_resize_partition(
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

    if volume.volume_type != disks_dbus::VolumeType::Partition {
        return Task::none();
    }

    let right_free_bytes = control
        .segments
        .get(control.selected_segment.saturating_add(1))
        .filter(|s| s.kind == DiskSegmentKind::FreeSpace)
        .map(|s| s.size)
        .unwrap_or(0);

    let max_size_bytes = volume.size.saturating_add(right_free_bytes);
    let min_size_bytes = volume
        .usage
        .as_ref()
        .map(|u| u.used)
        .unwrap_or(0)
        .min(max_size_bytes);

    if max_size_bytes.saturating_sub(min_size_bytes) < 1024 {
        return Task::none();
    }

    let new_size_bytes = volume.size.clamp(min_size_bytes, max_size_bytes);

    *dialog = Some(ShowDialog::ResizePartition(ResizePartitionDialog {
        volume,
        min_size_bytes,
        max_size_bytes,
        new_size_bytes,
        running: false,
    }));

    Task::none()
}

pub(super) fn edit_partition_message(
    _control: &mut VolumesControl,
    msg: EditPartitionMessage,
    dialog: &mut Option<ShowDialog>,
) -> Task<cosmic::Action<Message>> {
    let Some(ShowDialog::EditPartition(state)) = dialog.as_mut() else {
        return Task::none();
    };

    match msg {
        EditPartitionMessage::TypeUpdate(idx) => state.selected_type_index = idx,
        EditPartitionMessage::NameUpdate(name) => state.name = name,
        EditPartitionMessage::LegacyBiosBootableUpdate(v) => state.legacy_bios_bootable = v,
        EditPartitionMessage::SystemPartitionUpdate(v) => state.system_partition = v,
        EditPartitionMessage::HiddenUpdate(v) => state.hidden = v,
        EditPartitionMessage::Cancel => return Task::done(Message::CloseDialog.into()),
        EditPartitionMessage::Confirm => {
            if state.running {
                return Task::none();
            }

            let partition_type = state
                .partition_types
                .get(state.selected_type_index)
                .map(|t| t.ty.to_string());

            let Some(partition_type) = partition_type else {
                return Task::none();
            };

            state.running = true;

            let volume = state.volume.clone();
            let name = state.name.clone();
            let legacy = state.legacy_bios_bootable;
            let system = state.system_partition;
            let hidden = state.hidden;

            return Task::perform(
                async move {
                    let flags = VolumeModel::make_partition_flags_bits(legacy, system, hidden);

                    volume.edit_partition(partition_type, name, flags).await?;
                    DriveModel::get_drives().await
                },
                |result| match result {
                    Ok(drives) => Message::UpdateNav(drives, None).into(),
                    Err(e) => Message::Dialog(Box::new(ShowDialog::Info {
                        title: fl!("edit-partition").to_string(),
                        body: format!("{e:#}"),
                    }))
                    .into(),
                },
            );
        }
    }

    Task::none()
}

pub(super) fn resize_partition_message(
    _control: &mut VolumesControl,
    msg: ResizePartitionMessage,
    dialog: &mut Option<ShowDialog>,
) -> Task<cosmic::Action<Message>> {
    let Some(ShowDialog::ResizePartition(state)) = dialog.as_mut() else {
        return Task::none();
    };

    match msg {
        ResizePartitionMessage::SizeUpdate(size) => {
            state.new_size_bytes = size.clamp(state.min_size_bytes, state.max_size_bytes)
        }
        ResizePartitionMessage::Cancel => {
            return Task::done(Message::CloseDialog.into());
        }
        ResizePartitionMessage::Confirm => {
            if state.running {
                return Task::none();
            }

            // Disable when range is too small.
            if state.max_size_bytes.saturating_sub(state.min_size_bytes) < 1024 {
                return Task::none();
            }

            state.running = true;
            let volume = state.volume.clone();
            let new_size = state.new_size_bytes;

            return Task::perform(
                async move {
                    volume.resize(new_size).await?;
                    DriveModel::get_drives().await
                },
                |result| match result {
                    Ok(drives) => Message::UpdateNav(drives, None).into(),
                    Err(e) => Message::Dialog(Box::new(ShowDialog::Info {
                        title: fl!("resize-partition").to_string(),
                        body: format!("{e:#}"),
                    }))
                    .into(),
                },
            );
        }
    }

    Task::none()
}
