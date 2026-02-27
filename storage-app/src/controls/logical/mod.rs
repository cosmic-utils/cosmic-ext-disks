use crate::app::Message;
use crate::controls::layout::{row_container, transparent_button_class};
use crate::state::logical::LogicalState;
use cosmic::iced::Length;
use cosmic::widget::{self, icon};
use cosmic::{Apply, Element};
use std::collections::BTreeMap;
use storage_types::{LogicalEntity, LogicalEntityKind};

fn entity_icon(kind: LogicalEntityKind) -> &'static str {
    match kind {
        LogicalEntityKind::LvmVolumeGroup => "folder-visiting-symbolic",
        LogicalEntityKind::LvmLogicalVolume => "drive-harddisk-symbolic",
        LogicalEntityKind::LvmPhysicalVolume => "drive-harddisk-symbolic",
        LogicalEntityKind::MdRaidArray => "drive-multidisk-symbolic",
        LogicalEntityKind::MdRaidMember => "drive-harddisk-symbolic",
        LogicalEntityKind::BtrfsFilesystem => "folder-symbolic",
        LogicalEntityKind::BtrfsDevice => "drive-harddisk-symbolic",
        LogicalEntityKind::BtrfsSubvolume => "folder-symbolic",
    }
}

fn sorted_entities(logical: &LogicalState) -> Vec<&LogicalEntity> {
    let mut items: Vec<&LogicalEntity> = logical.entities.iter().collect();
    items.sort_by(|left, right| {
        left.name
            .cmp(&right.name)
            .then_with(|| left.id.cmp(&right.id))
    });
    items
}

fn child_map<'a>(
    entities: &'a [&'a LogicalEntity],
) -> BTreeMap<Option<String>, Vec<&'a LogicalEntity>> {
    let mut map: BTreeMap<Option<String>, Vec<&LogicalEntity>> = BTreeMap::new();
    for entity in entities {
        map.entry(entity.parent_id.clone())
            .or_default()
            .push(*entity);
    }

    for children in map.values_mut() {
        children.sort_by(|left, right| {
            left.name
                .cmp(&right.name)
                .then_with(|| left.id.cmp(&right.id))
        });
    }

    map
}

fn entity_row(
    logical: &LogicalState,
    entity: &LogicalEntity,
    depth: u16,
    controls_enabled: bool,
) -> Element<'static, Message> {
    let selected = logical.selected_entity_id.as_deref() == Some(entity.id.as_str());

    let mut select_button = widget::button::custom(
        widget::Row::with_children(vec![
            icon::from_name(entity_icon(entity.kind)).size(16).into(),
            widget::text::body(entity.name.clone())
                .font(cosmic::font::semibold())
                .into(),
        ])
        .spacing(8)
        .align_y(cosmic::iced::Alignment::Center)
        .width(Length::Fill),
    )
    .padding(0)
    .width(Length::Fill)
    .class(transparent_button_class(selected));

    if controls_enabled {
        select_button = select_button.on_press(Message::SidebarSelectLogical {
            entity_id: entity.id.clone(),
        });
    }

    let indent = depth * 20;
    let row = widget::Row::with_children(vec![
        widget::Space::new(indent, 0).into(),
        select_button.into(),
    ])
    .spacing(8)
    .align_y(cosmic::iced::Alignment::Center)
    .width(Length::Fill);

    row_container(row, selected, controls_enabled)
}

fn member_row(name: String, path: Option<String>, depth: u16) -> Element<'static, Message> {
    let indent = depth * 20;
    let text = if let Some(path) = path {
        format!("{name} ({path})")
    } else {
        name
    };

    widget::Row::with_children(vec![
        widget::Space::new(indent, 0).into(),
        widget::text::caption(text).into(),
    ])
    .spacing(8)
    .align_y(cosmic::iced::Alignment::Center)
    .width(Length::Fill)
    .into()
}

fn push_subtree(
    rows: &mut Vec<Element<'static, Message>>,
    logical: &LogicalState,
    entity: &LogicalEntity,
    children: &BTreeMap<Option<String>, Vec<&LogicalEntity>>,
    depth: u16,
    controls_enabled: bool,
) {
    rows.push(entity_row(logical, entity, depth, controls_enabled));

    if logical.selected_entity_id.as_deref() == Some(entity.id.as_str()) {
        for member in &entity.members {
            rows.push(member_row(
                member.name.clone(),
                member.device_path.clone(),
                depth + 1,
            ));
        }
    }

    if let Some(next) = children.get(&Some(entity.id.clone())) {
        for child in next {
            push_subtree(rows, logical, child, children, depth + 1, controls_enabled);
        }
    }
}

pub(crate) fn logical_topology_control(
    logical: &LogicalState,
    controls_enabled: bool,
) -> Element<'static, Message> {
    let entities = sorted_entities(logical);
    let children = child_map(&entities);

    let mut rows: Vec<Element<'static, Message>> = vec![
        widget::text::caption_heading("Topology")
            .apply(widget::container)
            .padding([0, 0, 6, 0])
            .into(),
    ];

    if let Some(roots) = children.get(&None) {
        for root in roots {
            push_subtree(&mut rows, logical, root, &children, 0, controls_enabled);
        }
    }

    widget::container(widget::scrollable(
        widget::Column::with_children(rows).spacing(4),
    ))
    .padding(10)
    .height(Length::Fill)
    .class(cosmic::style::Container::Card)
    .into()
}

#[cfg(test)]
mod tests {
    use super::*;
    use storage_types::{LogicalCapabilities, LogicalMember};

    fn entity(id: &str, name: &str, parent_id: Option<&str>) -> LogicalEntity {
        LogicalEntity {
            id: id.to_string(),
            kind: LogicalEntityKind::LvmLogicalVolume,
            name: name.to_string(),
            uuid: None,
            parent_id: parent_id.map(ToString::to_string),
            device_path: None,
            size_bytes: 1,
            used_bytes: None,
            free_bytes: None,
            health_status: None,
            progress_fraction: None,
            members: vec![LogicalMember {
                id: "m1".to_string(),
                name: "member-1".to_string(),
                device_path: Some("/dev/sdb1".to_string()),
                role: None,
                state: None,
                size_bytes: None,
            }],
            capabilities: LogicalCapabilities::default(),
            metadata: Default::default(),
        }
    }

    #[test]
    fn sorted_entities_orders_by_name_then_id() {
        let logical = LogicalState {
            entities: vec![
                entity("b2", "beta", None),
                entity("a2", "alpha", None),
                entity("a1", "alpha", None),
            ],
            ..Default::default()
        };

        let ids: Vec<String> = sorted_entities(&logical)
            .into_iter()
            .map(|item| item.id.clone())
            .collect();

        assert_eq!(ids, vec!["a1", "a2", "b2"]);
    }

    #[test]
    fn child_map_groups_by_parent() {
        let root = entity("root", "root", None);
        let child = entity("child", "child", Some("root"));
        let entities = vec![&root, &child];

        let grouped = child_map(&entities);

        assert_eq!(grouped.get(&None).map(Vec::len), Some(1));
        assert_eq!(
            grouped.get(&Some("root".to_string())).map(Vec::len),
            Some(1)
        );
    }
}
