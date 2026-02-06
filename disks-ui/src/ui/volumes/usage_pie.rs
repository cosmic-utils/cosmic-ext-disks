use cosmic::iced::{Alignment, Length};
use cosmic::widget::{self};
use cosmic::{Element, iced_widget};
use cosmic::cosmic_theme::palette::WithAlpha;
use disks_dbus::bytes_to_pretty;

use crate::app::Message;

/// Renders a thin pie chart showing used vs. free space with Used/Total text inside.
pub fn usage_pie<'a>(used: u64, total: u64) -> Element<'a, Message> {
    let percent = if total > 0 {
        ((used as f64 / total as f64) * 100.0) as u32
    } else {
        0
    };

    // Colors for used (accent) and free (muted)
    let used_text = format!("{} / {}", bytes_to_pretty(&used, false), bytes_to_pretty(&total, false));

    // For now, use a simple column with the text
    // TODO: Implement actual pie chart rendering using canvas or SVG
    widget::container(
        iced_widget::column![
            widget::text::caption_heading(format!("{}%", percent))
                .center(),
            widget::text::caption(used_text)
                .center(),
        ]
        .spacing(2)
        .align_x(Alignment::Center)
        .width(Length::Fixed(64.0))
    )
    .padding(4)
    .width(Length::Fixed(72.0))
    .height(Length::Fixed(72.0))
    .center_x(Length::Fixed(72.0))
    .center_y(Length::Fixed(72.0))
    .style(move |theme: &cosmic::Theme| {
        cosmic::iced_widget::container::Style {
            background: Some(cosmic::iced::Background::Color(
                theme.cosmic().accent_color().with_alpha(0.1).into(),
            )),
            border: cosmic::iced::Border {
                color: theme.cosmic().accent_color().into(),
                width: 2.0,
                radius: 36.0.into(),
            },
            ..Default::default()
        }
    })
    .into()
}
