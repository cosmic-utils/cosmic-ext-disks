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
    app::{
        ChangePassphraseDialog, ConfirmActionDialog, CreatePartitionDialog, DeletePartitionDialog,
        EditFilesystemLabelDialog, EditPartitionDialog, FilesystemTarget, FormatPartitionDialog,
        Message, ResizePartitionDialog, ShowDialog, TakeOwnershipDialog, UnlockEncryptedDialog,
    },
    fl,
    utils::{DiskSegmentKind, PartitionExtent, SegmentAnomaly, compute_disk_segments},
};
use disks_dbus::CreatePartitionInfo;
use disks_dbus::bytes_to_pretty;
use disks_dbus::{DriveModel, PartitionTypeInfo, VolumeKind, VolumeModel, VolumeNode};

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
    OpenFormatPartition,
    OpenEditPartition,
    OpenResizePartition,
    OpenEditFilesystemLabel,
    OpenCheckFilesystem,
    CheckFilesystemConfirm,
    OpenRepairFilesystem,
    RepairFilesystemConfirm,
    OpenTakeOwnership,
    OpenChangePassphrase,
    CreateMessage(CreateMessage),
    UnlockMessage(UnlockMessage),
    EditPartitionMessage(EditPartitionMessage),
    ResizePartitionMessage(ResizePartitionMessage),
    EditFilesystemLabelMessage(EditFilesystemLabelMessage),
    TakeOwnershipMessage(TakeOwnershipMessage),
    ChangePassphraseMessage(ChangePassphraseMessage),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EditPartitionMessage {
    TypeUpdate(usize),
    NameUpdate(String),
    LegacyBiosBootableUpdate(bool),
    SystemPartitionUpdate(bool),
    HiddenUpdate(bool),
    Confirm,
    Cancel,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResizePartitionMessage {
    SizeUpdate(u64),
    Confirm,
    Cancel,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EditFilesystemLabelMessage {
    LabelUpdate(String),
    Confirm,
    Cancel,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TakeOwnershipMessage {
    RecursiveUpdate(bool),
    Confirm,
    Cancel,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChangePassphraseMessage {
    CurrentUpdate(String),
    NewUpdate(String),
    ConfirmUpdate(String),
    Confirm,
    Cancel,
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
    Partition,
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

impl From<EditPartitionMessage> for VolumesControlMessage {
    fn from(val: EditPartitionMessage) -> Self {
        VolumesControlMessage::EditPartitionMessage(val)
    }
}

impl From<ResizePartitionMessage> for VolumesControlMessage {
    fn from(val: ResizePartitionMessage) -> Self {
        VolumesControlMessage::ResizePartitionMessage(val)
    }
}

impl From<EditFilesystemLabelMessage> for VolumesControlMessage {
    fn from(val: EditFilesystemLabelMessage) -> Self {
        VolumesControlMessage::EditFilesystemLabelMessage(val)
    }
}

impl From<TakeOwnershipMessage> for VolumesControlMessage {
    fn from(val: TakeOwnershipMessage) -> Self {
        VolumesControlMessage::TakeOwnershipMessage(val)
    }
}

impl From<ChangePassphraseMessage> for VolumesControlMessage {
    fn from(val: ChangePassphraseMessage) -> Self {
        VolumesControlMessage::ChangePassphraseMessage(val)
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

impl From<EditPartitionMessage> for Message {
    fn from(val: EditPartitionMessage) -> Self {
        Message::VolumesMessage(VolumesControlMessage::EditPartitionMessage(val))
    }
}

impl From<ResizePartitionMessage> for Message {
    fn from(val: ResizePartitionMessage) -> Self {
        Message::VolumesMessage(VolumesControlMessage::ResizePartitionMessage(val))
    }
}

impl From<EditFilesystemLabelMessage> for Message {
    fn from(val: EditFilesystemLabelMessage) -> Self {
        Message::VolumesMessage(VolumesControlMessage::EditFilesystemLabelMessage(val))
    }
}

impl From<TakeOwnershipMessage> for Message {
    fn from(val: TakeOwnershipMessage) -> Self {
        Message::VolumesMessage(VolumesControlMessage::TakeOwnershipMessage(val))
    }
}

impl From<ChangePassphraseMessage> for Message {
    fn from(val: ChangePassphraseMessage) -> Self {
        Message::VolumesMessage(VolumesControlMessage::ChangePassphraseMessage(val))
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
    pub volume: Option<VolumeModel>,
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
            volume: None,
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
            volume: None,
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

    pub fn new(volume: &VolumeModel) -> Self {
        let mut name = volume.name.clone();
        if name.is_empty() {
            name = fl!("filesystem");
        }

        let mut type_str = volume.id_type.clone().to_uppercase();
        type_str = format!("{} - {}", type_str, volume.partition_type.clone());

        Self {
            label: name,
            name: volume.name(),
            partition_type: type_str,
            size: volume.size,
            offset: volume.offset,
            state: false,
            kind: DiskSegmentKind::Partition,
            width: 0,
            volume: Some(volume.clone()),
            table_type: volume.table_type.clone(),
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
            .volumes_flat
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
                    let Some(p) = drive.volumes_flat.get(partition_id) else {
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
                    match s.volume {
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
                    match s.volume {
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
                    && let Some(p) = s.volume
                {
                    let mounted_children: Vec<VolumeNode> =
                        find_volume_node_for_partition(&self.model.volumes, &p)
                            .map(collect_mounted_descendants_leaf_first)
                            .unwrap_or_default();

                    return Task::perform(
                        async move {
                            // UDisks2 typically refuses to lock while the cleartext/child FS is mounted.
                            // Unmount any mounted descendants first, then lock the container.
                            for v in mounted_children {
                                v.unmount().await?;
                            }
                            p.lock().await?;
                            DriveModel::get_drives().await
                        },
                        |result| match result {
                            Ok(drives) => Message::UpdateNav(drives, None).into(),
                            Err(e) => {
                                eprintln!("{e:#}");
                                Message::Dialog(Box::new(ShowDialog::Info {
                                    title: fl!("lock-failed"),
                                    body: e.to_string(),
                                }))
                                .into()
                            }
                        },
                    );
                }
                return Task::none();
            }
            VolumesControlMessage::Delete => {
                let d = match dialog.as_mut() {
                    Some(d) => d,
                    None => {
                        eprintln!("Delete received with no active dialog; ignoring.");
                        return Task::none();
                    }
                };

                let ShowDialog::DeletePartition(delete_state) = d else {
                    eprintln!("Delete received while a different dialog is open; ignoring.");
                    return Task::none();
                };

                if delete_state.running {
                    return Task::none();
                }

                delete_state.running = true;

                let segment = self.segments.get(self.selected_segment).cloned();
                let task = match segment.clone() {
                    Some(s) => match s.volume {
                        Some(p) => {
                            let volume_node =
                                find_volume_node_for_partition(&self.model.volumes, &p).cloned();
                            let is_unlocked_crypto = matches!(
                                volume_node.as_ref(),
                                Some(v) if v.kind == VolumeKind::CryptoContainer && !v.locked
                            );
                            let mounted_children: Vec<VolumeNode> = if is_unlocked_crypto {
                                volume_node
                                    .as_ref()
                                    .map(collect_mounted_descendants_leaf_first)
                                    .unwrap_or_default()
                            } else {
                                Vec::new()
                            };

                            Task::perform(
                                async move {
                                    if is_unlocked_crypto {
                                        // UDisks2 typically refuses to lock while the cleartext/child FS is mounted.
                                        // Unmount any mounted descendants first, then lock the container.
                                        for v in mounted_children {
                                            v.unmount().await?;
                                        }
                                        p.lock().await?;
                                    }

                                    p.delete().await?;
                                    DriveModel::get_drives().await
                                },
                                |result| match result {
                                    Ok(drives) => Message::UpdateNav(drives, None).into(),
                                    Err(e) => {
                                        eprintln!("{e:#}");
                                        Message::Dialog(Box::new(ShowDialog::Info {
                                            title: fl!("delete-failed"),
                                            body: format!("{e:#}"),
                                        }))
                                        .into()
                                    }
                                },
                            )
                        }
                        None => Task::none(),
                    },
                    None => Task::none(),
                };

                return task;
            }

            VolumesControlMessage::OpenFormatPartition => {
                if dialog.is_some() {
                    return Task::none();
                }

                let Some(segment) = self.segments.get(self.selected_segment) else {
                    return Task::none();
                };
                let Some(volume) = segment.volume.clone() else {
                    return Task::none();
                };

                let table_type = if volume.table_type.trim().is_empty() {
                    "gpt".to_string()
                } else {
                    volume.table_type.clone()
                };

                let selected_partitition_type = common_partition_type_index_for(
                    &table_type,
                    if volume.id_type.trim().is_empty() {
                        None
                    } else {
                        Some(volume.id_type.as_str())
                    },
                );

                let info = CreatePartitionInfo {
                    name: volume.name.clone(),
                    size: volume.size,
                    max_size: volume.size,
                    offset: volume.offset,
                    erase: false,
                    selected_partitition_type,
                    table_type,
                    ..Default::default()
                };

                *dialog = Some(ShowDialog::FormatPartition(FormatPartitionDialog {
                    volume,
                    info,
                    running: false,
                }));
                return Task::none();
            }

            VolumesControlMessage::OpenEditPartition => {
                if dialog.is_some() {
                    return Task::none();
                }

                let Some(segment) = self.segments.get(self.selected_segment) else {
                    return Task::none();
                };
                let Some(volume) = segment.volume.clone() else {
                    return Task::none();
                };

                if volume.volume_type != disks_dbus::VolumeType::Partition {
                    return Task::none();
                }

                let partition_types =
                    disks_dbus::get_all_partition_type_infos(volume.table_type.as_str());
                if partition_types.is_empty() {
                    return Task::done(
                        Message::Dialog(Box::new(ShowDialog::Info {
                            title: fl!("app-title"),
                            body: fl!("edit-partition-no-types"),
                        }))
                        .into(),
                    );
                }

                let selected_type_index = partition_types
                    .iter()
                    .position(|t| t.ty == volume.partition_type_id)
                    .unwrap_or(0);

                let legacy_bios_bootable = volume.is_legacy_bios_bootable();
                let system_partition = volume.is_system_partition();
                let hidden = volume.is_hidden();
                let name = volume.name.clone();

                *dialog = Some(ShowDialog::EditPartition(EditPartitionDialog {
                    volume,
                    partition_types,
                    selected_type_index,
                    name,
                    legacy_bios_bootable,
                    system_partition,
                    hidden,
                    running: false,
                }));
                return Task::none();
            }

            VolumesControlMessage::OpenResizePartition => {
                if dialog.is_some() {
                    return Task::none();
                }

                let Some(segment) = self.segments.get(self.selected_segment) else {
                    return Task::none();
                };
                let Some(volume) = segment.volume.clone() else {
                    return Task::none();
                };

                if volume.volume_type != disks_dbus::VolumeType::Partition {
                    return Task::none();
                }

                let right_free_bytes = self
                    .segments
                    .get(self.selected_segment.saturating_add(1))
                    .filter(|s| s.kind == DiskSegmentKind::FreeSpace)
                    .map(|s| s.size)
                    .unwrap_or(0);

                let max_size_bytes = volume.size.saturating_add(right_free_bytes);
                let min_size_bytes = volume
                    .usage
                    .as_ref()
                    .map(|u| u.used)
                    .unwrap_or(0)
                    .min(max_size_bytes);

                if max_size_bytes.saturating_sub(min_size_bytes) < 1024 {
                    return Task::none();
                }

                let new_size_bytes = volume.size.clamp(min_size_bytes, max_size_bytes);

                *dialog = Some(ShowDialog::ResizePartition(ResizePartitionDialog {
                    volume,
                    min_size_bytes,
                    max_size_bytes,
                    new_size_bytes,
                    running: false,
                }));
                return Task::none();
            }

            VolumesControlMessage::OpenEditFilesystemLabel => {
                if dialog.is_some() {
                    return Task::none();
                }

                let target = if let Some(node) = self.selected_volume_node() {
                    if !node.can_mount() {
                        return Task::none();
                    }

                    FilesystemTarget::Node(node.clone())
                } else {
                    let Some(segment) = self.segments.get(self.selected_segment) else {
                        return Task::none();
                    };
                    let Some(volume) = segment.volume.clone() else {
                        return Task::none();
                    };

                    if !volume.can_mount() {
                        return Task::none();
                    }

                    FilesystemTarget::Volume(volume)
                };

                *dialog = Some(ShowDialog::EditFilesystemLabel(EditFilesystemLabelDialog {
                    target,
                    label: String::new(),
                    running: false,
                }));
                return Task::none();
            }

            VolumesControlMessage::OpenCheckFilesystem => {
                if dialog.is_some() {
                    return Task::none();
                }

                let target = if let Some(node) = self.selected_volume_node() {
                    if !node.can_mount() {
                        return Task::none();
                    }
                    FilesystemTarget::Node(node.clone())
                } else {
                    let Some(segment) = self.segments.get(self.selected_segment) else {
                        return Task::none();
                    };
                    let Some(volume) = segment.volume.clone() else {
                        return Task::none();
                    };
                    if !volume.can_mount() {
                        return Task::none();
                    }
                    FilesystemTarget::Volume(volume)
                };

                *dialog = Some(ShowDialog::ConfirmAction(ConfirmActionDialog {
                    title: fl!("check-filesystem").to_string(),
                    body: fl!("check-filesystem-warning").to_string(),
                    target,
                    ok_message: VolumesControlMessage::CheckFilesystemConfirm.into(),
                    running: false,
                }));
                return Task::none();
            }

            VolumesControlMessage::CheckFilesystemConfirm => {
                let Some(ShowDialog::ConfirmAction(state)) = dialog.as_mut() else {
                    return Task::none();
                };

                if state.running {
                    return Task::none();
                }
                state.running = true;

                let target = state.target.clone();
                return Task::perform(
                    async move {
                        match target {
                            FilesystemTarget::Volume(v) => v.check_filesystem().await?,
                            FilesystemTarget::Node(n) => n.check_filesystem().await?,
                        }
                        DriveModel::get_drives().await
                    },
                    |result| match result {
                        Ok(drives) => Message::UpdateNav(drives, None).into(),
                        Err(e) => Message::Dialog(Box::new(ShowDialog::Info {
                            title: fl!("check-filesystem").to_string(),
                            body: format!("{e:#}"),
                        }))
                        .into(),
                    },
                );
            }

            VolumesControlMessage::OpenRepairFilesystem => {
                if dialog.is_some() {
                    return Task::none();
                }

                let Some(segment) = self.segments.get(self.selected_segment) else {
                    return Task::none();
                };
                let Some(volume) = segment.volume.clone() else {
                    return Task::none();
                };

                if volume.volume_type != disks_dbus::VolumeType::Filesystem {
                    return Task::none();
                }

                *dialog = Some(ShowDialog::ConfirmAction(ConfirmActionDialog {
                    title: fl!("repair-filesystem").to_string(),
                    body: fl!("repair-filesystem-warning").to_string(),
                    target: FilesystemTarget::Volume(volume),
                    ok_message: VolumesControlMessage::RepairFilesystemConfirm.into(),
                    running: false,
                }));
                return Task::none();
            }

            VolumesControlMessage::RepairFilesystemConfirm => {
                let Some(ShowDialog::ConfirmAction(state)) = dialog.as_mut() else {
                    return Task::none();
                };

                if state.running {
                    return Task::none();
                }
                state.running = true;

                let target = state.target.clone();
                return Task::perform(
                    async move {
                        match target {
                            FilesystemTarget::Volume(v) => v.repair_filesystem().await?,
                            FilesystemTarget::Node(n) => n.repair_filesystem().await?,
                        }
                        DriveModel::get_drives().await
                    },
                    |result| match result {
                        Ok(drives) => Message::UpdateNav(drives, None).into(),
                        Err(e) => Message::Dialog(Box::new(ShowDialog::Info {
                            title: fl!("repair-filesystem").to_string(),
                            body: format!("{e:#}"),
                        }))
                        .into(),
                    },
                );
            }

            VolumesControlMessage::OpenTakeOwnership => {
                if dialog.is_some() {
                    return Task::none();
                }

                let Some(segment) = self.segments.get(self.selected_segment) else {
                    return Task::none();
                };
                let Some(volume) = segment.volume.clone() else {
                    return Task::none();
                };

                if volume.volume_type != disks_dbus::VolumeType::Filesystem {
                    return Task::none();
                }

                *dialog = Some(ShowDialog::TakeOwnership(TakeOwnershipDialog {
                    volume,
                    recursive: true,
                    running: false,
                }));
                return Task::none();
            }

            VolumesControlMessage::OpenChangePassphrase => {
                if dialog.is_some() {
                    return Task::none();
                }

                let Some(segment) = self.segments.get(self.selected_segment) else {
                    return Task::none();
                };
                let Some(volume) = segment.volume.clone() else {
                    return Task::none();
                };

                let is_crypto_container =
                    find_volume_node_for_partition(&self.model.volumes, &volume)
                        .is_some_and(|n| n.kind == VolumeKind::CryptoContainer);
                if !is_crypto_container {
                    return Task::none();
                }

                *dialog = Some(ShowDialog::ChangePassphrase(ChangePassphraseDialog {
                    volume,
                    current_passphrase: String::new(),
                    new_passphrase: String::new(),
                    confirm_passphrase: String::new(),
                    error: None,
                    running: false,
                }));
                return Task::none();
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

                    ShowDialog::AddPartition(state) => match create_message {
                        CreateMessage::SizeUpdate(size) => state.info.size = size,
                        CreateMessage::NameUpdate(name) => {
                            state.info.name = name;
                        }
                        CreateMessage::PasswordUpdate(password) => state.info.password = password,
                        CreateMessage::ConfirmedPasswordUpdate(confirmed_password) => {
                            state.info.confirmed_password = confirmed_password
                        }
                        CreateMessage::PasswordProectedUpdate(protect) => {
                            state.info.password_protected = protect
                        }
                        CreateMessage::EraseUpdate(erase) => state.info.erase = erase,
                        CreateMessage::PartitionTypeUpdate(p_type) => {
                            state.info.selected_partitition_type = p_type
                        }
                        CreateMessage::Continue => {
                            eprintln!("CreateMessage::Continue is not implemented; ignoring.");
                        }
                        CreateMessage::Cancel => return Task::done(Message::CloseDialog.into()),
                        CreateMessage::Partition => {
                            if state.running {
                                return Task::none();
                            }

                            state.running = true;

                            let mut create_partition_info: CreatePartitionInfo = state.info.clone();
                            if create_partition_info.name.is_empty() {
                                create_partition_info.name = fl!("untitled").to_string();
                            }

                            let model = self.model.clone();
                            return Task::perform(
                                async move {
                                    model.create_partition(create_partition_info).await?;
                                    DriveModel::get_drives().await
                                },
                                |result| match result {
                                    Ok(drives) => Message::UpdateNav(drives, None).into(),
                                    Err(e) => Message::Dialog(Box::new(ShowDialog::Info {
                                        title: fl!("app-title"),
                                        body: format!("{e:#}"),
                                    }))
                                    .into(),
                                },
                            );
                        }
                    },

                    ShowDialog::FormatPartition(state) => match create_message {
                        CreateMessage::NameUpdate(name) => {
                            state.info.name = name;
                        }
                        CreateMessage::EraseUpdate(erase) => state.info.erase = erase,
                        CreateMessage::PartitionTypeUpdate(p_type) => {
                            state.info.selected_partitition_type = p_type
                        }
                        CreateMessage::Cancel => return Task::done(Message::CloseDialog.into()),
                        CreateMessage::Partition => {
                            if state.running {
                                return Task::none();
                            }
                            state.running = true;

                            let volume = state.volume.clone();
                            let info = state.info.clone();
                            return Task::perform(
                                async move {
                                    let fs_type = common_partition_filesystem_type(
                                        info.table_type.as_str(),
                                        info.selected_partitition_type,
                                    )
                                    .ok_or_else(|| anyhow::anyhow!("Invalid filesystem selection"))?
                                    .to_string();

                                    volume
                                        .format(info.name.clone(), info.erase, fs_type)
                                        .await?;
                                    DriveModel::get_drives().await
                                },
                                |result| match result {
                                    Ok(drives) => Message::UpdateNav(drives, None).into(),
                                    Err(e) => Message::Dialog(Box::new(ShowDialog::Info {
                                        title: fl!("format-partition").to_string(),
                                        body: format!("{e:#}"),
                                    }))
                                    .into(),
                                },
                            );
                        }
                        _ => {}
                    },

                    ShowDialog::UnlockEncrypted(_) => {
                        eprintln!(
                            "CreateMessage received while an unlock dialog is open; ignoring."
                        );
                    }

                    ShowDialog::FormatDisk(_) => {
                        eprintln!(
                            "CreateMessage received while a format disk dialog is open; ignoring."
                        );
                    }

                    ShowDialog::SmartData(_) => {
                        eprintln!("CreateMessage received while a SMART dialog is open; ignoring.");
                    }

                    ShowDialog::NewDiskImage(_)
                    | ShowDialog::AttachDiskImage(_)
                    | ShowDialog::ImageOperation(_) => {
                        eprintln!(
                            "CreateMessage received while an image dialog is open; ignoring."
                        );
                    }

                    ShowDialog::EditPartition(_)
                    | ShowDialog::ResizePartition(_)
                    | ShowDialog::EditFilesystemLabel(_)
                    | ShowDialog::ConfirmAction(_)
                    | ShowDialog::TakeOwnership(_)
                    | ShowDialog::ChangePassphrase(_) => {
                        eprintln!(
                            "CreateMessage received while a different dialog is open; ignoring."
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
                        if state.running {
                            return Task::none();
                        }

                        state.running = true;

                        let partition_path = state.partition_path.clone();
                        let partition_name = state.partition_name.clone();
                        let passphrase = state.passphrase.clone();
                        let passphrase_for_task = passphrase.clone();

                        // Look up the partition in the current model.
                        let part = self
                            .model
                            .volumes_flat
                            .iter()
                            .find(|p| p.path.to_string() == partition_path)
                            .cloned();

                        let Some(p) = part else {
                            return Task::done(
                                Message::Dialog(Box::new(ShowDialog::Info {
                                    title: fl!("unlock-failed"),
                                    body: fl!("unlock-missing-partition", name = partition_name),
                                }))
                                .into(),
                            );
                        };

                        return Task::perform(
                            async move {
                                p.unlock(&passphrase_for_task).await?;
                                DriveModel::get_drives().await
                            },
                            move |result| match result {
                                Ok(drives) => Message::UpdateNav(drives, None).into(),
                                Err(e) => {
                                    eprintln!("Unlock encrypted dialog error: {e}");
                                    Message::Dialog(Box::new(ShowDialog::UnlockEncrypted(
                                        UnlockEncryptedDialog {
                                            partition_path: partition_path.clone(),
                                            partition_name: partition_name.clone(),
                                            passphrase: passphrase.clone(),
                                            error: Some(e.to_string()),
                                            running: false,
                                        },
                                    )))
                                    .into()
                                }
                            },
                        );
                    }
                }
            }

            VolumesControlMessage::EditPartitionMessage(msg) => {
                let Some(ShowDialog::EditPartition(state)) = dialog.as_mut() else {
                    return Task::none();
                };

                match msg {
                    EditPartitionMessage::TypeUpdate(idx) => state.selected_type_index = idx,
                    EditPartitionMessage::NameUpdate(name) => state.name = name,
                    EditPartitionMessage::LegacyBiosBootableUpdate(v) => {
                        state.legacy_bios_bootable = v
                    }
                    EditPartitionMessage::SystemPartitionUpdate(v) => state.system_partition = v,
                    EditPartitionMessage::HiddenUpdate(v) => state.hidden = v,
                    EditPartitionMessage::Cancel => return Task::done(Message::CloseDialog.into()),
                    EditPartitionMessage::Confirm => {
                        if state.running {
                            return Task::none();
                        }

                        let partition_type = state
                            .partition_types
                            .get(state.selected_type_index)
                            .map(|t| t.ty.to_string());

                        let Some(partition_type) = partition_type else {
                            return Task::none();
                        };

                        state.running = true;

                        let volume = state.volume.clone();
                        let name = state.name.clone();
                        let legacy = state.legacy_bios_bootable;
                        let system = state.system_partition;
                        let hidden = state.hidden;

                        return Task::perform(
                            async move {
                                let flags =
                                    VolumeModel::make_partition_flags_bits(legacy, system, hidden);

                                volume.edit_partition(partition_type, name, flags).await?;
                                DriveModel::get_drives().await
                            },
                            |result| match result {
                                Ok(drives) => Message::UpdateNav(drives, None).into(),
                                Err(e) => Message::Dialog(Box::new(ShowDialog::Info {
                                    title: fl!("edit-partition").to_string(),
                                    body: format!("{e:#}"),
                                }))
                                .into(),
                            },
                        );
                    }
                }

                return Task::none();
            }

            VolumesControlMessage::ResizePartitionMessage(msg) => {
                let Some(ShowDialog::ResizePartition(state)) = dialog.as_mut() else {
                    return Task::none();
                };

                match msg {
                    ResizePartitionMessage::SizeUpdate(size) => {
                        state.new_size_bytes =
                            size.clamp(state.min_size_bytes, state.max_size_bytes)
                    }
                    ResizePartitionMessage::Cancel => {
                        return Task::done(Message::CloseDialog.into());
                    }
                    ResizePartitionMessage::Confirm => {
                        if state.running {
                            return Task::none();
                        }

                        // Disable when range is too small.
                        if state.max_size_bytes.saturating_sub(state.min_size_bytes) < 1024 {
                            return Task::none();
                        }

                        state.running = true;
                        let volume = state.volume.clone();
                        let new_size = state.new_size_bytes;

                        return Task::perform(
                            async move {
                                volume.resize(new_size).await?;
                                DriveModel::get_drives().await
                            },
                            |result| match result {
                                Ok(drives) => Message::UpdateNav(drives, None).into(),
                                Err(e) => Message::Dialog(Box::new(ShowDialog::Info {
                                    title: fl!("resize-partition").to_string(),
                                    body: format!("{e:#}"),
                                }))
                                .into(),
                            },
                        );
                    }
                }

                return Task::none();
            }

            VolumesControlMessage::EditFilesystemLabelMessage(msg) => {
                let Some(ShowDialog::EditFilesystemLabel(state)) = dialog.as_mut() else {
                    return Task::none();
                };

                match msg {
                    EditFilesystemLabelMessage::LabelUpdate(label) => state.label = label,
                    EditFilesystemLabelMessage::Cancel => {
                        return Task::done(Message::CloseDialog.into());
                    }
                    EditFilesystemLabelMessage::Confirm => {
                        if state.running {
                            return Task::none();
                        }

                        state.running = true;
                        let target = state.target.clone();
                        let label = state.label.clone();

                        return Task::perform(
                            async move {
                                match target {
                                    FilesystemTarget::Volume(v) => {
                                        v.edit_filesystem_label(label).await?
                                    }
                                    FilesystemTarget::Node(n) => {
                                        n.edit_filesystem_label(&label).await?
                                    }
                                }
                                DriveModel::get_drives().await
                            },
                            |result| match result {
                                Ok(drives) => Message::UpdateNav(drives, None).into(),
                                Err(e) => Message::Dialog(Box::new(ShowDialog::Info {
                                    title: fl!("edit-filesystem").to_string(),
                                    body: format!("{e:#}"),
                                }))
                                .into(),
                            },
                        );
                    }
                }

                return Task::none();
            }

            VolumesControlMessage::TakeOwnershipMessage(msg) => {
                let Some(ShowDialog::TakeOwnership(state)) = dialog.as_mut() else {
                    return Task::none();
                };

                match msg {
                    TakeOwnershipMessage::RecursiveUpdate(v) => state.recursive = v,
                    TakeOwnershipMessage::Cancel => return Task::done(Message::CloseDialog.into()),
                    TakeOwnershipMessage::Confirm => {
                        if state.running {
                            return Task::none();
                        }

                        state.running = true;
                        let volume = state.volume.clone();
                        let recursive = state.recursive;

                        return Task::perform(
                            async move {
                                volume.take_ownership(recursive).await?;
                                DriveModel::get_drives().await
                            },
                            |result| match result {
                                Ok(drives) => Message::UpdateNav(drives, None).into(),
                                Err(e) => Message::Dialog(Box::new(ShowDialog::Info {
                                    title: fl!("take-ownership").to_string(),
                                    body: format!("{e:#}"),
                                }))
                                .into(),
                            },
                        );
                    }
                }

                return Task::none();
            }

            VolumesControlMessage::ChangePassphraseMessage(msg) => {
                let Some(ShowDialog::ChangePassphrase(state)) = dialog.as_mut() else {
                    return Task::none();
                };

                match msg {
                    ChangePassphraseMessage::CurrentUpdate(v) => {
                        state.current_passphrase = v;
                        state.error = None;
                    }
                    ChangePassphraseMessage::NewUpdate(v) => {
                        state.new_passphrase = v;
                        state.error = None;
                    }
                    ChangePassphraseMessage::ConfirmUpdate(v) => {
                        state.confirm_passphrase = v;
                        state.error = None;
                    }
                    ChangePassphraseMessage::Cancel => {
                        return Task::done(Message::CloseDialog.into());
                    }
                    ChangePassphraseMessage::Confirm => {
                        if state.running {
                            return Task::none();
                        }

                        if state.new_passphrase.is_empty()
                            || state.new_passphrase != state.confirm_passphrase
                        {
                            state.error = Some(fl!("passphrase-mismatch").to_string());
                            return Task::none();
                        }

                        state.running = true;
                        let volume = state.volume.clone();
                        let current = state.current_passphrase.clone();
                        let new = state.new_passphrase.clone();

                        return Task::perform(
                            async move {
                                volume.change_passphrase(&current, &new).await?;
                                DriveModel::get_drives().await
                            },
                            |result| match result {
                                Ok(drives) => Message::UpdateNav(drives, None).into(),
                                Err(e) => Message::Dialog(Box::new(ShowDialog::Info {
                                    title: fl!("change-passphrase").to_string(),
                                    body: format!("{e:#}"),
                                }))
                                .into(),
                            },
                        );
                    }
                }

                return Task::none();
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
                    .volume
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
            .volume
            .as_ref()
            .and_then(|p| find_volume_node_for_partition(&self.model.volumes, p));

        let selected_child_volume = self.selected_volume_node();

        match selected.kind {
            DiskSegmentKind::Partition => {
                if let Some(p) = selected.volume.as_ref() {
                    // Container actions are based on the selected partition (segment).
                    if let Some(v) = selected_volume
                        && v.kind == VolumeKind::CryptoContainer
                    {
                        if v.locked {
                            action_bar.push(tooltip_icon_button(
                                "dialog-password-symbolic",
                                fl!("unlock-button").to_string(),
                                Some(Message::Dialog(Box::new(ShowDialog::UnlockEncrypted(
                                    UnlockEncryptedDialog {
                                        partition_path: p.path.to_string(),
                                        partition_name: p.name(),
                                        passphrase: String::new(),
                                        error: None,
                                        running: false,
                                    },
                                )))),
                            ));
                        } else {
                            action_bar.push(tooltip_icon_button(
                                "changes-prevent-symbolic",
                                fl!("lock").to_string(),
                                Some(VolumesControlMessage::LockContainer.into()),
                            ));
                        }
                    }

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
                            action_bar.push(tooltip_icon_button(
                                icon_name,
                                fl!("mount-toggle").to_string(),
                                Some(msg.into()),
                            ));
                        }
                    } else if p.can_mount() {
                        let (icon_name, msg) = if p.is_mounted() {
                            (
                                "media-playback-stop-symbolic",
                                VolumesControlMessage::Unmount,
                            )
                        } else {
                            (
                                "media-playback-start-symbolic",
                                VolumesControlMessage::Mount,
                            )
                        };
                        action_bar.push(tooltip_icon_button(
                            icon_name,
                            fl!("mount-toggle").to_string(),
                            Some(msg.into()),
                        ));
                    }
                }
            }
            DiskSegmentKind::FreeSpace => {
                action_bar.push(tooltip_icon_button(
                    "list-add-symbolic",
                    fl!("create-partition").to_string(),
                    Some(Message::Dialog(Box::new(ShowDialog::AddPartition(
                        CreatePartitionDialog {
                            info: selected.get_create_info(),
                            running: false,
                        },
                    )))),
                ));
            }
            DiskSegmentKind::Reserved => {}
        }

        if selected.kind == DiskSegmentKind::Partition
            && let Some(p) = selected.volume.as_ref()
        {
            // Command visibility is based on the selected partition (segment), not a selected child.
            // Child selection only affects filesystem-targeted actions (label/check).

            // Always: Format Partition
            action_bar.push(tooltip_icon_button(
                "edit-clear-symbolic",
                fl!("format-partition").to_string(),
                Some(VolumesControlMessage::OpenFormatPartition.into()),
            ));

            // Partition-only: Edit Partition + Resize
            if selected_child_volume.is_none() && p.volume_type == disks_dbus::VolumeType::Partition
            {
                action_bar.push(tooltip_icon_button(
                    "document-edit-symbolic",
                    fl!("edit-partition").to_string(),
                    Some(VolumesControlMessage::OpenEditPartition.into()),
                ));

                let right_free_bytes = self
                    .segments
                    .get(self.selected_segment.saturating_add(1))
                    .filter(|s| s.kind == DiskSegmentKind::FreeSpace)
                    .map(|s| s.size)
                    .unwrap_or(0);
                let max_size = p.size.saturating_add(right_free_bytes);
                let min_size = p.usage.as_ref().map(|u| u.used).unwrap_or(0).min(max_size);

                let resize_enabled = max_size.saturating_sub(min_size) >= 1024;
                action_bar.push(tooltip_icon_button(
                    "transform-scale-symbolic",
                    fl!("resize-partition").to_string(),
                    resize_enabled.then_some(VolumesControlMessage::OpenResizePartition.into()),
                ));
            }

            // Partition + Filesystem: Edit filesystem label + Check filesystem
            let fs_target_available = selected_child_volume
                .map(|n| n.can_mount())
                .unwrap_or_else(|| p.can_mount());
            if fs_target_available {
                action_bar.push(tooltip_icon_button(
                    "tag-symbolic",
                    fl!("edit-filesystem").to_string(),
                    Some(VolumesControlMessage::OpenEditFilesystemLabel.into()),
                ));
                action_bar.push(tooltip_icon_button(
                    "emblem-ok-symbolic",
                    fl!("check-filesystem").to_string(),
                    Some(VolumesControlMessage::OpenCheckFilesystem.into()),
                ));
            }

            // Filesystem-only (VolumeType::Filesystem): Repair + Take Ownership
            if selected_child_volume.is_none()
                && p.volume_type == disks_dbus::VolumeType::Filesystem
            {
                action_bar.push(tooltip_icon_button(
                    "tools-symbolic",
                    fl!("repair-filesystem").to_string(),
                    Some(VolumesControlMessage::OpenRepairFilesystem.into()),
                ));
                action_bar.push(tooltip_icon_button(
                    "user-home-symbolic",
                    fl!("take-ownership").to_string(),
                    Some(VolumesControlMessage::OpenTakeOwnership.into()),
                ));
            }

            // Container-only (VolumeType::Container): Change Passphrase
            if selected_volume.is_some_and(|v| v.kind == VolumeKind::CryptoContainer) {
                action_bar.push(tooltip_icon_button(
                    "dialog-password-symbolic",
                    fl!("change-passphrase").to_string(),
                    Some(VolumesControlMessage::OpenChangePassphrase.into()),
                ));
            }

            // Delete partition
            if selected_child_volume.is_none()
                && p.volume_type != disks_dbus::VolumeType::Filesystem
            {
                action_bar.push(widget::horizontal_space().into());
                action_bar.push(tooltip_icon_button(
                    "edit-delete-symbolic",
                    fl!("delete", name = selected.name.clone()).to_string(),
                    Some(Message::Dialog(Box::new(ShowDialog::DeletePartition(
                        DeletePartitionDialog {
                            name: selected.name.clone(),
                            running: false,
                        },
                    )))),
                ));
            }
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

fn common_partition_filesystem_type(table_type: &str, index: usize) -> Option<&'static str> {
    match table_type {
        "gpt" => disks_dbus::COMMON_GPT_TYPES
            .get(index)
            .map(|p: &PartitionTypeInfo| p.filesystem_type),
        "dos" => disks_dbus::COMMON_DOS_TYPES
            .get(index)
            .map(|p: &PartitionTypeInfo| p.filesystem_type),
        _ => None,
    }
}

fn common_partition_type_index_for(table_type: &str, id_type: Option<&str>) -> usize {
    let Some(id_type) = id_type else {
        return 0;
    };

    let list: &[PartitionTypeInfo] = match table_type {
        "gpt" => &disks_dbus::COMMON_GPT_TYPES,
        "dos" => &disks_dbus::COMMON_DOS_TYPES,
        _ => return 0,
    };

    list.iter()
        .position(|p| p.filesystem_type.eq_ignore_ascii_case(id_type))
        .unwrap_or(0)
}

fn tooltip_icon_button(
    icon_name: &str,
    tooltip: String,
    msg: Option<Message>,
) -> Element<'_, Message> {
    let mut button = widget::button::custom(icon::from_name(icon_name));
    if let Some(m) = msg {
        button = button.on_press(m);
    }

    widget::tooltip(
        button,
        widget::text::body(tooltip),
        widget::tooltip::Position::Top,
    )
    .into()
}

fn collect_mounted_descendants_leaf_first(node: &VolumeNode) -> Vec<VolumeNode> {
    fn visit(node: &VolumeNode, out: &mut Vec<VolumeNode>) {
        for child in &node.children {
            visit(child, out);
        }

        if node.can_mount() && node.is_mounted() {
            out.push(node.clone());
        }
    }

    let mut out = Vec::new();
    visit(node, &mut out);
    out
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
    partition: &VolumeModel,
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
