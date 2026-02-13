use disks_dbus::BtrfsSubvolume;
use std::collections::HashMap;

/// State for BTRFS management UI
#[derive(Debug, Clone, Default)]
pub struct BtrfsState {
    /// Loading state for subvolumes
    pub loading: bool,
    /// List of subvolumes (None = not loaded yet, Some(Ok) = loaded, Some(Err) = error)
    pub subvolumes: Option<Result<Vec<BtrfsSubvolume>, String>>,
    /// Mount point for the BTRFS filesystem
    pub mount_point: Option<String>,
    /// Block device object path for D-Bus calls
    pub block_path: Option<String>,
    /// Filesystem usage (used bytes)
    pub used_space: Option<Result<u64, String>>,
    /// Loading state for usage info
    pub loading_usage: bool,
    /// Expander state: maps subvolume ID to expanded state (true = expanded)
    pub expanded_subvolumes: HashMap<u64, bool>,
    /// Default subvolume ID
    pub default_subvolume_id: Option<u64>,
    /// List of deleted subvolumes pending cleanup
    pub deleted_subvolumes: Option<Vec<BtrfsSubvolume>>,
    /// Whether to show deleted subvolumes in the UI
    pub show_deleted: bool,
    /// Currently selected subvolume (for properties/operations)
    pub selected_subvolume: Option<BtrfsSubvolume>,
    /// Whether to show the properties dialog
    pub show_properties_dialog: bool,
}

impl BtrfsState {
    /// Create a new state for the given mount point and block path
    pub fn new(mount_point: Option<String>, block_path: Option<String>) -> Self {
        Self {
            loading: false,
            subvolumes: None,
            mount_point,
            block_path,
            used_space: None,
            loading_usage: false,
            expanded_subvolumes: HashMap::new(),
            default_subvolume_id: None,
            deleted_subvolumes: None,
            show_deleted: false,
            selected_subvolume: None,
            show_properties_dialog: false,
        }
    }
}
