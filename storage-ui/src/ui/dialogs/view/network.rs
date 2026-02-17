// SPDX-License-Identifier: GPL-3.0-only

//! Network mount dialog views

use cosmic::widget::text::caption;
use cosmic::widget::{button, dialog, dropdown, text, text_input};
use cosmic::{Element, iced_widget};
use storage_common::rclone::SUPPORTED_REMOTE_TYPES;

use crate::app::Message;
use crate::ui::dialogs::message::RemoteConfigDialogMessage;
use crate::ui::dialogs::state::RemoteConfigDialog;

/// Create remote type options for dropdown
fn remote_type_options() -> Vec<String> {
    SUPPORTED_REMOTE_TYPES
        .iter()
        .map(|s| s.to_string())
        .collect()
}

/// Scope options for dropdown
fn scope_options() -> Vec<String> {
    vec!["User".to_string(), "System".to_string()]
}

/// Get scope index from ConfigScope
fn scope_to_index(scope: storage_common::rclone::ConfigScope) -> usize {
    match scope {
        storage_common::rclone::ConfigScope::User => 0,
        storage_common::rclone::ConfigScope::System => 1,
    }
}

/// Remote configuration dialog for creating/editing RClone remotes
pub fn remote_config<'a>(state: RemoteConfigDialog) -> Element<'a, Message> {
    let RemoteConfigDialog {
        name,
        remote_type_index,
        is_edit,
        running,
        error,
        ..
    } = state;

    let title = if is_edit {
        "Edit Remote"
    } else {
        "Add Remote"
    };

    let remote_types = remote_type_options();
    let scopes = scope_options();
    let scope_index = scope_to_index(state.scope);

    let mut content = iced_widget::column![
        caption("Remote Name"),
        text_input("Enter remote name", name.clone())
            .label("Name")
            .on_input(|t| RemoteConfigDialogMessage::NameUpdate(t).into()),
        caption("Remote Type"),
        dropdown(remote_types, Some(remote_type_index), |idx| {
            RemoteConfigDialogMessage::RemoteTypeIndexUpdate(idx).into()
        })
        .width(cosmic::iced::Length::Fill),
        caption("Configuration Scope"),
        dropdown(scopes, Some(scope_index), |idx| {
            RemoteConfigDialogMessage::ScopeUpdate(idx).into()
        })
        .width(cosmic::iced::Length::Fill),
    ]
    .spacing(12);

    if running {
        content = content.push(text("Saving...").size(11));
    }

    if let Some(error_msg) = error {
        content = content.push(text(error_msg).size(11));
    }

    let mut save_button = button::standard(if is_edit { "Save" } else { "Add" });
    if !running {
        save_button = save_button.on_press(RemoteConfigDialogMessage::Save.into());
    }

    dialog::dialog()
        .title(title)
        .control(content)
        .primary_action(save_button)
        .secondary_action(
            button::standard("Cancel").on_press(RemoteConfigDialogMessage::Cancel.into()),
        )
        .into()
}
