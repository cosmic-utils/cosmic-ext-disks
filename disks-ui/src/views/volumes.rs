use cosmic::{
    Element, Task,
    cosmic_theme::palette::WithAlpha,
    iced::{Alignment, Background, Length, Shadow},
    iced_widget::{self, column},
    widget::{
        self, checkbox, container, icon,
        text::{caption, caption_heading},
    },
};

use crate::{
    app::{Message, ShowDialog, UnlockEncryptedDialog},
    fl,
    utils::{DiskSegmentKind, PartitionExtent, SegmentAnomaly, compute_disk_segments},
};
use disks_dbus::CreatePartitionInfo;
use disks_dbus::bytes_to_pretty;
use disks_dbus::{DriveModel, PartitionModel, VolumeKind, VolumeNode};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VolumesControlMessage {
    SegmentSelected(usize),
    SelectVolume {
        segment_index: usize,
        object_path: String,
    },
    ToggleShowReserved(bool),
    Mount,
    Unmount,
    ChildMount(String),
    ChildUnmount(String),
    LockContainer,
    Delete,
    CreateMessage(CreateMessage),
    UnlockMessage(UnlockMessage),
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UnlockMessage {
    PassphraseUpdate(String),
    Confirm,
    Cancel,
}

impl From<CreateMessage> for VolumesControlMessage {
    fn from(val: CreateMessage) -> Self {
        VolumesControlMessage::CreateMessage(val)
    }
}

impl From<UnlockMessage> for VolumesControlMessage {
    fn from(val: UnlockMessage) -> Self {
        VolumesControlMessage::UnlockMessage(val)
    }
}

impl From<CreateMessage> for Message {
    fn from(val: CreateMessage) -> Self {
        Message::VolumesMessage(VolumesControlMessage::CreateMessage(val))
    }
}

impl From<UnlockMessage> for Message {
    fn from(val: UnlockMessage) -> Self {
        Message::VolumesMessage(VolumesControlMessage::UnlockMessage(val))
    }
}

impl From<VolumesControlMessage> for Message {
    fn from(val: VolumesControlMessage) -> Self {
        Message::VolumesMessage(val)
    }
}

pub struct VolumesControl {
    pub selected_segment: usize,
    pub selected_volume: Option<String>,
    pub segments: Vec<Segment>,
    pub show_reserved: bool,
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
    pub kind: DiskSegmentKind,
    pub width: u16,
    pub partition: Option<PartitionModel>,
    pub table_type: String,
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
    pub fn free_space(offset: u64, size: u64, table_type: String) -> Self {
        Self {
            label: fl!("free-space-segment"),
            name: "".into(),
            partition_type: "".into(),
            size,
            offset,
            state: false,
            kind: DiskSegmentKind::FreeSpace,
            width: 0,
            partition: None,
            table_type,
        }
    }

    pub fn reserved(offset: u64, size: u64, table_type: String) -> Self {
        Self {
            label: fl!("reserved-space-segment"),
            name: "".into(),
            partition_type: "".into(),
            size,
            offset,
            state: false,
            kind: DiskSegmentKind::Reserved,
            width: 0,
            partition: None,
            table_type,
        }
    }

    pub fn get_create_info(&self) -> CreatePartitionInfo {
        CreatePartitionInfo {
            max_size: self.size,
            offset: self.offset,
            size: self.size,
            table_type: self.table_type.clone(),
            ..Default::default()
        }
    }

    pub fn new(partition: &PartitionModel) -> Self {
        let mut name = partition.name.clone();
        if name.is_empty() {
            name = fl!("filesystem");
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
            kind: DiskSegmentKind::Partition,
            width: 0,
            partition: Some(partition.clone()),
            table_type: partition.table_type.clone(),
        }
    }

    pub fn get_segments(drive: &DriveModel, show_reserved: bool) -> Vec<Segment> {
        let table_type = drive.partition_table_type.clone().unwrap_or_default();
        const DOS_RESERVED_START_BYTES: u64 = 1024 * 1024;

        let usable_range = match table_type.as_str() {
            "gpt" => drive.gpt_usable_range.map(|r| (r.start, r.end)),
            "dos" => {
                if drive.size > DOS_RESERVED_START_BYTES {
                    Some((DOS_RESERVED_START_BYTES, drive.size))
                } else {
                    None
                }
            }
            _ => None,
        };

        let extents: Vec<PartitionExtent> = drive
            .partitions
            .iter()
            .enumerate()
            .map(|(id, p)| PartitionExtent {
                id,
                offset: p.offset,
                size: p.size,
            })
            .collect();

        let computation = compute_disk_segments(drive.size, extents, usable_range);
        for anomaly in computation.anomalies {
            match anomaly {
                SegmentAnomaly::PartitionOverlapsPrevious {
                    id,
                    partition_offset,
                    previous_end,
                } => {
                    eprintln!(
                        "partition segmentation anomaly: partition #{id} overlaps previous segment (offset={partition_offset}, previous_end={previous_end})"
                    );
                }
                SegmentAnomaly::PartitionStartsPastDisk {
                    id,
                    partition_offset,
                    disk_size,
                } => {
                    eprintln!(
                        "partition segmentation anomaly: partition #{id} starts past disk end (offset={partition_offset}, disk_size={disk_size})"
                    );
                }
                SegmentAnomaly::PartitionEndPastDisk {
                    id,
                    partition_end,
                    disk_size,
                } => {
                    eprintln!(
                        "partition segmentation anomaly: partition #{id} ends past disk end (end={partition_end}, disk_size={disk_size})"
                    );
                }
            }
        }

        let mut segments: Vec<Segment> = Vec::new();
        for seg in computation.segments {
            match seg.kind {
                DiskSegmentKind::FreeSpace => {
                    segments.push(Segment::free_space(
                        seg.offset,
                        seg.size,
                        table_type.clone(),
                    ));
                }
                DiskSegmentKind::Reserved => {
                    segments.push(Segment::reserved(seg.offset, seg.size, table_type.clone()));
                }
                DiskSegmentKind::Partition => {
                    let Some(partition_id) = seg.partition_id else {
                        continue;
                    };
                    let Some(p) = drive.partitions.get(partition_id) else {
                        continue;
                    };

                    let mut s = Segment::new(p);
                    // Use computed extents so clamping (e.g., end-past-disk) is reflected.
                    s.offset = seg.offset;
                    s.size = seg.size;
                    segments.push(s);
                }
            }
        }

        if !show_reserved {
            segments.retain(|s| {
                if s.kind == DiskSegmentKind::Reserved {
                    return false;
                }
                if s.kind == DiskSegmentKind::FreeSpace && s.size < 1048576 {
                    return false;
                }
                true
            });

            // Ensure the UI always has at least one segment to render/select.
            if segments.is_empty() && drive.size > 0 {
                if let Some((start, end)) = usable_range {
                    if end > start {
                        segments.push(Segment::free_space(
                            start,
                            end.saturating_sub(start),
                            table_type.clone(),
                        ));
                    } else {
                        segments.push(Segment::free_space(0, drive.size, table_type.clone()));
                    }
                } else {
                    segments.push(Segment::free_space(0, drive.size, table_type.clone()));
                }
            }
        }

        // Figure out Portion value (based on what we're showing).
        let visible_total = segments.iter().map(|s| s.size).sum::<u64>();
        let denom = visible_total.max(1);
        segments.iter_mut().for_each(|s| {
            s.width = (((s.size as f64 / denom as f64) * 1000.).log10().ceil() as u16).max(1);
        });

        segments
    }

    pub fn get_segment_control<'a>(&self) -> Element<'a, Message> {
        if self.kind == DiskSegmentKind::FreeSpace {
            container(
                iced_widget::column![
                    caption_heading(fl!("free-space-caption")).center(),
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
        } else if self.kind == DiskSegmentKind::Reserved {
            container(
                iced_widget::column![
                    caption_heading(fl!("reserved-space-caption")).center(),
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
    pub fn new(model: DriveModel, show_reserved: bool) -> Self {
        let mut segments: Vec<Segment> = Segment::get_segments(&model, show_reserved);
        if let Some(first) = segments.first_mut() {
            first.state = true;
        }

        Self {
            model,
            selected_segment: 0,
            selected_volume: None,
            segments,
            show_reserved,
        }
    }

    pub fn selected_volume_node(&self) -> Option<&VolumeNode> {
        let object_path = self.selected_volume.as_deref()?;
        find_volume_node(&self.model.volumes, object_path)
    }

    pub fn set_show_reserved(&mut self, show_reserved: bool) {
        if self.show_reserved == show_reserved {
            return;
        }

        self.show_reserved = show_reserved;
        self.segments = Segment::get_segments(&self.model, self.show_reserved);
        self.selected_segment = 0;
        self.selected_volume = None;
        if let Some(first) = self.segments.first_mut() {
            first.state = true;
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
                    self.selected_volume = None;
                    self.segments.iter_mut().for_each(|s| s.state = false);
                    if let Some(segment) = self.segments.get_mut(index) {
                        segment.state = true;
                    }
                }
            }
            VolumesControlMessage::SelectVolume {
                segment_index,
                object_path,
            } => {
                if dialog.is_none() {
                    self.selected_segment = segment_index;
                    self.selected_volume = Some(object_path);
                    self.segments.iter_mut().for_each(|s| s.state = false);
                    if let Some(segment) = self.segments.get_mut(segment_index) {
                        segment.state = true;
                    }
                }
            }
            VolumesControlMessage::ToggleShowReserved(show_reserved) => {
                self.set_show_reserved(show_reserved);
            }
            VolumesControlMessage::Mount => {
                let segment = self.segments.get(self.selected_segment).cloned();
                if let Some(s) = segment.clone() {
                    match s.partition {
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
                                        println!("{e:#}");
                                        Message::None.into()
                                    }
                                },
                            );
                        }
                        None => return Task::none(),
                    }
                }
                return Task::none();
            }
            VolumesControlMessage::Unmount => {
                let segment = self.segments.get(self.selected_segment).cloned();
                if let Some(s) = segment.clone() {
                    match s.partition {
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
                    }
                }
                return Task::none();
            }
            VolumesControlMessage::ChildMount(object_path) => {
                let node = find_volume_node(&self.model.volumes, &object_path).cloned();
                if let Some(v) = node {
                    return Task::perform(
                        async move {
                            v.mount().await?;
                            DriveModel::get_drives().await
                        },
                        |result| match result {
                            Ok(drives) => Message::UpdateNav(drives, None).into(),
                            Err(e) => {
                                eprintln!("{e:#}");
                                Message::None.into()
                            }
                        },
                    );
                }
                return Task::none();
            }
            VolumesControlMessage::ChildUnmount(object_path) => {
                let node = find_volume_node(&self.model.volumes, &object_path).cloned();
                if let Some(v) = node {
                    return Task::perform(
                        async move {
                            v.unmount().await?;
                            DriveModel::get_drives().await
                        },
                        |result| match result {
                            Ok(drives) => Message::UpdateNav(drives, None).into(),
                            Err(e) => {
                                eprintln!("{e:#}");
                                Message::None.into()
                            }
                        },
                    );
                }
                return Task::none();
            }

            VolumesControlMessage::LockContainer => {
                let segment = self.segments.get(self.selected_segment).cloned();
                if let Some(s) = segment
                    && let Some(p) = s.partition
                {
                    return Task::perform(
                        async move {
                            p.lock().await?;
                            DriveModel::get_drives().await
                        },
                        |result| match result {
                            Ok(drives) => Message::UpdateNav(drives, None).into(),
                            Err(e) => Message::Dialog(ShowDialog::Info {
                                title: fl!("lock-failed"),
                                body: e.to_string(),
                            })
                            .into(),
                        },
                    );
                }
                return Task::none();
            }
            VolumesControlMessage::Delete => {
                let segment = self.segments.get(self.selected_segment).cloned();
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
                    None => {
                        eprintln!("CreateMessage received with no active dialog; ignoring.");
                        return Task::none();
                    }
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
                        CreateMessage::Continue => {
                            eprintln!("CreateMessage::Continue is not implemented; ignoring.");
                        }
                        CreateMessage::Cancel => return Task::done(Message::CloseDialog.into()),
                        CreateMessage::Partition(mut create_partition_info) => {
                            //println!("{:?}", create_partition_info);

                            if create_partition_info.name.is_empty() {
                                create_partition_info.name = fl!("untitled").to_string();
                            }
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

                    ShowDialog::UnlockEncrypted(_) => {
                        eprintln!(
                            "CreateMessage received while an unlock dialog is open; ignoring."
                        );
                    }

                    ShowDialog::Info { .. } => {
                        eprintln!("CreateMessage received while an info dialog is open; ignoring.");
                    }
                }
            }

            VolumesControlMessage::UnlockMessage(unlock_message) => {
                let d = match dialog.as_mut() {
                    Some(d) => d,
                    None => {
                        eprintln!("UnlockMessage received with no active dialog; ignoring.");
                        return Task::none();
                    }
                };

                let ShowDialog::UnlockEncrypted(state) = d else {
                    eprintln!("UnlockMessage received while a different dialog is open; ignoring.");
                    return Task::none();
                };

                match unlock_message {
                    UnlockMessage::PassphraseUpdate(p) => {
                        state.passphrase = p;
                        state.error = None;
                        return Task::none();
                    }
                    UnlockMessage::Cancel => return Task::done(Message::CloseDialog.into()),
                    UnlockMessage::Confirm => {
                        let partition_path = state.partition_path.clone();
                        let partition_name = state.partition_name.clone();
                        let passphrase = state.passphrase.clone();

                        // Look up the partition in the current model.
                        let part = self
                            .model
                            .partitions
                            .iter()
                            .find(|p| p.path.to_string() == partition_path)
                            .cloned();

                        let Some(p) = part else {
                            return Task::done(
                                Message::Dialog(ShowDialog::Info {
                                    title: fl!("unlock-failed"),
                                    body: fl!("unlock-missing-partition", name = partition_name),
                                })
                                .into(),
                            );
                        };

                        return Task::perform(
                            async move {
                                p.unlock(&passphrase).await?;
                                DriveModel::get_drives().await
                            },
                            move |result| match result {
                                Ok(drives) => Message::UpdateNav(drives, None).into(),
                                Err(e) => Message::Dialog(ShowDialog::UnlockEncrypted(
                                    UnlockEncryptedDialog {
                                        partition_path: partition_path.clone(),
                                        partition_name: partition_name.clone(),
                                        passphrase: String::new(),
                                        error: Some(e.to_string()),
                                    },
                                ))
                                .into(),
                            },
                        );
                    }
                }
            }
        }

        Task::none()
    }

    pub fn view(&self) -> Element<'_, Message> {
        const SEGMENT_BUTTON_HEIGHT: f32 = 130.0;

        let show_reserved = checkbox(fl!("show-reserved"), self.show_reserved)
            .on_toggle(|v| VolumesControlMessage::ToggleShowReserved(v).into());

        let segment_buttons: Vec<Element<Message>> = self
            .segments
            .iter()
            .enumerate()
            .map(|(index, segment)| {
                // When a child filesystem is selected, visually de-emphasize the container.
                let container_selected = segment.state && self.selected_volume.is_none();
                let active_state = ToggleState::active_or(&container_selected, ToggleState::Normal);
                let hovered_state =
                    ToggleState::active_or(&container_selected, ToggleState::Hovered);

                // For encrypted container partitions, render a split tile:
                // - top half: the container itself (selects the segment)
                // - bottom half: contained volumes (select filesystem/LV)
                let container_volume = segment
                    .partition
                    .as_ref()
                    .and_then(|p| find_volume_node_for_partition(&self.model.volumes, p))
                    .filter(|v| v.kind == VolumeKind::CryptoContainer);

                if let Some(v) = container_volume {
                    let state_text = if v.locked {
                        fl!("locked")
                    } else {
                        fl!("unlocked")
                    };

                    let top = cosmic::widget::button::custom(
                        container(
                            iced_widget::column![
                                caption_heading(segment.name.clone()).center(),
                                caption(bytes_to_pretty(&segment.size, false)).center(),
                                caption(state_text).center(),
                            ]
                            .spacing(4)
                            .width(Length::Fill)
                            .align_x(Alignment::Center),
                        )
                        .padding(6)
                        .align_x(Alignment::Center)
                        .align_y(Alignment::Center),
                    )
                    .on_press(Message::VolumesMessage(
                        VolumesControlMessage::SegmentSelected(index),
                    ))
                    .class(cosmic::theme::Button::Custom {
                        active: Box::new(move |_b, theme| get_button_style(active_state, theme)),
                        disabled: Box::new(|theme| get_button_style(ToggleState::Disabled, theme)),
                        hovered: Box::new(move |_, theme| get_button_style(hovered_state, theme)),
                        pressed: Box::new(|_, theme| get_button_style(ToggleState::Pressed, theme)),
                    })
                    .height(Length::FillPortion(1));

                    let bottom_content: Element<Message> = if v.locked {
                        // No children while locked.
                        container(
                            iced_widget::column![caption(fl!("locked")).center()]
                                .width(Length::Fill)
                                .height(Length::Fill)
                                .align_x(Alignment::Center),
                        )
                        .padding(6)
                        .into()
                    } else {
                        // Prefer showing the immediate children if any; if the only child is a
                        // container (e.g. cleartext PV), show its children instead.
                        let direct = &v.children;
                        let mut col = iced_widget::column![].spacing(8);
                        col = col.width(Length::Fill).height(Length::Fill);

                        if direct.len() == 1 && !direct[0].children.is_empty() {
                            col = col.push(volume_row_compact(
                                index,
                                &direct[0],
                                &direct[0].children,
                                self.selected_volume.as_deref(),
                            ));
                        } else {
                            col = col.push(volume_row_compact(
                                index,
                                v,
                                direct,
                                self.selected_volume.as_deref(),
                            ));
                        }

                        col.into()
                    };

                    // No extra outer padding here; child tiles should align with container extents.
                    let bottom = container(bottom_content)
                        .padding(0)
                        .height(Length::FillPortion(1))
                        .width(Length::Fill);

                    return container(
                        iced_widget::column![top, bottom]
                            .spacing(6)
                            .height(Length::Fixed(SEGMENT_BUTTON_HEIGHT)),
                    )
                    .width(Length::FillPortion(segment.width))
                    .into();
                }

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
                    .height(Length::Fixed(SEGMENT_BUTTON_HEIGHT))
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

        let selected_volume = selected
            .partition
            .as_ref()
            .and_then(|p| find_volume_node_for_partition(&self.model.volumes, p));

        let selected_child_volume = self.selected_volume_node();

        match selected.kind {
            DiskSegmentKind::Partition => {
                if let Some(p) = selected.partition.as_ref() {
                    // If a child filesystem/LV is selected, mount/unmount applies to it.
                    if let Some(v) = selected_child_volume {
                        if v.can_mount() {
                            let msg = if v.is_mounted() {
                                VolumesControlMessage::ChildUnmount(v.object_path.to_string())
                            } else {
                                VolumesControlMessage::ChildMount(v.object_path.to_string())
                            };
                            let icon_name = if v.is_mounted() {
                                "media-playback-stop-symbolic"
                            } else {
                                "media-playback-start-symbolic"
                            };
                            action_bar.push(
                                widget::button::custom(icon::from_name(icon_name))
                                    .on_press(msg.into())
                                    .into(),
                            );
                        }
                    } else if let Some(v) = selected_volume {
                        if v.kind == VolumeKind::CryptoContainer {
                            if v.locked {
                                action_bar.push(
                                    widget::button::custom(icon::from_name(
                                        "dialog-password-symbolic",
                                    ))
                                    .on_press(Message::Dialog(ShowDialog::UnlockEncrypted(
                                        UnlockEncryptedDialog {
                                            partition_path: p.path.to_string(),
                                            partition_name: p.name(),
                                            passphrase: String::new(),
                                            error: None,
                                        },
                                    )))
                                    .into(),
                                );
                            } else {
                                action_bar.push(
                                    widget::button::custom(icon::from_name(
                                        "changes-prevent-symbolic",
                                    ))
                                    .on_press(VolumesControlMessage::LockContainer.into())
                                    .into(),
                                );
                            }
                        } else if p.can_mount() {
                            let button = if p.is_mounted() {
                                widget::button::custom(icon::from_name(
                                    "media-playback-stop-symbolic",
                                ))
                                .on_press(VolumesControlMessage::Unmount.into())
                            } else {
                                widget::button::custom(icon::from_name(
                                    "media-playback-start-symbolic",
                                ))
                                .on_press(VolumesControlMessage::Mount.into())
                            };

                            action_bar.push(button.into());
                        }
                    }
                }
            }
            DiskSegmentKind::FreeSpace => {
                action_bar.push(
                    widget::button::custom(icon::from_name("list-add-symbolic"))
                        .on_press(Message::Dialog(ShowDialog::AddPartition(
                            selected.get_create_info(),
                        )))
                        .into(),
                );
            }
            DiskSegmentKind::Reserved => {}
        }

        //TODO Get better icons
        if selected.kind == DiskSegmentKind::Partition {
            action_bar.push(widget::button::custom(icon::from_name("edit-find-symbolic")).into());
            action_bar.push(widget::horizontal_space().into());
            action_bar.push(
                widget::button::custom(icon::from_name("edit-delete-symbolic"))
                    .on_press(Message::Dialog(ShowDialog::DeletePartition(
                        selected.name.clone(),
                    )))
                    .into(),
            );
        }

        let root = column![
            cosmic::widget::Row::from_vec(vec![show_reserved.into()])
                .spacing(10)
                .width(Length::Fill),
            cosmic::widget::Row::from_vec(segment_buttons)
                .spacing(10)
                .width(Length::Fill),
            widget::Row::from_vec(action_bar).width(Length::Fill)
        ]
        .spacing(10);

        container(root)
            .width(Length::Fill)
            .padding(10)
            .class(cosmic::style::Container::Card)
            .into()
    }
}

fn find_volume_node<'a>(volumes: &'a [VolumeNode], object_path: &str) -> Option<&'a VolumeNode> {
    for v in volumes {
        if v.object_path.to_string() == object_path {
            return Some(v);
        }
        if let Some(child) = find_volume_node(&v.children, object_path) {
            return Some(child);
        }
    }
    None
}

fn find_volume_node_for_partition<'a>(
    volumes: &'a [VolumeNode],
    partition: &PartitionModel,
) -> Option<&'a VolumeNode> {
    let target = partition.path.to_string();
    find_volume_node(volumes, &target)
}
fn volume_row_compact<'a>(
    segment_index: usize,
    parent: &VolumeNode,
    children: &'a [VolumeNode],
    selected_volume: Option<&str>,
) -> Element<'a, Message> {
    let total = parent.size.max(1);
    let mut buttons: Vec<Element<Message>> = Vec::new();

    for child in children {
        let child_object_path = child.object_path.to_string();
        let denom = total;
        let width = (((child.size as f64 / denom as f64) * 1000.).log10().ceil() as u16).max(1);

        let col = iced_widget::column![
            cosmic::widget::text::caption_heading(child.label.clone()).center(),
        ]
        .spacing(4)
        .width(Length::Fill)
        .align_x(Alignment::Center);

        let is_selected = selected_volume.is_some_and(|p| p == child_object_path);
        let active_state = if is_selected {
            ToggleState::Active
        } else {
            ToggleState::Normal
        };
        let hovered_state = if is_selected {
            ToggleState::Active
        } else {
            ToggleState::Hovered
        };

        let b = cosmic::widget::button::custom(container(col).padding(6))
            .on_press(
                VolumesControlMessage::SelectVolume {
                    segment_index,
                    object_path: child_object_path,
                }
                .into(),
            )
            .class(cosmic::theme::Button::Custom {
                active: Box::new(move |_b, theme| get_button_style(active_state, theme)),
                disabled: Box::new(|theme| get_button_style(ToggleState::Disabled, theme)),
                hovered: Box::new(move |_, theme| get_button_style(hovered_state, theme)),
                pressed: Box::new(|_, theme| get_button_style(ToggleState::Pressed, theme)),
            })
            .height(Length::Fill)
            .width(Length::FillPortion(width));

        buttons.push(b.into());
    }

    cosmic::widget::Row::from_vec(buttons)
        .spacing(10)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
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
        ToggleState::Disabled => {
            base.border_color = theme.cosmic().primary.base.with_alpha(0.35).into();
            base.outline_color = theme.cosmic().primary.base.with_alpha(0.35).into();
            base.background = Some(Background::Color(
                theme.cosmic().primary.base.with_alpha(0.08).into(),
            ));
        }
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
