# D-Bus API Contract: Service Hardening

**Feature**: 001-service-hardening
**Date**: 2026-02-15
**Interface**: `org.cosmic.ext.StorageService.Filesystems`

## Existing Methods (No Changes)

These methods remain unchanged:

| Method | Signature | Description |
|--------|-----------|-------------|
| `list_filesystems` | `() -> String` | List all filesystems |
| `get_supported_filesystems` | `() -> String` | Get supported filesystem types |
| `format` | `(device, fs_type, label, options_json) -> ()` | Format device |
| `mount` | `(device, mount_point, options_json) -> String` | Mount filesystem |
| `check` | `(device, repair) -> String` | Check filesystem |
| `set_label` | `(device, label) -> ()` | Set filesystem label |
| `get_usage` | `(mount_point) -> String` | Get usage stats |
| `get_mount_options` | `(device) -> String` | Get fstab options |
| `default_mount_options` | `(device) -> ()` | Clear fstab options |
| `edit_mount_options` | `(...) -> ()` | Set fstab options |
| `take_ownership` | `(device, recursive) -> ()` | Take ownership |

---

## Modified Methods

### unmount

**Existing signature** (unchanged):
```
unmount(device_or_mount: String, force: bool, kill_processes: bool) -> String
```

**Behavior change**: When `kill_processes=true` and the mount point is a protected system path, returns error in `UnmountResult`.

**Return value** (JSON `UnmountResult`):

```json
// Success case (unchanged)
{
  "success": true,
  "error": null,
  "blocking_processes": []
}

// Protected path error (NEW)
{
  "success": false,
  "error": "Cannot kill processes on system path '/'. This filesystem is critical for system operation.",
  "blocking_processes": []
}

// Busy error (unchanged)
{
  "success": false,
  "error": "Device is busy",
  "blocking_processes": [
    {"pid": 1234, "command": "bash", "username": "user"}
  ]
}
```

**Protected paths**:
- `/` (root)
- `/boot`, `/boot/efi`
- `/home`
- `/usr`, `/var`, `/etc`
- `/opt`, `/srv`, `/tmp`

**Path matching**: Exact match or subdirectory. Symlinks are resolved to canonical paths.

---

## New Methods

### get_filesystem_tools

**Purpose**: Get detailed information about available filesystem tools for UI feature enablement.

**Signature**:
```
get_filesystem_tools() -> String
```

**Authorization**: `org.cosmic.ext.storage-service.filesystem-read` (allow_active)

**Return value** (JSON array of `FilesystemToolInfo`):

```json
[
  {
    "fs_type": "ext4",
    "fs_name": "EXT4",
    "command": "mkfs.ext4",
    "package_hint": "e2fsprogs",
    "available": true
  },
  {
    "fs_type": "xfs",
    "fs_name": "XFS",
    "command": "mkfs.xfs",
    "package_hint": "xfsprogs",
    "available": false
  },
  {
    "fs_type": "btrfs",
    "fs_name": "Btrfs",
    "command": "mkfs.btrfs",
    "package_hint": "btrfs-progs",
    "available": true
  },
  {
    "fs_type": "vfat",
    "fs_name": "FAT32",
    "command": "mkfs.vfat",
    "package_hint": "dosfstools",
    "available": true
  },
  {
    "fs_type": "ntfs",
    "fs_name": "NTFS",
    "command": "mkfs.ntfs",
    "package_hint": "ntfs-3g",
    "available": false
  },
  {
    "fs_type": "exfat",
    "fs_name": "exFAT",
    "command": "mkfs.exfat",
    "package_hint": "exfatprogs",
    "available": true
  },
  {
    "fs_type": "f2fs",
    "fs_name": "F2FS",
    "command": "mkfs.f2fs",
    "package_hint": "f2fs-tools",
    "available": false
  },
  {
    "fs_type": "udf",
    "fs_name": "UDF",
    "command": "mkudffs",
    "package_hint": "udftools",
    "available": false
  }
]
```

**Usage in UI**:
1. Call on app startup
2. Cache results in UI state
3. Enable/disable filesystem options in format dialog based on `available`
4. Show installation hints from `package_hint` for unavailable tools

---

## Backwards Compatibility

| Change | Compatibility |
|--------|---------------|
| `unmount` behavior change | **Fully compatible** - existing error handling path handles new error message |
| New `get_filesystem_tools` method | **Additive** - clients that don't call it are unaffected |

**Versioning**: No version bump required for additive changes.

---

## Error Codes

No new D-Bus error types. All errors returned via `UnmountResult.error` field as strings.

| Error Pattern | Cause |
|---------------|-------|
| `Cannot kill processes on system path '...'` | Protected path + kill_processes=true |
| `Device is busy` | Unmount failed, processes using mount |
| `Authorization failed: ...` | Polkit denial |
| `Failed to ...` | General operation failure |

---

## Internal Library Contract: storage-dbus

*Added during planning phase: Connection caching for storage-dbus layer.*

This documents the internal API changes in `storage-dbus` that are not exposed via D-Bus but affect service behavior.

### Modified Function: get_disks_with_volumes

**Module**: `storage-dbus/src/disk/discovery.rs`

**Before** (current):
```rust
pub async fn get_disks_with_volumes() -> Result<Vec<(DiskInfo, Vec<VolumeInfo>)>>
```

**After** (with connection caching):
```rust
pub async fn get_disks_with_volumes(
    manager: &DiskManager
) -> Result<Vec<(DiskInfo, Vec<VolumeInfo>)>, DiskError>
```

**Contract**:
- **Precondition**: `manager` must be initialized (connection established)
- **Postcondition**: Returns disk enumeration using manager's cached connection
- **Performance**: O(1) connection overhead after first call (vs O(n) before)

### Modified Struct: DiskManager

**Module**: `storage-dbus/src/disk/manager.rs`

**New method**:
```rust
impl DiskManager {
    /// Get a reference to the cached D-Bus connection
    pub fn connection(&self) -> &Arc<Connection>;
}
```

**Contract**:
- Returns reference to Arc-wrapped connection
- Connection is valid for DiskManager's lifetime
- Thread-safe: Arc allows sharing across threads

### Call Site Updates Required

All call sites in `storage-service` must be updated:

```rust
// Before
let disks = get_disks_with_volumes().await?;

// After
let disks = get_disks_with_volumes(&self.manager).await?;
```

**Migration**: Search for `get_disks_with_volumes()` calls and add manager parameter.
