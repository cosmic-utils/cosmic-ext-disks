# Architecture Overview — cosmic-ext-storage

**Last updated**: 2026-02-14
**Maintainer**: This document provides project context for speckit workflows

---

## Project Identity

- **Name**: COSMIC Ext Storage (cosmic-ext-storage)
- **Purpose**: Disk utility application for the COSMIC desktop
- **Type**: Rust workspace with multiple crates
- **Platform**: Linux only (systemd-based distributions)

---

## Workspace Structure

```
cosmic-ext-storage/
├── storage-ui/           # COSMIC GUI application (libcosmic-based)
├── storage-service/      # D-Bus service for privileged operations
├── storage-dbus/         # UDisks2 D-Bus abstraction layer
├── storage-common/       # Shared data models and types
├── storage-sys/          # Low-level system operations
├── storage-btrfs/        # BTRFS-specific utilities
└── .specify/             # Spec kit configuration and memory
    ├── memory/
    │   ├── constitution.md   # Project governance
    │   └── architecture.md   # This file
    └── specs/                # Feature specs by branch
```

---

## Crate Responsibilities

| Crate | Purpose | Key Types |
|-------|---------|-----------|
| `storage-ui` | COSMIC GUI application | `AppModel`, `UiDrive`, `UiVolume` |
| `storage-service` | D-Bus service (root) | `DisksInterface`, `PartitionsInterface` |
| `storage-dbus` | UDisks2 abstraction | `DriveModel`, `VolumeNode`, `VolumeModel` |
| `storage-common` | Shared domain types | `DiskInfo`, `VolumeInfo`, `PartitionInfo` |
| `storage-sys` | Low-level system ops | Command execution, sysfs reading |
| `storage-btrfs` | BTRFS utilities | `SubvolumeManager`, snapshot operations |

### Crate Dependencies

```
storage-ui
    ├── storage-common
    └── storage-service (via D-Bus)

storage-service
    ├── storage-dbus
    ├── storage-common
    ├── storage-btrfs
    └── storage-sys

storage-dbus
    └── storage-common

storage-btrfs
    └── storage-common

storage-sys
    └── (standalone)
```

---

## Architecture Layers

```
┌─────────────────────────────────────────────────────────────┐
│                     storage-ui (GUI)                        │
│  COSMIC/libcosmic application, views, user interaction      │
│  - AppModel: Application state and subscriptions            │
│  - UiDrive/UiVolume: UI wrapper models                      │
│  - ClientPool: Shared D-Bus client instances                │
└─────────────────────────┬───────────────────────────────────┘
                          │ D-Bus (system bus)
                          ▼
┌─────────────────────────────────────────────────────────────┐
│                  storage-service (D-Bus Service)            │
│  Polkit auth, JSON serialization, signal emission           │
│  - Runs as root (euid == 0)                                 │
│  - Exposes org.cosmic.ext.StorageService                    │
│  - Orchestrates operations, delegates to libraries          │
└─────────────────────────┬───────────────────────────────────┘
                          │ D-Bus / library calls
                          ▼
┌─────────────────────────────────────────────────────────────┐
│                  storage-dbus (Library)                     │
│  UDisks2 proxies, drive/volume enumeration, operations      │
│  - DriveModel: Disk enumeration and operations              │
│  - VolumeNode: Hierarchical volume tree                     │
│  - DiskManager: Device change detection                     │
└─────────────────────────┬───────────────────────────────────┘
                          │ D-Bus (system bus)
                          ▼
┌─────────────────────────────────────────────────────────────┐
│                    UDisks2 (System Service)                 │
│  org.freedesktop.UDisks2                                    │
│  Manager, Drive, Block, Partition, Filesystem, Encrypted    │
└─────────────────────────────────────────────────────────────┘
```

---

## Key Types (storage-common)

### DiskInfo

Represents a physical or virtual disk:

```rust
pub struct DiskInfo {
    pub device: String,           // e.g., "/dev/sda"
    pub vendor: Option<String>,
    pub model: Option<String>,
    pub serial: Option<String>,
    pub size: u64,                // bytes
    pub rotation_rate: Option<u32>,
    pub media_removable: bool,
    pub partition_table_type: Option<String>,  // "gpt", "dos", or None
}
```

### VolumeInfo

Represents any volume (partition, LUKS container, LVM LV, BTRFS subvolume):

```rust
pub struct VolumeInfo {
    pub device_path: String,      // e.g., "/dev/sda1"
    pub parent_path: Option<String>,  // For hierarchy (LUKS -> cleartext)
    pub kind: VolumeKind,         // Partition, Luks, Lvm, Btrfs, etc.
    pub label: Option<String>,
    pub size: u64,
    pub filesystem_type: Option<String>,
    pub mount_point: Option<String>,
    pub usage: Option<UsageInfo>,
    pub children: Vec<VolumeInfo>,  // Nested volumes
}
```

### VolumeKind

Discriminant for volume types:

```rust
pub enum VolumeKind {
    Partition,
    FreeSpace,
    Luks,
    LuksCleartext,
    LvmPhysicalVolume,
    LvmLogicalVolume,
    BtrfsSubvolume,
    LoopDevice,
}
```

---

## Volume Hierarchy Model

Volumes form a tree structure. Examples:

```
/dev/sda (Disk)
├── /dev/sda1 (Partition, ext4, mounted at /boot)
├── /dev/sda2 (Partition, LUKS)
│   └── /dev/mapper/luks-xxx (LUKS Cleartext, LVM PV)
│       └── /dev/mapper/vg0-root (LVM LV, ext4, mounted at /)
└── /dev/sda3 (Partition, BTRFS)
    └── subvolume @ (mounted at /)
    └── subvolume @home (mounted at /home)
```

The `VolumeNode` type in `storage-dbus` builds this tree during enumeration.
`VolumeInfo.parent_path` links children to parents for UI tree construction.

---

## Data Flow

### Startup

1. UI initializes localization (Fluent/i18n)
2. AppModel creates `ClientPool` with D-Bus clients
3. UI calls `DisksClient::list_disks()` → storage-service
4. storage-service calls `storage-dbus::DriveModel::get_drives()`
5. storage-dbus enumerates via UDisks2, builds volume tree
6. UI renders drives in nav, volumes as segments

### Device Changes

1. UDisks2 emits `InterfacesAdded`/`InterfacesRemoved` signals
2. `storage-dbus::DiskManager` receives and filters events
3. UI subscription triggers refresh

### User Operations (e.g., Mount)

1. User clicks mount button in UI
2. UI calls `FilesystemsClient::mount()` → storage-service
3. storage-service checks Polkit authorization
4. storage-service calls UDisks2 Filesystem.Mount()
5. Result returned to UI, volume state updated

### Refresh Strategy

| Trigger | Method | Scope |
|---------|--------|-------|
| App startup | `UiDrive::refresh()` | Full tree rebuild |
| Partition create/delete | `UiDrive::refresh()` | Full tree rebuild |
| Mount/unmount | `UiDrive::refresh_volume()` | Single volume atomic update |
| Device event | `AppModel` subscription | Full refresh of affected disk |

---

## Key Technologies

| Category | Technology |
|----------|------------|
| Language | Rust (edition 2024) |
| UI Framework | libcosmic (COSMIC desktop) |
| Async Runtime | Tokio |
| D-Bus | zbus 5.x, udisks2 crate |
| Serialization | serde + serde_json |
| Logging | tracing |
| i18n | fluent + i18n-embed |

---

## D-Bus Interfaces

storage-service exposes these interfaces at `org.cosmic.ext.StorageService`:

| Interface | Methods |
|-----------|---------|
| Disks | list_disks, list_volumes, get_disk_info, eject, power_off |
| Partitions | create_partition, delete_partition, resize_partition |
| Filesystems | format, mount, unmount, check, set_label |
| LUKS | unlock, lock, change_passphrase |
| LVM | list_volume_groups, create_volume_group |
| BTRFS | get_subvolumes, create_subvolume, create_snapshot |
| Image | export_image, restore_image |

### Serialization

All D-Bus methods return JSON strings (not native D-Bus types):

```rust
// Service side
async fn list_disks(&self) -> zbus::fdo::Result<String> {
    let disks: Vec<DiskInfo> = /* ... */;
    serde_json::to_string(&disks).map_err(|e| zbus::fdo::Error::Failed(e.to_string()))
}

// Client side
pub async fn list_disks(&self) -> Result<Vec<DiskInfo>, ClientError> {
    let json = self.proxy.list_disks().await?;
    serde_json::from_str(&json).map_err(ClientError::ParseError)
}
```

**Rationale**: Simpler handling of complex nested types than zvariant.

---

## Error Handling

### Error Types by Crate

| Crate | Error Type | Base |
|-------|------------|------|
| storage-ui | `ClientError` | thiserror |
| storage-service | `zbus::fdo::Error` | zbus |
| storage-dbus | `DiskError` | thiserror |
| storage-btrfs | `BtrfsError` | thiserror |
| storage-sys | `SysError` | thiserror |

### ClientError Variants

```rust
pub enum ClientError {
    Connection(String),
    ServiceNotAvailable,
    PermissionDenied(String),
    MethodCall { message: String, dbus_name: Option<String> },
    ParseError(String),
    Timeout(String),
}
```

---

## Configuration

- **App ID**: `com.cosmos.Disks`
- **Config**: `~/.config/cosmic/com.cosmos.Disks/` (via cosmic_config)
- **Service**: `org.cosmic.ext.StorageService` on system bus
- **Logs**: `~/.local/state/cosmic_ext_storage/` (file logging)

---

## Testing Strategy

| Level | Location | Focus |
|-------|----------|-------|
| Unit | `src/` inline | Individual functions, pure logic |
| Integration | `tests/` | D-Bus serialization round-trips |
| Manual | N/A | Full disk operations on test hardware |

---

## Known Constraints

1. **Root Required**: storage-service must run as root for privileged operations
2. **UDisks2 Dependency**: Requires udisks2 system service running
3. **Filesystem Tools**: ntfs-3g, exfatprogs, dosfstools needed for full support
4. **Single Instance**: Only one storage-service can run at a time (D-Bus name)

---

## Related Documentation

- [Constitution](./constitution.md) - Project governance and principles
