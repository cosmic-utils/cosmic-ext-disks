// SPDX-License-Identifier: GPL-3.0-only

//! View components for network mount management

use super::message::NetworkMessage;
use super::state::{NetworkEditorState, NetworkState};
use cosmic::cosmic_theme::palette::WithAlpha;
use cosmic::iced::Length;
use cosmic::widget::{self, button, dropdown, icon, text_input};
use cosmic::{iced_widget, Apply, Element};
use storage_common::rclone::{rclone_provider, supported_remote_types, ConfigScope, MountStatus};

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
    let config_name = mount.config.name.clone();

    // Name text
    let name_text = widget::text::body(config_name).font(cosmic::font::semibold());

    // Scope icon (left aligned)
    let scope_icon_widget = widget::tooltip(
        icon::from_name(scope_icon(scope)).size(16),
        widget::text::body(scope_label(scope)),
        widget::tooltip::Position::FollowCursor,
    );

    // Main select button
    let mut select_button = widget::button::custom(
        widget::Row::with_children(vec![scope_icon_widget.into(), name_text.into()])
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

    // Compose the row
    let row = widget::Row::with_children(vec![
        widget::Space::new(20, 0).into(), // Indent to match drive tree
        select_button.into(),
    ])
    .spacing(8)
    .align_y(cosmic::iced::Alignment::Center)
    .width(Length::Fill);

    row_container(row, selected, controls_enabled)
}

/// Section header for sidebar
fn section_header(label: &str, controls_enabled: bool) -> Element<'static, NetworkMessage> {
    let label_widget = widget::text::caption_heading(label.to_string());

    let mut children: Vec<Element<'static, NetworkMessage>> = vec![label_widget.into()];

    if controls_enabled {
        let add_btn = widget::button::custom(icon::from_name("list-add-symbolic").size(14))
            .padding(2)
            .class(cosmic::theme::Button::Link)
            .on_press(NetworkMessage::BeginCreateRemote);
        children.push(widget::Space::new(Length::Fill, 0).into());
        children.push(add_btn.into());
    }

    widget::Row::with_children(children)
        .padding([8, 12, 4, 12])
        .align_y(cosmic::iced::Alignment::Center)
        .into()
}

/// Render the Network section for the sidebar
pub fn network_section(
    state: &NetworkState,
    controls_enabled: bool,
) -> Element<'static, NetworkMessage> {
    let mut children: Vec<Element<'static, NetworkMessage>> = Vec::new();

    // Header
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

fn field_placeholder(option: &storage_common::rclone::RcloneProviderOption) -> String {
    if let Some(example) = option.examples.first() {
        return example.value.clone();
    }
    if !option.default_value.is_empty() {
        return option.default_value.clone();
    }
    String::new()
}

fn status_label(status: &MountStatus) -> &'static str {
    match status {
        MountStatus::Mounted => "Mounted",
        MountStatus::Unmounted => "Unmounted",
        MountStatus::Mounting => "Mounting",
        MountStatus::Unmounting => "Unmounting",
        MountStatus::Error(_) => "Error",
    }
}

fn action_button(
    icon_name: &'static str,
    label: &'static str,
    message: Option<NetworkMessage>,
    enabled: bool,
) -> Element<'static, NetworkMessage> {
    let mut button = widget::button::icon(icon::from_name(icon_name).size(16));
    if enabled {
        if let Some(message) = message {
            button = button.on_press(message);
        }
    }
    widget::tooltip(
        button,
        widget::text(label),
        widget::tooltip::Position::Bottom,
    )
    .into()
}

fn status_badge(text: String, running: bool, unsaved: bool) -> Element<'static, NetworkMessage> {
    use cosmic::iced_widget::container;

    let badge = widget::text::caption(text);

    widget::container(badge)
        .style(move |theme| {
            let mut color = theme.cosmic().background.component.on.with_alpha(0.6);
            if running {
                color = theme.cosmic().accent_color();
            } else if unsaved {
                color = theme.cosmic().warning_color();
            }

            container::Style {
                text_color: Some(color.into()),
                ..Default::default()
            }
        })
        .into()
}

fn editor_header(
    editor: &NetworkEditorState,
    selected_mount: Option<&super::state::NetworkMountState>,
    controls_enabled: bool,
) -> Element<'static, NetworkMessage> {
    let title = if editor.name.trim().is_empty() {
        "New Remote".to_string()
    } else {
        editor.name.clone()
    };

    let status_text = selected_mount
        .map(|m| status_label(&m.status).to_string())
        .unwrap_or_else(|| "Not saved".to_string());
    let unsaved = editor.is_new || selected_mount.is_none();
    let status_badge = status_badge(status_text, editor.running, unsaved);

    let can_control =
        selected_mount.is_some_and(|m| !m.loading) && controls_enabled && !editor.running;
    let is_mounted = selected_mount.map(|m| m.is_mounted()).unwrap_or(false);

    let mut actions: Vec<Element<'static, NetworkMessage>> = Vec::new();

    if let Some(mount) = selected_mount {
        actions.push(action_button(
            "media-playback-start-symbolic",
            "Start",
            Some(NetworkMessage::MountRemote {
                name: mount.config.name.clone(),
                scope: mount.config.scope,
            }),
            can_control && !is_mounted,
        ));

        actions.push(action_button(
            "media-playback-stop-symbolic",
            "Stop",
            Some(NetworkMessage::UnmountRemote {
                name: mount.config.name.clone(),
                scope: mount.config.scope,
            }),
            can_control && is_mounted,
        ));

        actions.push(action_button(
            "view-refresh-symbolic",
            "Restart",
            Some(NetworkMessage::RestartRemote {
                name: mount.config.name.clone(),
                scope: mount.config.scope,
            }),
            can_control,
        ));

        actions.push(action_button(
            "emblem-system-symbolic",
            "Test",
            Some(NetworkMessage::TestRemote {
                name: mount.config.name.clone(),
                scope: mount.config.scope,
            }),
            can_control,
        ));
    } else {
        actions.push(action_button(
            "media-playback-start-symbolic",
            "Start",
            None,
            false,
        ));
        actions.push(action_button(
            "media-playback-stop-symbolic",
            "Stop",
            None,
            false,
        ));
        actions.push(action_button(
            "view-refresh-symbolic",
            "Restart",
            None,
            false,
        ));
        actions.push(action_button("emblem-system-symbolic", "Test", None, false));
    }

    actions.push(action_button(
        "document-save-symbolic",
        if editor.is_new { "Create" } else { "Save" },
        Some(NetworkMessage::SaveRemote),
        controls_enabled && !editor.running,
    ));

    if !editor.is_new {
        let delete_message = selected_mount.map(|mount| NetworkMessage::DeleteRemote {
            name: mount.config.name.clone(),
            scope: mount.config.scope,
        });
        actions.push(action_button(
            "edit-delete-symbolic",
            "Delete",
            delete_message,
            controls_enabled && !editor.running && selected_mount.is_some(),
        ));
    }

    let actions_row = widget::Row::from_vec(actions)
        .spacing(4)
        .align_y(cosmic::iced::Alignment::Center)
        .apply(widget::container)
        .padding([0, 10, 0, 0]);

    iced_widget::row![
        widget::text::title2(title),
        widget::Space::new(Length::Fill, 0),
        status_badge,
        actions_row
    ]
    .spacing(12)
    .align_y(cosmic::iced::Alignment::Center)
    .into()
}

fn editor_form(
    editor: &NetworkEditorState,
    provider: Option<&storage_common::rclone::RcloneProvider>,
    controls_enabled: bool,
) -> Element<'static, NetworkMessage> {
    let remote_types: Vec<String> = supported_remote_types().to_vec();
    let remote_type_index = remote_types
        .iter()
        .position(|t| t.eq_ignore_ascii_case(&editor.remote_type))
        .unwrap_or(0);

    let scopes = vec!["User".to_string(), "System".to_string()];
    let scope_index = match editor.scope {
        ConfigScope::User => 0,
        ConfigScope::System => 1,
    };

    let mut form = iced_widget::column![
        widget::text::caption("Remote Name"),
        text_input("Enter remote name", editor.name.clone())
            .label("Name")
            .on_input(|value| NetworkMessage::EditorNameChanged(value)),
        widget::text::caption("Remote Type"),
        dropdown(remote_types, Some(remote_type_index), |idx| {
            NetworkMessage::EditorTypeIndexChanged(idx)
        })
        .width(Length::Fill),
        widget::text::caption("Configuration Scope"),
        dropdown(scopes, Some(scope_index), |idx| {
            NetworkMessage::EditorScopeChanged(idx)
        })
        .width(Length::Fill)
    ]
    .spacing(10);

    if let Some(provider) = provider {
        let mut basic = Vec::new();
        let mut advanced = Vec::new();
        let mut hidden = Vec::new();

        for option in &provider.options {
            let option_name = option.name.clone();
            let value = editor
                .options
                .get(&option_name)
                .cloned()
                .unwrap_or_default();

            let label = if option.required {
                format!("{} *", option_name)
            } else {
                option_name.clone()
            };

            let placeholder = field_placeholder(option);
            let input = if option.is_secure() {
                text_input::secure_input(placeholder, value.clone(), None, true)
                    .label(label)
                    .on_input(move |v| NetworkMessage::EditorFieldChanged {
                        key: option_name.clone(),
                        value: v,
                    })
            } else {
                text_input(placeholder, value.clone())
                    .label(label)
                    .on_input(move |v| NetworkMessage::EditorFieldChanged {
                        key: option_name.clone(),
                        value: v,
                    })
            };

            let help: Option<Element<'static, NetworkMessage>> = if option.help.trim().is_empty() {
                None
            } else {
                Some(widget::text::caption(option.help.trim().to_string()).into())
            };

            let mut field_column = iced_widget::column![input].spacing(4);
            if let Some(help) = help {
                field_column = field_column.push(help);
            }

            let target = if option.is_hidden() {
                &mut hidden
            } else if option.advanced {
                &mut advanced
            } else {
                &mut basic
            };

            target.push(field_column.into());
        }

        if !basic.is_empty() {
            form = form.push(widget::text::caption_heading("Required & Common Options"));
            form = form.push(widget::column::with_children(basic).spacing(8));
        }

        if !advanced.is_empty() {
            form = form.push(
                widget::checkbox("Show advanced options", editor.show_advanced)
                    .on_toggle(NetworkMessage::EditorShowAdvanced),
            );
            if editor.show_advanced {
                form = form.push(widget::column::with_children(advanced).spacing(8));
            }
        }

        if !hidden.is_empty() {
            form = form.push(
                widget::checkbox("Show internal options", editor.show_hidden)
                    .on_toggle(NetworkMessage::EditorShowHidden),
            );
            if editor.show_hidden {
                form = form.push(widget::column::with_children(hidden).spacing(8));
            }
        }
    }

    let provider_option_names: Vec<String> = provider
        .map(|p| p.options.iter().map(|o| o.name.clone()).collect())
        .unwrap_or_default();
    let mut custom_keys: Vec<String> = editor
        .options
        .keys()
        .filter(|k| {
            !provider_option_names
                .iter()
                .any(|p| p.eq_ignore_ascii_case(k))
        })
        .cloned()
        .collect();
    custom_keys.sort();

    if !custom_keys.is_empty() {
        let custom_rows: Vec<Element<'static, NetworkMessage>> = custom_keys
            .iter()
            .map(|key| {
                let key_name = key.clone();
                let key_for_input = key_name.clone();
                let key_for_remove = key_name.clone();
                let value = editor.options.get(&key_name).cloned().unwrap_or_default();
                let mut row = iced_widget::row![text_input("", value)
                    .label(key_name.clone())
                    .on_input(move |v| NetworkMessage::EditorFieldChanged {
                        key: key_for_input.clone(),
                        value: v,
                    })
                    .width(Length::Fill),]
                .spacing(6)
                .align_y(cosmic::iced::Alignment::Center);

                if controls_enabled {
                    let remove_btn = button::standard("Remove").on_press(
                        NetworkMessage::EditorRemoveCustomOption {
                            key: key_for_remove.clone(),
                        },
                    );
                    row = row.push(remove_btn);
                }

                row.into()
            })
            .collect();

        form = form.push(widget::text::caption_heading("Additional Options"));
        form = form.push(widget::column::with_children(custom_rows).spacing(6));
    }

    let mut custom_add_row = iced_widget::row![
        text_input("Key", editor.new_option_key.clone())
            .label("Option")
            .on_input(|v| NetworkMessage::EditorNewOptionKeyChanged(v))
            .width(Length::Fill),
        text_input("Value", editor.new_option_value.clone())
            .label("Value")
            .on_input(|v| NetworkMessage::EditorNewOptionValueChanged(v))
            .width(Length::Fill),
    ]
    .spacing(8)
    .align_y(cosmic::iced::Alignment::Center);

    if controls_enabled {
        let add_btn =
            button::standard("Add Option").on_press(NetworkMessage::EditorAddCustomOption);
        custom_add_row = custom_add_row.push(add_btn);
    }

    form = form.push(widget::text::caption_heading("Add Custom Option"));
    form = form.push(custom_add_row);

    form.into()
}

pub fn network_main_view(
    state: &NetworkState,
    controls_enabled: bool,
) -> Element<'static, NetworkMessage> {
    let Some(editor) = &state.editor else {
        let mut empty = iced_widget::column![
            widget::text::title2("Network Mounts"),
            widget::text::body("Select a remote from the sidebar or create a new one."),
        ]
        .spacing(10)
        .width(Length::Fill);

        if controls_enabled {
            empty = empty
                .push(button::standard("New Remote").on_press(NetworkMessage::BeginCreateRemote));
        }

        return widget::container(empty)
            .padding(20)
            .width(Length::Fill)
            .height(Length::Fill)
            .into();
    };

    let selected_mount = state
        .selected
        .as_ref()
        .and_then(|(name, scope)| state.get_mount(name, *scope));

    let provider = rclone_provider(&editor.remote_type);

    let form = editor_form(editor, provider, controls_enabled);
    let form = widget::container(form).width(Length::Fill).max_width(720);

    let mut layout = iced_widget::column![editor_header(editor, selected_mount, controls_enabled)]
        .spacing(16)
        .width(Length::Fill);

    if let Some(mount) = selected_mount {
        if mount.is_mounted() {
            let mount_point = mount.config.mount_point().to_string_lossy().to_string();
            let mount_row = iced_widget::row![
                widget::text::caption("Mounted at:"),
                widget::button::link(mount_point.clone())
                    .padding(0)
                    .on_press(NetworkMessage::OpenMountPath(mount_point))
            ]
            .spacing(4)
            .align_y(cosmic::iced::Alignment::Center);
            layout = layout.push(mount_row);
        } else {
            layout = layout.push(widget::text::caption("Not mounted"));
        }
    }

    if !editor.is_new {
        let checked = editor.mount_on_boot.unwrap_or(false);
        let mut mount_on_boot = widget::checkbox("Mount on boot", checked);
        if controls_enabled && !editor.running && editor.mount_on_boot.is_some() {
            mount_on_boot = mount_on_boot.on_toggle(NetworkMessage::ToggleMountOnBoot);
        }
        layout = layout.push(mount_on_boot);
    }

    layout = layout.push(form);

    if editor.running {
        layout = layout.push(widget::text::caption("Saving configuration..."));
    }

    if let Some(error) = &editor.error {
        layout = layout.push(widget::text::caption(error.clone()));
    }

    widget::scrollable(layout)
        .width(Length::Fill)
        .height(Length::Fill)
        .apply(widget::container)
        .padding(20)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}
