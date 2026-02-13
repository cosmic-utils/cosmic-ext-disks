use crate::utils::btrfs::{Subvolume, UsageInfo};

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
    /// Filesystem usage information
    pub usage_info: Option<Result<UsageInfo, String>>,
    /// Compression algorithm (None = not loaded, Some(None) = disabled, Some(Some(algo)) = enabled)
    pub compression: Option<Option<String>>,
    /// Loading state for usage info
    pub loading_usage: bool,
}

impl BtrfsState {
    /// Create a new state for the given mount point
    pub fn new(mount_point: Option<String>) -> Self {
        Self {
            expanded: true, // Start expanded by default for better UX
            loading: false,
            subvolumes: None,
            mount_point,
            usage_info: None,
            compression: None,
            loading_usage: false,
        }
    }
}
