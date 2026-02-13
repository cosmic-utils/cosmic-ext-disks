use crate::ui::app::message::Message;
use crate::ui::app::state::AppModel;
use crate::ui::btrfs::BtrfsState;
use crate::ui::dialogs::state::ShowDialog;
use crate::ui::volumes::VolumesControl;
use cosmic::app::Task;
use cosmic::widget::icon;
use disks_dbus::DriveModel;
use std::collections::HashMap;

pub(super) fn update_nav(
    app: &mut AppModel,
    drive_models: Vec<DriveModel>,
    selected: Option<String>,
) -> Task<Message> {
    // Cache drive models for the custom sidebar tree.
    app.sidebar.set_drives(drive_models.clone());

    // Some actions (unlock/format/create/delete) trigger a refresh; close the dialog if
    // it is in a running state so it doesn't linger after success.
    let should_close = match app.dialog.as_ref() {
        Some(ShowDialog::UnlockEncrypted(s)) => s.running,
        Some(ShowDialog::FormatDisk(s)) => s.running,
        Some(ShowDialog::AddPartition(s)) => s.running,
        Some(ShowDialog::FormatPartition(s)) => s.running,
        Some(ShowDialog::EditPartition(s)) => s.running,
        Some(ShowDialog::ResizePartition(s)) => s.running,
        Some(ShowDialog::EditFilesystemLabel(s)) => s.running,
        Some(ShowDialog::EditMountOptions(s)) => s.running,
        Some(ShowDialog::ConfirmAction(s)) => s.running,
        Some(ShowDialog::TakeOwnership(s)) => s.running,
        Some(ShowDialog::ChangePassphrase(s)) => s.running,
        Some(ShowDialog::EditEncryptionOptions(s)) => s.running,
        Some(ShowDialog::DeletePartition(s)) => s.running,
        _ => false,
    };

    if should_close {
        app.dialog = None;
    }

    let selected = selected.or_else(|| {
        app.nav
            .active_data::<DriveModel>()
            .map(|d| d.block_path.clone())
    });

    // Volumes-level preference; keep it stable across nav rebuilds.
    let show_reserved = app
        .nav
        .active_data::<VolumesControl>()
        .map(|v| v.show_reserved)
        .unwrap_or(app.config.show_reserved);

    app.nav.clear();

    let mut drive_entities: HashMap<String, cosmic::widget::nav_bar::Id> = HashMap::new();

    let selected = selected.or_else(|| drive_models.first().map(|d| d.block_path.clone()));

    for drive in drive_models {
        let icon_name = if drive.removable {
            "drive-removable-media-symbolic"
        } else {
            "disks-symbolic"
        };

        let should_activate = selected.as_ref().is_some_and(|s| &drive.block_path == s);

        let mut volumes_control = VolumesControl::new(drive.clone(), show_reserved);

        // Initialize BTRFS state for the first segment if it's BTRFS
        if let Some(segment) = volumes_control.segments.first()
            && let Some(volume) = &segment.volume
        {
            let is_btrfs = volume.id_type.to_lowercase() == "btrfs"
                || (volume.has_filesystem && volume.id_type.to_lowercase() == "btrfs");

            if is_btrfs {
                let mount_point = volume.mount_points.first().cloned();
                volumes_control.btrfs_state = Some(BtrfsState::new(mount_point));
            }
        }

        let mut nav_item = app
            .nav
            .insert()
            .text(drive.name())
            .data::<VolumesControl>(volumes_control)
            .data::<DriveModel>(drive.clone())
            .icon(icon::from_name(icon_name));

        if should_activate {
            nav_item = nav_item.activate();
        }

        let id = nav_item.id();
        drive_entities.insert(drive.block_path.clone(), id);
    }

    app.sidebar.set_drive_entities(drive_entities);

    //  Trigger BTRFS data loading for activated drive
    if let Some(volumes_control) = app.nav.active_data::<VolumesControl>()
        && let Some(btrfs_state) = &volumes_control.btrfs_state
        && let Some(mount_point) = &btrfs_state.mount_point
    {
        let mut tasks = Vec::new();

        // Load subvolumes if not already loaded/loading
        if btrfs_state.subvolumes.is_none() && !btrfs_state.loading {
            tasks.push(Task::done(
                Message::BtrfsLoadSubvolumes {
                    mount_point: mount_point.clone(),
                }
                .into(),
            ));
        }

        // Load usage info if not already loaded/loading
        if btrfs_state.usage_info.is_none() && !btrfs_state.loading_usage {
            tasks.push(Task::done(
                Message::BtrfsLoadUsage {
                    mount_point: mount_point.clone(),
                }
                .into(),
            ));
        }

        if !tasks.is_empty() {
            return Task::batch(tasks);
        }
    }

    Task::none()
}
