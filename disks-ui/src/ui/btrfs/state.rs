use crate::utils::btrfs::Subvolume;

/// State for BTRFS management UI
#[derive(Debug, Clone, Default)]
pub struct BtrfsState {
    /// Whether the BTRFS section is expanded
    pub expanded: bool,
    /// Loading state for subvolumes
    pub loading: bool,
    /// List of subvolumes (None = not loaded yet, Some(Ok) = loaded, Some(Err) = error)
    pub subvolumes: Option<Result<Vec<Subvolume>, String>>,
    /// Mount point for the BTRFS filesystem
    pub mount_point: Option<String>,
}

impl BtrfsState {
    /// Create a new state for the given mount point
    pub fn new(mount_point: Option<String>) -> Self {
        Self {
            expanded: false,
            loading: false,
            subvolumes: None,
            mount_point,
        }
    }
}
