use disks_dbus::pretty_to_bytes;
use std::borrow::Cow;

use cosmic::{
    Element, Theme,
    cosmic_theme::Spacing,
    iced::{self, Alignment, Background, Border, Color, Length, Shadow, alignment},
    iced_widget::{self, row},
    widget::{self, button, container},
};

//spinner with editable value
pub fn input_spinner<'a, Message: 'static + Clone>(
    value_string: impl Into<Cow<'a, str>>,
    value: f64,
    step: f64,
    min: f64,
    max: f64,
    on_edit: impl Fn(f64) -> Message + 'static + Clone,
) -> Element<'a, Message> {
    let text_edit = on_edit.clone();
    container(row![
        button::text("-").on_press((on_edit.clone())(value - step)),
        widget::text_input("", value_string.into())
            .width(Length::Fill)
            .on_input(move |v| {
                match pretty_to_bytes(&v) {
                    Ok(v) => (text_edit)((v as f64).clamp(min, max)),
                    Err(_) => (text_edit)(value),
                }
            }),
        button::text("+").on_press((on_edit)(value + step)),
    ])
    .into()
}

pub fn labelled_spinner<'a, Message: 'static + Clone>(
    label: impl Into<Cow<'a, str>>,
    value_string: impl Into<Cow<'a, str>>,
    value: f64,
    step: f64,
    min: f64,
    max: f64,
    on_press: impl Fn(f64) -> Message + 'static + Clone,
) -> Element<'a, Message> {
    iced_widget::row![
        widget::text(label.into())
            .align_x(Alignment::End)
            .width(Length::FillPortion(1)),
        // .class(theme::Text::Color(cosmic::theme::system_preference().cosmic().text_button.base.color.into())),
        container(input_spinner(value_string, value, step, min, max, on_press))
            .width(Length::FillPortion(3)),
    ]
    .align_y(alignment::Vertical::Center)
    .spacing(Spacing::default().space_s)
    .into()
}

/// Displays information in a labeled row format.
/// Planned for future property editors and detail views.
#[allow(dead_code)]
pub fn labelled_info<'a, Message: 'static + Clone>(
    label: impl Into<String>,
    info: impl Into<String>,
) -> Element<'a, Message> {
    iced_widget::row![
        widget::text(label.into())
            .align_x(Alignment::End)
            .width(Length::FillPortion(1)),
        // .class(theme::Text::Color(cosmic::theme::system_preference().cosmic().text_button.base.color.into())),
        widget::text(info.into()).width(Length::FillPortion(3)),
    ]
    .spacing(Spacing::default().space_s)
    .into()
}

/// Displays a clickable link in a labeled row format.
/// Planned for future help links and external references.
#[allow(dead_code)]
pub fn link_info<'a, Message: 'static + Clone>(
    label: impl Into<String>,
    info: impl Into<String>,
    message: Message,
) -> Element<'a, Message> {
    iced_widget::row![
        widget::text(label.into())
            .align_x(Alignment::End)
            .width(Length::FillPortion(1)),
        // .class(theme::Text::Color(cosmic::theme::system_preference().cosmic().text_button.base.color.into())),
        container(
            cosmic::widget::button::link(info.into())
                .width(Length::Shrink)
                .padding(0)
                .on_press(message)
        )
        .width(Length::FillPortion(3)),
    ]
    .spacing(Spacing::default().space_s)
    .into()
}

/// Generic alert container with customizable styling.
/// Base function for warning/error/success/info alert variants.
#[allow(dead_code)]
fn alert<'a, Message: 'static + Clone>(
    message: impl Into<String>,
    on_close: Message,
    style: fn(&Theme) -> widget::container::Style,
) -> widget::Container<'a, Message, Theme> {
    widget::warning(message.into())
        .on_close(on_close)
        .into_widget()
        .style(style)
}

/// Warning alert for non-critical issues requiring user attention.
#[allow(dead_code)]
pub fn warning<'a, Message: 'static + Clone>(
    message: impl Into<String>,
    on_close: Message,
) -> widget::Container<'a, Message, Theme> {
    alert(message, on_close, warning_style)
}

/// Error alert for critical issues requiring immediate action.
#[allow(dead_code)]
pub fn error<'a, Message: 'static + Clone>(
    message: impl Into<String>,
    on_close: Message,
) -> widget::Container<'a, Message, Theme> {
    alert(message, on_close, error_style)
}

/// Success alert for confirming completed operations.
#[allow(dead_code)]
pub fn success<'a, Message: 'static + Clone>(
    message: impl Into<String>,
    on_close: Message,
) -> widget::Container<'a, Message, Theme> {
    alert(message, on_close, success_style)
}

/// Informational alert for general notices.
#[allow(dead_code)]
pub fn info<'a, Message: 'static + Clone>(
    message: impl Into<String>,
    on_close: Message,
) -> widget::Container<'a, Message, Theme> {
    alert(message, on_close, info_style)
}

/// Theme styling for warning alerts.
#[allow(dead_code)]
pub fn warning_style(theme: &Theme) -> widget::container::Style {
    let cosmic = theme.cosmic();
    widget::container::Style {
        icon_color: Some(theme.cosmic().warning.on.into()),
        text_color: Some(theme.cosmic().warning.on.into()),
        background: Some(Background::Color(theme.cosmic().warning_color().into())),
        border: Border {
            color: Color::TRANSPARENT,
            width: 1.0,
            radius: cosmic.corner_radii.radius_0.into(),
        },
        shadow: Shadow {
            color: Color::TRANSPARENT,
            offset: iced::Vector::new(0.0, 0.0),
            blur_radius: 0.0,
        },
    }
}

/// Theme styling for error alerts.
#[allow(dead_code)]
pub fn error_style(theme: &Theme) -> widget::container::Style {
    let cosmic = theme.cosmic();
    widget::container::Style {
        icon_color: Some(theme.cosmic().destructive.on.into()),
        text_color: Some(theme.cosmic().destructive.on.into()),
        background: Some(Background::Color(theme.cosmic().destructive_color().into())),
        border: Border {
            color: Color::TRANSPARENT,
            width: 1.0,
            radius: cosmic.corner_radii.radius_0.into(),
        },
        shadow: Shadow {
            color: Color::TRANSPARENT,
            offset: iced::Vector::new(0.0, 0.0),
            blur_radius: 0.0,
        },
    }
}

/// Theme styling for success alerts.
#[allow(dead_code)]
pub fn success_style(theme: &Theme) -> widget::container::Style {
    let cosmic = theme.cosmic();
    widget::container::Style {
        icon_color: Some(theme.cosmic().success.on.into()),
        text_color: Some(theme.cosmic().success.on.into()),
        background: Some(Background::Color(theme.cosmic().success_color().into())),
        border: Border {
            color: Color::TRANSPARENT,
            width: 1.0,
            radius: cosmic.corner_radii.radius_0.into(),
        },
        shadow: Shadow {
            color: Color::TRANSPARENT,
            offset: iced::Vector::new(0.0, 0.0),
            blur_radius: 0.0,
        },
    }
}

/// Theme styling for informational alerts.
#[allow(dead_code)]
pub fn info_style(theme: &Theme) -> widget::container::Style {
    let cosmic = theme.cosmic();
    widget::container::Style {
        icon_color: Some(theme.cosmic().accent.on.into()),
        text_color: Some(theme.cosmic().accent.on.into()),
        background: Some(Background::Color(theme.cosmic().accent_color().into())),
        border: Border {
            color: Color::TRANSPARENT,
            width: 1.0,
            radius: cosmic.corner_radii.radius_0.into(),
        },
        shadow: Shadow {
            color: Color::TRANSPARENT,
            offset: iced::Vector::new(0.0, 0.0),
            blur_radius: 0.0,
        },
    }
}
