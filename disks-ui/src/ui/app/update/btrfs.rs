use super::super::state::AppModel;
use super::Message;
use crate::fl;
use crate::ui::dialogs::state::{ConfirmActionDialog, FilesystemTarget, ShowDialog};
use crate::ui::error::{UiErrorContext, log_error_and_show_dialog};
use crate::ui::volumes::VolumesControl;
use cosmic::app::Task;
use disks_dbus::{BtrfsFilesystem, DriveModel};
use std::path::PathBuf;

/// Handle BTRFS management messages
pub(super) fn handle_btrfs_message(app: &mut AppModel, message: Message) -> Task<Message> {
    match message {
        Message::BtrfsLoadSubvolumes {
            block_path: _,
            mount_point,
        } => {
            // Set loading state
            if let Some(volumes_control) = app.nav.active_data_mut::<VolumesControl>()
                && let Some(btrfs_state) = &mut volumes_control.btrfs_state
            {
                btrfs_state.loading = true;
            }

            // Spawn async task to load subvolumes via btrfsutil
            let mount_point_for_async = mount_point.clone();
            Task::perform(
                async move {
                    let mount_path = PathBuf::from(&mount_point);
                    let btrfs = BtrfsFilesystem::new(mount_path).await?;
                    let subvolumes = btrfs.list_subvolumes().await?;
                    Ok(subvolumes)
                },
                move |result: anyhow::Result<Vec<disks_dbus::BtrfsSubvolume>>| {
                    let result = result.map_err(|e| format!("{:#}", e));
                    Message::BtrfsSubvolumesLoaded {
                        mount_point: mount_point_for_async.clone(),
                        result,
                    }
                    .into()
                },
            )
        }

        Message::BtrfsSubvolumesLoaded {
            mount_point,
            result,
        } => {
            // Update state with loaded subvolumes
            if let Some(volumes_control) = app.nav.active_data_mut::<VolumesControl>()
                && let Some(btrfs_state) = &mut volumes_control.btrfs_state
                && btrfs_state.mount_point.as_deref() == Some(&mount_point)
            {
                btrfs_state.loading = false;
                btrfs_state.subvolumes = Some(result);
            }
            Task::none()
        }

        Message::BtrfsDeleteSubvolume {
            block_path,
            mount_point,
            path,
        } => {
            // Show confirmation dialog
            let subvol_name = path.rsplit('/').next().unwrap_or(&path).to_string();

            // Get a dummy FilesystemTarget (required by ConfirmActionDialog but not used for BTRFS)
            let target = if let Some(volumes_control) = app.nav.active_data::<VolumesControl>() {
                if let Some(segment) = volumes_control
                    .segments
                    .get(volumes_control.selected_segment)
                {
                    if let Some(volume) = &segment.volume {
                        FilesystemTarget::Volume(volume.clone())
                    } else {
                        return Task::none();
                    }
                } else {
                    return Task::none();
                }
            } else {
                return Task::none();
            };

            app.dialog = Some(ShowDialog::ConfirmAction(ConfirmActionDialog {
                title: fl!("btrfs-delete-subvolume"),
                body: fl!("btrfs-delete-confirm", name = subvol_name.as_str()),
                target,
                ok_message: Message::BtrfsDeleteSubvolumeConfirm {
                    block_path,
                    mount_point,
                    path,
                },
                running: false,
            }));

            Task::none()
        }

        Message::BtrfsDeleteSubvolumeConfirm {
            block_path: _,
            mount_point,
            path,
        } => {
            // Set dialog to running state
            if let Some(ShowDialog::ConfirmAction(state)) = &mut app.dialog {
                state.running = true;
            }

            // Perform the actual delete via btrfsutil
            Task::perform(
                async move {
                    let mount_path = PathBuf::from(&mount_point);
                    let btrfs = BtrfsFilesystem::new(mount_path).await?;
                    let subvol_path = PathBuf::from(&path);
                    btrfs.delete_subvolume(&subvol_path).await?;
                    DriveModel::get_drives().await
                },
                |result| match result {
                    Ok(drives) => {
                        // Close dialog and refresh drives (subvolume list will reload)
                        Message::UpdateNav(drives, None).into()
                    }
                    Err(e) => {
                        let ctx = UiErrorContext::new("delete_subvolume");
                        log_error_and_show_dialog(fl!("btrfs-delete-subvolume-failed"), e, ctx)
                            .into()
                    }
                },
            )
        }

        Message::BtrfsLoadUsage {
            block_path: _,
            mount_point: _,
        } => {
            // TODO: Implement BTRFS usage info via alternative method
            // The old get_used_space() was UDisks2-specific
            // Could use statvfs() or subprocess call to `btrfs filesystem usage`
            Task::none()
        }

        Message::BtrfsUsageLoaded {
            mount_point: _,
            used_space: _,
        } => {
            // TODO: Removed temporarily - needs alternative implementation
            Task::none()
        }

        Message::BtrfsToggleSubvolumeExpanded {
            mount_point,
            subvolume_id,
        } => {
            // Toggle the expanded state for a subvolume's snapshots
            if let Some(volumes_control) = app.nav.active_data_mut::<VolumesControl>()
                && let Some(btrfs_state) = &mut volumes_control.btrfs_state
                && btrfs_state.mount_point.as_deref() == Some(&mount_point)
            {
                let expanded = btrfs_state
                    .expanded_subvolumes
                    .entry(subvolume_id)
                    .or_insert(false);
                *expanded = !*expanded;
            }
            Task::none()
        }

        Message::BtrfsLoadDefaultSubvolume { mount_point } => {
            let mount_point_for_async = mount_point.clone();
            Task::perform(
                async move {
                    let mount_path = PathBuf::from(&mount_point);
                    let btrfs = BtrfsFilesystem::new(mount_path).await?;
                    let default_subvol = btrfs.get_default_subvolume().await?;
                    Ok(default_subvol)
                },
                move |result: anyhow::Result<disks_dbus::BtrfsSubvolume>| {
                    let result = result.map_err(|e| format!("{:#}", e));
                    Message::BtrfsDefaultSubvolumeLoaded {
                        mount_point: mount_point_for_async.clone(),
                        result,
                    }
                    .into()
                },
            )
        }

        Message::BtrfsDefaultSubvolumeLoaded {
            mount_point,
            result,
        } => {
            if let Some(volumes_control) = app.nav.active_data_mut::<VolumesControl>()
                && let Some(btrfs_state) = &mut volumes_control.btrfs_state
                && btrfs_state.mount_point.as_deref() == Some(&mount_point)
            {
                match result {
                    Ok(subvol) => {
                        btrfs_state.default_subvolume_id = Some(subvol.id);
                    }
                    Err(e) => {
                        tracing::warn!("Failed to load default subvolume: {}", e);
                    }
                }
            }
            Task::none()
        }

        Message::BtrfsSetDefaultSubvolume {
            mount_point,
            subvolume_id,
        } => {
            // Find the subvolume path by ID
            let subvol_path = if let Some(volumes_control) = app.nav.active_data::<VolumesControl>()
                && let Some(btrfs_state) = &volumes_control.btrfs_state
                && let Some(Ok(subvolumes)) = &btrfs_state.subvolumes
            {
                subvolumes
                    .iter()
                    .find(|s| s.id == subvolume_id)
                    .map(|s| s.path.clone())
            } else {
                None
            };

            if let Some(path) = subvol_path {
                let mount_point_clone = mount_point.clone();
                let mount_point_for_closure = mount_point.clone();
                Task::perform(
                    async move {
                        let mount_path = PathBuf::from(&mount_point_clone);
                        let btrfs = BtrfsFilesystem::new(mount_path).await?;
                        btrfs.set_default_subvolume(&path).await?;
                        Ok(())
                    },
                    move |result: anyhow::Result<()>| {
                        match result {
                            Ok(()) => {
                                // Reload subvolumes to update default flag
                                Message::BtrfsLoadSubvolumes {
                                    block_path: String::new(),
                                    mount_point: mount_point_for_closure.clone(),
                                }
                            }
                            Err(e) => {
                                let ctx = UiErrorContext::new("set_default_subvolume");
                                log_error_and_show_dialog(
                                    fl!("btrfs-set-default-failed"),
                                    e,
                                    ctx,
                                )
                            }
                        }
                        .into()
                    },
                )
            } else {
                Task::none()
            }
        }

        Message::BtrfsToggleReadonly {
            mount_point,
            subvolume_id,
        } => {
            // Find the subvolume by ID
            let subvol_info =
                if let Some(volumes_control) = app.nav.active_data::<VolumesControl>()
                    && let Some(btrfs_state) = &volumes_control.btrfs_state
                    && let Some(Ok(subvolumes)) = &btrfs_state.subvolumes
                {
                    subvolumes
                        .iter()
                        .find(|s| s.id == subvolume_id)
                        .map(|s| (s.path.clone(), s.is_readonly))
                } else {
                    None
                };

            if let Some((path, current_readonly)) = subvol_info {
                let new_readonly = !current_readonly;
                let mount_point_for_async = mount_point.clone();
                let mount_point_for_closure = mount_point.clone();
                Task::perform(
                    async move {
                        let mount_path = PathBuf::from(&mount_point);
                        let btrfs = BtrfsFilesystem::new(mount_path).await?;
                        btrfs.set_readonly(&path, new_readonly).await?;
                        Ok(())
                    },
                    move |result: anyhow::Result<()>| {
                        let result = result.map_err(|e| format!("{:#}", e));
                        Message::BtrfsReadonlyToggled {
                            mount_point: mount_point_for_closure.clone(),
                            result,
                        }
                        .into()
                    },
                )
            } else {
                Task::none()
            }
        }

        Message::BtrfsReadonlyToggled {
            mount_point,
            result,
        } => {
            match result {
                Ok(()) => {
                    // Reload subvolumes to update readonly flag
                    handle_btrfs_message(
                        app,
                        Message::BtrfsLoadSubvolumes {
                            block_path: String::new(),
                            mount_point,
                        },
                    )
                }
                Err(e) => {
                    let ctx = UiErrorContext::new("toggle_readonly");
                    Task::done(log_error_and_show_dialog(fl!("btrfs-readonly-failed"), anyhow::anyhow!(e), ctx).into())
                }
            }
        }

        Message::BtrfsShowProperties {
            mount_point: _,
            subvolume_id,
        } => {
            // Find and store the selected subvolume
            if let Some(volumes_control) = app.nav.active_data_mut::<VolumesControl>()
                && let Some(btrfs_state) = &mut volumes_control.btrfs_state
                && let Some(Ok(subvolumes)) = &btrfs_state.subvolumes
            {
                let subvol = subvolumes.iter().find(|s| s.id == subvolume_id).cloned();
                if let Some(subvol) = subvol {
                    btrfs_state.selected_subvolume = Some(subvol);
                    btrfs_state.show_properties_dialog = true;
                }
            }
            Task::none()
        }

        Message::BtrfsCloseProperties { mount_point } => {
            if let Some(volumes_control) = app.nav.active_data_mut::<VolumesControl>()
                && let Some(btrfs_state) = &mut volumes_control.btrfs_state
                && btrfs_state.mount_point.as_deref() == Some(&mount_point)
            {
                btrfs_state.show_properties_dialog = false;
                btrfs_state.selected_subvolume = None;
            }
            Task::none()
        }

        Message::BtrfsLoadDeletedSubvolumes { mount_point } => {
            let mount_point_for_async = mount_point.clone();
            Task::perform(
                async move {
                    let mount_path = PathBuf::from(&mount_point);
                    let btrfs = BtrfsFilesystem::new(mount_path).await?;
                    let deleted = btrfs.list_deleted_subvolumes().await?;
                    Ok(deleted)
                },
                move |result: anyhow::Result<Vec<disks_dbus::BtrfsSubvolume>>| {
                    let result = result.map_err(|e| format!("{:#}", e));
                    Message::BtrfsDeletedSubvolumesLoaded {
                        mount_point: mount_point_for_async.clone(),
                        result,
                    }
                    .into()
                },
            )
        }

        Message::BtrfsDeletedSubvolumesLoaded {
            mount_point,
            result,
        } => {
            if let Some(volumes_control) = app.nav.active_data_mut::<VolumesControl>()
                && let Some(btrfs_state) = &mut volumes_control.btrfs_state
                && btrfs_state.mount_point.as_deref() == Some(&mount_point)
            {
                match result {
                    Ok(deleted) => {
                        btrfs_state.deleted_subvolumes = Some(deleted);
                    }
                    Err(e) => {
                        tracing::warn!("Failed to load deleted subvolumes: {}", e);
                    }
                }
            }
            Task::none()
        }

        Message::BtrfsToggleShowDeleted { mount_point } => {
            if let Some(volumes_control) = app.nav.active_data_mut::<VolumesControl>()
                && let Some(btrfs_state) = &mut volumes_control.btrfs_state
                && btrfs_state.mount_point.as_deref() == Some(&mount_point)
            {
                btrfs_state.show_deleted = !btrfs_state.show_deleted;
                
                // Load deleted subvolumes if we're showing them and haven't loaded yet
                if btrfs_state.show_deleted && btrfs_state.deleted_subvolumes.is_none() {
                    return handle_btrfs_message(
                        app,
                        Message::BtrfsLoadDeletedSubvolumes {
                            mount_point: mount_point.clone(),
                        },
                    );
                }
            }
            Task::none()
        }

        Message::BtrfsRefreshAll { mount_point } => {
            // Reload all BTRFS data
            Task::batch(vec![
                handle_btrfs_message(
                    app,
                    Message::BtrfsLoadSubvolumes {
                        block_path: String::new(),
                        mount_point: mount_point.clone(),
                    },
                ),
                handle_btrfs_message(
                    app,
                    Message::BtrfsLoadDefaultSubvolume {
                        mount_point: mount_point.clone(),
                    },
                ),
            ])
        }

        _ => Task::none(),
    }
}
