use crate::app::Message;
use crate::models::{UiDrive, UiVolume};
use crate::ui::network::{NetworkState, view::network_section};
use crate::ui::sidebar::state::{SidebarNodeKey, SidebarState};
use cosmic::cosmic_theme::palette::WithAlpha;
use cosmic::iced::Length;
use cosmic::widget::{self, icon};
use cosmic::{Apply, Element};
use storage_common::VolumeKind;

/// Fixed width for expander button (icon 16px + padding 2px * 2)
const EXPANDER_WIDTH: u16 = 20;

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

fn section_for_drive(drive: &UiDrive) -> Section {
    if drive.disk.is_loop || drive.disk.backing_file.is_some() {
        return Section::Images;
    }

    if drive.disk.removable {
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

fn drive_title(drive: &UiDrive) -> String {
    if let Some(path) = drive.disk.backing_file.as_deref()
        && !path.trim().is_empty()
        && let Some(name) = path.rsplit('/').next()
        && !name.trim().is_empty()
    {
        return name.to_string();
    }

    let vendor = drive.disk.vendor.trim();
    let model = drive.disk.model.trim();

    if vendor.is_empty() && model.is_empty() {
        return drive.disk.display_name();
    }

    if vendor.is_empty() {
        return model.to_string();
    }

    if model.is_empty() {
        return vendor.to_string();
    }

    if model.to_lowercase().starts_with(&vendor.to_lowercase()) {
        model.to_string()
    } else {
        format!("{vendor} {model}")
    }
}

fn transparent_button_class(selected: bool) -> cosmic::theme::Button {
    cosmic::theme::Button::Custom {
        active: Box::new(move |_b, theme| transparent_button_style(selected, false, theme)),
        disabled: Box::new(move |theme| transparent_button_style(selected, true, theme)),
        hovered: Box::new(move |_b, theme| transparent_button_style(selected, false, theme)),
        pressed: Box::new(move |_b, theme| transparent_button_style(selected, false, theme)),
    }
}

fn transparent_button_style(
    selected: bool,
    disabled: bool,
    theme: &cosmic::theme::Theme,
) -> cosmic::widget::button::Style {
    use cosmic::cosmic_theme::palette::WithAlpha;

    let component = &theme.cosmic().background.component;

    let mut on = component.on;
    if !disabled && selected {
        on = theme.cosmic().accent_color();
    } else if disabled {
        on = on.with_alpha(0.35);
    }

    cosmic::widget::button::Style {
        shadow_offset: Default::default(),
        background: None,
        overlay: None,
        border_radius: (theme.cosmic().corner_radii.radius_xs).into(),
        border_width: 0.0,
        border_color: component.base.with_alpha(0.0).into(),
        outline_width: 0.0,
        outline_color: component.base.with_alpha(0.0).into(),
        icon_color: Some(on.into()),
        text_color: Some(on.into()),
    }
}

fn section_header(label: String) -> Element<'static, Message> {
    widget::text::caption_heading(label)
        .apply(widget::container)
        .padding([8, 12, 4, 12])
        .into()
}

fn row_container<'a>(
    row: impl Into<Element<'a, Message>>,
    selected: bool,
    enabled: bool,
) -> Element<'a, Message> {
    widget::container(row)
        .padding([6, 8])
        .class(cosmic::style::Container::custom(move |theme| {
            use cosmic::iced::{Border, Shadow};

            // Match the visual background used by `cosmic::style::Container::Card`.
            let component = &theme.cosmic().background.component;

            let mut on = component.on;

            if !enabled {
                // Keep the card background, but visually de-emphasize content.
                on = component.on.with_alpha(0.35);
            } else if selected {
                on = theme.cosmic().accent_color();
            }

            cosmic::iced_widget::container::Style {
                icon_color: Some(on.into()),
                text_color: Some(on.into()),
                background: None,
                border: Border {
                    radius: theme.cosmic().corner_radii.radius_s.into(),
                    ..Default::default()
                },
                shadow: Shadow::default(),
            }
        }))
        .into()
}

fn drive_row(
    sidebar: &SidebarState,
    drive: &UiDrive,
    active_drive: Option<&str>,
    controls_enabled: bool,
) -> Element<'static, Message> {
    let key = SidebarNodeKey::Drive(drive.block_path().to_string());
    let selected = active_drive.is_some_and(|a| a == drive.block_path());

    let expanded = sidebar.is_expanded(&key);
    let has_children = !drive.volumes.is_empty();

    let expander = if has_children {
        let mut button =
            widget::button::custom(icon::from_name(expander_icon(expanded)).size(16)).padding(2);
        button = button.class(transparent_button_class(selected));
        if controls_enabled {
            button = button.on_press(Message::SidebarToggleExpanded(key.clone()));
        }
        button.into()
    } else {
        widget::Space::new(EXPANDER_WIDTH, EXPANDER_WIDTH).into()
    };

    let drive_icon_name = if drive.disk.removable {
        "drive-removable-media-symbolic"
    } else {
        "disks-symbolic"
    };

    let title = drive_title(drive);

    let mut select_button = widget::button::custom(
        widget::Row::with_children(vec![
            icon::from_name(drive_icon_name).size(16).into(),
            widget::text::body(title)
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
        select_button =
            select_button.on_press(Message::SidebarSelectDrive(drive.block_path().to_string()));
    }

    let mut actions: Vec<Element<'static, Message>> = Vec::new();

    // Primary action: eject/remove for removable drives and loop-backed images.
    if drive.disk.is_loop || drive.disk.removable || drive.disk.ejectable {
        let mut eject_btn =
            widget::button::custom(icon::from_name("media-eject-symbolic").size(16)).padding(4);
        eject_btn = eject_btn.class(transparent_button_class(selected));
        if controls_enabled {
            eject_btn =
                eject_btn.on_press(Message::SidebarDriveEject(drive.block_path().to_string()));
        }
        actions.push(eject_btn.into());
    }

    let row = widget::Row::with_children(vec![
        expander,
        select_button.into(),
        widget::Row::with_children(actions).spacing(4).into(),
    ])
    .spacing(8)
    .align_y(cosmic::iced::Alignment::Center)
    .width(Length::Fill);

    row_container(row, selected, controls_enabled)
}

fn volume_row(
    sidebar: &SidebarState,
    drive_block_path: &str,
    node: &UiVolume,
    depth: u16,
    controls_enabled: bool,
) -> Element<'static, Message> {
    let key = SidebarNodeKey::Volume(node.device_path().unwrap_or_default());
    let selected = sidebar.selected_child.as_ref() == Some(&key);

    let expanded = sidebar.is_expanded(&key);
    let has_children = !node.children.is_empty();

    let expander = if has_children {
        let mut button =
            widget::button::custom(icon::from_name(expander_icon(expanded)).size(16)).padding(2);
        button = button.class(transparent_button_class(selected));
        if controls_enabled {
            button = button.on_press(Message::SidebarToggleExpanded(key.clone()));
        }
        button.into()
    } else {
        widget::Space::new(EXPANDER_WIDTH, EXPANDER_WIDTH).into()
    };

    let title_text = if node.volume.label.trim().is_empty() {
        match node.volume.device_path.as_deref() {
            Some(p) => p.to_string(),
            None => node.device_path().unwrap_or_default(),
        }
    } else {
        node.volume.label.clone()
    };

    let select_msg = Message::SidebarSelectChild {
        device_path: node.device_path().unwrap_or_default(),
    };

    let mut select_button = widget::button::custom(
        widget::Row::with_children(vec![
            icon::from_name(volume_icon(&node.volume.kind))
                .size(16)
                .into(),
            widget::text::body(title_text)
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
        select_button = select_button.on_press(select_msg.clone());
    }

    let mut actions: Vec<Element<'static, Message>> = Vec::new();

    if node.is_mounted() {
        let mut unmount_btn =
            widget::button::custom(icon::from_name("media-eject-symbolic").size(16)).padding(4);
        unmount_btn = unmount_btn.class(transparent_button_class(selected));
        if controls_enabled {
            unmount_btn = unmount_btn.on_press(Message::SidebarVolumeUnmount {
                drive: drive_block_path.to_string(),
                device_path: node.device_path().unwrap_or_default(),
            });
        }
        actions.push(unmount_btn.into());
    }

    // Indent accounts for expander width + spacing between elements
    // Each level indents by one expander width (20px) + row spacing (8px)
    const ROW_SPACING: u16 = 8;
    let indent = depth * (EXPANDER_WIDTH + ROW_SPACING);

    let row = widget::Row::with_children(vec![
        expander,
        select_button.into(),
        widget::Row::with_children(actions).spacing(4).into(),
    ])
    .spacing(ROW_SPACING)
    .align_y(cosmic::iced::Alignment::Center)
    .width(Length::Fill);

    let item = row_container(row, selected, controls_enabled);

    if indent > 0 {
        widget::Row::with_children(vec![widget::Space::new(indent, 0).into(), item])
            .spacing(0)
            .align_y(cosmic::iced::Alignment::Center)
            .width(Length::Fill)
            .into()
    } else {
        item
    }
}

fn push_volume_tree(
    out: &mut Vec<Element<'static, Message>>,
    sidebar: &SidebarState,
    drive_block_path: &str,
    node: &UiVolume,
    depth: u16,
    controls_enabled: bool,
) {
    out.push(volume_row(
        sidebar,
        drive_block_path,
        node,
        depth,
        controls_enabled,
    ));

    let key = SidebarNodeKey::Volume(node.device_path().unwrap_or_default());
    let expanded = sidebar.is_expanded(&key);

    if expanded {
        // Sort children by device_path to maintain disk offset order
        let mut sorted_children: Vec<&UiVolume> = node.children.iter().collect();
        sorted_children.sort_by(|a, b| {
            a.device_path()
                .as_deref()
                .unwrap_or("")
                .cmp(b.device_path().as_deref().unwrap_or(""))
        });

        for child in sorted_children {
            push_volume_tree(
                out,
                sidebar,
                drive_block_path,
                child,
                depth + 1,
                controls_enabled,
            );
        }
    }
}

pub(crate) fn sidebar(
    app_nav: &cosmic::widget::nav_bar::Model,
    sidebar: &SidebarState,
    network: &NetworkState,
    controls_enabled: bool,
) -> Element<'static, Message> {
    let active_drive = sidebar.active_drive_block_path(app_nav);

    let mut logical: Vec<&UiDrive> = Vec::new();
    let mut internal: Vec<&UiDrive> = Vec::new();
    let mut external: Vec<&UiDrive> = Vec::new();
    let mut images: Vec<&UiDrive> = Vec::new();

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
        |rows: &mut Vec<Element<'static, Message>>, section: Section, drives: Vec<&UiDrive>| {
            if drives.is_empty() {
                return;
            }
            rows.push(section_header(section.label()));

            for drive in drives {
                rows.push(drive_row(
                    sidebar,
                    drive,
                    active_drive.as_deref(),
                    controls_enabled,
                ));

                let drive_key = SidebarNodeKey::Drive(drive.block_path().to_string());
                if sidebar.is_expanded(&drive_key) {
                    // Sort volumes by offset to maintain disk order
                    let mut sorted_volumes: Vec<&UiVolume> = drive.volumes.iter().collect();

                    // To get offset, we need to look up the corresponding PartitionInfo by matching device_path with device
                    sorted_volumes.sort_by(|a, b| {
                        // Find offset for each volume by matching device_path with partitions
                        let offset_a = a
                            .volume
                            .device_path
                            .as_ref()
                            .and_then(|dev| drive.partitions.iter().find(|p| &p.device == dev))
                            .map(|p| p.offset)
                            .unwrap_or(0);
                        let offset_b = b
                            .volume
                            .device_path
                            .as_ref()
                            .and_then(|dev| drive.partitions.iter().find(|p| &p.device == dev))
                            .map(|p| p.offset)
                            .unwrap_or(0);
                        offset_a.cmp(&offset_b)
                    });

                    for v in sorted_volumes {
                        push_volume_tree(rows, sidebar, drive.block_path(), v, 1, controls_enabled);
                    }
                }
            }
        };

    add_section(&mut rows, Section::Logical, logical);
    add_section(&mut rows, Section::Internal, internal);
    add_section(&mut rows, Section::External, external);
    add_section(&mut rows, Section::Images, images);

    // Network section (RClone, Samba, FTP)
    if network.rclone_available || !network.mounts.is_empty() {
        rows.push(network_section(network, controls_enabled).map(Message::Network));
    }

    // Image operations buttons at bottom - reduced size with wrapping for 50/50 layout
    let image_buttons = widget::row::with_capacity(2)
        .push(
            widget::button::custom(
                widget::text::caption(crate::fl!("new-disk-image"))
                    .width(Length::Fill)
                    .center()
                    .wrapping(cosmic::iced::widget::text::Wrapping::Word),
            )
            .class(cosmic::theme::Button::Link)
            .on_press(Message::NewDiskImage)
            .width(Length::Fill)
            .padding(8),
        )
        .push(
            widget::button::custom(
                widget::text::caption(crate::fl!("attach-disk-image"))
                    .width(Length::Fill)
                    .center()
                    .wrapping(cosmic::iced::widget::text::Wrapping::Word),
            )
            .class(cosmic::theme::Button::Link)
            .on_press(Message::AttachDisk)
            .width(Length::Fill)
            .padding(8),
        )
        .spacing(5)
        .padding([10, 10]);

    widget::container::Container::new(
        widget::column::with_capacity(2)
            .push(
                widget::scrollable(widget::Column::with_children(rows).spacing(2))
                    .height(Length::Fill),
            )
            .push(image_buttons),
    )
    .class(cosmic::style::Container::Card)
    .into()
}
