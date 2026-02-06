use crate::ui::dialogs::state::ShowDialog;
use crate::ui::volumes::VolumesControl;
use cosmic::widget::icon;
use disks_dbus::DriveModel;
use std::collections::HashMap;

use crate::ui::app::state::AppModel;

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

    let selected = selected.or_else(|| {
        drive_models
            .first()
            .map(|d| d.block_path.clone())
    });

    for drive in drive_models {
        let icon_name = if drive.removable {
            "drive-removable-media-symbolic"
        } else {
            "disks-symbolic"
        };

        let should_activate = selected.as_ref().is_some_and(|s| &drive.block_path == s);

        let mut nav_item = app
            .nav
            .insert()
            .text(drive.name())
            .data::<VolumesControl>(VolumesControl::new(drive.clone(), show_reserved))
            .data::<DriveModel>(drive.clone())
            .icon(icon::from_name(icon_name));

        if should_activate {
            nav_item = nav_item.activate();
        }

        let id = nav_item.id();
        drive_entities.insert(drive.block_path.clone(), id);
    }

    app.sidebar.set_drive_entities(drive_entities);
}
