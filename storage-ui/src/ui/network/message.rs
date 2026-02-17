// SPDX-License-Identifier: GPL-3.0-only

//! Messages for network mount management

use storage_common::rclone::{ConfigScope, RemoteConfig};

/// Messages for network mount operations
#[derive(Debug, Clone)]
pub enum NetworkMessage {
    /// Load all configured remotes
    LoadRemotes,
    /// Remotes loaded from service
    RemotesLoaded(Result<Vec<RemoteConfig>, String>),
    /// Select a remote in the sidebar
    SelectRemote { name: String, scope: ConfigScope },
    /// Mount a remote
    MountRemote { name: String, scope: ConfigScope },
    /// Unmount a remote
    UnmountRemote { name: String, scope: ConfigScope },
    /// Restart a remote (unmount then mount)
    RestartRemote { name: String, scope: ConfigScope },
    /// Mount operation completed
    MountCompleted {
        name: String,
        scope: ConfigScope,
        result: Result<(), String>,
    },
    /// Unmount operation completed
    UnmountCompleted {
        name: String,
        scope: ConfigScope,
        result: Result<(), String>,
    },
    /// Test remote configuration
    TestRemote { name: String, scope: ConfigScope },
    /// Test result received
    TestCompleted {
        name: String,
        result: Result<String, String>,
    },
    /// Refresh mount status for a remote
    RefreshStatus { name: String, scope: ConfigScope },
    /// Status refreshed
    StatusRefreshed {
        name: String,
        scope: ConfigScope,
        mounted: bool,
    },
    /// Open add remote dialog
    OpenAddRemote,
    /// Open edit remote dialog
    OpenEditRemote { name: String, scope: ConfigScope },
    /// Delete remote (with confirmation)
    DeleteRemote { name: String, scope: ConfigScope },
    /// Confirm delete remote
    ConfirmDeleteRemote { name: String, scope: ConfigScope },
    /// Delete completed
    DeleteCompleted {
        name: String,
        result: Result<(), String>,
    },
    /// Cancel current operation / close dialog
    Cancel,
}
