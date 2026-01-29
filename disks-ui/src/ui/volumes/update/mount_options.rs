use cosmic::Task;

use crate::app::Message;
use crate::fl;
use crate::ui::dialogs::message::EditMountOptionsMessage;
use crate::ui::dialogs::state::{EditMountOptionsDialog, FilesystemTarget, ShowDialog};
use disks_dbus::DriveModel;

use super::super::VolumesControl;

pub(super) fn open_edit_mount_options(
    control: &mut VolumesControl,
    dialog: &mut Option<ShowDialog>,
) -> Task<cosmic::Action<Message>> {
    if dialog.is_some() {
        return Task::none();
    }

    let target = if let Some(node) = control.selected_volume_node() {
        if !node.can_mount() {
            return Task::none();
        }
        FilesystemTarget::Node(node.clone())
    } else {
        let Some(segment) = control.segments.get(control.selected_segment) else {
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

    let (device_path, suggested_name, suggested_fstype, suggested_mountpoint) = match &target {
        FilesystemTarget::Volume(v) => {
            let device = v
                .device_path
                .clone()
                .unwrap_or_else(|| format!("/dev/{}", v.path.split('/').next_back().unwrap_or("")));
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

    Task::perform(
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
    )
}

pub(super) fn edit_mount_options_message(
    _control: &mut VolumesControl,
    msg: EditMountOptionsMessage,
    dialog: &mut Option<ShowDialog>,
) -> Task<cosmic::Action<Message>> {
    let Some(ShowDialog::EditMountOptions(state)) = dialog.as_mut() else {
        return Task::none();
    };

    match msg {
        EditMountOptionsMessage::UseDefaultsUpdate(v) => {
            state.use_defaults = v;
            state.error = None;
            Task::none()
        }
        EditMountOptionsMessage::MountAtStartupUpdate(v) => {
            state.mount_at_startup = v;
            state.error = None;
            Task::none()
        }
        EditMountOptionsMessage::RequireAuthUpdate(v) => {
            state.require_auth = v;
            state.error = None;
            Task::none()
        }
        EditMountOptionsMessage::ShowInUiUpdate(v) => {
            state.show_in_ui = v;
            state.error = None;
            Task::none()
        }
        EditMountOptionsMessage::OtherOptionsUpdate(v) => {
            state.other_options = v;
            state.error = None;
            Task::none()
        }
        EditMountOptionsMessage::DisplayNameUpdate(v) => {
            state.display_name = v;
            state.error = None;
            Task::none()
        }
        EditMountOptionsMessage::IconNameUpdate(v) => {
            state.icon_name = v;
            state.error = None;
            Task::none()
        }
        EditMountOptionsMessage::SymbolicIconNameUpdate(v) => {
            state.symbolic_icon_name = v;
            state.error = None;
            Task::none()
        }
        EditMountOptionsMessage::MountPointUpdate(v) => {
            state.mount_point = v;
            state.error = None;
            Task::none()
        }
        EditMountOptionsMessage::IdentifyAsIndexUpdate(v) => {
            state.identify_as_index = v;
            state.error = None;
            Task::none()
        }
        EditMountOptionsMessage::FilesystemTypeUpdate(v) => {
            state.filesystem_type = v;
            state.error = None;
            Task::none()
        }
        EditMountOptionsMessage::Cancel => Task::done(Message::CloseDialog.into()),
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

            Task::perform(
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
            )
        }
    }
}
