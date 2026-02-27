use crate::app::Message;
use crate::controls::logical::logical_topology_control;
use crate::fl;
use crate::state::logical::{LogicalDetailTab, LogicalState};
use cosmic::iced::Length;
use cosmic::widget;
use cosmic::{Element, iced_widget};
use storage_types::{LogicalEntity, LogicalOperation, bytes_to_pretty};

fn tab_button(label: &'static str, active: bool, msg: Message) -> Element<'static, Message> {
    let mut button = widget::button::text(label);
    button = button.class(if active {
        cosmic::theme::Button::Suggested
    } else {
        cosmic::theme::Button::Text
    });
    button.on_press(msg).into()
}

fn operation_label(operation: LogicalOperation) -> &'static str {
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

fn is_destructive(operation: LogicalOperation) -> bool {
    matches!(
        operation,
        LogicalOperation::Delete
            | LogicalOperation::RemoveMember
            | LogicalOperation::Deactivate
            | LogicalOperation::Stop
            | LogicalOperation::Repair
    )
}

fn operations_for_entity(entity: &LogicalEntity) -> Vec<LogicalOperation> {
    let mut operations = entity.capabilities.supported.clone();
    operations.sort_by_key(|operation| operation_label(*operation));
    operations
}

fn operations_body(entity: &LogicalEntity) -> Element<'static, Message> {
    let mut column = iced_widget::column![widget::text::title4("Operations")].spacing(8);

    let operations = operations_for_entity(entity);

    if operations.is_empty() {
        return column
            .push(widget::text::caption("No operations available"))
            .into();
    }

    for operation in operations {
        let blocked_reason = entity
            .capabilities
            .blocked_reason(operation)
            .map(ToString::to_string);

        let mut button = widget::button::text(operation_label(operation));
        if blocked_reason.is_none() {
            button = button.on_press(Message::OpenLogicalOperationDialog {
                entity_id: entity.id.clone(),
                operation,
            });
        }

        let styled = button.class(if blocked_reason.is_some() {
            cosmic::theme::Button::Text
        } else if is_destructive(operation) {
            cosmic::theme::Button::Destructive
        } else {
            cosmic::theme::Button::Suggested
        });

        let row = if let Some(reason) = blocked_reason {
            iced_widget::row![styled, widget::text::caption(format!("blocked: {reason}"))]
                .spacing(8)
        } else {
            iced_widget::row![styled].spacing(8)
        };

        column = column.push(row);
    }

    column.into()
}

pub(crate) fn logical_detail_view(logical: &LogicalState) -> Element<'_, Message> {
    let Some(entity) = logical.selected_entity() else {
        return widget::container(widget::text::title1(fl!("no-disk-selected")))
            .width(Length::Fill)
            .height(Length::Fill)
            .align_x(cosmic::iced::alignment::Horizontal::Center)
            .align_y(cosmic::iced::alignment::Vertical::Center)
            .into();
    };

    let title_row = iced_widget::row![
        widget::text::title2(entity.name.clone()),
        widget::text::caption(format!(
            "status: {}",
            entity.health_status.as_deref().unwrap_or("unknown")
        )),
    ]
    .spacing(8)
    .align_y(cosmic::iced::Alignment::Center);

    let summary_metrics = iced_widget::row![
        widget::text::caption(format!("Kind: {:?}", entity.kind)),
        widget::text::caption(format!(
            "Size: {}",
            bytes_to_pretty(&entity.size_bytes, false)
        )),
        widget::text::caption(format!("ID: {}", entity.id)),
    ]
    .spacing(12)
    .align_y(cosmic::iced::Alignment::Center);

    let toolbar =
        iced_widget::row![widget::button::text("Refresh").on_press(Message::LoadLogicalEntities)]
            .spacing(8)
            .align_y(cosmic::iced::Alignment::Center);

    let tabs = {
        let mut row = iced_widget::row![
            tab_button(
                "Overview",
                logical.detail_tab == LogicalDetailTab::Overview,
                Message::LogicalDetailTabSelected(LogicalDetailTab::Overview),
            ),
            tab_button(
                "Members",
                logical.detail_tab == LogicalDetailTab::Members,
                Message::LogicalDetailTabSelected(LogicalDetailTab::Members),
            ),
            tab_button(
                "Operations",
                logical.detail_tab == LogicalDetailTab::Operations,
                Message::LogicalDetailTabSelected(LogicalDetailTab::Operations),
            ),
        ]
        .spacing(6);

        if matches!(
            entity.kind,
            storage_types::LogicalEntityKind::BtrfsFilesystem
        ) {
            row = row.push(tab_button(
                "BTRFS",
                logical.detail_tab == LogicalDetailTab::Btrfs,
                Message::LogicalDetailTabSelected(LogicalDetailTab::Btrfs),
            ));
        }

        row
    };

    let body: Element<'_, Message> = match logical.detail_tab {
        LogicalDetailTab::Overview => {
            let used_line = entity
                .used_bytes
                .map(|value| bytes_to_pretty(&value, false))
                .map(|used| format!("Used: {used}"))
                .unwrap_or_else(|| "Used: unknown".to_string());

            iced_widget::column![
                widget::text::caption(used_line),
                widget::text::caption(
                    entity
                        .free_bytes
                        .map(|value| bytes_to_pretty(&value, false))
                        .map(|free| format!("Free: {free}"))
                        .unwrap_or_else(|| "Free: unknown".to_string())
                ),
            ]
            .spacing(6)
            .into()
        }
        LogicalDetailTab::Members => {
            let mut members = iced_widget::column![widget::text::title4("Members")].spacing(6);
            if entity.members.is_empty() {
                members = members.push(widget::text::caption("No members"));
            } else {
                for member in &entity.members {
                    let member_path = member.device_path.clone().unwrap_or_default();
                    members = members.push(widget::text::caption(format!(
                        "{} {}",
                        member.name, member_path
                    )));
                }
            }

            members.into()
        }
        LogicalDetailTab::Operations => operations_body(entity),
        LogicalDetailTab::Btrfs => iced_widget::column![
            widget::text::title4("BTRFS Context"),
            widget::text::caption("BTRFS-specific actions are launched from Operations."),
        ]
        .spacing(6)
        .into(),
    };

    let mut right = iced_widget::column![title_row, summary_metrics, toolbar, tabs, body]
        .spacing(10)
        .width(Length::Fill);

    if let Some(status) = &logical.operation_status {
        right = right.push(widget::text::caption(status));
    }

    widget::container(
        iced_widget::row![
            widget::container(logical_topology_control(logical, true))
                .width(Length::Fixed(280.0))
                .height(Length::Fill),
            widget::container(right)
                .width(Length::Fill)
                .height(Length::Fill),
        ]
        .spacing(12)
        .width(Length::Fill)
        .height(Length::Fill),
    )
    .padding(20)
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn destructive_operations_are_marked() {
        assert!(is_destructive(LogicalOperation::Delete));
        assert!(is_destructive(LogicalOperation::Repair));
        assert!(!is_destructive(LogicalOperation::Activate));
    }
}
