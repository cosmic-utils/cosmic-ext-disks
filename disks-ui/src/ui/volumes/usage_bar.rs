use cosmic::iced::{Alignment, Color, Length};
use cosmic::widget::{self};
use cosmic::{Apply, Element, iced_widget};
use disks_dbus::bytes_to_pretty;

use crate::app::Message;
use crate::ui::volumes::Segment;
use crate::utils::DiskSegmentKind;

/// Renders a color-coded usage bar showing stacked volume usage with a legend.
#[allow(dead_code)]  // Replaced by pie chart in disk header (Task 23)
pub fn usage_bar<'a>(segments: &'a [Segment], total_size: u64) -> Element<'a, Message> {
    // Filter to only actual partitions (not free space or reserved)
    let partitions: Vec<&Segment> = segments
        .iter()
        .filter(|s| s.kind == DiskSegmentKind::Partition)
        .collect();

    if partitions.is_empty() {
        return widget::container(widget::text::caption("No partitions"))
            .padding(10)
            .width(Length::Fill)
            .center_x(Length::Fill)
            .into();
    }

    // Build the stacked usage bar
    let mut bar_segments = Vec::new();

    for (index, segment) in partitions.iter().enumerate() {
        let color = get_segment_color(index);
        let width_portion = if total_size > 0 {
            ((segment.size as f64 / total_size as f64) * 1000.0).round() as u16
        } else {
            1
        };

        let segment_bar = widget::container(widget::Space::new(0, 0))
            .style(
                move |theme: &cosmic::Theme| cosmic::iced_widget::container::Style {
                    background: Some(cosmic::iced::Background::Color(color)),
                    border: cosmic::iced::Border {
                        color: theme.cosmic().background.base.into(),
                        width: 1.0,
                        radius: 0.0.into(),
                    },
                    ..Default::default()
                },
            )
            .height(Length::Fixed(6.0))
            .width(Length::FillPortion(width_portion));

        bar_segments.push(segment_bar.into());
    }

    let usage_bar_row = cosmic::widget::Row::from_vec(bar_segments)
        .spacing(0)
        .width(Length::Fill);

    // Build the legend
    let mut legend_items = Vec::new();

    for (index, segment) in partitions.iter().enumerate() {
        let color = get_segment_color(index);

        // Color swatch
        let swatch = widget::container(widget::Space::new(12, 12))
            .style(
                move |_theme: &cosmic::Theme| cosmic::iced_widget::container::Style {
                    background: Some(cosmic::iced::Background::Color(color)),
                    border: cosmic::iced::Border {
                        color: Color::from_rgb(0.5, 0.5, 0.5),
                        width: 1.0,
                        radius: 2.0.into(),
                    },
                    ..Default::default()
                },
            )
            .padding(0);

        // Label: name + size
        let label = widget::text::caption(format!(
            "{} ({})",
            segment.name,
            bytes_to_pretty(&segment.size, false)
        ));

        let legend_item = iced_widget::Row::new()
            .push(swatch)
            .push(widget::Space::new(6, 0))
            .push(label)
            .spacing(0)
            .align_y(Alignment::Center);

        legend_items.push(legend_item.into());
    }

    // Wrap legend items with spacing
    let legend = cosmic::widget::Row::from_vec(legend_items)
        .spacing(20)
        .align_y(Alignment::Center)
        .apply(widget::container)
        .center_x(Length::Fill)
        .padding([8, 0, 0, 0]);

    // Combine bar and legend
    iced_widget::column![usage_bar_row, legend]
        .spacing(8)
        .width(Length::Fill)
        .into()
}

/// Generate a distinct color for each segment based on index.
#[allow(dead_code)]
fn get_segment_color(index: usize) -> Color {
    // Color palette with distinct, accessible colors
    let colors = [
        Color::from_rgb(0.2, 0.6, 0.86),   // Blue
        Color::from_rgb(0.95, 0.61, 0.07), // Orange
        Color::from_rgb(0.33, 0.66, 0.41), // Green
        Color::from_rgb(0.89, 0.29, 0.35), // Red
        Color::from_rgb(0.62, 0.42, 0.84), // Purple
        Color::from_rgb(0.96, 0.76, 0.05), // Yellow
        Color::from_rgb(0.0, 0.74, 0.83),  // Cyan
        Color::from_rgb(0.94, 0.44, 0.63), // Pink
        Color::from_rgb(0.54, 0.77, 0.29), // Lime
        Color::from_rgb(0.80, 0.36, 0.36), // Brown
    ];

    colors[index % colors.len()]
}
