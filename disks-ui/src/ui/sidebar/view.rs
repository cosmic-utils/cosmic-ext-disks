use crate::app::Message;
use crate::fl;
use crate::ui::sidebar::state::{SidebarNodeKey, SidebarState};
use cosmic::iced::Length;
use cosmic::widget::{self, icon};
use cosmic::{Apply, Element};
use disks_dbus::{DriveModel, VolumeKind, VolumeNode};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Section {
    Logical,
    Internal,
    External,
    Images,
}

impl Section {
    fn label(&self) -> String {
        match self {
            Section::Logical => "Logical".to_string(),
            Section::Internal => "Internal".to_string(),
            Section::External => "External".to_string(),
            Section::Images => "Images".to_string(),
        }
    }
}

fn section_for_drive(drive: &DriveModel) -> Section {
    if drive.is_loop || drive.backing_file.is_some() {
        return Section::Images;
    }

    if drive.removable {
        return Section::External;
    }

    Section::Internal
}

fn volume_icon(kind: &VolumeKind) -> &'static str {
    match kind {
        VolumeKind::CryptoContainer => "dialog-password-symbolic",
        VolumeKind::Filesystem => "folder-symbolic",
        VolumeKind::LvmPhysicalVolume => "folder-symbolic",
        VolumeKind::LvmLogicalVolume => "folder-symbolic",
        VolumeKind::Partition => "drive-harddisk-symbolic",
        VolumeKind::Block => "drive-harddisk-symbolic",
    }
}

fn expander_icon(expanded: bool) -> &'static str {
    if expanded {
        "go-down-symbolic"
    } else {
        "go-next-symbolic"
    }
}

fn section_header(label: String) -> Element<'static, Message> {
    widget::text::caption_heading(label)
        .apply(widget::container)
        .padding([8, 12, 4, 12])
        .into()
}

fn menu_item(
    label: String,
    icon_name: &'static str,
    msg: Message,
) -> widget::Button<'static, Message> {
    let row = widget::Row::with_children(vec![
        icon::from_name(icon_name).size(16).into(),
        widget::text(label).into(),
    ])
    .spacing(8)
    .align_y(cosmic::iced::Alignment::Center);

    widget::menu::menu_button(vec![row.into()]).on_press(msg)
}

fn kebab_menu(drive_block_path: &str) -> Element<'static, Message> {
    let drive = drive_block_path.to_string();

    let col = widget::Column::with_children(vec![
        menu_item(
            fl!("eject"),
            "media-eject-symbolic",
            Message::SidebarDriveAction {
                drive: drive.clone(),
                action: crate::ui::app::message::SidebarDriveAction::Eject,
            },
        )
        .into(),
        menu_item(
            fl!("power-off"),
            "system-shutdown-symbolic",
            Message::SidebarDriveAction {
                drive: drive.clone(),
                action: crate::ui::app::message::SidebarDriveAction::PowerOff,
            },
        )
        .into(),
        menu_item(
            fl!("format-disk"),
            "edit-clear-all-symbolic",
            Message::SidebarDriveAction {
                drive: drive.clone(),
                action: crate::ui::app::message::SidebarDriveAction::Format,
            },
        )
        .into(),
        menu_item(
            fl!("smart-data-self-tests"),
            "utilities-system-monitor-symbolic",
            Message::SidebarDriveAction {
                drive: drive.clone(),
                action: crate::ui::app::message::SidebarDriveAction::SmartData,
            },
        )
        .into(),
        menu_item(
            fl!("standby-now"),
            "media-playback-pause-symbolic",
            Message::SidebarDriveAction {
                drive: drive.clone(),
                action: crate::ui::app::message::SidebarDriveAction::StandbyNow,
            },
        )
        .into(),
        menu_item(
            fl!("wake-up-from-standby"),
            "media-playback-start-symbolic",
            Message::SidebarDriveAction {
                drive,
                action: crate::ui::app::message::SidebarDriveAction::Wakeup,
            },
        )
        .into(),
    ])
    .spacing(0)
    .width(Length::Shrink);

    widget::container(col).padding(4).into()
}

fn row_container<'a>(
    row: impl Into<Element<'a, Message>>,
    _selected: bool,
) -> Element<'a, Message> {
    widget::container(row).padding([6, 8]).into()
}

fn drive_row(
    sidebar: &SidebarState,
    drive: &DriveModel,
    active_drive: Option<&str>,
) -> Element<'static, Message> {
    let key = SidebarNodeKey::Drive(drive.block_path.clone());
    let selected = active_drive.is_some_and(|a| a == drive.block_path);

    let expanded = sidebar.is_expanded(&key);
    let has_children = !drive.volumes.is_empty();

    let expander = if has_children {
        widget::button::custom(icon::from_name(expander_icon(expanded)).size(16))
            .padding(2)
            .on_press(Message::SidebarToggleExpanded(key.clone()))
            .into()
    } else {
        widget::Space::new(16, 16).into()
    };

    let drive_icon_name = if drive.removable {
        "drive-removable-media-symbolic"
    } else {
        "disks-symbolic"
    };

    let title_button = widget::button::custom(widget::text(drive.name()))
        .padding(0)
        .width(Length::Fill)
        .on_press(Message::SidebarSelectDrive(drive.block_path.clone()));

    let mut actions: Vec<Element<'static, Message>> = Vec::new();

    // Primary action: eject/remove for removable drives and loop-backed images.
    if drive.is_loop || drive.removable || drive.ejectable {
        actions.push(
            widget::button::custom(icon::from_name("media-eject-symbolic").size(16))
                .padding(4)
                .on_press(Message::SidebarDriveEject(drive.block_path.clone()))
                .into(),
        );
    }

    // Kebab menu.
    let menu_key = SidebarNodeKey::Drive(drive.block_path.clone());
    let open = sidebar.open_menu_for.as_ref() == Some(&menu_key);

    let kebab_button = widget::button::custom(icon::from_name("open-menu-symbolic").size(16))
        .padding(4)
        .on_press(Message::SidebarOpenMenu(menu_key.clone()));

    let kebab = if open {
        widget::popover(kebab_button)
            .position(cosmic::widget::popover::Position::Bottom)
            .on_close(Message::SidebarCloseMenu)
            .popup(kebab_menu(&drive.block_path))
            .into()
    } else {
        widget::popover(kebab_button)
            .position(cosmic::widget::popover::Position::Bottom)
            .on_close(Message::SidebarCloseMenu)
            .into()
    };

    actions.push(kebab);

    let row = widget::Row::with_children(vec![
        expander,
        icon::from_name(drive_icon_name).size(16).into(),
        title_button.into(),
        widget::Row::with_children(actions).spacing(4).into(),
    ])
    .spacing(8)
    .align_y(cosmic::iced::Alignment::Center)
    .width(Length::Fill);

    row_container(row, selected)
}

fn volume_row(
    sidebar: &SidebarState,
    drive_block_path: &str,
    node: &VolumeNode,
    depth: u16,
) -> Element<'static, Message> {
    let key = SidebarNodeKey::Volume(node.object_path.to_string());
    let selected = sidebar.selected_child.as_ref() == Some(&key);

    let expanded = sidebar.is_expanded(&key);
    let has_children = !node.children.is_empty();

    let expander = if has_children {
        widget::button::custom(icon::from_name(expander_icon(expanded)).size(16))
            .padding(2)
            .on_press(Message::SidebarToggleExpanded(key.clone()))
            .into()
    } else {
        widget::Space::new(16, 16).into()
    };

    let title_text = if node.label.trim().is_empty() {
        match node.device_path.as_deref() {
            Some(p) => p.to_string(),
            None => node.object_path.to_string(),
        }
    } else {
        node.label.clone()
    };

    let title_button = widget::button::custom(widget::text(title_text))
        .padding(0)
        .width(Length::Fill)
        .on_press(Message::SidebarSelectChild {
            object_path: node.object_path.to_string(),
        });

    let mut actions: Vec<Element<'static, Message>> = Vec::new();

    if node.is_mounted() {
        actions.push(
            widget::button::custom(icon::from_name("media-eject-symbolic").size(16))
                .padding(4)
                .on_press(Message::SidebarVolumeUnmount {
                    drive: drive_block_path.to_string(),
                    object_path: node.object_path.to_string(),
                })
                .into(),
        );
    }

    // Kebab menu (Disk actions), targeting the parent drive.
    let menu_key = SidebarNodeKey::Volume(node.object_path.to_string());
    let open = sidebar.open_menu_for.as_ref() == Some(&menu_key);

    let kebab_button = widget::button::custom(icon::from_name("open-menu-symbolic").size(16))
        .padding(4)
        .on_press(Message::SidebarOpenMenu(menu_key.clone()));

    let kebab = if open {
        widget::popover(kebab_button)
            .position(cosmic::widget::popover::Position::Bottom)
            .on_close(Message::SidebarCloseMenu)
            .popup(kebab_menu(drive_block_path))
            .into()
    } else {
        widget::popover(kebab_button)
            .position(cosmic::widget::popover::Position::Bottom)
            .on_close(Message::SidebarCloseMenu)
            .into()
    };

    actions.push(kebab);

    let indent = depth * 18;

    let row = widget::Row::with_children(vec![
        widget::Space::new(indent, 0).into(),
        expander,
        icon::from_name(volume_icon(&node.kind)).size(16).into(),
        title_button.into(),
        widget::Row::with_children(actions).spacing(4).into(),
    ])
    .spacing(8)
    .align_y(cosmic::iced::Alignment::Center)
    .width(Length::Fill);

    row_container(row, selected)
}

fn push_volume_tree(
    out: &mut Vec<Element<'static, Message>>,
    sidebar: &SidebarState,
    drive_block_path: &str,
    node: &VolumeNode,
    depth: u16,
) {
    out.push(volume_row(sidebar, drive_block_path, node, depth));

    let key = SidebarNodeKey::Volume(node.object_path.to_string());
    let expanded = sidebar.is_expanded(&key);

    if expanded {
        for child in &node.children {
            push_volume_tree(out, sidebar, drive_block_path, child, depth + 1);
        }
    }
}

pub(crate) fn sidebar(
    app_nav: &cosmic::widget::nav_bar::Model,
    sidebar: &SidebarState,
) -> Element<'static, Message> {
    let active_drive = sidebar.active_drive_block_path(app_nav);

    let mut logical: Vec<&DriveModel> = Vec::new();
    let mut internal: Vec<&DriveModel> = Vec::new();
    let mut external: Vec<&DriveModel> = Vec::new();
    let mut images: Vec<&DriveModel> = Vec::new();

    for d in &sidebar.drives {
        match section_for_drive(d) {
            Section::Logical => logical.push(d),
            Section::Internal => internal.push(d),
            Section::External => external.push(d),
            Section::Images => images.push(d),
        }
    }

    let mut rows: Vec<Element<'static, Message>> = Vec::new();

    let add_section =
        |rows: &mut Vec<Element<'static, Message>>, section: Section, drives: Vec<&DriveModel>| {
            if drives.is_empty() {
                return;
            }
            rows.push(section_header(section.label()));

            for drive in drives {
                rows.push(drive_row(sidebar, drive, active_drive.as_deref()));

                let drive_key = SidebarNodeKey::Drive(drive.block_path.clone());
                if sidebar.is_expanded(&drive_key) {
                    for v in &drive.volumes {
                        push_volume_tree(rows, sidebar, &drive.block_path, v, 1);
                    }
                }
            }
        };

    add_section(&mut rows, Section::Logical, logical);
    add_section(&mut rows, Section::Internal, internal);
    add_section(&mut rows, Section::External, external);
    add_section(&mut rows, Section::Images, images);

    widget::scrollable(widget::Column::with_children(rows).spacing(2))
        .height(Length::Fill)
        .into()
}
