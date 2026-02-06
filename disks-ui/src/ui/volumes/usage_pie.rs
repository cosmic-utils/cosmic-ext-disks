use cosmic::iced::{Alignment, Length};
use cosmic::widget::{self};
use cosmic::{Element, iced_widget};
use cosmic::cosmic_theme::palette::WithAlpha;
use disks_dbus::bytes_to_pretty;

use crate::app::Message;

/// Renders a pie chart showing used vs. free space with percentage inside and Used/Total below.
pub fn usage_pie<'a>(used: u64, total: u64) -> Element<'a, Message> {
    let percent = if total > 0 {
        ((used as f64 / total as f64) * 100.0) as u32
    } else {
        0
    };

    let used_text = format!("{} / {}", bytes_to_pretty(&used, false), bytes_to_pretty(&total, false));

    // Pie circle with only percentage inside
    let pie_circle = widget::container(
        widget::text::caption_heading(format!("{}%", percent))
            .center()
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
                width: 4.0,  // 2x thicker
                radius: 36.0.into(),
            },
            ..Default::default()
        }
    });

    // Column: pie circle above, Used/Total text below
    iced_widget::column![
        pie_circle,
        widget::text::caption(used_text)
            .center()
    ]
    .spacing(4)
    .align_x(Alignment::Center)
    .into()
}
