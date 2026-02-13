use super::super::state::AppModel;
use super::Message;
use crate::fl;
use crate::ui::dialogs::state::{ConfirmActionDialog, FilesystemTarget, ShowDialog};
use crate::ui::error::{UiErrorContext, log_error_and_show_dialog};
use crate::ui::volumes::VolumesControl;
use cosmic::app::Task;
use disks_dbus::{BtrfsFilesystem, DiskManager, DriveModel, OwnedObjectPath};

/// Handle BTRFS management messages
pub(super) fn handle_btrfs_message(app: &mut AppModel, message: Message) -> Task<Message> {
    match message {
        Message::BtrfsLoadSubvolumes {
            block_path,
            mount_point,
        } => {
            // Set loading state
            if let Some(volumes_control) = app.nav.active_data_mut::<VolumesControl>()
                && let Some(btrfs_state) = &mut volumes_control.btrfs_state
            {
                btrfs_state.loading = true;
            }

            // Spawn async task to load subvolumes via D-Bus
            let mount_point_for_async = mount_point.clone();
            Task::perform(
                async move {
                    let manager = DiskManager::new().await?;
                    let connection = manager.connection();
                    let block_obj_path: OwnedObjectPath = block_path.as_str().try_into()?;
                    let btrfs = BtrfsFilesystem::new(connection, block_obj_path);

                    // Get all subvolumes (not just snapshots)
                    let subvolumes = btrfs.get_subvolumes(false).await?;
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
            block_path,
            mount_point: _,
            path,
        } => {
            // Set dialog to running state
            if let Some(ShowDialog::ConfirmAction(state)) = &mut app.dialog {
                state.running = true;
            }

            // Perform the actual delete via D-Bus
            Task::perform(
                async move {
                    let manager = DiskManager::new().await?;
                    let connection = manager.connection();
                    let block_obj_path: OwnedObjectPath = block_path.as_str().try_into()?;
                    let btrfs = BtrfsFilesystem::new(connection, block_obj_path);

                    btrfs.remove_subvolume(&path).await?;
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
            block_path,
            mount_point,
        } => {
            // Set loading state
            if let Some(volumes_control) = app.nav.active_data_mut::<VolumesControl>()
                && let Some(btrfs_state) = &mut volumes_control.btrfs_state
            {
                btrfs_state.loading_usage = true;
            }

            // Spawn async task to load usage info via D-Bus
            let mount_point_for_async = mount_point.clone();
            Task::perform(
                async move {
                    let manager = DiskManager::new().await?;
                    let connection = manager.connection();
                    let block_obj_path: OwnedObjectPath = block_path.as_str().try_into()?;
                    let btrfs = BtrfsFilesystem::new(connection, block_obj_path);

                    let used_space = btrfs.get_used_space().await?;
                    Ok(used_space)
                },
                move |result: anyhow::Result<u64>| {
                    let used_space = result.map_err(|e| format!("{:#}", e));
                    Message::BtrfsUsageLoaded {
                        mount_point: mount_point_for_async.clone(),
                        used_space,
                    }
                    .into()
                },
            )
        }

        Message::BtrfsUsageLoaded {
            mount_point,
            used_space,
        } => {
            // Update state with loaded usage info
            if let Some(volumes_control) = app.nav.active_data_mut::<VolumesControl>()
                && let Some(btrfs_state) = &mut volumes_control.btrfs_state
                && btrfs_state.mount_point.as_deref() == Some(&mount_point)
            {
                btrfs_state.loading_usage = false;
                btrfs_state.used_space = Some(used_space);
            }
            Task::none()
        }

        _ => Task::none(),
    }
}
