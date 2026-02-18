# Data Model: RClone Mount Management

**Feature**: 072-rclone-mounts
**Date**: 2026-02-17

## Entity Overview

```
┌─────────────────┐     ┌──────────────────┐
│  RemoteConfig   │────<│  NetworkMount    │
│  (configuration)│     │  (runtime state) │
└─────────────────┘     └──────────────────┘
        │                       │
        │                       │
        v                       v
┌─────────────────┐     ┌──────────────────┐
│  ConfigScope    │     │  MountStatus     │
│  (enum)         │     │  (enum)          │
└─────────────────┘     └──────────────────┘
```

## Entities

### ConfigScope (Enum)

Defines whether a configuration is per-user or system-wide.

```rust
pub enum ConfigScope {
    User,    // ~/.config/rclone/rclone.conf
    System,  // /etc/rclone.conf
}
```

| Value | Config Path | Mount Prefix | Auth Required |
|-------|-------------|--------------|---------------|
| `User` | `~/.config/rclone/rclone.conf` | `~/mnt/<remote>/` | No |
| `System` | `/etc/rclone.conf` | `/mnt/rclone/<remote>/` | Yes (polkit) |

### MountStatus (Enum)

Runtime state of a network mount.

```rust
pub enum MountStatus {
    Unmounted,   // Not currently mounted
    Mounting,    // Mount operation in progress
    Mounted,     // Successfully mounted and accessible
    Unmounting,  // Unmount operation in progress
    Error(String), // Error state with message
}
```

**State Transitions**:
```
Unmounted --[start]--> Mounting --[success]--> Mounted
    ^                      |                       |
    |                    [fail]                  [stop]
    |                      v                       v
    +------------------ Error <--- Unmounting <---+
```

### RemoteConfig (Struct)

Configuration for a single RClone remote.

```rust
pub struct RemoteConfig {
    /// Unique name for this remote (e.g., "my-drive")
    pub name: String,

    /// Backend type (e.g., "drive", "s3", "ftp")
    pub remote_type: String,

    /// Configuration scope (user or system)
    pub scope: ConfigScope,

    /// Raw configuration key-value pairs from rclone.conf
    pub options: HashMap<String, String>,

    /// Whether sensitive fields (tokens, secrets) are present
    pub has_secrets: bool,
}
```

**Validation Rules**:
- `name` must be non-empty, alphanumeric with dashes/underscores
- `remote_type` must be a valid rclone backend type
- `options` must include required keys for the remote type

### NetworkMount (Struct)

Represents a mountable network storage resource with runtime state.

```rust
pub struct NetworkMount {
    /// Reference to the remote configuration
    pub remote_name: String,

    /// Configuration scope
    pub scope: ConfigScope,

    /// Current mount status
    pub status: MountStatus,

    /// Mount point path (resolved from scope)
    pub mount_point: PathBuf,

    /// Mount type for future extensibility (rclone, samba, ftp)
    pub mount_type: MountType,
}
```

### MountType (Enum)

Type of network mount backend. Designed for future extensibility.

```rust
pub enum MountType {
    RClone,
    // Future:
    // Samba,
    // Ftp,
}
```

### RemoteConfigList (Struct)

Container for listing remotes with scope information.

```rust
pub struct RemoteConfigList {
    pub remotes: Vec<RemoteConfig>,
    pub user_config_path: Option<PathBuf>,
    pub system_config_path: Option<PathBuf>,
}
```

## Serialization

All entities implement:
- `Serialize` / `Deserialize` (serde) for D-Bus transport
- `Clone`, `Debug`, `PartialEq` for testing

JSON format is used for D-Bus method returns (following existing pattern in `storage-service/src/btrfs.rs`).

## File Locations

### storage-common/src/rclone.rs

```rust
// All data model types for RClone functionality
pub use ConfigScope::{self, *};
pub use MountStatus::{self, *};
pub use MountType::{self, *};
pub use RemoteConfig;
pub use RemoteConfigList;
pub use NetworkMount;
```

## D-Bus Transport Format

Following existing patterns, complex types are serialized to JSON strings:

```rust
// Example D-Bus method signature
async fn list_remotes(&self) -> zbus::fdo::Result<String>; // Returns JSON RemoteConfigList
```

This matches the pattern in `btrfs.rs:30-50` where `SubvolumeList` is returned as JSON.
