use crate::app::Message;
use crate::fl;
use crate::ui::dialogs::message::{
    ChangePassphraseMessage, EditEncryptionOptionsMessage, TakeOwnershipMessage, UnlockMessage,
};
use crate::ui::dialogs::state::{
    ChangePassphraseDialog, EditEncryptionOptionsDialog, TakeOwnershipDialog, UnlockEncryptedDialog,
};
use crate::ui::wizard::{wizard_action_row, wizard_shell};
use cosmic::{
    Element, iced_widget,
    widget::text::caption,
    widget::{button, checkbox, dialog, text_input},
};

pub fn take_ownership<'a>(state: TakeOwnershipDialog) -> Element<'a, Message> {
    let TakeOwnershipDialog {
        target: _,
        recursive,
        running,
    } = state;

    let mut content = iced_widget::column![
        caption(fl!("take-ownership-warning")),
        checkbox(fl!("take-ownership-recursive"), recursive)
            .on_toggle(|v| TakeOwnershipMessage::RecursiveUpdate(v).into()),
    ]
    .spacing(12);

    if running {
        content = content.push(caption(fl!("working")));
    }

    let mut apply = button::destructive(fl!("take-ownership"));
    if !running {
        apply = apply.on_press(TakeOwnershipMessage::Confirm.into());
    }

    dialog::dialog()
        .title(fl!("take-ownership"))
        .control(content)
        .primary_action(apply)
        .secondary_action(
            button::standard(fl!("cancel")).on_press(TakeOwnershipMessage::Cancel.into()),
        )
        .into()
}

pub fn change_passphrase<'a>(state: ChangePassphraseDialog) -> Element<'a, Message> {
    let ChangePassphraseDialog {
        volume: _,
        current_passphrase,
        new_passphrase,
        confirm_passphrase,
        error,
        running,
    } = state;

    let current_for_input = current_passphrase.clone();
    let new_for_input = new_passphrase.clone();
    let confirm_for_input = confirm_passphrase.clone();

    let mut content = iced_widget::column![
        text_input::secure_input("", current_for_input, None, true)
            .label(fl!("current-passphrase"))
            .on_input(|v| ChangePassphraseMessage::CurrentUpdate(v).into()),
        text_input::secure_input("", new_for_input, None, true)
            .label(fl!("new-passphrase"))
            .on_input(|v| ChangePassphraseMessage::NewUpdate(v).into()),
        text_input::secure_input("", confirm_for_input, None, true)
            .label(fl!("confirm"))
            .on_input(|v| ChangePassphraseMessage::ConfirmUpdate(v).into()),
    ]
    .spacing(12);

    if let Some(err) = error.as_ref() {
        content = content.push(caption(err.clone()));
    }

    if running {
        content = content.push(caption(fl!("working")));
    }

    let mut apply = button::standard(fl!("apply"));
    if !running {
        apply = apply.on_press(ChangePassphraseMessage::Confirm.into());
    }

    let footer = wizard_action_row(
        vec![],
        vec![
            button::standard(fl!("cancel"))
                .on_press(ChangePassphraseMessage::Cancel.into())
                .into(),
            apply.into(),
        ],
    );

    let shell = wizard_shell(
        caption(fl!("change-passphrase")).into(),
        content.into(),
        footer,
    );

    dialog::dialog()
        .title(fl!("change-passphrase"))
        .control(shell)
        .into()
}

pub fn edit_encryption_options<'a>(state: EditEncryptionOptionsDialog) -> Element<'a, Message> {
    let EditEncryptionOptionsDialog {
        volume: _,
        use_defaults,
        unlock_at_startup,
        require_auth,
        other_options,
        name,
        passphrase,
        show_passphrase,
        error,
        running,
    } = state;

    let name_for_input = name.clone();

    let controls_enabled = !use_defaults;

    let mut defaults_cb = checkbox(fl!("user-session-defaults"), use_defaults);
    if !running {
        defaults_cb =
            defaults_cb.on_toggle(|v| EditEncryptionOptionsMessage::UseDefaultsUpdate(v).into());
    }

    let mut startup_cb = checkbox(fl!("unlock-at-startup"), unlock_at_startup);
    if controls_enabled && !running {
        startup_cb =
            startup_cb.on_toggle(|v| EditEncryptionOptionsMessage::UnlockAtStartupUpdate(v).into());
    }

    let mut auth_cb = checkbox(fl!("require-auth-to-unlock"), require_auth);
    if controls_enabled && !running {
        auth_cb = auth_cb.on_toggle(|v| EditEncryptionOptionsMessage::RequireAuthUpdate(v).into());
    }

    let mut other_opts_input =
        text_input(fl!("other-options"), other_options).label(fl!("other-options"));
    if controls_enabled && !running {
        other_opts_input = other_opts_input
            .on_input(|t| EditEncryptionOptionsMessage::OtherOptionsUpdate(t).into());
    }

    let mut name_input = text_input(fl!("name"), name_for_input).label(fl!("name"));
    if controls_enabled && !running {
        name_input = name_input.on_input(|t| EditEncryptionOptionsMessage::NameUpdate(t).into());
    }

    let mut passphrase_input = if show_passphrase {
        text_input(fl!("passphrase"), passphrase.clone()).label(fl!("passphrase"))
    } else {
        text_input::secure_input("", passphrase.clone(), None, true).label(fl!("passphrase"))
    };
    if controls_enabled && !running {
        passphrase_input =
            passphrase_input.on_input(|t| EditEncryptionOptionsMessage::PassphraseUpdate(t).into());
    }

    let mut show_pass_cb = checkbox(fl!("show-passphrase"), show_passphrase);
    if controls_enabled && !running {
        show_pass_cb = show_pass_cb
            .on_toggle(|v| EditEncryptionOptionsMessage::ShowPassphraseUpdate(v).into());
    }

    let mut content = iced_widget::column![
        defaults_cb,
        startup_cb,
        auth_cb,
        other_opts_input,
        name_input,
        passphrase_input,
        show_pass_cb,
    ]
    .spacing(12);

    if let Some(err) = error.as_ref() {
        content = content.push(caption(err.clone()));
    }
    if running {
        content = content.push(caption(fl!("working")));
    }

    let can_apply = use_defaults || (!name.trim().is_empty() && !running);
    let mut apply = button::standard(fl!("apply"));
    if can_apply {
        apply = apply.on_press(EditEncryptionOptionsMessage::Confirm.into());
    }

    let footer = wizard_action_row(
        vec![],
        vec![
            button::standard(fl!("cancel"))
                .on_press(EditEncryptionOptionsMessage::Cancel.into())
                .into(),
            apply.into(),
        ],
    );

    let shell = wizard_shell(
        caption(fl!("edit-encryption-options")).into(),
        content.into(),
        footer,
    );

    dialog::dialog()
        .title(fl!("edit-encryption-options"))
        .control(shell)
        .into()
}

pub fn unlock_encrypted<'a>(state: UnlockEncryptedDialog) -> Element<'a, Message> {
    let mut content = iced_widget::column![
        text_input::secure_input("", state.passphrase.clone(), None, true)
            .label(fl!("passphrase"))
            .on_input(|v| UnlockMessage::PassphraseUpdate(v).into()),
    ]
    .spacing(12);

    if let Some(err) = state.error.as_ref() {
        content = content.push(caption(err.clone()));
    }

    if state.running {
        content = content.push(caption(fl!("working")));
    }

    let mut unlock_button = button::destructive(fl!("unlock-button"));
    if !state.running {
        unlock_button = unlock_button.on_press(UnlockMessage::Confirm.into());
    }

    dialog::dialog()
        .title(fl!("unlock", name = state.partition_name))
        .control(content)
        .primary_action(unlock_button)
        .secondary_action(button::standard(fl!("cancel")).on_press(UnlockMessage::Cancel.into()))
        .into()
}
