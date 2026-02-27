use crate::app::Message;
use crate::message::dialogs::{
    LogicalBtrfsDialogMessage, LogicalControlDialogMessage, LogicalLvmDialogMessage,
    LogicalMdRaidDialogMessage,
};
use crate::state::app::AppModel;
use crate::state::dialogs::{
    LogicalBtrfsWizardDialog, LogicalControlDialog, LogicalLvmWizardDialog,
    LogicalMdRaidWizardDialog, LogicalWizardStep, ShowDialog,
};
use cosmic::app::Task;
use storage_contracts::client::LogicalClient;
use storage_types::{LogicalEntityKind, LogicalOperation};

fn md_name_from_device(device: &str) -> String {
    device
        .rsplit('/')
        .next()
        .unwrap_or(device)
        .trim()
        .to_string()
}

fn parse_csv(csv: &str) -> Vec<String> {
    csv.split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(ToString::to_string)
        .collect()
}

pub(super) fn open_operation_dialog(
    app: &mut AppModel,
    entity_id: String,
    operation: LogicalOperation,
) -> Task<Message> {
    let Some(entity) = app
        .logical
        .entities
        .iter()
        .find(|candidate| candidate.id == entity_id)
        .cloned()
    else {
        app.logical.operation_status = Some("Selected logical entity no longer exists".to_string());
        return Task::none();
    };

    let members_csv = entity
        .members
        .iter()
        .filter_map(|member| member.device_path.as_ref())
        .cloned()
        .collect::<Vec<_>>()
        .join(",");

    let default_member = entity
        .members
        .iter()
        .find_map(|member| member.device_path.clone())
        .unwrap_or_default();

    let default_mount_point = entity
        .metadata
        .get("mount_point")
        .cloned()
        .unwrap_or_default();

    let default_level = entity
        .metadata
        .get("level")
        .cloned()
        .unwrap_or_else(|| "raid1".to_string());

    let default_vg_name = entity
        .parent_id
        .as_deref()
        .and_then(|parent| parent.strip_prefix("lvm-vg:"))
        .unwrap_or(&entity.name)
        .to_string();

    let control_op = matches!(
        operation,
        LogicalOperation::Activate
            | LogicalOperation::Deactivate
            | LogicalOperation::Start
            | LogicalOperation::Stop
            | LogicalOperation::Check
            | LogicalOperation::Repair
    );

    if control_op {
        app.dialog = Some(ShowDialog::LogicalControl(LogicalControlDialog {
            entity_id: entity.id,
            operation,
            lv_path: entity.device_path.clone().unwrap_or_default(),
            array_device: entity.device_path.clone().unwrap_or_default(),
            md_name: entity
                .device_path
                .as_deref()
                .map(md_name_from_device)
                .unwrap_or_default(),
            action: match operation {
                LogicalOperation::Check => "check".to_string(),
                LogicalOperation::Repair => "repair".to_string(),
                _ => String::new(),
            },
            running: false,
            error: None,
        }));

        return Task::none();
    }

    match entity.kind {
        LogicalEntityKind::LvmVolumeGroup
        | LogicalEntityKind::LvmLogicalVolume
        | LogicalEntityKind::LvmPhysicalVolume => {
            app.dialog = Some(ShowDialog::LogicalLvmWizard(LogicalLvmWizardDialog {
                entity_id: entity.id,
                operation,
                step: LogicalWizardStep::Configure,
                vg_name: default_vg_name,
                lv_name: entity.name,
                lv_path: entity.device_path.unwrap_or_default(),
                pv_device: default_member,
                size_bytes: entity.size_bytes.max(1),
                devices_csv: members_csv,
                running: false,
                error: None,
            }));
        }
        LogicalEntityKind::MdRaidArray | LogicalEntityKind::MdRaidMember => {
            app.dialog = Some(ShowDialog::LogicalMdRaidWizard(LogicalMdRaidWizardDialog {
                entity_id: entity.id,
                operation,
                step: LogicalWizardStep::Configure,
                array_device: entity.device_path.unwrap_or_default(),
                level: default_level,
                devices_csv: members_csv,
                member_device: default_member,
                running: false,
                error: None,
            }));
        }
        LogicalEntityKind::BtrfsFilesystem
        | LogicalEntityKind::BtrfsDevice
        | LogicalEntityKind::BtrfsSubvolume => {
            app.dialog = Some(ShowDialog::LogicalBtrfsWizard(LogicalBtrfsWizardDialog {
                entity_id: entity.id,
                operation,
                step: LogicalWizardStep::Configure,
                member_device: default_member,
                mount_point: default_mount_point,
                size_spec: "max".to_string(),
                label: String::new(),
                subvolume_id: "0".to_string(),
                running: false,
                error: None,
            }));
        }
    }

    Task::none()
}

pub(super) fn lvm_dialog(app: &mut AppModel, msg: LogicalLvmDialogMessage) -> Task<Message> {
    let Some(ShowDialog::LogicalLvmWizard(state)) = app.dialog.as_mut() else {
        return Task::none();
    };

    match msg {
        LogicalLvmDialogMessage::PrevStep => state.step = LogicalWizardStep::Configure,
        LogicalLvmDialogMessage::NextStep => state.step = LogicalWizardStep::Review,
        LogicalLvmDialogMessage::SetStep(step) => state.step = step,
        LogicalLvmDialogMessage::VgNameUpdate(value) => state.vg_name = value,
        LogicalLvmDialogMessage::LvNameUpdate(value) => state.lv_name = value,
        LogicalLvmDialogMessage::LvPathUpdate(value) => state.lv_path = value,
        LogicalLvmDialogMessage::PvDeviceUpdate(value) => state.pv_device = value,
        LogicalLvmDialogMessage::SizeBytesUpdate(value) => state.size_bytes = value,
        LogicalLvmDialogMessage::DevicesCsvUpdate(value) => state.devices_csv = value,
        LogicalLvmDialogMessage::Cancel => {
            if !state.running {
                app.dialog = None;
            }
        }
        LogicalLvmDialogMessage::Submit => {
            if state.running {
                return Task::none();
            }

            state.running = true;
            state.error = None;

            let operation = state.operation;
            let vg_name = state.vg_name.clone();
            let lv_name = state.lv_name.clone();
            let lv_path = state.lv_path.clone();
            let pv_device = state.pv_device.clone();
            let size_bytes = state.size_bytes;
            let devices_csv = state.devices_csv.clone();

            return Task::perform(
                async move {
                    let client = LogicalClient::new().await.map_err(|e| e.to_string())?;
                    match operation {
                        LogicalOperation::Create => {
                            if lv_name.trim().is_empty() {
                                client
                                    .lvm_create_volume_group(
                                        vg_name,
                                        serde_json::to_string(&parse_csv(&devices_csv))
                                            .map_err(|e| e.to_string())?,
                                    )
                                    .await
                                    .map_err(|e| e.to_string())
                            } else {
                                client
                                    .lvm_create_logical_volume(vg_name, lv_name, size_bytes)
                                    .await
                                    .map_err(|e| e.to_string())
                            }
                        }
                        LogicalOperation::Delete => {
                            if lv_path.trim().is_empty() {
                                client
                                    .lvm_delete_volume_group(vg_name)
                                    .await
                                    .map_err(|e| e.to_string())
                            } else {
                                client
                                    .lvm_delete_logical_volume(lv_path)
                                    .await
                                    .map_err(|e| e.to_string())
                            }
                        }
                        LogicalOperation::AddMember => client
                            .lvm_add_physical_volume(vg_name, pv_device)
                            .await
                            .map_err(|e| e.to_string()),
                        LogicalOperation::RemoveMember => client
                            .lvm_remove_physical_volume(vg_name, pv_device)
                            .await
                            .map_err(|e| e.to_string()),
                        LogicalOperation::Resize => client
                            .lvm_resize_logical_volume(lv_path, size_bytes)
                            .await
                            .map_err(|e| e.to_string()),
                        _ => Err("Unsupported LVM wizard operation".to_string()),
                    }
                },
                |result| {
                    Message::LogicalLvmDialog(LogicalLvmDialogMessage::Complete(result)).into()
                },
            );
        }
        LogicalLvmDialogMessage::Complete(result) => {
            state.running = false;
            match result {
                Ok(()) => {
                    let op = state.operation;
                    let id = state.entity_id.clone();
                    app.dialog = None;
                    app.logical.operation_status = Some(format!("{op:?} succeeded for {id}"));
                    return Task::done(cosmic::Action::App(Message::LoadLogicalEntities));
                }
                Err(error) => {
                    state.error = Some(error.clone());
                    app.logical.operation_status = Some(format!(
                        "{:?} failed for {}: {}",
                        state.operation, state.entity_id, error
                    ));
                }
            }
        }
    }

    Task::none()
}

pub(super) fn mdraid_dialog(app: &mut AppModel, msg: LogicalMdRaidDialogMessage) -> Task<Message> {
    let Some(ShowDialog::LogicalMdRaidWizard(state)) = app.dialog.as_mut() else {
        return Task::none();
    };

    match msg {
        LogicalMdRaidDialogMessage::PrevStep => state.step = LogicalWizardStep::Configure,
        LogicalMdRaidDialogMessage::NextStep => state.step = LogicalWizardStep::Review,
        LogicalMdRaidDialogMessage::SetStep(step) => state.step = step,
        LogicalMdRaidDialogMessage::ArrayDeviceUpdate(value) => state.array_device = value,
        LogicalMdRaidDialogMessage::LevelUpdate(value) => state.level = value,
        LogicalMdRaidDialogMessage::DevicesCsvUpdate(value) => state.devices_csv = value,
        LogicalMdRaidDialogMessage::MemberDeviceUpdate(value) => state.member_device = value,
        LogicalMdRaidDialogMessage::Cancel => {
            if !state.running {
                app.dialog = None;
            }
        }
        LogicalMdRaidDialogMessage::Submit => {
            if state.running {
                return Task::none();
            }

            state.running = true;
            state.error = None;

            let operation = state.operation;
            let array_device = state.array_device.clone();
            let level = state.level.clone();
            let devices_csv = state.devices_csv.clone();
            let member_device = state.member_device.clone();

            return Task::perform(
                async move {
                    let client = LogicalClient::new().await.map_err(|e| e.to_string())?;
                    match operation {
                        LogicalOperation::Create => client
                            .mdraid_create_array(
                                array_device,
                                level,
                                serde_json::to_string(&parse_csv(&devices_csv))
                                    .map_err(|e| e.to_string())?,
                            )
                            .await
                            .map_err(|e| e.to_string()),
                        LogicalOperation::Delete => client
                            .mdraid_delete_array(array_device)
                            .await
                            .map_err(|e| e.to_string()),
                        LogicalOperation::AddMember => client
                            .mdraid_add_member(array_device, member_device)
                            .await
                            .map_err(|e| e.to_string()),
                        LogicalOperation::RemoveMember => client
                            .mdraid_remove_member(array_device, member_device)
                            .await
                            .map_err(|e| e.to_string()),
                        _ => Err("Unsupported MD RAID wizard operation".to_string()),
                    }
                },
                |result| {
                    Message::LogicalMdRaidDialog(LogicalMdRaidDialogMessage::Complete(result))
                        .into()
                },
            );
        }
        LogicalMdRaidDialogMessage::Complete(result) => {
            state.running = false;
            match result {
                Ok(()) => {
                    let op = state.operation;
                    let id = state.entity_id.clone();
                    app.dialog = None;
                    app.logical.operation_status = Some(format!("{op:?} succeeded for {id}"));
                    return Task::done(cosmic::Action::App(Message::LoadLogicalEntities));
                }
                Err(error) => {
                    state.error = Some(error.clone());
                    app.logical.operation_status = Some(format!(
                        "{:?} failed for {}: {}",
                        state.operation, state.entity_id, error
                    ));
                }
            }
        }
    }

    Task::none()
}

pub(super) fn btrfs_dialog(app: &mut AppModel, msg: LogicalBtrfsDialogMessage) -> Task<Message> {
    let Some(ShowDialog::LogicalBtrfsWizard(state)) = app.dialog.as_mut() else {
        return Task::none();
    };

    match msg {
        LogicalBtrfsDialogMessage::PrevStep => state.step = LogicalWizardStep::Configure,
        LogicalBtrfsDialogMessage::NextStep => state.step = LogicalWizardStep::Review,
        LogicalBtrfsDialogMessage::SetStep(step) => state.step = step,
        LogicalBtrfsDialogMessage::MemberDeviceUpdate(value) => state.member_device = value,
        LogicalBtrfsDialogMessage::MountPointUpdate(value) => state.mount_point = value,
        LogicalBtrfsDialogMessage::SizeSpecUpdate(value) => state.size_spec = value,
        LogicalBtrfsDialogMessage::LabelUpdate(value) => state.label = value,
        LogicalBtrfsDialogMessage::SubvolumeIdUpdate(value) => state.subvolume_id = value,
        LogicalBtrfsDialogMessage::Cancel => {
            if !state.running {
                app.dialog = None;
            }
        }
        LogicalBtrfsDialogMessage::Submit => {
            if state.running {
                return Task::none();
            }

            state.running = true;
            state.error = None;

            let operation = state.operation;
            let member_device = state.member_device.clone();
            let mount_point = state.mount_point.clone();
            let size_spec = state.size_spec.clone();
            let label = state.label.clone();
            let subvolume_id = state.subvolume_id.clone();

            return Task::perform(
                async move {
                    let client = LogicalClient::new().await.map_err(|e| e.to_string())?;
                    match operation {
                        LogicalOperation::AddMember => client
                            .btrfs_add_device(member_device, mount_point)
                            .await
                            .map_err(|e| e.to_string()),
                        LogicalOperation::RemoveMember => client
                            .btrfs_remove_device(member_device, mount_point)
                            .await
                            .map_err(|e| e.to_string()),
                        LogicalOperation::Resize => client
                            .btrfs_resize(size_spec, mount_point)
                            .await
                            .map_err(|e| e.to_string()),
                        LogicalOperation::SetLabel => client
                            .btrfs_set_label(mount_point, label)
                            .await
                            .map_err(|e| e.to_string()),
                        LogicalOperation::SetDefaultSubvolume => {
                            let id = subvolume_id.trim().parse::<u64>().map_err(|_| {
                                "Subvolume ID must be a valid unsigned integer".to_string()
                            })?;
                            client
                                .btrfs_set_default_subvolume(id, mount_point)
                                .await
                                .map_err(|e| e.to_string())
                        }
                        _ => Err("Unsupported BTRFS wizard operation".to_string()),
                    }
                },
                |result| {
                    Message::LogicalBtrfsDialog(LogicalBtrfsDialogMessage::Complete(result)).into()
                },
            );
        }
        LogicalBtrfsDialogMessage::Complete(result) => {
            state.running = false;
            match result {
                Ok(()) => {
                    let op = state.operation;
                    let id = state.entity_id.clone();
                    app.dialog = None;
                    app.logical.operation_status = Some(format!("{op:?} succeeded for {id}"));
                    return Task::done(cosmic::Action::App(Message::LoadLogicalEntities));
                }
                Err(error) => {
                    state.error = Some(error.clone());
                    app.logical.operation_status = Some(format!(
                        "{:?} failed for {}: {}",
                        state.operation, state.entity_id, error
                    ));
                }
            }
        }
    }

    Task::none()
}

pub(super) fn control_dialog(
    app: &mut AppModel,
    msg: LogicalControlDialogMessage,
) -> Task<Message> {
    let Some(ShowDialog::LogicalControl(state)) = app.dialog.as_mut() else {
        return Task::none();
    };

    match msg {
        LogicalControlDialogMessage::LvPathUpdate(value) => state.lv_path = value,
        LogicalControlDialogMessage::ArrayDeviceUpdate(value) => state.array_device = value,
        LogicalControlDialogMessage::MdNameUpdate(value) => state.md_name = value,
        LogicalControlDialogMessage::ActionUpdate(value) => state.action = value,
        LogicalControlDialogMessage::Cancel => {
            if !state.running {
                app.dialog = None;
            }
        }
        LogicalControlDialogMessage::Submit => {
            if state.running {
                return Task::none();
            }
            state.running = true;
            state.error = None;

            let operation = state.operation;
            let lv_path = state.lv_path.clone();
            let array_device = state.array_device.clone();
            let md_name = state.md_name.clone();
            let action = state.action.clone();

            return Task::perform(
                async move {
                    let client = LogicalClient::new().await.map_err(|e| e.to_string())?;
                    match operation {
                        LogicalOperation::Activate => client
                            .lvm_activate_logical_volume(lv_path)
                            .await
                            .map_err(|e| e.to_string()),
                        LogicalOperation::Deactivate => client
                            .lvm_deactivate_logical_volume(lv_path)
                            .await
                            .map_err(|e| e.to_string()),
                        LogicalOperation::Start => client
                            .mdraid_start_array(array_device)
                            .await
                            .map_err(|e| e.to_string()),
                        LogicalOperation::Stop => client
                            .mdraid_stop_array(array_device)
                            .await
                            .map_err(|e| e.to_string()),
                        LogicalOperation::Check | LogicalOperation::Repair => client
                            .mdraid_request_sync_action(md_name, action)
                            .await
                            .map_err(|e| e.to_string()),
                        _ => Err("Unsupported control operation".to_string()),
                    }
                },
                |result| {
                    Message::LogicalControlDialog(LogicalControlDialogMessage::Complete(result))
                        .into()
                },
            );
        }
        LogicalControlDialogMessage::Complete(result) => {
            state.running = false;
            match result {
                Ok(()) => {
                    let op = state.operation;
                    let id = state.entity_id.clone();
                    app.dialog = None;
                    app.logical.operation_status = Some(format!("{op:?} succeeded for {id}"));
                    return Task::done(cosmic::Action::App(Message::LoadLogicalEntities));
                }
                Err(error) => {
                    state.error = Some(error.clone());
                    app.logical.operation_status = Some(format!(
                        "{:?} failed for {}: {}",
                        state.operation, state.entity_id, error
                    ));
                }
            }
        }
    }

    Task::none()
}
