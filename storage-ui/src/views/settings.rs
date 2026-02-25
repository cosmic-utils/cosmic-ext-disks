use cosmic::{Element, cosmic_theme, iced::Alignment, theme, widget};
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

    // Settings section (top, grouped by domain)
    let settings_title = widget::text::title4("Settings");

    // Filesystem tools status section - use tools from service
    let missing_tools: Vec<_> = filesystem_tools.iter().filter(|t| !t.available).collect();

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

        // Filesystem domain section
        let filesystem_domain = widget::column()
            .push(widget::text::caption_heading("Filesystem"))
            .push(widget::divider::horizontal::default())
            .push(tools_section)
            .align_x(Alignment::Start);

        let show_reserved_toggle = widget::checkbox("Show Reserved Space", config.show_reserved)
            .on_toggle(Message::ToggleShowReserved);

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

        let usage_domain = widget::column()
            .push(widget::text::caption_heading("Usage"))
            .push(show_reserved_toggle)
            .push(widget::text::caption(fl!("usage-scan-parallelism-label")))
            .push(usage_parallelism_dropdown)
            .spacing(space_s);

        let repo_footer = widget::row::with_capacity(4)
            .push(widget::Space::new(cosmic::iced::Length::Fill, 0))
            .push(commit_caption)
            .push(
                widget::button::custom(
                    widget::row::with_capacity(2)
                        .push(widget::icon::from_name("web-github-symbolic").size(16))
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

        widget::column()
            .push(settings_title)
            .push(usage_domain)
            .push(widget::divider::horizontal::default())
            .push(filesystem_domain)
            .push(widget::divider::horizontal::default())
            .push(repo_footer)
            .spacing(space_m)
            .into()
    } else {
        let tools_title = widget::text::title4(fl!("fs-tools-all-installed-title"));
        let tools_ok = widget::text::body(fl!("fs-tools-all-installed"));

        let tools_section = widget::column()
            .push(tools_title)
            .push(tools_ok)
            .spacing(space_s);

        let filesystem_domain = widget::column()
            .push(widget::text::caption_heading("Filesystem"))
            .push(widget::divider::horizontal::default())
            .push(tools_section)
            .align_x(Alignment::Start);

        let show_reserved_toggle = widget::checkbox("Show Reserved Space", config.show_reserved)
            .on_toggle(Message::ToggleShowReserved);

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

        let usage_domain = widget::column()
            .push(widget::text::caption_heading("Usage"))
            .push(show_reserved_toggle)
            .push(widget::text::caption(fl!("usage-scan-parallelism-label")))
            .push(usage_parallelism_dropdown)
            .spacing(space_s);

        let repo_footer = widget::row::with_capacity(4)
            .push(widget::Space::new(cosmic::iced::Length::Fill, 0))
            .push(commit_caption)
            .push(
                widget::button::custom(
                    widget::row::with_capacity(2)
                        .push(widget::icon::from_name("web-github-symbolic").size(16))
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

        widget::column()
            .push(settings_title)
            .push(usage_domain)
            .push(widget::divider::horizontal::default())
            .push(filesystem_domain)
            .push(widget::divider::horizontal::default())
            .push(repo_footer)
            .spacing(space_m)
            .into()
    }
}
