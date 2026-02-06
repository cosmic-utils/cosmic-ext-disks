use crate::ui::dialogs::state::ShowDialog;
use crate::ui::volumes::VolumesControl;
use cosmic::widget::icon;
use disks_dbus::DriveModel;
use std::collections::HashMap;

use super::super::state::AppModel;

pub(super) fn update_nav(
    app: &mut AppModel,
    drive_models: Vec<DriveModel>,
    selected: Option<String>,
) {
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

    let selected = match selected {
        Some(s) => Some(s),
        None => app
            .nav
            .active_data::<DriveModel>()
            .map(|d| d.block_path.clone()),
    };

    // Volumes-level preference; keep it stable across nav rebuilds.
    let show_reserved = app
        .nav
        .active_data::<VolumesControl>()
        .map(|v| v.show_reserved)
        .unwrap_or(false);

    app.nav.clear();

    let mut drive_entities: HashMap<String, cosmic::widget::nav_bar::Id> = HashMap::new();

    let selected = match selected {
        Some(s) => Some(s),
        None => {
            if selected.is_none() && !drive_models.is_empty() {
                Some(drive_models.first().unwrap().block_path.clone())
            } else {
                None
            }
        }
    };

    for drive in drive_models {
        let icon_name = match drive.removable {
            true => "drive-removable-media-symbolic",
            false => "disks-symbolic",
        };

        match selected {
            Some(ref s) => {
                if drive.block_path == s.clone() {
                    let id = app
                        .nav
                        .insert()
                        .text(drive.name())
                        .data::<VolumesControl>(VolumesControl::new(drive.clone(), show_reserved))
                        .data::<DriveModel>(drive.clone())
                        .icon(icon::from_name(icon_name))
                        .activate()
                        .id();

                    drive_entities.insert(drive.block_path.clone(), id);
                } else {
                    let id = app
                        .nav
                        .insert()
                        .text(drive.name())
                        .data::<VolumesControl>(VolumesControl::new(drive.clone(), show_reserved))
                        .data::<DriveModel>(drive.clone())
                        .icon(icon::from_name(icon_name))
                        .id();

                    drive_entities.insert(drive.block_path.clone(), id);
                }
            }
            None => {
                let id = app
                    .nav
                    .insert()
                    .text(drive.name())
                    .data::<VolumesControl>(VolumesControl::new(drive.clone(), show_reserved))
                    .data::<DriveModel>(drive.clone())
                    .icon(icon::from_name(icon_name))
                    .id();

                drive_entities.insert(drive.block_path.clone(), id);
            }
        }
    }

    app.sidebar.set_drive_entities(drive_entities);
}
