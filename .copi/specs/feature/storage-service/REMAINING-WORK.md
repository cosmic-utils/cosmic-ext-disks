# Remaining Work for storage-ui Migration

## Overview

The operation call replacements are complete (with TODO comments). What remains is the **type system migration** - replacing the core data structures that represent disk/volume state throughout the application.

---

## Core Type Replacements Needed

### 1. DriveModel → DiskInfo (24 imports)

**Current Usage:**
```rust
use disks_dbus::DriveModel;

// Startup
let drives: Vec<DriveModel> = DriveModel::get_drives().await?;

// Access properties
drive.name
drive.block_path
drive.size
drive.volumes // nested VolumeNode tree
drive.volumes_flat // flat Vec<VolumeModel>
```

**Target Usage:**
```rust
use storage_models::DiskInfo;
use crate::client::DisksClient;

// Startup
let disks_client = DisksClient::new().await?;
let disks: Vec<DiskInfo> = disks_client.list_disks().await?;

// Access properties (different names)
disk.label // was: drive.name
disk.device // was: drive.block_path
disk.size_bytes // was: drive.size
// volumes accessed separately via queries
```

**Files Affected:**
- `ui/app/mod.rs` - Initial load
- `ui/app/state.rs` - State storage (Vec<DriveModel>)
- `ui/app/message.rs` - Message types
- `ui/app/view.rs` - Rendering
- `ui/app/update/*.rs` - All update handlers (8 files)
- `ui/sidebar/*.rs` - Sidebar rendering (2 files)
- `ui/volumes/*.rs` - Volume management (10 files)
- `ui/dialogs/state.rs` - Dialog state

### 2. VolumeModel (6 imports)

**Current:**
```rust
pub struct VolumeModel {
    pub volume_type: VolumeType,
    pub table_path: OwnedObjectPath,
    pub name: String,
    pub partition_type_id: String,
    pub partition_type: String,
    pub id_type: String,
    pub uuid: String,
    pub number: u32,
    pub flags: BitFlags<PartitionFlags>,
    pub offset: u64,
    pub size: u64,
    pub path: OwnedObjectPath,
    pub device_path: Option<String>,
    pub has_filesystem: bool,
    pub mount_points: Vec<String>,
    pub usage: Option<Usage>,
}
```

**Target:** Use `VolumeInfo` from storage-models (recursive tree structure)

**Files:**
- `ui/volumes/state.rs`
- `ui/volumes/helpers.rs`
- `ui/volumes/update/partition.rs`
- `ui/dialogs/state.rs`

### 3. VolumeNode (10 imports)

**Current:**
```rust
pub struct VolumeNode {
    pub kind: VolumeKind,
    pub label: String,
    pub size: u64,
    pub id_type: String,
    pub object_path: OwnedObjectPath,
    pub device_path: Option<String>,
    pub has_filesystem: bool,
    pub mount_points: Vec<String>,
    pub usage: Option<Usage>,
    pub locked: bool,
    pub children: Vec<VolumeNode>, // recursive
}
```

**Target:** `VolumeInfo` from storage-models (already recursive)

**Files:**
- `ui/sidebar/view.rs`
- `ui/volumes/view.rs`
- `ui/volumes/disk_header.rs`
- `ui/volumes/helpers.rs`
- `ui/volumes/state.rs`
- `ui/volumes/update/partition.rs`
- `ui/volumes/update/encryption.rs`
- `ui/dialogs/state.rs`
- `ui/app/update/mod.rs`

### 4. DiskError → ClientError (3 files)

**Current:** Checking for `DiskError::ResourceBusy`

**Target:** Check `ClientError` or handle differently via UnmountResult

**Files:**
- `ui/volumes/update/mount.rs` (2 sites)
- `ui/app/update/mod.rs` (1 site)

### 5. DiskManager (Already has TODOs)

- `ui/app/subscriptions.rs` - Device events
- `ui/app/update/mod.rs` - Module config

---

## Implementation Strategy

### Step 1: Add Client Infrastructure to AppModel

```rust
// ui/app/state.rs
use crate::client::*;
use std::sync::Arc;

pub struct AppModel {
    // Existing fields...
    
    // NEW: Storage service clients
    pub disks_client: Arc<DisksClient>,
    pub partitions_client: Arc<PartitionsClient>,
    pub filesystems_client: Arc<FilesystemsClient>,
    pub luks_client: Arc<LuksClient>,
    pub btrfs_client: Arc<BtrfsClient>,
    pub image_client: Arc<ImageClient>,
}

impl AppModel {
    pub async fn initialize_clients() -> Result<Self, ClientError> {
        // Initialize all clients
        // Handle connection failures gracefully
    }
}
```

### Step 2: Replace State Storage Types

```rust
// OLD
pub struct AppModel {
    pub drives: Vec<DriveModel>,
}

// NEW
pub struct AppModel {
    pub disks: Vec<DiskInfo>,
    // Note: Volumes will be queried on-demand or cached separately
}
```

### Step 3: Update All Property Accesses

Search and replace patterns:
- `drive.name` → `disk.label`
- `drive.block_path` → `disk.device`
- `drive.size` → `disk.size_bytes`
- `drive.volumes` → Query volumes separately
- `volume.path` → `volume.object_path` (or similar)

### Step 4: Implement Client Operations

Remove all TODO comments and implement real client calls:

```rust
// OLD
v.unmount().await?;

// NEW
let filesystems_client = &self.filesystems_client; // from AppModel
filesystems_client.unmount(&device_path, false, false).await?;
```

### Step 5: Update Volume Querying

**Current:** Volumes come nested in DriveModel
**New:** Query volumes separately when needed

```rust
// When user selects a disk, query its volumes
let volumes = disks_client.get_volumes(&disk_device).await?;
// Or use a separate volumes cache/state
```

### Step 6: Error Handling Migration

```rust
// OLD
if let Some(disk_err) = e.downcast_ref::<DiskError>()
    && matches!(disk_err, DiskError::ResourceBusy { .. })
{
    // Handle busy
}

// NEW
match filesystems_client.unmount(...).await {
    Ok(UnmountResult::Success) => {},
    Ok(UnmountResult::Busy { processes, .. }) => {
        // Handle busy with process list
    }
    Err(e) => {
        // Handle other errors
    }
}
```

---

## Testing Strategy

### Phase-by-Phase Testing

**After Step 1 (Client Infrastructure):**
- Verify app starts
- Verify clients connect to service
- Test graceful degradation if service not running

**After Step 2 (State Types):**
- Verify data loads into new structures
- Check compilation of property accesses

**After Step 3 (Property Updates):**
- Verify UI renders correctly
- Check all display strings

**After Step 4 (Operations):**
- Test each operation individually:
  - Mount/unmount (with process killing)
  - Create/delete/resize partition
  - Format filesystem
  - Lock/unlock LUKS
  - Btrfs operations
  - SMART info
  - Drive commands (eject, power off)

**After Step 5 (Volume Queries):**
- Test navigation between disks
- Test volume hierarchy display
- Test child node operations

**After Step 6 (Error Handling):**
- Test error conditions
- Test permission denied scenarios
- Test resource busy scenarios

---

## Critical Dependencies

Before this work can proceed:

1. ✅ **storage-models** - Type definitions complete
2. ⏸️ **storage-service** - D-Bus service must be implemented and running
3. ⏸️ **Client implementations** - All client modules must be complete
4. ⏸️ **Polkit policies** - Must be installed for authentication

---

## Estimated Effort

Based on 35 files with type changes and ~60 operation sites:

- **Step 1 (Clients):** 1-2 days
- **Step 2 (State):** 2-3 days
- **Step 3 (Properties):** 3-4 days
- **Step 4 (Operations):** 5-7 days (remove all TODOs)
- **Step 5 (Volumes):** 3-5 days
- **Step 6 (Errors):** 2-3 days

**Total: 16-24 days (3.5-5 weeks)**

With testing and debugging: **4-6 weeks**

---

## Success Criteria

- [ ] Zero `use disks_dbus::` imports (except maybe re-exported types)
- [ ] `storage-dbus` dependency removed from Cargo.toml
- [ ] All clients initialized in AppModel
- [ ] All operations use client methods
- [ ] All property accesses use new field names
- [ ] All tests pass
- [ ] App runs without storage-dbus
- [ ] Full feature parity with old architecture
- [ ] No performance regressions

---

**Document Status:** Work-in-Progress Guide
**Last Updated:** 2026-02-14
**Next Action:** Await storage-service implementation (Phase 3B)
