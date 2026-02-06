use crate::{
    fl,
    ui::volumes::helpers,
    utils::{DiskSegmentKind, PartitionExtent, SegmentAnomaly, compute_disk_segments},
};
use disks_dbus::{CreatePartitionInfo, DriveModel, VolumeModel, VolumeNode};

pub struct VolumesControl {
    pub selected_segment: usize,
    pub selected_volume: Option<String>,
    pub segments: Vec<Segment>,
    pub show_reserved: bool,
    pub(crate) model: DriveModel,
}

#[derive(Clone, Debug)]
pub struct Segment {
    pub label: String,
    pub name: String,
    #[allow(dead_code)]
    pub partition_type: String,
    pub size: u64,
    pub offset: u64,
    pub state: bool,
    pub kind: DiskSegmentKind,
    pub width: u16,
    pub volume: Option<VolumeModel>,
    #[allow(dead_code)]
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

    #[allow(dead_code)]
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
                    tracing::warn!(
                        id,
                        partition_offset,
                        previous_end,
                        "partition segmentation anomaly: overlaps previous segment"
                    );
                }
                SegmentAnomaly::PartitionStartsPastDisk {
                    id,
                    partition_offset,
                    disk_size,
                } => {
                    tracing::warn!(
                        id,
                        partition_offset,
                        disk_size,
                        "partition segmentation anomaly: starts past disk end"
                    );
                }
                SegmentAnomaly::PartitionEndPastDisk {
                    id,
                    partition_end,
                    disk_size,
                } => {
                    tracing::warn!(
                        id,
                        partition_end,
                        disk_size,
                        "partition segmentation anomaly: ends past disk end"
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

        // Convert tiny free space (<10MB) to reserved space to hide alignment/reserved gaps,
        // UNLESS the whole drive is free space OR the drive is very small (<100MB).
        const TINY_FREE_THRESHOLD: u64 = 10 * 1024 * 1024; // 10MB
        const SMALL_DRIVE_THRESHOLD: u64 = 100 * 1024 * 1024; // 100MB

        let all_free_space = segments
            .iter()
            .all(|s| s.kind == DiskSegmentKind::FreeSpace);
        let is_small_drive = drive.size < SMALL_DRIVE_THRESHOLD;

        if !all_free_space && !is_small_drive {
            for segment in segments.iter_mut() {
                if segment.kind == DiskSegmentKind::FreeSpace && segment.size < TINY_FREE_THRESHOLD
                {
                    segment.kind = DiskSegmentKind::Reserved;
                }
            }
        }

        if segments.len() > 1 {
            let mut merged: Vec<Segment> = Vec::with_capacity(segments.len());
            for segment in segments.into_iter() {
                if let Some(last) = merged.last_mut()
                    && last.kind == DiskSegmentKind::Reserved
                    && segment.kind == DiskSegmentKind::Reserved
                    && last.offset.saturating_add(last.size) == segment.offset
                {
                    last.size = last.size.saturating_add(segment.size);
                } else {
                    merged.push(segment);
                }
            }
            segments = merged;
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
        helpers::find_volume_node(&self.model.volumes, object_path)
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
}
