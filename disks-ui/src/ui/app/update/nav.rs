use crate::ui::dialogs::state::ShowDialog;
use crate::ui::volumes::VolumesControl;
use cosmic::widget::icon;
use disks_dbus::DriveModel;

use super::super::state::AppModel;

pub(super) fn update_nav(
    app: &mut AppModel,
    drive_models: Vec<DriveModel>,
    selected: Option<String>,
) {
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
                    app.nav
                        .insert()
                        .text(drive.name())
                        .data::<VolumesControl>(VolumesControl::new(drive.clone(), show_reserved))
                        .data::<DriveModel>(drive)
                        .icon(icon::from_name(icon_name))
                        .activate();
                } else {
                    app.nav
                        .insert()
                        .text(drive.name())
                        .data::<VolumesControl>(VolumesControl::new(drive.clone(), show_reserved))
                        .data::<DriveModel>(drive)
                        .icon(icon::from_name(icon_name));
                }
            }
            None => {
                app.nav
                    .insert()
                    .text(drive.name())
                    .data::<VolumesControl>(VolumesControl::new(drive.clone(), show_reserved))
                    .data::<DriveModel>(drive)
                    .icon(icon::from_name(icon_name));
            }
        }
    }
}
