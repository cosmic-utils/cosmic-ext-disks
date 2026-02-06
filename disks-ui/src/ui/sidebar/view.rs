use crate::app::Message;
use crate::ui::sidebar::state::{SidebarNodeKey, SidebarState};
use cosmic::cosmic_theme::palette::WithAlpha;
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

fn drive_title(drive: &DriveModel) -> String {
    if let Some(path) = drive.backing_file.as_deref()
        && !path.trim().is_empty()
        && let Some(name) = path.rsplit('/').next()
        && !name.trim().is_empty()
    {
        return name.to_string();
    }

    let vendor = drive.vendor.trim();
    let model = drive.model.trim();

    if vendor.is_empty() && model.is_empty() {
        return drive.name();
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
    drive: &DriveModel,
    active_drive: Option<&str>,
    controls_enabled: bool,
) -> Element<'static, Message> {
    let key = SidebarNodeKey::Drive(drive.block_path.clone());
    let selected = active_drive.is_some_and(|a| a == drive.block_path);

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
        widget::Space::new(16, 16).into()
    };

    let drive_icon_name = if drive.removable {
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
            select_button.on_press(Message::SidebarSelectDrive(drive.block_path.clone()));
    }

    let mut actions: Vec<Element<'static, Message>> = Vec::new();

    // Primary action: eject/remove for removable drives and loop-backed images.
    if drive.is_loop || drive.removable || drive.ejectable {
        let mut eject_btn =
            widget::button::custom(icon::from_name("media-eject-symbolic").size(16)).padding(4);
        eject_btn = eject_btn.class(transparent_button_class(selected));
        if controls_enabled {
            eject_btn = eject_btn.on_press(Message::SidebarDriveEject(drive.block_path.clone()));
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
    node: &VolumeNode,
    depth: u16,
    controls_enabled: bool,
) -> Element<'static, Message> {
    let key = SidebarNodeKey::Volume(node.object_path.to_string());
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

    let select_msg = Message::SidebarSelectChild {
        object_path: node.object_path.to_string(),
    };

    let mut select_button = widget::button::custom(
        widget::Row::with_children(vec![
            icon::from_name(volume_icon(&node.kind)).size(16).into(),
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
                object_path: node.object_path.to_string(),
            });
        }
        actions.push(unmount_btn.into());
    }

    let indent = depth * 18;

    let row = widget::Row::with_children(vec![
        expander,
        select_button.into(),
        widget::Row::with_children(actions).spacing(4).into(),
    ])
    .spacing(8)
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
    node: &VolumeNode,
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

    let key = SidebarNodeKey::Volume(node.object_path.to_string());
    let expanded = sidebar.is_expanded(&key);

    if expanded {
        for child in &node.children {
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
    controls_enabled: bool,
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
                rows.push(drive_row(
                    sidebar,
                    drive,
                    active_drive.as_deref(),
                    controls_enabled,
                ));

                let drive_key = SidebarNodeKey::Drive(drive.block_path.clone());
                if sidebar.is_expanded(&drive_key) {
                    for v in &drive.volumes {
                        push_volume_tree(rows, sidebar, &drive.block_path, v, 1, controls_enabled);
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
