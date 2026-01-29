use crate::app::Message;
use crate::fl;
use crate::ui::dialogs::message::EditMountOptionsMessage;
use crate::ui::dialogs::state::EditMountOptionsDialog;
use cosmic::{
    Element, iced_widget,
    widget::text::{caption, caption_heading},
    widget::{button, checkbox, dialog, dropdown, text_input},
};

pub fn edit_mount_options<'a>(state: EditMountOptionsDialog) -> Element<'a, Message> {
    let EditMountOptionsDialog {
        target: _,
        use_defaults,
        mount_at_startup,
        require_auth,
        show_in_ui,
        other_options,
        display_name,
        icon_name,
        symbolic_icon_name,
        mount_point,
        identify_as_options,
        identify_as_index,
        filesystem_type,
        error,
        running,
    } = state;

    let other_options_for_input = other_options.clone();
    let mount_point_for_input = mount_point.clone();
    let filesystem_type_for_input = filesystem_type.clone();

    let controls_enabled = !use_defaults;

    let mut defaults_cb = checkbox(fl!("user-session-defaults"), use_defaults);
    if !running {
        defaults_cb =
            defaults_cb.on_toggle(|v| EditMountOptionsMessage::UseDefaultsUpdate(v).into());
    }

    let mut mount_start_cb = checkbox(fl!("mount-at-startup"), mount_at_startup);
    if controls_enabled && !running {
        mount_start_cb =
            mount_start_cb.on_toggle(|v| EditMountOptionsMessage::MountAtStartupUpdate(v).into());
    }

    let mut auth_cb = checkbox(fl!("require-auth-to-mount"), require_auth);
    if controls_enabled && !running {
        auth_cb = auth_cb.on_toggle(|v| EditMountOptionsMessage::RequireAuthUpdate(v).into());
    }

    let mut show_cb = checkbox(fl!("show-in-ui"), show_in_ui);
    if controls_enabled && !running {
        show_cb = show_cb.on_toggle(|v| EditMountOptionsMessage::ShowInUiUpdate(v).into());
    }

    let mut identify_dropdown = dropdown(identify_as_options, Some(identify_as_index), |v| {
        EditMountOptionsMessage::IdentifyAsIndexUpdate(v).into()
    });
    if !controls_enabled || running {
        // Best-effort: disable via style by removing interaction.
        identify_dropdown = dropdown(Vec::<String>::new(), None, |_| Message::None);
    }

    let mut other_opts_input =
        text_input(fl!("other-options"), other_options_for_input).label(fl!("other-options"));
    if controls_enabled && !running {
        other_opts_input =
            other_opts_input.on_input(|t| EditMountOptionsMessage::OtherOptionsUpdate(t).into());
    }

    let mut mount_point_input =
        text_input(fl!("mount-point"), mount_point_for_input).label(fl!("mount-point"));
    if controls_enabled && !running {
        mount_point_input =
            mount_point_input.on_input(|t| EditMountOptionsMessage::MountPointUpdate(t).into());
    }

    let mut fs_type_input =
        text_input(fl!("filesystem-type"), filesystem_type_for_input).label(fl!("filesystem-type"));
    if controls_enabled && !running {
        fs_type_input =
            fs_type_input.on_input(|t| EditMountOptionsMessage::FilesystemTypeUpdate(t).into());
    }

    let mut display_name_input =
        text_input(fl!("display-name"), display_name).label(fl!("display-name"));
    if controls_enabled && !running {
        display_name_input =
            display_name_input.on_input(|t| EditMountOptionsMessage::DisplayNameUpdate(t).into());
    }

    let mut icon_input = text_input(fl!("icon-name"), icon_name).label(fl!("icon-name"));
    if controls_enabled && !running {
        icon_input = icon_input.on_input(|t| EditMountOptionsMessage::IconNameUpdate(t).into());
    }

    let mut sym_icon_input =
        text_input(fl!("symbolic-icon-name"), symbolic_icon_name).label(fl!("symbolic-icon-name"));
    if controls_enabled && !running {
        sym_icon_input =
            sym_icon_input.on_input(|t| EditMountOptionsMessage::SymbolicIconNameUpdate(t).into());
    }

    let mut content = iced_widget::column![
        defaults_cb,
        mount_start_cb,
        auth_cb,
        show_cb,
        caption_heading(fl!("identify-as")),
        identify_dropdown,
        other_opts_input,
        mount_point_input,
        fs_type_input,
        display_name_input,
        icon_input,
        sym_icon_input,
    ]
    .spacing(12);

    if let Some(err) = error.as_ref() {
        content = content.push(caption(err.clone()));
    }
    if running {
        content = content.push(caption(fl!("working")));
    }

    let can_apply = use_defaults
        || (!running
            && !mount_point.trim().is_empty()
            && !filesystem_type.trim().is_empty()
            && !other_options.trim().is_empty());

    let mut apply = button::standard(fl!("apply"));
    if can_apply {
        apply = apply.on_press(EditMountOptionsMessage::Confirm.into());
    }

    dialog::dialog()
        .title(fl!("edit-mount-options"))
        .control(content)
        .primary_action(apply)
        .secondary_action(
            button::standard(fl!("cancel")).on_press(EditMountOptionsMessage::Cancel.into()),
        )
        .into()
}
