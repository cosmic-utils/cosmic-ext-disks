#![allow(clippy::too_many_arguments)]

use crate::app::Message;
use crate::controls::wizard::{
    WizardBreadcrumbStatus, WizardBreadcrumbStep, wizard_breadcrumb, wizard_step_is_clickable,
    wizard_step_nav, wizard_step_shell,
};
use crate::message::dialogs::{
    LogicalBtrfsDialogMessage, LogicalControlDialogMessage, LogicalLvmDialogMessage,
    LogicalMdRaidDialogMessage,
};
use crate::state::dialogs::{
    LogicalBtrfsWizardDialog, LogicalControlDialog, LogicalLvmWizardDialog,
    LogicalMdRaidWizardDialog, LogicalWizardStep,
};
use cosmic::{Element, iced_widget, widget::button, widget::text::caption, widget::text_input};
use storage_types::{LogicalOperation, bytes_to_pretty};

fn operation_title(operation: LogicalOperation) -> &'static str {
    match operation {
        LogicalOperation::Create => "Create",
        LogicalOperation::Delete => "Delete",
        LogicalOperation::Resize => "Resize",
        LogicalOperation::AddMember => "Add member",
        LogicalOperation::RemoveMember => "Remove member",
        LogicalOperation::Activate => "Activate",
        LogicalOperation::Deactivate => "Deactivate",
        LogicalOperation::Start => "Start",
        LogicalOperation::Stop => "Stop",
        LogicalOperation::Check => "Check",
        LogicalOperation::Repair => "Repair",
        LogicalOperation::SetLabel => "Set label",
        LogicalOperation::SetDefaultSubvolume => "Set default subvolume",
    }
}

fn wizard_chrome<M: Clone + 'static>(
    title: String,
    step: LogicalWizardStep,
    configure_msg: M,
    prev_msg: M,
    next_msg: Option<M>,
    submit_msg: Option<M>,
    cancel_msg: M,
    content: Element<'_, M>,
) -> Element<'_, M> {
    let current = step.number();
    let steps = [
        (LogicalWizardStep::Configure, "Configure".to_string()),
        (LogicalWizardStep::Review, "Review".to_string()),
    ];

    let breadcrumb = wizard_breadcrumb(
        steps
            .iter()
            .map(|(wizard_step, label)| {
                let step_number = wizard_step.number();
                let status = if step_number == current {
                    WizardBreadcrumbStatus::Current
                } else if step_number < current {
                    WizardBreadcrumbStatus::Completed
                } else {
                    WizardBreadcrumbStatus::Upcoming
                };

                let on_press = if wizard_step_is_clickable(step_number, current) {
                    Some(configure_msg.clone())
                } else {
                    None
                };

                WizardBreadcrumbStep {
                    label: label.clone(),
                    status,
                    on_press,
                }
            })
            .collect(),
    );

    let footer = wizard_step_nav(
        cancel_msg,
        if step == LogicalWizardStep::Review {
            Some(prev_msg)
        } else {
            None
        },
        if step == LogicalWizardStep::Review {
            "Apply".to_string()
        } else {
            "Next".to_string()
        },
        if step == LogicalWizardStep::Review {
            submit_msg
        } else {
            next_msg
        },
    );

    wizard_step_shell(caption(title).into(), breadcrumb, content, footer)
}

pub fn logical_lvm_wizard<'a>(state: LogicalLvmWizardDialog) -> Element<'a, Message> {
    let title = format!("LVM {}", operation_title(state.operation));

    let mut configure = iced_widget::column![]
        .spacing(12)
        .push(caption("Entity"))
        .push(caption(state.entity_id.clone()));

    match state.operation {
        LogicalOperation::Create => {
            configure = configure
                .push(
                    text_input("VG name", state.vg_name.clone())
                        .on_input(|v| LogicalLvmDialogMessage::VgNameUpdate(v).into()),
                )
                .push(
                    text_input(
                        "Devices CSV (/dev/sda2,/dev/sdb2)",
                        state.devices_csv.clone(),
                    )
                    .on_input(|v| LogicalLvmDialogMessage::DevicesCsvUpdate(v).into()),
                );
        }
        LogicalOperation::Delete => {
            configure = configure.push(
                text_input("VG name", state.vg_name.clone())
                    .on_input(|v| LogicalLvmDialogMessage::VgNameUpdate(v).into()),
            );
        }
        LogicalOperation::AddMember | LogicalOperation::RemoveMember => {
            configure = configure
                .push(
                    text_input("VG name", state.vg_name.clone())
                        .on_input(|v| LogicalLvmDialogMessage::VgNameUpdate(v).into()),
                )
                .push(
                    text_input("PV device", state.pv_device.clone())
                        .on_input(|v| LogicalLvmDialogMessage::PvDeviceUpdate(v).into()),
                );
        }
        LogicalOperation::Resize => {
            configure = configure
                .push(
                    text_input("LV path", state.lv_path.clone())
                        .on_input(|v| LogicalLvmDialogMessage::LvPathUpdate(v).into()),
                )
                .push(
                    text_input("Size bytes", state.size_bytes.to_string()).on_input(|v| {
                        let parsed = v.trim().parse::<u64>().unwrap_or(0);
                        LogicalLvmDialogMessage::SizeBytesUpdate(parsed).into()
                    }),
                );
        }
        LogicalOperation::Activate | LogicalOperation::Deactivate => {
            configure = configure.push(
                text_input("LV path", state.lv_path.clone())
                    .on_input(|v| LogicalLvmDialogMessage::LvPathUpdate(v).into()),
            );
        }
        _ => {
            configure = configure
                .push(
                    text_input("VG name", state.vg_name.clone())
                        .on_input(|v| LogicalLvmDialogMessage::VgNameUpdate(v).into()),
                )
                .push(
                    text_input("LV name", state.lv_name.clone())
                        .on_input(|v| LogicalLvmDialogMessage::LvNameUpdate(v).into()),
                )
                .push(
                    text_input("Size bytes", state.size_bytes.to_string()).on_input(|v| {
                        let parsed = v.trim().parse::<u64>().unwrap_or(0);
                        LogicalLvmDialogMessage::SizeBytesUpdate(parsed).into()
                    }),
                );
        }
    }

    if let Some(error) = state.error.clone() {
        configure = configure.push(caption(error));
    }
    if state.running {
        configure = configure.push(caption("working"));
    }

    let review = iced_widget::column![
        caption(format!("Operation: {}", operation_title(state.operation))),
        caption(format!("VG: {}", state.vg_name)),
        caption(format!("LV: {}", state.lv_name)),
        caption(format!("LV Path: {}", state.lv_path)),
        caption(format!("PV: {}", state.pv_device)),
        caption(format!(
            "Size: {}",
            bytes_to_pretty(&state.size_bytes, false)
        )),
        caption(format!("Devices: {}", state.devices_csv)),
    ]
    .spacing(8)
    .into();

    wizard_chrome(
        title,
        state.step,
        LogicalLvmDialogMessage::SetStep(LogicalWizardStep::Configure).into(),
        LogicalLvmDialogMessage::PrevStep.into(),
        Some(LogicalLvmDialogMessage::NextStep.into()),
        if state.running {
            None
        } else {
            Some(LogicalLvmDialogMessage::Submit.into())
        },
        LogicalLvmDialogMessage::Cancel.into(),
        if state.step == LogicalWizardStep::Configure {
            configure.into()
        } else {
            review
        },
    )
}

pub fn logical_mdraid_wizard<'a>(state: LogicalMdRaidWizardDialog) -> Element<'a, Message> {
    let title = format!("MD RAID {}", operation_title(state.operation));

    let mut configure = iced_widget::column![]
        .spacing(12)
        .push(caption("Entity"))
        .push(caption(state.entity_id.clone()));

    match state.operation {
        LogicalOperation::Create => {
            configure = configure
                .push(
                    text_input("Array device", state.array_device.clone())
                        .on_input(|v| LogicalMdRaidDialogMessage::ArrayDeviceUpdate(v).into()),
                )
                .push(
                    text_input("Level (raid1/raid5)", state.level.clone())
                        .on_input(|v| LogicalMdRaidDialogMessage::LevelUpdate(v).into()),
                )
                .push(
                    text_input("Devices CSV", state.devices_csv.clone())
                        .on_input(|v| LogicalMdRaidDialogMessage::DevicesCsvUpdate(v).into()),
                );
        }
        LogicalOperation::Delete => {
            configure = configure.push(
                text_input("Array device", state.array_device.clone())
                    .on_input(|v| LogicalMdRaidDialogMessage::ArrayDeviceUpdate(v).into()),
            );
        }
        _ => {
            configure = configure
                .push(
                    text_input("Array device", state.array_device.clone())
                        .on_input(|v| LogicalMdRaidDialogMessage::ArrayDeviceUpdate(v).into()),
                )
                .push(
                    text_input("Member device", state.member_device.clone())
                        .on_input(|v| LogicalMdRaidDialogMessage::MemberDeviceUpdate(v).into()),
                );
        }
    }

    if let Some(error) = state.error.clone() {
        configure = configure.push(caption(error));
    }
    if state.running {
        configure = configure.push(caption("working"));
    }

    let review = iced_widget::column![
        caption(format!("Operation: {}", operation_title(state.operation))),
        caption(format!("Array: {}", state.array_device)),
        caption(format!("Level: {}", state.level)),
        caption(format!("Devices: {}", state.devices_csv)),
        caption(format!("Member: {}", state.member_device)),
    ]
    .spacing(8)
    .into();

    wizard_chrome(
        title,
        state.step,
        LogicalMdRaidDialogMessage::SetStep(LogicalWizardStep::Configure).into(),
        LogicalMdRaidDialogMessage::PrevStep.into(),
        Some(LogicalMdRaidDialogMessage::NextStep.into()),
        if state.running {
            None
        } else {
            Some(LogicalMdRaidDialogMessage::Submit.into())
        },
        LogicalMdRaidDialogMessage::Cancel.into(),
        if state.step == LogicalWizardStep::Configure {
            configure.into()
        } else {
            review
        },
    )
}

pub fn logical_btrfs_wizard<'a>(state: LogicalBtrfsWizardDialog) -> Element<'a, Message> {
    let title = format!("BTRFS {}", operation_title(state.operation));

    let mut configure = iced_widget::column![]
        .spacing(12)
        .push(caption("Entity"))
        .push(caption(state.entity_id.clone()));

    match state.operation {
        LogicalOperation::AddMember | LogicalOperation::RemoveMember => {
            configure = configure
                .push(
                    text_input("Member device", state.member_device.clone())
                        .on_input(|v| LogicalBtrfsDialogMessage::MemberDeviceUpdate(v).into()),
                )
                .push(
                    text_input("Mount point", state.mount_point.clone())
                        .on_input(|v| LogicalBtrfsDialogMessage::MountPointUpdate(v).into()),
                );
        }
        LogicalOperation::Resize => {
            configure = configure
                .push(
                    text_input("Size spec", state.size_spec.clone())
                        .on_input(|v| LogicalBtrfsDialogMessage::SizeSpecUpdate(v).into()),
                )
                .push(
                    text_input("Mount point", state.mount_point.clone())
                        .on_input(|v| LogicalBtrfsDialogMessage::MountPointUpdate(v).into()),
                );
        }
        LogicalOperation::SetLabel => {
            configure = configure
                .push(
                    text_input("Label", state.label.clone())
                        .on_input(|v| LogicalBtrfsDialogMessage::LabelUpdate(v).into()),
                )
                .push(
                    text_input("Mount point", state.mount_point.clone())
                        .on_input(|v| LogicalBtrfsDialogMessage::MountPointUpdate(v).into()),
                );
        }
        _ => {
            configure = configure
                .push(
                    text_input("Subvolume ID", state.subvolume_id.clone())
                        .on_input(|v| LogicalBtrfsDialogMessage::SubvolumeIdUpdate(v).into()),
                )
                .push(
                    text_input("Mount point", state.mount_point.clone())
                        .on_input(|v| LogicalBtrfsDialogMessage::MountPointUpdate(v).into()),
                );
        }
    }

    if let Some(error) = state.error.clone() {
        configure = configure.push(caption(error));
    }
    if state.running {
        configure = configure.push(caption("working"));
    }

    let review = iced_widget::column![
        caption(format!("Operation: {}", operation_title(state.operation))),
        caption(format!("Member: {}", state.member_device)),
        caption(format!("Mount: {}", state.mount_point)),
        caption(format!("Size spec: {}", state.size_spec)),
        caption(format!("Label: {}", state.label)),
        caption(format!("Subvolume ID: {}", state.subvolume_id)),
    ]
    .spacing(8)
    .into();

    wizard_chrome(
        title,
        state.step,
        LogicalBtrfsDialogMessage::SetStep(LogicalWizardStep::Configure).into(),
        LogicalBtrfsDialogMessage::PrevStep.into(),
        Some(LogicalBtrfsDialogMessage::NextStep.into()),
        if state.running {
            None
        } else {
            Some(LogicalBtrfsDialogMessage::Submit.into())
        },
        LogicalBtrfsDialogMessage::Cancel.into(),
        if state.step == LogicalWizardStep::Configure {
            configure.into()
        } else {
            review
        },
    )
}

pub fn logical_control_dialog<'a>(state: LogicalControlDialog) -> Element<'a, Message> {
    let title = operation_title(state.operation).to_string();

    let mut content = iced_widget::column![]
        .spacing(12)
        .push(caption("Entity"))
        .push(caption(state.entity_id.clone()));

    match state.operation {
        LogicalOperation::Activate | LogicalOperation::Deactivate => {
            content = content.push(
                text_input("LV path", state.lv_path.clone())
                    .on_input(|v| LogicalControlDialogMessage::LvPathUpdate(v).into()),
            );
        }
        LogicalOperation::Start | LogicalOperation::Stop => {
            content = content.push(
                text_input("Array device", state.array_device.clone())
                    .on_input(|v| LogicalControlDialogMessage::ArrayDeviceUpdate(v).into()),
            );
        }
        LogicalOperation::Check | LogicalOperation::Repair => {
            content = content
                .push(
                    text_input("md name", state.md_name.clone())
                        .on_input(|v| LogicalControlDialogMessage::MdNameUpdate(v).into()),
                )
                .push(
                    text_input("action", state.action.clone())
                        .on_input(|v| LogicalControlDialogMessage::ActionUpdate(v).into()),
                );
        }
        _ => {}
    }

    if let Some(error) = state.error.clone() {
        content = content.push(caption(error));
    }
    if state.running {
        content = content.push(caption("working"));
    }

    let mut apply = button::suggested("Apply");
    if !state.running {
        apply = apply.on_press(LogicalControlDialogMessage::Submit.into());
    }

    let footer = iced_widget::row![
        button::standard("Cancel").on_press(LogicalControlDialogMessage::Cancel.into()),
        cosmic::widget::Space::new(cosmic::iced::Length::Fill, 0),
        apply,
    ]
    .spacing(8);

    crate::controls::wizard::wizard_shell(caption(title).into(), content.into(), footer.into())
}
