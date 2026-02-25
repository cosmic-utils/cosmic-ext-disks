use cosmic::{Element, cosmic_theme, iced::Alignment, iced::Length, theme, widget};
use storage_common::FilesystemToolInfo;

use crate::{
    app::{Message, REPOSITORY},
    config::Config,
    fl,
};

pub fn settings<'a>(
    config: &Config,
    filesystem_tools: &[FilesystemToolInfo],
) -> Element<'a, Message> {
    let cosmic_theme::Spacing {
        space_xxs,
        space_s,
        space_m,
        ..
    } = theme::active().cosmic().spacing;

    let hash = env!("VERGEN_GIT_SHA");
    let short_hash: String = hash.chars().take(7).collect();
    let date = env!("VERGEN_GIT_COMMIT_DATE");

    let commit_caption = widget::button::custom(widget::text::caption(fl!(
        "git-description",
        hash = short_hash.as_str(),
        date = date
    )))
    .class(cosmic::theme::Button::Link)
    .on_press(Message::LaunchUrl(format!("{REPOSITORY}/commits/{hash}")))
    .padding(0);

    let show_reserved_toggle = widget::checkbox("Show Reserved Space", config.show_reserved)
        .on_toggle(Message::ToggleShowReserved);

    let volumes_section = widget::column()
        .push(widget::text::title4("Volumes"))
        .push(show_reserved_toggle)
        .spacing(space_s)
        .align_x(Alignment::Start);

    let parallelism_options = vec![
        fl!("usage-parallelism-low"),
        fl!("usage-parallelism-balanced"),
        fl!("usage-parallelism-high"),
    ];

    let usage_parallelism_dropdown = widget::dropdown(
        parallelism_options,
        Some(config.usage_scan_parallelism.to_index()),
        Message::UsageScanParallelismChanged,
    )
    .width(cosmic::iced::Length::Fill);

    let usage_section = widget::column()
        .push(widget::text::title4("Usage"))
        .push(widget::text::caption(fl!("usage-scan-parallelism-label")))
        .push(usage_parallelism_dropdown)
        .push(widget::divider::horizontal::default())
        .spacing(space_s)
        .align_x(Alignment::Start);

    // Filesystem tools status section - use tools from service
    let missing_tools: Vec<_> = filesystem_tools.iter().filter(|t| !t.available).collect();

    let github_icon = widget::icon::icon(
        widget::icon::from_svg_bytes(include_bytes!("../../resources/icons/github.svg"))
            .symbolic(true),
    )
    .size(16);

    let repo_footer = widget::row::with_capacity(4)
        .push(widget::Space::new(Length::Fill, 0))
        .push(commit_caption)
        .push(
            widget::button::custom(
                widget::row::with_capacity(2)
                    .push(github_icon)
                    .push(widget::text::caption("GitHub"))
                    .spacing(space_xxs)
                    .align_y(Alignment::Center),
            )
            .class(cosmic::theme::Button::Link)
            .on_press(Message::OpenRepositoryUrl)
            .padding(0),
        )
        .spacing(space_xxs)
        .align_y(Alignment::Center);

    let top_sections = widget::column()
        .push(volumes_section)
        .push(widget::divider::horizontal::default())
        .push(usage_section)
        .spacing(space_m);

    if !missing_tools.is_empty() {
        let tools_title = widget::text::title4(fl!("fs-tools-missing-title"));
        let tools_description = widget::text::body(fl!("fs-tools-missing-desc"));

        let mut tools_list = widget::column().spacing(space_xxs);
        for tool in &missing_tools {
            let tool_text = widget::text::body(format!(
                "• {} - {}",
                tool.package_hint,
                fl!("fs-tools-required-for", fs_name = tool.fs_name.clone())
            ));
            tools_list = tools_list.push(tool_text);
        }

        let tools_section = widget::column()
            .push(tools_title)
            .push(tools_description)
            .push(tools_list)
            .spacing(space_s);

        // Missing-filesystem-tools section is docked near the bottom
        let filesystem_domain = widget::column()
            .push(tools_section)
            .spacing(space_s)
            .align_x(Alignment::Start);

        widget::column()
            .push(top_sections)
            .push(widget::divider::horizontal::default())
            .push(filesystem_domain)
            .push(widget::divider::horizontal::default())
            .push(repo_footer)
            .spacing(space_m)
            .into()
    } else {
        widget::column()
            .push(top_sections)
            .push(widget::divider::horizontal::default())
            .push(repo_footer)
            .spacing(space_m)
            .into()
    }
}
