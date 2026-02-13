/// Messages for BTRFS management operations
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum Message {
    /// Toggle the BTRFS management section expansion
    ToggleExpanded,
    /// Toggle a subvolume's snapshots expansion
    ToggleSubvolumeExpanded(u64), // subvolume ID
}
