mod fs_tools;
mod segments;
mod ui;
pub mod unit_size_input;

// Explicit exports from fs_tools module
pub use fs_tools::{get_fs_tool_status, get_missing_tools};

// Explicit exports from unit_size_input module
pub use unit_size_input::SizeUnit;

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
