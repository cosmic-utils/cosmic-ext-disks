use cosmic::iced::Alignment;
use cosmic::iced::Length;
use cosmic::widget::{self, button};
use cosmic::{Element, iced_widget};

pub(crate) fn wizard_shell<'a, Message: Clone + 'static>(
    header: Element<'a, Message>,
    content: Element<'a, Message>,
    footer: Element<'a, Message>,
) -> Element<'a, Message> {
    let content = widget::scrollable(
        widget::container(content)
            .padding([8, 0])
            .width(Length::Fill),
    )
    .width(Length::Fill)
    .height(Length::Fill);

    let layout = iced_widget::column![header, content, footer]
        .spacing(12)
        .width(Length::Fill)
        .height(Length::Fill);

    widget::container(layout)
        .padding(20)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

pub(crate) fn wizard_action_row<'a, Message: Clone + 'static>(
    left_actions: Vec<Element<'a, Message>>,
    right_actions: Vec<Element<'a, Message>>,
) -> Element<'a, Message> {
    let mut row = iced_widget::row![]
        .spacing(8)
        .align_y(Alignment::Center)
        .width(Length::Fill);

    for action in left_actions {
        row = row.push(action);
    }

    row = row.push(widget::Space::new(Length::Fill, 0));

    for action in right_actions {
        row = row.push(action);
    }

    row.into()
}

pub(crate) fn option_tile_grid<'a, Message: Clone + 'static>(
    tiles: Vec<Element<'a, Message>>,
) -> Element<'a, Message> {
    widget::flex_row(tiles).row_spacing(12).column_spacing(12).into()
}

pub(crate) fn selectable_tile<'a, Message: Clone + 'static>(
    content: Element<'a, Message>,
    selected: bool,
    on_press: Option<Message>,
    width: Length,
    height: Length,
) -> Element<'a, Message> {
    let mut tile = button::custom(
        widget::container(content)
            .padding(16)
            .width(width)
            .height(height)
            .align_x(cosmic::iced::alignment::Horizontal::Center)
            .align_y(cosmic::iced::alignment::Vertical::Center),
    )
    .class(if selected {
        cosmic::theme::Button::Suggested
    } else {
        cosmic::theme::Button::Standard
    });

    if let Some(message) = on_press {
        tile = tile.on_press(message);
    }

    tile.into()
}
