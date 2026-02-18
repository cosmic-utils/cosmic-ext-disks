// SPDX-License-Identifier: GPL-3.0-only

//! View components for network mount management

use super::message::NetworkMessage;
use super::state::{
    NetworkEditorState, NetworkState, NetworkWizardState, QUICK_SETUP_PROVIDERS, SECTION_ORDER,
    WizardStep,
};
use cosmic::cosmic_theme::palette::WithAlpha;
use cosmic::iced::Length;
use cosmic::widget::{self, button, dropdown, icon, text_input};
use cosmic::{Apply, Element, iced_widget};
use std::collections::BTreeMap;
use storage_common::rclone::{ConfigScope, MountStatus, rclone_provider, supported_remote_types};

// ─── Sidebar helpers ─────────────────────────────────────────────────────────

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
fn sidebar_section_header(label: &str, controls_enabled: bool) -> Element<'static, NetworkMessage> {
    let label_widget = widget::text::caption_heading(label.to_string());

    let mut children: Vec<Element<'static, NetworkMessage>> = vec![label_widget.into()];

    if controls_enabled {
        let add_btn = widget::button::custom(icon::from_name("list-add-symbolic").size(20))
            .padding(4)
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
    children.push(sidebar_section_header("Network", controls_enabled));

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

// ─── Shared helpers ──────────────────────────────────────────────────────────

/// Convert a snake_case field name to Title Case for display.
/// e.g. "access_key_id" -> "Access Key Id", "host" -> "Host"
fn pretty_field_name(name: &str) -> String {
    name.split('_')
        .filter(|s| !s.is_empty())
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                Some(first) => {
                    let upper = first.to_uppercase().to_string();
                    format!("{}{}", upper, chars.as_str())
                }
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
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
    if enabled && let Some(message) = message {
        button = button.on_press(message);
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

/// Build a single option field widget (used by both editor and wizard)
fn option_field_widget<F>(
    option: &storage_common::rclone::RcloneProviderOption,
    value: &str,
    on_change: F,
) -> Element<'static, NetworkMessage>
where
    F: Fn(String) -> NetworkMessage + 'static,
{
    let display_name = pretty_field_name(&option.name);
    let label = if option.required {
        format!("{} *", display_name)
    } else {
        display_name
    };

    let placeholder = field_placeholder(option);
    let input = if option.is_secure() {
        text_input::secure_input(placeholder, value.to_owned(), None, true)
            .label(label)
            .on_input(on_change)
    } else {
        text_input(placeholder, value.to_owned())
            .label(label)
            .on_input(on_change)
    };

    let mut col = iced_widget::column![input].spacing(4);

    let help = option.help.trim();
    if !help.is_empty() {
        col = col.push(widget::text::caption(help.to_string()));
    }

    col.into()
}

/// Expander icon helper
fn expander_icon(expanded: bool) -> &'static str {
    if expanded {
        "go-down-symbolic"
    } else {
        "go-next-symbolic"
    }
}

// ─── Editor (for existing remotes and advanced new-remote) ───────────────────

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

/// Build a collapsible section expander for the editor
fn section_expander(
    section_id: &str,
    display_name: &str,
    field_count: usize,
    expanded: bool,
) -> Element<'static, NetworkMessage> {
    let section_id = section_id.to_string();

    let expander_row = iced_widget::row![
        icon::from_name(expander_icon(expanded)).size(16),
        widget::text::body(format!("{display_name} ({field_count})"))
            .font(cosmic::font::semibold()),
    ]
    .spacing(8)
    .align_y(cosmic::iced::Alignment::Center);

    widget::button::custom(expander_row)
        .padding([8, 4])
        .width(Length::Fill)
        .class(cosmic::theme::Button::Text)
        .on_press(NetworkMessage::EditorToggleSection(section_id))
        .into()
}

fn editor_form(
    editor: &NetworkEditorState,
    provider: Option<&storage_common::rclone::RcloneProvider>,
    _controls_enabled: bool,
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
        text_input("Enter remote name", editor.name.clone())
            .label("Remote Name")
            .on_input(NetworkMessage::EditorNameChanged),
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

    // Show advanced / hidden toggle row
    if let Some(provider) = provider {
        let has_advanced = provider
            .options
            .iter()
            .any(|o| o.advanced && !o.is_hidden());
        let has_hidden = provider.options.iter().any(|o| o.is_hidden());

        if has_advanced || has_hidden {
            let mut toggle_row: Vec<Element<'static, NetworkMessage>> = Vec::new();

            if has_advanced {
                toggle_row.push(
                    widget::checkbox("Show advanced options", editor.show_advanced)
                        .on_toggle(NetworkMessage::EditorShowAdvanced)
                        .into(),
                );
            }
            if has_hidden {
                toggle_row.push(
                    widget::checkbox("Show internal options", editor.show_hidden)
                        .on_toggle(NetworkMessage::EditorShowHidden)
                        .into(),
                );
            }

            form = form.push(
                widget::Row::from_vec(toggle_row)
                    .spacing(20)
                    .align_y(cosmic::iced::Alignment::Center),
            );
        }
    }

    // Group options by section
    if let Some(provider) = provider {
        // Collect options into sections, respecting visibility filters
        let mut sections: BTreeMap<
            usize,
            (&str, Vec<&storage_common::rclone::RcloneProviderOption>),
        > = BTreeMap::new();

        for option in &provider.options {
            // Skip hidden options unless show_hidden is on
            if option.is_hidden() && !editor.show_hidden {
                continue;
            }
            // Skip advanced options unless show_advanced is on
            if option.advanced && !option.is_hidden() && !editor.show_advanced {
                continue;
            }

            let section = option.section.as_str();
            let order = SECTION_ORDER
                .iter()
                .position(|s| *s == section)
                .unwrap_or(SECTION_ORDER.len());

            sections
                .entry(order)
                .or_insert_with(|| (section, Vec::new()))
                .1
                .push(option);
        }

        for (section_id, options) in sections.values() {
            let display_name = super::state::section_display_name(section_id);
            let expanded = editor.expanded_sections.contains(*section_id);
            let field_count = options.len();

            form = form.push(section_expander(
                section_id,
                display_name,
                field_count,
                expanded,
            ));

            if expanded {
                let fields: Vec<Element<'static, NetworkMessage>> = options
                    .iter()
                    .map(|option| {
                        let option_name = option.name.clone();
                        let value = editor
                            .options
                            .get(&option_name)
                            .cloned()
                            .unwrap_or_default();
                        option_field_widget(option, &value, move |v| {
                            NetworkMessage::EditorFieldChanged {
                                key: option_name.clone(),
                                value: v,
                            }
                        })
                    })
                    .collect();

                form = form.push(
                    widget::column::with_children(fields)
                        .spacing(8)
                        .padding([0, 0, 0, 24]),
                );
            }
        }
    }

    // Custom options (not in provider definition)
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
        form = form.push(widget::text::caption_heading("Additional Options"));
        let custom_rows: Vec<Element<'static, NetworkMessage>> = custom_keys
            .iter()
            .map(|key| {
                let key_for_input = key.clone();
                let key_for_remove = key.clone();
                let value = editor.options.get(key).cloned().unwrap_or_default();
                let display_key = pretty_field_name(key);
                iced_widget::row![
                    text_input("", value)
                        .label(display_key)
                        .on_input(move |v| NetworkMessage::EditorFieldChanged {
                            key: key_for_input.clone(),
                            value: v,
                        })
                        .width(Length::Fill),
                    button::standard("Remove").on_press(NetworkMessage::EditorRemoveCustomOption {
                        key: key_for_remove.clone(),
                    }),
                ]
                .spacing(6)
                .align_y(cosmic::iced::Alignment::Center)
                .into()
            })
            .collect();
        form = form.push(widget::column::with_children(custom_rows).spacing(6));
    }

    // Add custom option row
    form = form.push(widget::text::caption_heading("Add Custom Option"));
    form = form.push(
        iced_widget::row![
            text_input("Key", editor.new_option_key.clone())
                .label("Option")
                .on_input(NetworkMessage::EditorNewOptionKeyChanged)
                .width(Length::Fill),
            text_input("Value", editor.new_option_value.clone())
                .label("Value")
                .on_input(NetworkMessage::EditorNewOptionValueChanged)
                .width(Length::Fill),
            // Wrap button in a column with top padding to align with the input boxes
            // (the text_inputs have a label above them that adds ~20px)
            widget::container(
                button::standard("Add Option").on_press(NetworkMessage::EditorAddCustomOption),
            )
            .padding([20, 0, 0, 0]),
        ]
        .spacing(8)
        .align_y(cosmic::iced::Alignment::End),
    );

    form.into()
}

fn editor_view(
    state: &NetworkState,
    editor: &NetworkEditorState,
    controls_enabled: bool,
) -> Element<'static, NetworkMessage> {
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

// ─── Wizard ──────────────────────────────────────────────────────────────────

/// Progress indicator showing dots for each step
fn wizard_progress(current: &WizardStep) -> Element<'static, NetworkMessage> {
    let current_num = current.number();
    let total = WizardStep::total();

    let mut dots: Vec<Element<'static, NetworkMessage>> = Vec::new();
    let steps = [
        WizardStep::SelectType,
        WizardStep::NameAndScope,
        WizardStep::Connection,
        WizardStep::Authentication,
        WizardStep::Review,
    ];

    for (i, step) in steps.iter().enumerate() {
        let step_num = i + 1;
        let is_current = step_num == current_num;
        let is_done = step_num < current_num;

        let label = step.label();
        let text = if is_current {
            widget::text::body(label.to_string()).font(cosmic::font::semibold())
        } else {
            widget::text::body(label.to_string())
        };

        let styled_text: Element<'static, NetworkMessage> = widget::container(text)
            .style(move |theme| {
                let color = if is_current {
                    theme.cosmic().accent_color()
                } else if is_done {
                    theme.cosmic().background.component.on
                } else {
                    theme.cosmic().background.component.on.with_alpha(0.4)
                };
                cosmic::iced_widget::container::Style {
                    text_color: Some(color.into()),
                    ..Default::default()
                }
            })
            .into();

        dots.push(styled_text);

        if step_num < total {
            dots.push(
                widget::container(widget::text::caption("  >  ".to_string()))
                    .style(move |theme| cosmic::iced_widget::container::Style {
                        text_color: Some(
                            theme
                                .cosmic()
                                .background
                                .component
                                .on
                                .with_alpha(0.3)
                                .into(),
                        ),
                        ..Default::default()
                    })
                    .into(),
            );
        }
    }

    widget::Row::from_vec(dots)
        .align_y(cosmic::iced::Alignment::Center)
        .into()
}

/// Navigation buttons for wizard (Back / Next / Create)
fn wizard_nav(wizard: &NetworkWizardState) -> Element<'static, NetworkMessage> {
    let mut nav: Vec<Element<'static, NetworkMessage>> = Vec::new();

    // Cancel button (always present)
    nav.push(
        button::standard("Cancel")
            .on_press(NetworkMessage::WizardCancel)
            .into(),
    );

    nav.push(widget::Space::new(Length::Fill, 0).into());

    // Back button (not on first step)
    if wizard.step != WizardStep::SelectType {
        nav.push(
            button::standard("Back")
                .on_press(NetworkMessage::WizardBack)
                .into(),
        );
    }

    // Next / Create button
    let can_advance = wizard.can_advance();
    if wizard.step == WizardStep::Review {
        let mut create_btn = button::suggested("Create");
        if can_advance {
            create_btn = create_btn.on_press(NetworkMessage::WizardCreate);
        }
        nav.push(create_btn.into());
    } else {
        let mut next_btn = button::suggested("Next");
        if can_advance {
            next_btn = next_btn.on_press(NetworkMessage::WizardNext);
        }
        nav.push(next_btn.into());
    }

    widget::Row::from_vec(nav)
        .spacing(8)
        .align_y(cosmic::iced::Alignment::Center)
        .width(Length::Fill)
        .into()
}

/// Step 1: Type selection grid
fn wizard_select_type(wizard: &NetworkWizardState) -> Element<'static, NetworkMessage> {
    let mut cards: Vec<Element<'static, NetworkMessage>> = Vec::new();

    for provider in QUICK_SETUP_PROVIDERS {
        let type_name = provider.type_name.to_string();
        let is_selected = wizard.remote_type == type_name;

        let card_content = iced_widget::column![
            icon::from_name(provider.icon).size(32),
            widget::text::body(provider.label.to_string()).font(cosmic::font::semibold()),
            widget::text::caption(provider.description.to_string()),
        ]
        .spacing(6)
        .align_x(cosmic::iced::Alignment::Center)
        .width(Length::Fill);

        let card = widget::button::custom(
            widget::container(card_content)
                .padding(16)
                .width(Length::Fixed(150.0))
                .height(Length::Fixed(120.0))
                .align_x(cosmic::iced::alignment::Horizontal::Center)
                .align_y(cosmic::iced::alignment::Vertical::Center),
        )
        .class(if is_selected {
            cosmic::theme::Button::Suggested
        } else {
            cosmic::theme::Button::Standard
        })
        .on_press(NetworkMessage::WizardSelectType(type_name));

        cards.push(card.into());
    }

    // "Advanced..." card
    let advanced_content = iced_widget::column![
        icon::from_name("preferences-other-symbolic").size(32),
        widget::text::body("Advanced...".to_string()).font(cosmic::font::semibold()),
        widget::text::caption("All provider types".to_string()),
    ]
    .spacing(6)
    .align_x(cosmic::iced::Alignment::Center)
    .width(Length::Fill);

    let advanced_card = widget::button::custom(
        widget::container(advanced_content)
            .padding(16)
            .width(Length::Fixed(150.0))
            .height(Length::Fixed(120.0))
            .align_x(cosmic::iced::alignment::Horizontal::Center)
            .align_y(cosmic::iced::alignment::Vertical::Center),
    )
    .class(cosmic::theme::Button::Standard)
    .on_press(NetworkMessage::WizardAdvanced);

    cards.push(advanced_card.into());

    // Wrap cards in a responsive grid using Wrap
    let grid = widget::flex_row(cards).row_spacing(12).column_spacing(12);

    iced_widget::column![
        widget::text::title3("Choose a remote type"),
        widget::text::body(
            "Select a common provider below, or choose Advanced to see all available types."
                .to_string()
        ),
        widget::Space::new(0, 8),
        grid,
    ]
    .spacing(8)
    .width(Length::Fill)
    .into()
}

/// Step 2: Name & Scope
fn wizard_name_scope(wizard: &NetworkWizardState) -> Element<'static, NetworkMessage> {
    let scopes = vec!["User".to_string(), "System".to_string()];
    let scope_index = match wizard.scope {
        ConfigScope::User => 0,
        ConfigScope::System => 1,
    };

    let provider_label = rclone_provider(&wizard.remote_type)
        .map(|p| p.description.clone())
        .unwrap_or_else(|| wizard.remote_type.clone());

    let mut col = iced_widget::column![
        widget::text::title3("Name your remote"),
        widget::text::body(format!("Type: {provider_label}")),
        widget::Space::new(0, 8),
        text_input("my-remote", wizard.name.clone())
            .label("Remote Name")
            .on_input(NetworkMessage::WizardSetName),
        widget::text::caption("Use only letters, numbers, dashes, and underscores.".to_string()),
        widget::Space::new(0, 4),
        widget::text::caption("Configuration Scope"),
        dropdown(scopes, Some(scope_index), |idx| {
            NetworkMessage::WizardSetScope(idx)
        })
        .width(Length::Fill),
        widget::text::caption(
            "User scope stores in your home directory. System scope is shared across all users."
                .to_string()
        ),
    ]
    .spacing(8)
    .width(Length::Fill)
    .max_width(500);

    if let Some(error) = &wizard.error {
        col = col.push(widget::text::caption(error.clone()));
    }

    col.into()
}

/// Step 3: Connection settings
fn wizard_connection(wizard: &NetworkWizardState) -> Element<'static, NetworkMessage> {
    let provider = rclone_provider(&wizard.remote_type);

    let mut col = iced_widget::column![
        widget::text::title3("Connection settings"),
        widget::text::body("Configure how to connect to the remote.".to_string()),
        widget::Space::new(0, 8),
    ]
    .spacing(8)
    .width(Length::Fill)
    .max_width(500);

    if let Some(provider) = provider {
        let connection_fields: Vec<_> = provider
            .options
            .iter()
            .filter(|o| o.section == "connection" && !o.advanced && !o.is_hidden())
            .collect();

        if connection_fields.is_empty() {
            col = col.push(widget::text::body(
                "No connection settings required for this provider.".to_string(),
            ));
        } else {
            for option in connection_fields {
                let option_name = option.name.clone();
                let value = wizard
                    .options
                    .get(&option_name)
                    .cloned()
                    .unwrap_or_default();
                col = col.push(option_field_widget(option, &value, move |v| {
                    NetworkMessage::WizardFieldChanged {
                        key: option_name.clone(),
                        value: v,
                    }
                }));
            }
        }
    } else {
        col = col.push(widget::text::body("Unknown provider type.".to_string()));
    }

    if let Some(error) = &wizard.error {
        col = col.push(widget::text::caption(error.clone()));
    }

    col.into()
}

/// Step 4: Authentication settings
fn wizard_authentication(wizard: &NetworkWizardState) -> Element<'static, NetworkMessage> {
    let provider = rclone_provider(&wizard.remote_type);

    let mut col = iced_widget::column![
        widget::text::title3("Authentication"),
        widget::text::body("Enter credentials for the remote.".to_string()),
        widget::Space::new(0, 8),
    ]
    .spacing(8)
    .width(Length::Fill)
    .max_width(500);

    if let Some(provider) = provider {
        let auth_fields: Vec<_> = provider
            .options
            .iter()
            .filter(|o| o.section == "authentication" && !o.advanced && !o.is_hidden())
            .collect();

        if auth_fields.is_empty() {
            col = col.push(widget::text::body(
                "No authentication required for this provider, or authentication is handled via OAuth.".to_string(),
            ));
        } else {
            for option in auth_fields {
                let option_name = option.name.clone();
                let value = wizard
                    .options
                    .get(&option_name)
                    .cloned()
                    .unwrap_or_default();
                col = col.push(option_field_widget(option, &value, move |v| {
                    NetworkMessage::WizardFieldChanged {
                        key: option_name.clone(),
                        value: v,
                    }
                }));
            }
        }
    } else {
        col = col.push(widget::text::body("Unknown provider type.".to_string()));
    }

    if let Some(error) = &wizard.error {
        col = col.push(widget::text::caption(error.clone()));
    }

    col.into()
}

/// Step 5: Review & Create
fn wizard_review(wizard: &NetworkWizardState) -> Element<'static, NetworkMessage> {
    let provider = rclone_provider(&wizard.remote_type);
    let provider_label = provider
        .map(|p| p.description.clone())
        .unwrap_or_else(|| wizard.remote_type.clone());

    let scope_label = match wizard.scope {
        ConfigScope::User => "User",
        ConfigScope::System => "System",
    };

    let mut col = iced_widget::column![
        widget::text::title3("Review"),
        widget::text::body("Review your remote configuration before creating it.".to_string()),
        widget::Space::new(0, 8),
    ]
    .spacing(8)
    .width(Length::Fill)
    .max_width(500);

    // Summary table
    let mut summary = iced_widget::column![
        summary_row("Name", &wizard.name),
        summary_row("Type", &provider_label),
        summary_row("Scope", scope_label),
    ]
    .spacing(6);

    // Show configured options (non-empty only)
    if let Some(provider) = provider {
        for option in &provider.options {
            if let Some(value) = wizard.options.get(&option.name)
                && !value.is_empty()
            {
                let display_value = if option.is_secure() {
                    "********".to_string()
                } else {
                    value.clone()
                };
                summary = summary.push(summary_row(
                    &pretty_field_name(&option.name),
                    &display_value,
                ));
            }
        }
    }

    col = col.push(
        widget::container(summary)
            .padding(16)
            .class(cosmic::style::Container::Card),
    );

    col = col.push(widget::Space::new(0, 4));
    col = col.push(widget::text::caption(
        "You can configure additional options after creating the remote.".to_string(),
    ));

    if wizard.running {
        col = col.push(widget::text::caption("Creating remote...".to_string()));
    }

    if let Some(error) = &wizard.error {
        col = col.push(widget::text::caption(error.clone()));
    }

    col.into()
}

/// Single row in the review summary
fn summary_row(label: &str, value: &str) -> Element<'static, NetworkMessage> {
    iced_widget::row![
        widget::text::body(format!("{label}:")).font(cosmic::font::semibold()),
        widget::text::body(value.to_string()),
    ]
    .spacing(8)
    .align_y(cosmic::iced::Alignment::Center)
    .into()
}

/// Full wizard view
fn wizard_view(wizard: &NetworkWizardState) -> Element<'static, NetworkMessage> {
    let step_content: Element<'static, NetworkMessage> = match wizard.step {
        WizardStep::SelectType => wizard_select_type(wizard),
        WizardStep::NameAndScope => wizard_name_scope(wizard),
        WizardStep::Connection => wizard_connection(wizard),
        WizardStep::Authentication => wizard_authentication(wizard),
        WizardStep::Review => wizard_review(wizard),
    };

    let header = iced_widget::column![
        widget::text::title2("New Remote"),
        wizard_progress(&wizard.step),
    ]
    .spacing(8)
    .width(Length::Fill);

    let content = widget::scrollable(
        widget::container(step_content)
            .padding([8, 0])
            .width(Length::Fill),
    )
    .width(Length::Fill)
    .height(Length::Fill);

    let layout = iced_widget::column![header, content, wizard_nav(wizard),]
        .spacing(12)
        .width(Length::Fill)
        .height(Length::Fill);

    widget::container(layout)
        .padding(20)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

// ─── Main view router ────────────────────────────────────────────────────────

pub fn network_main_view(
    state: &NetworkState,
    controls_enabled: bool,
) -> Element<'static, NetworkMessage> {
    // Wizard takes priority over editor when present
    if let Some(wizard) = &state.wizard {
        return wizard_view(wizard);
    }

    if let Some(editor) = &state.editor {
        return editor_view(state, editor, controls_enabled);
    }

    // Empty state - no editor or wizard active
    let mut empty = iced_widget::column![
        widget::text::title2("Network Mounts"),
        widget::text::body("Select a remote from the sidebar or create a new one."),
    ]
    .spacing(10)
    .width(Length::Fill);

    if controls_enabled {
        empty =
            empty.push(button::standard("New Remote").on_press(NetworkMessage::BeginCreateRemote));
    }

    widget::container(empty)
        .padding(20)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}
