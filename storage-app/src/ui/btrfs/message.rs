use storage_types::BtrfsSubvolume;

/// Messages for BTRFS management operations
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum Message {
    /// Toggle the BTRFS management section expansion
    ToggleExpanded,
    /// Toggle a subvolume's snapshots expansion
    ToggleSubvolumeExpanded(u64), // subvolume ID
    /// Load the default subvolume
    LoadDefaultSubvolume,
    /// Default subvolume loaded
    DefaultSubvolumeLoaded(Result<BtrfsSubvolume, String>),
    /// Set a subvolume as the default
    SetDefaultSubvolume { subvolume_id: u64 },
    /// Toggle readonly flag on a subvolume
    ToggleReadonly { subvolume_id: u64 },
    /// Readonly toggle completed
    ReadonlyToggled(Result<(), String>),
    /// Show properties dialog for a subvolume
    ShowProperties { subvolume_id: u64 },
    /// Close the properties dialog
    CloseProperties,
    /// Load deleted subvolumes pending cleanup
    LoadDeletedSubvolumes,
    /// Deleted subvolumes loaded
    DeletedSubvolumesLoaded(Result<Vec<BtrfsSubvolume>, String>),
    /// Toggle showing deleted subvolumes in the UI
    ToggleShowDeleted,
    /// Refresh all BTRFS data (subvolumes, usage, default, etc.)
    RefreshAll,
}
