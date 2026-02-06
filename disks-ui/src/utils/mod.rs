mod segments;
mod ui;

// Explicit exports from segments module
pub use segments::{DiskSegmentKind, PartitionExtent, SegmentAnomaly, compute_disk_segments};

// Explicit exports from ui module
pub use ui::labelled_spinner;

// Re-export unused utility functions for future features
#[allow(unused_imports)]
pub use ui::{
    error, error_style, info, info_style, labelled_info, link_info, success, success_style,
    warning, warning_style,
};
