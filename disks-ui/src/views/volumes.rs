use cosmic::{
    Element, Task,
    cosmic_theme::palette::WithAlpha,
    iced::{Alignment, Background, Length, Shadow},
    iced_widget::{self, column},
    widget::{
        self, container, icon,
        text::{caption, caption_heading},
    },
};

use crate::app::{Message, ShowDialog};
use hardware::bytes_to_pretty;
use hardware::disks::{DriveModel, PartitionModel};
use hardware::{CreatePartitionInfo, Drive, Partition};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VolumesControlMessage {
    SegmentSelected(usize),
    Mount,
    Unmount,
    Delete,
    CreateMessage(CreateMessage),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CreateMessage {
    SizeUpdate(u64),
    NameUpdate(String),
    PasswordUpdate(String),
    ConfirmedPasswordUpdate(String),
    PasswordProectedUpdate(bool),
    EraseUpdate(bool),
    PartitionTypeUpdate(usize),
    #[allow(dead_code)]
    Continue,
    Cancel,
    Partition(CreatePartitionInfo),
}

impl Into<VolumesControlMessage> for CreateMessage {
    fn into(self) -> VolumesControlMessage {
        VolumesControlMessage::CreateMessage(self)
    }
}

impl Into<Message> for CreateMessage {
    fn into(self) -> Message {
        Message::VolumesMessage(VolumesControlMessage::CreateMessage(self))
    }
}

impl Into<Message> for VolumesControlMessage {
    fn into(self) -> Message {
        Message::VolumesMessage(self)
    }
}

pub struct VolumesControl {
    pub selected_segment: usize,
    pub segments: Vec<Segment>,
    #[allow(dead_code)]
    pub model: DriveModel,
}

#[derive(Clone, Debug)]
pub struct Segment {
    pub label: String,
    pub name: String,
    pub partition_type: String,
    pub size: u64,
    pub offset: u64,
    pub state: bool,
    pub is_free_space: bool,
    pub width: u16,
    pub partition: Option<PartitionModel>,
}

#[derive(Copy, Clone)]
pub enum ToggleState {
    Normal,
    Active,
    Disabled,
    Hovered,
    Pressed,
}

impl ToggleState {
    pub fn active_or(selected: &bool, toggle: ToggleState) -> Self {
        if *selected {
            ToggleState::Active
        } else {
            toggle
        }
    }
}

impl Segment {
    pub fn free_space(offset: u64, size: u64) -> Self {
        Self {
            label: "Free Space".into(),
            name: "".into(),
            partition_type: "".into(),
            size,
            offset,
            state: false,
            is_free_space: true,
            width: 0,
            partition: None,
        }
    }

    pub fn get_create_info(&self) -> CreatePartitionInfo {
        CreatePartitionInfo {
            max_size: self.size,
            offset: self.offset,
            size: self.size,
            ..Default::default()
        }
    }

    pub fn new(partition: &PartitionModel) -> Self {
        let mut name = partition.name.clone();
        if name.len() < 1 {
            name = "Filesystem".into();
        }

        let mut type_str = partition.id_type.clone().to_uppercase();
        type_str = format!("{} - {}", type_str, partition.partition_type.clone());

        Self {
            label: name,
            name: partition.name(),
            partition_type: type_str,
            size: partition.size,
            offset: partition.offset,
            state: false,
            is_free_space: false,
            width: 0,
            partition: Some(partition.clone()),
        }
    }

    pub fn get_segments(drive: &DriveModel) -> Vec<Segment> {
        if drive.partitions.len() == 0 {
            return vec![Segment::free_space(0, drive.size)];
        }

        let mut ordered_partitions = drive.partitions.clone();

        ordered_partitions.sort_by(|a, b| a.offset.cmp(&b.offset));

        let mut segments = vec![];
        let mut current_offset = ordered_partitions.first().unwrap().offset; //TODO: HANDLE UNWRAP

        if current_offset > 1048576 {
            //TODO: There seems to be 1024KB at the start of all drives.
            //      We need to make sure this is ALWAYS present, or the same size.
            current_offset = 0;
        }

        for p in ordered_partitions {
            if p.offset > current_offset {
                //add in a free space segment.
                segments.push(Segment::free_space(
                    current_offset,
                    p.offset - current_offset,
                ));
                current_offset = p.offset;
            }

            segments.push(Segment::new(&p));
            current_offset += p.size;
        }

        //TODO: Hack to hide weird end portion... find out what this is.
        if current_offset < drive.size - 5242880 {
            segments.push(Segment::free_space(
                current_offset,
                drive.size - current_offset,
            ));
        }

        //Figure out Portion value
        segments.iter_mut().for_each(|s| {
            s.width = (((s.size as f64 / drive.size as f64) * 1000.).log10().ceil() as u16).max(1);
        });

        segments
    }

    pub fn get_segment_control<'a>(&self) -> Element<'a, Message> {
        if self.is_free_space {
            container(
                iced_widget::column![
                    caption_heading("Free space").center(),
                    caption(bytes_to_pretty(&self.size, false)).center()
                ]
                .spacing(5)
                .width(Length::Fill)
                .align_x(Alignment::Center),
            )
            .padding(5)
            .align_x(Alignment::Center)
            .align_y(Alignment::Center)
            .into()
        } else {
            container(
                iced_widget::column![
                    caption_heading(self.name.clone()).center(),
                    caption(self.label.clone()).center(),
                    caption(self.partition_type.clone()).center(),
                    caption(bytes_to_pretty(&self.size, false)).center()
                ]
                .spacing(5)
                .align_x(Alignment::Center),
            )
            .padding(5)
            .align_x(Alignment::Center)
            .align_y(Alignment::Center)
            .into()
        }
    }
}

impl VolumesControl {
    pub fn new(model: DriveModel) -> Self {
        let mut segments: Vec<Segment> = Segment::get_segments(&model);
        segments.first_mut().unwrap().state = true; //TODO: HANDLE UNWRAP.

        Self {
            model,
            selected_segment: 0,
            segments: segments,
        }
    }

    pub fn update(
        &mut self,
        message: VolumesControlMessage,
        dialog: &mut Option<ShowDialog>,
    ) -> Task<cosmic::Action<Message>> {
        match message {
            VolumesControlMessage::SegmentSelected(index) => {
                if dialog.is_none() {
                    self.selected_segment = index;
                    self.segments.iter_mut().for_each(|s| s.state = false);
                    self.segments.get_mut(index).unwrap().state = true;
                }
            }
            VolumesControlMessage::Mount => {
                let segment = self.segments.get(self.selected_segment.clone()).cloned();
                match segment.clone() {
                    Some(s) => match s.partition {
                        Some(p) => {
                            return Task::perform(
                                async move {
                                    match p.mount().await {
                                        Ok(_) => match DriveModel::get_drives().await {
                                            Ok(drives) => Ok(drives),
                                            Err(e) => Err(e),
                                        },
                                        Err(e) => Err(e),
                                    }
                                },
                                |result| match result {
                                    Ok(drives) => Message::UpdateNav(drives, None).into(),
                                    Err(e) => {
                                        println!("{e}");
                                        Message::None.into()
                                    }
                                },
                            );
                        }
                        None => return Task::none(),
                    },
                    None => {}
                }
                return Task::none();
            }
            VolumesControlMessage::Unmount => {
                let segment = self.segments.get(self.selected_segment.clone()).cloned();
                match segment.clone() {
                    Some(s) => match s.partition {
                        Some(p) => {
                            return Task::perform(
                                async move {
                                    match p.unmount().await {
                                        Ok(_) => match DriveModel::get_drives().await {
                                            Ok(drives) => Ok(drives),
                                            Err(e) => Err(e),
                                        },
                                        Err(e) => Err(e),
                                    }
                                },
                                |result| match result {
                                    Ok(drives) => Message::UpdateNav(drives, None).into(),
                                    Err(e) => {
                                        println!("{e}");
                                        Message::None.into()
                                    }
                                },
                            );
                        }
                        None => return Task::none(),
                    },
                    None => {}
                }
                return Task::none();
            }
            VolumesControlMessage::Delete => {
                let segment = self.segments.get(self.selected_segment.clone()).cloned();
                let task = match segment.clone() {
                    Some(s) => match s.partition {
                        Some(p) => Task::perform(
                            async move {
                                match p.delete().await {
                                    Ok(_) => match DriveModel::get_drives().await {
                                        Ok(drives) => Ok(drives),
                                        Err(e) => Err(e),
                                    },
                                    Err(e) => Err(e),
                                }
                            },
                            |result| match result {
                                Ok(drives) => Message::UpdateNav(drives, None).into(),
                                Err(e) => {
                                    println!("{e}");
                                    Message::None.into()
                                }
                            },
                        ),
                        None => Task::none(),
                    },
                    None => Task::none(),
                };

                return Task::done(Message::CloseDialog.into()).chain(task);
            }
            VolumesControlMessage::CreateMessage(create_message) => {
                let d = match dialog.as_mut() {
                    Some(d) => d,
                    None => panic!("invalid state"), //TODO: Better handling,
                };

                match d {
                    ShowDialog::DeletePartition(_) => {}

                    ShowDialog::AddPartition(create) => match create_message {
                        CreateMessage::SizeUpdate(size) => create.size = size,
                        CreateMessage::NameUpdate(name) => {
                            create.name = name;
                        }
                        CreateMessage::PasswordUpdate(password) => create.password = password,
                        CreateMessage::ConfirmedPasswordUpdate(confirmed_password) => {
                            create.confirmed_password = confirmed_password
                        }
                        CreateMessage::PasswordProectedUpdate(protect) => {
                            create.password_protected = protect
                        }
                        CreateMessage::EraseUpdate(erase) => create.erase = erase,
                        CreateMessage::PartitionTypeUpdate(p_type) => {
                            create.selected_partitition_type = p_type
                        }
                        CreateMessage::Continue => todo!(),
                        CreateMessage::Cancel => todo!(),
                        CreateMessage::Partition(create_partition_info) => {
                            let model = self.model.clone();
                            let task = Task::perform(
                                async move {
                                    match model.create_partition(create_partition_info).await {
                                        Ok(_) => match DriveModel::get_drives().await {
                                            Ok(drives) => Ok(drives),
                                            Err(e) => Err(e),
                                        },
                                        Err(e) => Err(e),
                                    }
                                },
                                |result| match result {
                                    Ok(drives) => Message::UpdateNav(drives, None).into(),
                                    Err(e) => {
                                        println!("{e}");
                                        Message::None.into()
                                    }
                                },
                            );

                            return Task::done(Message::CloseDialog.into()).chain(task);
                        }
                    },
                }
            }
        }
        Task::none()
    }

    pub fn view(&self) -> Element<Message> {
        let segment_buttons: Vec<Element<Message>> = self
            .segments
            .iter()
            .enumerate()
            .map(|(index, segment)| {
                let active_state = ToggleState::active_or(&segment.state, ToggleState::Normal);
                let hovered_state = ToggleState::active_or(&segment.state, ToggleState::Hovered);

                cosmic::widget::button::custom(segment.get_segment_control())
                    .on_press(Message::VolumesMessage(
                        VolumesControlMessage::SegmentSelected(index),
                    ))
                    .class(cosmic::theme::Button::Custom {
                        active: Box::new(move |_b, theme| get_button_style(active_state, theme)),
                        disabled: Box::new(|theme| get_button_style(ToggleState::Disabled, theme)),
                        hovered: Box::new(move |_, theme| get_button_style(hovered_state, theme)),
                        pressed: Box::new(|_, theme| get_button_style(ToggleState::Pressed, theme)),
                    })
                    .height(Length::Fixed(100.))
                    .width(Length::FillPortion(segment.width))
                    .into()
            })
            .collect();

        let selected = match self.segments.get(self.selected_segment).cloned() {
            Some(segment) => segment,
            None => {
                // Handle the case where selected_segment is out of range
                return container(
                    column![
                        cosmic::widget::Row::from_vec(vec![])
                            .spacing(10)
                            .width(Length::Fill),
                        widget::Row::from_vec(vec![]).width(Length::Fill)
                    ]
                    .spacing(10),
                )
                .width(Length::Fill)
                .padding(10)
                .class(cosmic::style::Container::Card)
                .into();
            }
        };
        let mut action_bar: Vec<Element<Message>> = vec![];

        action_bar.push(match selected.partition {
            Some(p) => {
                match p.usage //TODO: More solid check than using the output of df to see if mounted.
              {
                  Some(_) => widget::button::custom(icon::from_name( "media-playback-stop-symbolic")).on_press(VolumesControlMessage::Unmount.into()),
                  None =>widget::button::custom(icon::from_name( "media-playback-start-symbolic")).on_press(VolumesControlMessage::Mount.into()),
              }
            }
            None =>widget::button::custom(icon::from_name( "list-add-symbolic")).on_press(Message::Dialog(ShowDialog::AddPartition(selected.get_create_info())).into()),
        }.into());

        //TODO Get better icons
        if !selected.is_free_space {
            action_bar.push(widget::button::custom(icon::from_name("edit-find-symbolic")).into());
            action_bar.push(widget::horizontal_space().into());
            action_bar.push(
                widget::button::custom(icon::from_name("edit-delete-symbolic"))
                    .on_press(
                        Message::Dialog(ShowDialog::DeletePartition(selected.name.clone())).into(),
                    )
                    .into(),
            );
        }

        container(
            column![
                cosmic::widget::Row::from_vec(segment_buttons)
                    .spacing(10)
                    .width(Length::Fill),
                widget::Row::from_vec(action_bar).width(Length::Fill)
            ]
            .spacing(10),
        )
        .width(Length::Fill)
        .padding(10)
        .class(cosmic::style::Container::Card)
        .into()
    }
}

fn get_button_style(
    state: ToggleState,
    theme: &cosmic::theme::Theme,
) -> cosmic::widget::button::Style {
    let mut base = cosmic::widget::button::Style {
        shadow_offset: Shadow::default().offset,
        background: Some(cosmic::iced::Background::Color(
            theme.cosmic().primary.base.into(),
        )), // Some(cosmic::iced::Background::Color(Color::TRANSPARENT)),
        overlay: None,
        border_radius: (theme.cosmic().corner_radii.radius_xs).into(),
        border_width: 0.,
        border_color: theme.cosmic().primary.base.into(),
        outline_width: 2.,
        outline_color: theme.cosmic().primary.base.into(),
        icon_color: None,
        text_color: None,
    };

    match state {
        ToggleState::Normal => {}
        ToggleState::Active => {
            base.border_color = theme.cosmic().accent_color().into();
            base.outline_color = theme.cosmic().accent_color().into();
            base.background = Some(Background::Color(
                theme.cosmic().accent_color().with_alpha(0.2).into(),
            ));
        }
        ToggleState::Disabled => todo!(),
        ToggleState::Hovered => {
            base.text_color = Some(theme.cosmic().accent_button.base.into());
            base.background = Some(Background::Color(theme.cosmic().button.hover.into()));
        }
        ToggleState::Pressed => {
            base.border_color = theme.cosmic().accent_color().into();
            base.outline_color = theme.cosmic().accent_color().into();
        }
    }

    base
}
