use cosmic::Task;

use crate::app::Message;
use crate::fl;
use crate::ui::dialogs::message::{
    ChangePassphraseMessage, CreateMessage, EditEncryptionOptionsMessage,
    EditFilesystemLabelMessage, EditMountOptionsMessage, EditPartitionMessage,
    ResizePartitionMessage, TakeOwnershipMessage, UnlockMessage,
};
use crate::ui::dialogs::state::{
    ChangePassphraseDialog, ConfirmActionDialog, EditEncryptionOptionsDialog,
    EditFilesystemLabelDialog, EditMountOptionsDialog, EditPartitionDialog, FilesystemTarget,
    FormatPartitionDialog, ResizePartitionDialog, ShowDialog, TakeOwnershipDialog,
    UnlockEncryptedDialog,
};
use crate::ui::volumes::helpers;
use crate::utils::DiskSegmentKind;
use disks_dbus::{CreatePartitionInfo, DriveModel, VolumeKind, VolumeModel, VolumeNode};

use super::{VolumesControl, VolumesControlMessage};

impl VolumesControl {
    pub fn update(
        &mut self,
        message: VolumesControlMessage,
        dialog: &mut Option<ShowDialog>,
    ) -> Task<cosmic::Action<Message>> {
        match message {
            VolumesControlMessage::SegmentSelected(index) => {
                if dialog.is_none() {
                    let Some(last_index) = self.segments.len().checked_sub(1) else {
                        self.selected_segment = 0;
                        self.selected_volume = None;
                        return Task::none();
                    };

                    let index = index.min(last_index);
                    self.selected_segment = index;
                    self.selected_volume = None;
                    self.segments.iter_mut().for_each(|s| s.state = false);
                    if let Some(segment) = self.segments.get_mut(index) {
                        segment.state = true;
                    }
                }
            }
            VolumesControlMessage::SelectVolume {
                segment_index,
                object_path,
            } => {
                if dialog.is_none() {
                    let Some(last_index) = self.segments.len().checked_sub(1) else {
                        self.selected_segment = 0;
                        self.selected_volume = None;
                        return Task::none();
                    };

                    let segment_index = segment_index.min(last_index);
                    self.selected_segment = segment_index;
                    self.selected_volume = Some(object_path);
                    self.segments.iter_mut().for_each(|s| s.state = false);
                    if let Some(segment) = self.segments.get_mut(segment_index) {
                        segment.state = true;
                    }
                }
            }
            VolumesControlMessage::ToggleShowReserved(show_reserved) => {
                self.set_show_reserved(show_reserved);
            }
            VolumesControlMessage::Mount => {
                let segment = self.segments.get(self.selected_segment).cloned();
                if let Some(s) = segment.clone() {
                    match s.volume {
                        Some(p) => {
                            return Task::perform(
                                async move {
                                    match p.mount().await {
                                        Ok(_) => match DriveModel::get_drives().await {
                                            Ok(drives) => Ok(drives),
                                            Err(e) => Err(e),
                                        },
                                        Err(e) => Err(e),
                                    }
                                },
                                |result| match result {
                                    Ok(drives) => Message::UpdateNav(drives, None).into(),
                                    Err(e) => {
                                        tracing::error!(?e, "mount failed");
                                        Message::None.into()
                                    }
                                },
                            );
                        }
                        None => return Task::none(),
                    }
                }
                return Task::none();
            }
            VolumesControlMessage::Unmount => {
                let segment = self.segments.get(self.selected_segment).cloned();
                if let Some(s) = segment.clone() {
                    match s.volume {
                        Some(p) => {
                            return Task::perform(
                                async move {
                                    match p.unmount().await {
                                        Ok(_) => match DriveModel::get_drives().await {
                                            Ok(drives) => Ok(drives),
                                            Err(e) => Err(e),
                                        },
                                        Err(e) => Err(e),
                                    }
                                },
                                |result| match result {
                                    Ok(drives) => Message::UpdateNav(drives, None).into(),
                                    Err(e) => {
                                        tracing::error!(%e, "unmount failed");
                                        Message::None.into()
                                    }
                                },
                            );
                        }
                        None => return Task::none(),
                    }
                }
                return Task::none();
            }
            VolumesControlMessage::ChildMount(object_path) => {
                let node = helpers::find_volume_node(&self.model.volumes, &object_path).cloned();
                if let Some(v) = node {
                    return Task::perform(
                        async move {
                            v.mount().await?;
                            DriveModel::get_drives().await
                        },
                        |result| match result {
                            Ok(drives) => Message::UpdateNav(drives, None).into(),
                            Err(e) => {
                                tracing::error!(?e, "child mount failed");
                                Message::None.into()
                            }
                        },
                    );
                }
                return Task::none();
            }
            VolumesControlMessage::ChildUnmount(object_path) => {
                let node = helpers::find_volume_node(&self.model.volumes, &object_path).cloned();
                if let Some(v) = node {
                    return Task::perform(
                        async move {
                            v.unmount().await?;
                            DriveModel::get_drives().await
                        },
                        |result| match result {
                            Ok(drives) => Message::UpdateNav(drives, None).into(),
                            Err(e) => {
                                tracing::error!(?e, "child unmount failed");
                                Message::None.into()
                            }
                        },
                    );
                }
                return Task::none();
            }

            VolumesControlMessage::LockContainer => {
                let segment = self.segments.get(self.selected_segment).cloned();
                if let Some(s) = segment
                    && let Some(p) = s.volume
                {
                    let mounted_children: Vec<VolumeNode> =
                        helpers::find_volume_node_for_partition(&self.model.volumes, &p)
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
                                tracing::error!(?e, "lock container failed");
                                Message::Dialog(Box::new(ShowDialog::Info {
                                    title: fl!("lock-failed"),
                                    body: e.to_string(),
                                }))
                                .into()
                            }
                        },
                    );
                }
                return Task::none();
            }
            VolumesControlMessage::Delete => {
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

                let segment = self.segments.get(self.selected_segment).cloned();
                let task = match segment.clone() {
                    Some(s) => match s.volume {
                        Some(p) => {
                            let volume_node =
                                helpers::find_volume_node_for_partition(&self.model.volumes, &p)
                                    .cloned();
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
                };

                return task;
            }

            VolumesControlMessage::OpenFormatPartition => {
                if dialog.is_some() {
                    return Task::none();
                }

                let Some(segment) = self.segments.get(self.selected_segment) else {
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
                return Task::none();
            }

            VolumesControlMessage::OpenEditPartition => {
                if dialog.is_some() {
                    return Task::none();
                }

                let Some(segment) = self.segments.get(self.selected_segment) else {
                    return Task::none();
                };
                let Some(volume) = segment.volume.clone() else {
                    return Task::none();
                };

                if volume.volume_type != disks_dbus::VolumeType::Partition {
                    return Task::none();
                }

                let partition_types =
                    disks_dbus::get_all_partition_type_infos(volume.table_type.as_str());
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
                return Task::none();
            }

            VolumesControlMessage::OpenResizePartition => {
                if dialog.is_some() {
                    return Task::none();
                }

                let Some(segment) = self.segments.get(self.selected_segment) else {
                    return Task::none();
                };
                let Some(volume) = segment.volume.clone() else {
                    return Task::none();
                };

                if volume.volume_type != disks_dbus::VolumeType::Partition {
                    return Task::none();
                }

                let right_free_bytes = self
                    .segments
                    .get(self.selected_segment.saturating_add(1))
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
                return Task::none();
            }

            VolumesControlMessage::OpenEditFilesystemLabel => {
                if dialog.is_some() {
                    return Task::none();
                }

                let target = if let Some(node) = self.selected_volume_node() {
                    if !node.can_mount() {
                        return Task::none();
                    }

                    FilesystemTarget::Node(node.clone())
                } else {
                    let Some(segment) = self.segments.get(self.selected_segment) else {
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
                return Task::none();
            }

            VolumesControlMessage::OpenEditMountOptions => {
                if dialog.is_some() {
                    return Task::none();
                }

                let target = if let Some(node) = self.selected_volume_node() {
                    if !node.can_mount() {
                        return Task::none();
                    }
                    FilesystemTarget::Node(node.clone())
                } else {
                    let Some(segment) = self.segments.get(self.selected_segment) else {
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

                let (device_path, suggested_name, suggested_fstype, suggested_mountpoint) =
                    match &target {
                        FilesystemTarget::Volume(v) => {
                            let device = v.device_path.clone().unwrap_or_else(|| {
                                format!("/dev/{}", v.path.split('/').next_back().unwrap_or(""))
                            });
                            let name = if v.name.trim().is_empty() {
                                v.partition_type.clone()
                            } else {
                                v.name.clone()
                            };
                            let fstype = if v.id_type.trim().is_empty() {
                                "auto".to_string()
                            } else {
                                v.id_type.clone()
                            };
                            let mountpoint = v.mount_points.first().cloned().unwrap_or_else(|| {
                                let slug = name.replace(' ', "-");
                                format!("/mnt/{slug}")
                            });
                            (device, name, fstype, mountpoint)
                        }
                        FilesystemTarget::Node(n) => {
                            let device = n.device_path.clone().unwrap_or_else(|| {
                                format!(
                                    "/dev/{}",
                                    n.object_path.split('/').next_back().unwrap_or("")
                                )
                            });
                            let name = if n.label.trim().is_empty() {
                                "Filesystem".to_string()
                            } else {
                                n.label.clone()
                            };
                            let fstype = if n.id_type.trim().is_empty() {
                                "auto".to_string()
                            } else {
                                n.id_type.clone()
                            };
                            let mountpoint = n.mount_points.first().cloned().unwrap_or_else(|| {
                                let slug = name.replace(' ', "-");
                                format!("/mnt/{slug}")
                            });
                            (device, name, fstype, mountpoint)
                        }
                    };

                let mut identify_as_options = vec![device_path];
                // Provide a UUID= option when we have one (VolumeModel only).
                if let FilesystemTarget::Volume(v) = &target
                    && !v.uuid.trim().is_empty()
                {
                    identify_as_options.push(format!("UUID={}", v.uuid.trim()));
                }

                return Task::perform(
                    async move {
                        let loaded = match &target {
                            FilesystemTarget::Volume(v) => v.get_mount_options_settings().await,
                            FilesystemTarget::Node(n) => n.get_mount_options_settings().await,
                        };

                        let mut error: Option<String> = None;
                        let settings = match loaded {
                            Ok(v) => v,
                            Err(e) => {
                                error = Some(format!("{e:#}"));
                                None
                            }
                        };

                        let (
                            use_defaults,
                            mount_at_startup,
                            require_auth,
                            show_in_ui,
                            other_options,
                            display_name,
                            icon_name,
                            symbolic_icon_name,
                            mount_point,
                            identify_as,
                            filesystem_type,
                        ) = if let Some(s) = settings {
                            (
                                false,
                                s.mount_at_startup,
                                s.require_auth,
                                s.show_in_ui,
                                s.other_options,
                                if s.display_name.is_empty() {
                                    suggested_name.clone()
                                } else {
                                    s.display_name
                                },
                                s.icon_name,
                                s.symbolic_icon_name,
                                if s.mount_point.is_empty() {
                                    suggested_mountpoint.clone()
                                } else {
                                    s.mount_point
                                },
                                s.identify_as,
                                if s.filesystem_type.is_empty() {
                                    suggested_fstype.clone()
                                } else {
                                    s.filesystem_type
                                },
                            )
                        } else {
                            (
                                true,
                                true,
                                false,
                                true,
                                // GNOME Disks defaults to `nosuid,nodev,nofail,x-gvfs-show`.
                                // We keep `x-gvfs-show` controlled by the checkbox.
                                "nosuid,nodev,nofail".to_string(),
                                suggested_name.clone(),
                                String::new(),
                                String::new(),
                                suggested_mountpoint.clone(),
                                identify_as_options.first().cloned().unwrap_or_default(),
                                suggested_fstype.clone(),
                            )
                        };

                        let mut identify_as_options = identify_as_options;
                        if !identify_as.trim().is_empty()
                            && !identify_as_options.iter().any(|v| v == identify_as.trim())
                        {
                            identify_as_options.push(identify_as.clone());
                        }

                        let identify_as_index = identify_as_options
                            .iter()
                            .position(|v| v == identify_as.trim())
                            .unwrap_or(0);

                        ShowDialog::EditMountOptions(EditMountOptionsDialog {
                            target,
                            use_defaults,
                            mount_at_startup,
                            require_auth,
                            show_in_ui,
                            other_options,
                            display_name,
                            icon_name,
                            symbolic_icon_name,
                            mount_point,
                            identify_as_options,
                            identify_as_index,
                            filesystem_type,
                            error,
                            running: false,
                        })
                    },
                    |dialog_state| Message::Dialog(Box::new(dialog_state)).into(),
                );
            }

            VolumesControlMessage::OpenCheckFilesystem => {
                if dialog.is_some() {
                    return Task::none();
                }

                let target = if let Some(node) = self.selected_volume_node() {
                    if !node.can_mount() {
                        return Task::none();
                    }
                    FilesystemTarget::Node(node.clone())
                } else {
                    let Some(segment) = self.segments.get(self.selected_segment) else {
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
                return Task::none();
            }

            VolumesControlMessage::CheckFilesystemConfirm => {
                let Some(ShowDialog::ConfirmAction(state)) = dialog.as_mut() else {
                    return Task::none();
                };

                if state.running {
                    return Task::none();
                }
                state.running = true;

                let target = state.target.clone();
                return Task::perform(
                    async move {
                        match target {
                            FilesystemTarget::Volume(v) => v.check_filesystem().await?,
                            FilesystemTarget::Node(n) => n.check_filesystem().await?,
                        }
                        DriveModel::get_drives().await
                    },
                    |result| match result {
                        Ok(drives) => Message::UpdateNav(drives, None).into(),
                        Err(e) => Message::Dialog(Box::new(ShowDialog::Info {
                            title: fl!("check-filesystem").to_string(),
                            body: format!("{e:#}"),
                        }))
                        .into(),
                    },
                );
            }

            VolumesControlMessage::OpenRepairFilesystem => {
                if dialog.is_some() {
                    return Task::none();
                }

                let target = if let Some(node) = self.selected_volume_node() {
                    if !node.can_mount() {
                        return Task::none();
                    }
                    FilesystemTarget::Node(node.clone())
                } else {
                    let Some(segment) = self.segments.get(self.selected_segment) else {
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
                return Task::none();
            }

            VolumesControlMessage::RepairFilesystemConfirm => {
                let Some(ShowDialog::ConfirmAction(state)) = dialog.as_mut() else {
                    return Task::none();
                };

                if state.running {
                    return Task::none();
                }
                state.running = true;

                let target = state.target.clone();
                return Task::perform(
                    async move {
                        match target {
                            FilesystemTarget::Volume(v) => v.repair_filesystem().await?,
                            FilesystemTarget::Node(n) => n.repair_filesystem().await?,
                        }
                        DriveModel::get_drives().await
                    },
                    |result| match result {
                        Ok(drives) => Message::UpdateNav(drives, None).into(),
                        Err(e) => Message::Dialog(Box::new(ShowDialog::Info {
                            title: fl!("repair-filesystem").to_string(),
                            body: format!("{e:#}"),
                        }))
                        .into(),
                    },
                );
            }

            VolumesControlMessage::OpenTakeOwnership => {
                if dialog.is_some() {
                    return Task::none();
                }

                let target = if let Some(node) = self.selected_volume_node() {
                    if !node.can_mount() {
                        return Task::none();
                    }
                    FilesystemTarget::Node(node.clone())
                } else {
                    let Some(segment) = self.segments.get(self.selected_segment) else {
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
                return Task::none();
            }

            VolumesControlMessage::OpenChangePassphrase => {
                if dialog.is_some() {
                    return Task::none();
                }

                let Some(segment) = self.segments.get(self.selected_segment) else {
                    return Task::none();
                };
                let Some(volume) = segment.volume.clone() else {
                    return Task::none();
                };

                let is_crypto_container =
                    helpers::find_volume_node_for_partition(&self.model.volumes, &volume)
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
                return Task::none();
            }

            VolumesControlMessage::OpenEditEncryptionOptions => {
                if dialog.is_some() {
                    return Task::none();
                }

                let Some(segment) = self.segments.get(self.selected_segment) else {
                    return Task::none();
                };
                let Some(volume) = segment.volume.clone() else {
                    return Task::none();
                };

                let is_crypto_container =
                    helpers::find_volume_node_for_partition(&self.model.volumes, &volume)
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

                return Task::perform(
                    async move {
                        let loaded = volume.get_encryption_options_settings().await;
                        let mut error: Option<String> = None;
                        let settings = match loaded {
                            Ok(v) => v,
                            Err(e) => {
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
                );
            }

            VolumesControlMessage::EditMountOptionsMessage(msg) => {
                let Some(ShowDialog::EditMountOptions(state)) = dialog.as_mut() else {
                    return Task::none();
                };

                match msg {
                    EditMountOptionsMessage::UseDefaultsUpdate(v) => {
                        state.use_defaults = v;
                        state.error = None;
                        return Task::none();
                    }
                    EditMountOptionsMessage::MountAtStartupUpdate(v) => {
                        state.mount_at_startup = v;
                        state.error = None;
                        return Task::none();
                    }
                    EditMountOptionsMessage::RequireAuthUpdate(v) => {
                        state.require_auth = v;
                        state.error = None;
                        return Task::none();
                    }
                    EditMountOptionsMessage::ShowInUiUpdate(v) => {
                        state.show_in_ui = v;
                        state.error = None;
                        return Task::none();
                    }
                    EditMountOptionsMessage::OtherOptionsUpdate(v) => {
                        state.other_options = v;
                        state.error = None;
                        return Task::none();
                    }
                    EditMountOptionsMessage::DisplayNameUpdate(v) => {
                        state.display_name = v;
                        state.error = None;
                        return Task::none();
                    }
                    EditMountOptionsMessage::IconNameUpdate(v) => {
                        state.icon_name = v;
                        state.error = None;
                        return Task::none();
                    }
                    EditMountOptionsMessage::SymbolicIconNameUpdate(v) => {
                        state.symbolic_icon_name = v;
                        state.error = None;
                        return Task::none();
                    }
                    EditMountOptionsMessage::MountPointUpdate(v) => {
                        state.mount_point = v;
                        state.error = None;
                        return Task::none();
                    }
                    EditMountOptionsMessage::IdentifyAsIndexUpdate(v) => {
                        state.identify_as_index = v;
                        state.error = None;
                        return Task::none();
                    }
                    EditMountOptionsMessage::FilesystemTypeUpdate(v) => {
                        state.filesystem_type = v;
                        state.error = None;
                        return Task::none();
                    }
                    EditMountOptionsMessage::Cancel => {
                        return Task::done(Message::CloseDialog.into());
                    }
                    EditMountOptionsMessage::Confirm => {
                        if state.running {
                            return Task::none();
                        }
                        state.running = true;

                        let target = state.target.clone();
                        let use_defaults = state.use_defaults;
                        let mount_at_startup = state.mount_at_startup;
                        let require_auth = state.require_auth;
                        let show_in_ui = state.show_in_ui;
                        let other_options = state.other_options.clone();
                        let display_name = state.display_name.clone();
                        let icon_name = state.icon_name.clone();
                        let symbolic_icon_name = state.symbolic_icon_name.clone();
                        let mount_point = state.mount_point.clone();
                        let identify_as = state
                            .identify_as_options
                            .get(state.identify_as_index)
                            .cloned()
                            .unwrap_or_default();
                        let filesystem_type = state.filesystem_type.clone();

                        return Task::perform(
                            async move {
                                match target {
                                    FilesystemTarget::Volume(v) => {
                                        if use_defaults {
                                            v.default_mount_options().await?;
                                        } else {
                                            v.edit_mount_options(
                                                mount_at_startup,
                                                show_in_ui,
                                                require_auth,
                                                if display_name.trim().is_empty() {
                                                    None
                                                } else {
                                                    Some(display_name)
                                                },
                                                if icon_name.trim().is_empty() {
                                                    None
                                                } else {
                                                    Some(icon_name)
                                                },
                                                if symbolic_icon_name.trim().is_empty() {
                                                    None
                                                } else {
                                                    Some(symbolic_icon_name)
                                                },
                                                other_options,
                                                mount_point,
                                                identify_as,
                                                filesystem_type,
                                            )
                                            .await?;
                                        }
                                    }
                                    FilesystemTarget::Node(n) => {
                                        if use_defaults {
                                            n.default_mount_options().await?;
                                        } else {
                                            n.edit_mount_options(
                                                mount_at_startup,
                                                show_in_ui,
                                                require_auth,
                                                if display_name.trim().is_empty() {
                                                    None
                                                } else {
                                                    Some(display_name)
                                                },
                                                if icon_name.trim().is_empty() {
                                                    None
                                                } else {
                                                    Some(icon_name)
                                                },
                                                if symbolic_icon_name.trim().is_empty() {
                                                    None
                                                } else {
                                                    Some(symbolic_icon_name)
                                                },
                                                other_options,
                                                mount_point,
                                                identify_as,
                                                filesystem_type,
                                            )
                                            .await?;
                                        }
                                    }
                                }
                                DriveModel::get_drives().await
                            },
                            move |result| match result {
                                Ok(drives) => Message::UpdateNav(drives, None).into(),
                                Err(e) => Message::Dialog(Box::new(ShowDialog::Info {
                                    title: fl!("edit-mount-options"),
                                    body: format!("{e:#}"),
                                }))
                                .into(),
                            },
                        );
                    }
                }
            }

            VolumesControlMessage::EditEncryptionOptionsMessage(msg) => {
                let Some(ShowDialog::EditEncryptionOptions(state)) = dialog.as_mut() else {
                    return Task::none();
                };

                match msg {
                    EditEncryptionOptionsMessage::UseDefaultsUpdate(v) => {
                        state.use_defaults = v;
                        state.error = None;
                        return Task::none();
                    }
                    EditEncryptionOptionsMessage::UnlockAtStartupUpdate(v) => {
                        state.unlock_at_startup = v;
                        state.error = None;
                        return Task::none();
                    }
                    EditEncryptionOptionsMessage::RequireAuthUpdate(v) => {
                        state.require_auth = v;
                        state.error = None;
                        return Task::none();
                    }
                    EditEncryptionOptionsMessage::OtherOptionsUpdate(v) => {
                        state.other_options = v;
                        state.error = None;
                        return Task::none();
                    }
                    EditEncryptionOptionsMessage::NameUpdate(v) => {
                        state.name = v;
                        state.error = None;
                        return Task::none();
                    }
                    EditEncryptionOptionsMessage::PassphraseUpdate(v) => {
                        state.passphrase = v;
                        state.error = None;
                        return Task::none();
                    }
                    EditEncryptionOptionsMessage::ShowPassphraseUpdate(v) => {
                        state.show_passphrase = v;
                        return Task::none();
                    }
                    EditEncryptionOptionsMessage::Cancel => {
                        return Task::done(Message::CloseDialog.into());
                    }
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

                        return Task::perform(
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
                                Err(e) => Message::Dialog(Box::new(ShowDialog::Info {
                                    title: fl!("edit-encryption-options"),
                                    body: format!("{e:#}"),
                                }))
                                .into(),
                            },
                        );
                    }
                }
            }
            VolumesControlMessage::CreateMessage(create_message) => {
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

                            let model = self.model.clone();
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
                        tracing::warn!(
                            "create message received while an unlock dialog is open; ignoring"
                        );
                    }

                    ShowDialog::FormatDisk(_) => {
                        tracing::warn!(
                            "create message received while a format disk dialog is open; ignoring"
                        );
                    }

                    ShowDialog::SmartData(_) => {
                        tracing::warn!(
                            "create message received while a SMART dialog is open; ignoring"
                        );
                    }

                    ShowDialog::NewDiskImage(_)
                    | ShowDialog::AttachDiskImage(_)
                    | ShowDialog::ImageOperation(_) => {
                        tracing::warn!(
                            "create message received while an image dialog is open; ignoring"
                        );
                    }

                    ShowDialog::EditPartition(_)
                    | ShowDialog::ResizePartition(_)
                    | ShowDialog::EditFilesystemLabel(_)
                    | ShowDialog::ConfirmAction(_)
                    | ShowDialog::TakeOwnership(_)
                    | ShowDialog::ChangePassphrase(_) => {
                        tracing::warn!(
                            "create message received while a different dialog is open; ignoring"
                        );
                    }

                    ShowDialog::Info { .. } => {
                        tracing::warn!(
                            "create message received while an info dialog is open; ignoring"
                        );
                    }
                }
            }

            VolumesControlMessage::UnlockMessage(unlock_message) => {
                let d = match dialog.as_mut() {
                    Some(d) => d,
                    None => {
                        tracing::warn!("unlock message received with no active dialog; ignoring");
                        return Task::none();
                    }
                };

                let ShowDialog::UnlockEncrypted(state) = d else {
                    tracing::warn!(
                        "unlock message received while a different dialog is open; ignoring"
                    );
                    return Task::none();
                };

                match unlock_message {
                    UnlockMessage::PassphraseUpdate(p) => {
                        state.passphrase = p;
                        state.error = None;
                        return Task::none();
                    }
                    UnlockMessage::Cancel => return Task::done(Message::CloseDialog.into()),
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
                        let part = self
                            .model
                            .volumes_flat
                            .iter()
                            .find(|p| p.path.to_string() == partition_path)
                            .cloned();

                        let Some(p) = part else {
                            return Task::done(
                                Message::Dialog(Box::new(ShowDialog::Info {
                                    title: fl!("unlock-failed"),
                                    body: fl!("unlock-missing-partition", name = partition_name),
                                }))
                                .into(),
                            );
                        };

                        return Task::perform(
                            async move {
                                p.unlock(&passphrase_for_task).await?;
                                DriveModel::get_drives().await
                            },
                            move |result| match result {
                                Ok(drives) => Message::UpdateNav(drives, None).into(),
                                Err(e) => {
                                    tracing::error!(%e, "unlock encrypted dialog error");
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
                        );
                    }
                }
            }

            VolumesControlMessage::EditPartitionMessage(msg) => {
                let Some(ShowDialog::EditPartition(state)) = dialog.as_mut() else {
                    return Task::none();
                };

                match msg {
                    EditPartitionMessage::TypeUpdate(idx) => state.selected_type_index = idx,
                    EditPartitionMessage::NameUpdate(name) => state.name = name,
                    EditPartitionMessage::LegacyBiosBootableUpdate(v) => {
                        state.legacy_bios_bootable = v
                    }
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
                                let flags =
                                    VolumeModel::make_partition_flags_bits(legacy, system, hidden);

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

                return Task::none();
            }

            VolumesControlMessage::ResizePartitionMessage(msg) => {
                let Some(ShowDialog::ResizePartition(state)) = dialog.as_mut() else {
                    return Task::none();
                };

                match msg {
                    ResizePartitionMessage::SizeUpdate(size) => {
                        state.new_size_bytes =
                            size.clamp(state.min_size_bytes, state.max_size_bytes)
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

                return Task::none();
            }

            VolumesControlMessage::EditFilesystemLabelMessage(msg) => {
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
                                    FilesystemTarget::Volume(v) => {
                                        v.edit_filesystem_label(label).await?
                                    }
                                    FilesystemTarget::Node(n) => {
                                        n.edit_filesystem_label(&label).await?
                                    }
                                }
                                DriveModel::get_drives().await
                            },
                            |result| match result {
                                Ok(drives) => Message::UpdateNav(drives, None).into(),
                                Err(e) => Message::Dialog(Box::new(ShowDialog::Info {
                                    title: fl!("edit-filesystem").to_string(),
                                    body: format!("{e:#}"),
                                }))
                                .into(),
                            },
                        );
                    }
                }

                return Task::none();
            }

            VolumesControlMessage::TakeOwnershipMessage(msg) => {
                let Some(ShowDialog::TakeOwnership(state)) = dialog.as_mut() else {
                    return Task::none();
                };

                match msg {
                    TakeOwnershipMessage::RecursiveUpdate(v) => state.recursive = v,
                    TakeOwnershipMessage::Cancel => return Task::done(Message::CloseDialog.into()),
                    TakeOwnershipMessage::Confirm => {
                        if state.running {
                            return Task::none();
                        }

                        state.running = true;
                        let target = state.target.clone();
                        let recursive = state.recursive;

                        return Task::perform(
                            async move {
                                match target {
                                    FilesystemTarget::Volume(v) => {
                                        v.take_ownership(recursive).await?
                                    }
                                    FilesystemTarget::Node(n) => {
                                        n.take_ownership(recursive).await?
                                    }
                                }
                                DriveModel::get_drives().await
                            },
                            |result| match result {
                                Ok(drives) => Message::UpdateNav(drives, None).into(),
                                Err(e) => Message::Dialog(Box::new(ShowDialog::Info {
                                    title: fl!("take-ownership").to_string(),
                                    body: format!("{e:#}"),
                                }))
                                .into(),
                            },
                        );
                    }
                }

                return Task::none();
            }

            VolumesControlMessage::ChangePassphraseMessage(msg) => {
                let Some(ShowDialog::ChangePassphrase(state)) = dialog.as_mut() else {
                    return Task::none();
                };

                match msg {
                    ChangePassphraseMessage::CurrentUpdate(v) => {
                        state.current_passphrase = v;
                        state.error = None;
                    }
                    ChangePassphraseMessage::NewUpdate(v) => {
                        state.new_passphrase = v;
                        state.error = None;
                    }
                    ChangePassphraseMessage::ConfirmUpdate(v) => {
                        state.confirm_passphrase = v;
                        state.error = None;
                    }
                    ChangePassphraseMessage::Cancel => {
                        return Task::done(Message::CloseDialog.into());
                    }
                    ChangePassphraseMessage::Confirm => {
                        if state.running {
                            return Task::none();
                        }

                        if state.new_passphrase.is_empty()
                            || state.new_passphrase != state.confirm_passphrase
                        {
                            state.error = Some(fl!("passphrase-mismatch").to_string());
                            return Task::none();
                        }

                        state.running = true;
                        let volume = state.volume.clone();
                        let current = state.current_passphrase.clone();
                        let new = state.new_passphrase.clone();

                        return Task::perform(
                            async move {
                                volume.change_passphrase(&current, &new).await?;
                                DriveModel::get_drives().await
                            },
                            |result| match result {
                                Ok(drives) => Message::UpdateNav(drives, None).into(),
                                Err(e) => Message::Dialog(Box::new(ShowDialog::Info {
                                    title: fl!("change-passphrase").to_string(),
                                    body: format!("{e:#}"),
                                }))
                                .into(),
                            },
                        );
                    }
                }

                return Task::none();
            }
        }

        Task::none()
    }
}
