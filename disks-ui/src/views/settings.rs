use cosmic::{Element, cosmic_theme, iced::Alignment, theme, widget};

use crate::{
    app::{APP_ICON, Message, REPOSITORY},
    config::Config,
    fl,
    utils::get_missing_tools,
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

    let mut about_section = widget::column()
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

    // Filesystem tools status section
    let missing_tools = get_missing_tools();

    if !missing_tools.is_empty() {
        let tools_title = widget::text::title4("Missing Filesystem Tools");
        let tools_description = widget::text::body(
            "The following tools are not installed. Install them to enable full filesystem support:",
        );

        let mut tools_list = widget::column().spacing(space_xxs);
        for tool in &missing_tools {
            let tool_text = widget::text::body(format!(
                "• {} - required for {} support",
                tool.package_hint, tool.fs_name
            ));
            tools_list = tools_list.push(tool_text);
        }

        let tools_section = widget::column()
            .push(tools_title)
            .push(tools_description)
            .push(tools_list)
            .spacing(space_s);

        about_section = about_section
            .push(widget::divider::horizontal::default())
            .push(tools_section)
            .align_x(Alignment::Start);
    } else {
        // Show a positive message when all tools are available
        let tools_title = widget::text::title4("Filesystem Tools");
        let tools_ok = widget::text::body("All filesystem tools are installed.");

        let tools_section = widget::column()
            .push(tools_title)
            .push(tools_ok)
            .spacing(space_s);

        about_section = about_section
            .push(widget::divider::horizontal::default())
            .push(tools_section)
            .align_x(Alignment::Start);
    }

    // Settings section
    let settings_title = widget::text::title4("Settings");

    let show_reserved_toggle = widget::checkbox("Show Reserved Space", config.show_reserved)
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
