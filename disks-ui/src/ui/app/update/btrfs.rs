use super::super::state::AppModel;
use super::Message;
use crate::fl;
use crate::ui::dialogs::state::{ConfirmActionDialog, FilesystemTarget, ShowDialog};
use crate::ui::error::{UiErrorContext, log_error_and_show_dialog};
use crate::ui::volumes::VolumesControl;
use crate::utils::btrfs;
use cosmic::app::Task;
use disks_dbus::DriveModel;

/// Handle BTRFS management messages
pub(super) fn handle_btrfs_message(app: &mut AppModel, message: Message) -> Task<Message> {
    match message {
        Message::BtrfsLoadSubvolumes { mount_point } => {
            // Set loading state
            if let Some(volumes_control) = app.nav.active_data_mut::<VolumesControl>()
                && let Some(btrfs_state) = &mut volumes_control.btrfs_state
            {
                btrfs_state.loading = true;
            }

            // Spawn async task to load subvolumes
            let mount_point_for_async = mount_point.clone();
            Task::perform(
                async move {
                    match btrfs::list_subvolumes(&mount_point_for_async).await {
                        Ok(subvolumes) => Ok(subvolumes),
                        Err(e) => Err(format!("{:#}", e)),
                    }
                },
                move |result| {
                    Message::BtrfsSubvolumesLoaded {
                        mount_point: mount_point.clone(),
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

        Message::BtrfsDeleteSubvolume { path } => {
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
                ok_message: Message::BtrfsDeleteSubvolumeConfirm { path },
                running: false,
            }));

            Task::none()
        }

        Message::BtrfsDeleteSubvolumeConfirm { path } => {
            // Set dialog to running state
            if let Some(ShowDialog::ConfirmAction(state)) = &mut app.dialog {
                state.running = true;
            }

            // Perform the actual delete
            Task::perform(
                async move {
                    btrfs::delete_subvolume(&path).await?;
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

        Message::BtrfsLoadUsage { mount_point } => {
            // Set loading state
            if let Some(volumes_control) = app.nav.active_data_mut::<VolumesControl>()
                && let Some(btrfs_state) = &mut volumes_control.btrfs_state
            {
                btrfs_state.loading_usage = true;
            }

            // Spawn async task to load usage info and compression
            let mount_point_for_async = mount_point.clone();
            Task::perform(
                async move {
                    let usage = btrfs::get_filesystem_usage(&mount_point_for_async)
                        .await
                        .map_err(|e| format!("{:#}", e));

                    let compression = btrfs::get_compression(&mount_point_for_async)
                        .await
                        .map_err(|e| format!("{:#}", e));

                    (usage, compression)
                },
                move |(usage_result, compression_result)| {
                    Message::BtrfsUsageLoaded {
                        mount_point: mount_point.clone(),
                        usage_result,
                        compression_result,
                    }
                    .into()
                },
            )
        }

        Message::BtrfsUsageLoaded {
            mount_point,
            usage_result,
            compression_result,
        } => {
            // Update state with loaded usage info
            if let Some(volumes_control) = app.nav.active_data_mut::<VolumesControl>()
                && let Some(btrfs_state) = &mut volumes_control.btrfs_state
                && btrfs_state.mount_point.as_deref() == Some(&mount_point)
            {
                btrfs_state.loading_usage = false;
                btrfs_state.usage_info = Some(usage_result);
                btrfs_state.compression = Some(compression_result.ok().flatten());
            }
            Task::none()
        }

        _ => Task::none(),
    }
}
