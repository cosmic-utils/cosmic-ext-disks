use crate::utils::DiskSegmentKind;
use disks_dbus::{DriveModel, VolumeModel};

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
