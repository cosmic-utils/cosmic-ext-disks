use cosmic::{Element, cosmic_theme, iced::Alignment, theme, widget};

use crate::{
    app::{APP_ICON, Message, REPOSITORY},
    config::Config,
    fl,
};

pub fn settings<'a>(config: &Config) -> Element<'a, Message> {
    let cosmic_theme::Spacing {
        space_xxs,
        space_s,
        space_m,
        ..
    } = theme::active().cosmic().spacing;

    // About section (keep existing content)
    let icon = widget::svg(widget::svg::Handle::from_memory(APP_ICON));
    let title = widget::text::title3(fl!("app-title"));

    let hash = env!("VERGEN_GIT_SHA");
    let short_hash: String = hash.chars().take(7).collect();
    let date = env!("VERGEN_GIT_COMMIT_DATE");

    let link = widget::button::link(REPOSITORY)
        .on_press(Message::OpenRepositoryUrl)
        .padding(0);

    let about_section = widget::column()
        .push(icon)
        .push(title)
        .push(link)
        .push(
            widget::button::link(fl!(
                "git-description",
                hash = short_hash.as_str(),
                date = date
            ))
            .on_press(Message::LaunchUrl(format!("{REPOSITORY}/commits/{hash}")))
            .padding(0),
        )
        .align_x(Alignment::Center)
        .spacing(space_xxs);

    // Settings section
    let settings_title = widget::text::title4("Settings");

    let show_reserved_toggle = widget::checkbox(
        "Show Reserved Space",
        config.show_reserved,
    )
    .on_toggle(Message::ToggleShowReserved);

    let settings_section = widget::column()
        .push(settings_title)
        .push(show_reserved_toggle)
        .spacing(space_s);

    // Combine sections
    widget::column()
        .push(about_section)
        .push(widget::divider::horizontal::default())
        .push(settings_section)
        .spacing(space_m)
        .into()
}
