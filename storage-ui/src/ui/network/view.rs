// SPDX-License-Identifier: GPL-3.0-only

//! View components for network mount management

use super::message::NetworkMessage;
use super::state::NetworkState;
use cosmic::cosmic_theme::palette::WithAlpha;
use cosmic::iced::Length;
use cosmic::widget::{self, icon};
use cosmic::{Apply, Element};
use storage_common::rclone::{ConfigScope, MountStatus};

/// Icon for mount status
fn mount_status_icon(status: &MountStatus) -> &'static str {
    match status {
        MountStatus::Mounted => "folder-remote-symbolic",
        MountStatus::Unmounted => "folder-symbolic",
        MountStatus::Mounting | MountStatus::Unmounting => "folder-download-symbolic",
        MountStatus::Error(_) => "dialog-warning-symbolic",
    }
}

/// Icon for scope badge
fn scope_icon(scope: ConfigScope) -> &'static str {
    match scope {
        ConfigScope::User => "user-home-symbolic",
        ConfigScope::System => "computer-symbolic",
    }
}

/// Scope label for accessibility/tooltips
fn scope_label(scope: ConfigScope) -> &'static str {
    match scope {
        ConfigScope::User => "User",
        ConfigScope::System => "System",
    }
}

/// Container for a row matching sidebar style
fn row_container(
    row: impl Into<Element<'static, NetworkMessage>>,
    selected: bool,
    enabled: bool,
) -> Element<'static, NetworkMessage> {
    widget::container(row)
        .padding([6, 8])
        .class(cosmic::style::Container::custom(move |theme| {
            use cosmic::iced::{Border, Shadow};

            let component = &theme.cosmic().background.component;

            let mut on = component.on;

            if !enabled {
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

/// Transparent button class matching sidebar style
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

/// Render a single network mount item for the sidebar
pub fn network_mount_item(
    state: &NetworkState,
    name: &str,
    scope: ConfigScope,
    controls_enabled: bool,
) -> Element<'static, NetworkMessage> {
    let mount = match state.get_mount(name, scope) {
        Some(m) => m,
        None => return widget::text::body("Unknown mount").into(),
    };

    // Extract all data needed before building UI
    let selected = state.is_selected(name, scope);
    let loading = mount.loading;
    let is_mounted = mount.is_mounted();
    let status = mount.status.clone();
    let config_name = mount.config.name.clone();

    // Status icon
    let status_icon = if loading {
        widget::tooltip(
            icon::from_name("folder-download-symbolic").size(16),
            widget::text::body(if status == MountStatus::Mounting {
                "Mounting..."
            } else {
                "Unmounting..."
            }),
            widget::tooltip::Position::FollowCursor,
        )
        .into()
    } else {
        icon::from_name(mount_status_icon(&status)).size(16).into()
    };

    // Name text
    let name_text = widget::text::body(config_name);

    // Scope badge
    let scope_badge = widget::tooltip(
        icon::from_name(scope_icon(scope)).size(12),
        widget::text::body(scope_label(scope)),
        widget::tooltip::Position::FollowCursor,
    );

    // Main select button
    let mut select_button = widget::button::custom(
        widget::Row::with_children(vec![status_icon, name_text.into(), scope_badge.into()])
            .spacing(8)
            .align_y(cosmic::iced::Alignment::Center)
            .width(Length::Fill),
    )
    .padding(0)
    .width(Length::Fill)
    .class(transparent_button_class(selected));

    if controls_enabled && !loading {
        select_button = select_button.on_press(NetworkMessage::SelectRemote {
            name: name.to_string(),
            scope,
        });
    }

    // Action buttons
    let mut actions: Vec<Element<'static, NetworkMessage>> = Vec::new();

    // Mount/Unmount button
    if !loading {
        let (action_msg, action_icon) = if is_mounted {
            (
                NetworkMessage::UnmountRemote {
                    name: name.to_string(),
                    scope,
                },
                "media-eject-symbolic",
            )
        } else {
            (
                NetworkMessage::MountRemote {
                    name: name.to_string(),
                    scope,
                },
                "folder-download-symbolic",
            )
        };

        let mut action_btn =
            widget::button::custom(icon::from_name(action_icon).size(16)).padding(4);
        action_btn = action_btn.class(transparent_button_class(selected));
        if controls_enabled {
            action_btn = action_btn.on_press(action_msg);
        }
        actions.push(action_btn.into());

        // Test configuration button
        let mut test_btn = widget::button::custom(
            icon::from_name("network-wireless-signal-excellent-symbolic").size(16),
        )
        .padding(4);
        test_btn = test_btn.class(transparent_button_class(selected));
        if controls_enabled {
            test_btn = test_btn.on_press(NetworkMessage::TestRemote {
                name: name.to_string(),
                scope,
            });
        }
        actions.push(test_btn.into());

        // Edit configuration button
        let mut edit_btn =
            widget::button::custom(icon::from_name("document-edit-symbolic").size(16)).padding(4);
        edit_btn = edit_btn.class(transparent_button_class(selected));
        if controls_enabled {
            edit_btn = edit_btn.on_press(NetworkMessage::OpenEditRemote {
                name: name.to_string(),
                scope,
            });
        }
        actions.push(edit_btn.into());

        // Delete configuration button
        let mut delete_btn =
            widget::button::custom(icon::from_name("user-trash-symbolic").size(16)).padding(4);
        delete_btn = delete_btn.class(transparent_button_class(selected));
        if controls_enabled {
            delete_btn = delete_btn.on_press(NetworkMessage::DeleteRemote {
                name: name.to_string(),
                scope,
            });
        }
        actions.push(delete_btn.into());
    }

    // Spacing for action row
    let action_row = widget::Row::with_children(actions).spacing(4);

    // Compose the row
    let row = widget::Row::with_children(vec![
        widget::Space::new(20, 0).into(), // Indent to match drive tree
        select_button.into(),
        action_row.into(),
    ])
    .spacing(8)
    .align_y(cosmic::iced::Alignment::Center)
    .width(Length::Fill);

    row_container(row, selected, controls_enabled)
}

/// Section header for sidebar with optional add button
fn section_header(label: &str, controls_enabled: bool) -> Element<'static, NetworkMessage> {
    let label_widget = widget::text::caption_heading(label.to_string());

    if controls_enabled {
        // Add button with plus icon
        let add_btn = widget::button::custom(icon::from_name("list-add-symbolic").size(14))
            .padding(2)
            .class(cosmic::theme::Button::Link)
            .on_press(NetworkMessage::OpenAddRemote);

        widget::Row::with_children(vec![label_widget.into(), widget::Space::new(Length::Fill, 0).into(), add_btn.into()])
            .padding([8, 12, 4, 12])
            .align_y(cosmic::iced::Alignment::Center)
            .into()
    } else {
        label_widget
            .apply(widget::container)
            .padding([8, 12, 4, 12])
            .into()
    }
}

/// Render the Network section for the sidebar
pub fn network_section(
    state: &NetworkState,
    controls_enabled: bool,
) -> Element<'static, NetworkMessage> {
    let mut children: Vec<Element<'static, NetworkMessage>> = Vec::new();

    // Header with add button
    children.push(section_header("Network", controls_enabled));

    // Loading state
    if state.loading {
        children.push(
            widget::container(widget::text::body("Loading remotes..."))
                .padding([4, 12])
                .into(),
        );
    } else if !state.rclone_available {
        children.push(
            widget::container(widget::text::body("RClone not available"))
                .padding([4, 12])
                .into(),
        );
    } else if state.mounts.is_empty() {
        // Empty state
        children.push(
            widget::container(widget::text::body("No network mounts configured"))
                .padding([4, 12])
                .into(),
        );
    } else {
        // Collect mount info to avoid borrowing issues
        let mount_info: Vec<(String, ConfigScope)> = state
            .mounts
            .values()
            .map(|m| (m.config.name.clone(), m.config.scope))
            .collect();

        // Separate user and system mounts
        let (user_mounts, system_mounts): (Vec<_>, Vec<_>) = mount_info
            .into_iter()
            .partition(|(_, scope)| *scope == ConfigScope::User);

        let has_user_mounts = !user_mounts.is_empty();

        for (name, scope) in &user_mounts {
            children.push(network_mount_item(state, name, *scope, controls_enabled));
        }

        if !system_mounts.is_empty() {
            if has_user_mounts {
                children.push(widget::Space::new(0, 4).into());
            }
            for (name, scope) in &system_mounts {
                children.push(network_mount_item(state, name, *scope, controls_enabled));
            }
        }
    }

    widget::column::with_children(children)
        .spacing(2)
        .width(Length::Fill)
        .into()
}
