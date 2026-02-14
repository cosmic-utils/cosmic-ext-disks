# disks-ui Migration Analysis — Replace disks-dbus with storage-service Clients

**Analysis Date:** 2026-02-14  
**Phase:** Phase 3A Complete, Phase 3B Prerequisite  
**Task Reference:** [disk-partition-ops-tasks.md](./disk-partition-ops-tasks.md) Task 12  
**Status:** Ready for Implementation (After Phase 3B)

---

## Executive Summary

**Objective:** Replace ALL disks-dbus usage in disks-ui with storage-service D-Bus clients and storage-models types.

**Total Impact:**
- **Files affected:** ~35 files
- **Import statements:** 20+ need updating  
- **Operation call sites:** 60+ need client replacements
- **Estimated effort:** 15-21 days (3-4 weeks)

**Strategy:** Incremental migration in 5 phases (type imports → client init → discovery → operations → cleanup)

---

## Analysis Results

### CATEGORY 1: TYPE-ONLY IMPORTS

**Summary:** 20+ import statements that can be changed from `disks_dbus::` to `storage_models::` with zero logic changes.

#### Utility Functions (10 files)

- `bytes_to_pretty`, `pretty_to_bytes` → `storage_models::`
- `get_step`, `get_numeric` → `storage_models::`
- `GPT_ALIGNMENT_BYTES` → `storage_models::`
- `COMMON_GPT_TYPES`, `COMMON_DOS_TYPES` → `storage_models::partition_types::`

**Files:** utils/ui.rs, utils/segments.rs, ui/dialogs/view/partition.rs, ui/dialogs/view/image.rs, ui/volumes/usage_pie.rs, ui/volumes/view.rs, ui/app/view.rs

#### Type Definitions (15 files)

- `VolumeKind` → `storage_models::VolumeKind` (8 files)
- `VolumeType` → `storage_models::VolumeType` (4 files)
- `CreatePartitionInfo` → `storage_models::CreatePartitionInfo` (4 files)
- `PartitionTypeInfo` → `storage_models::partition_types::PartitionTypeInfo` (3 files)
- `ProcessInfo` → `storage_models::ProcessInfo` (1 file)

#### Btrfs Types (3 files)

- `BtrfsSubvolume` → `storage_models::btrfs::BtrfsSubvolume`

**Files:** ui/btrfs/view.rs, ui/btrfs/state.rs, ui/btrfs/message.rs

---

### CATEGORY 2: OPERATIONAL TYPES

**Summary:** Types that require D-Bus operations to obtain. Need to replace data source with storage-service client calls.

#### DriveModel (10 files)

**Current:**
```rust
use disks_dbus::DriveModel;
let drives = DriveModel::get_drives().await?;
```

**Replacement:**
```rust
use storage_models::disk::DiskInfo;
let drives = disks_client.get_drives().await?; // Returns Vec<DiskInfo>
```

**Files:** app/mod.rs, sidebar/state.rs, sidebar/view.rs, volumes/disk_header.rs, app/view.rs, app/message.rs, app/update/nav.rs, app/update/drive.rs, volumes/update/create.rs, volumes/state.rs

#### VolumeModel & VolumeNode (8 files)

**Current:**
```rust
use disks_dbus::{VolumeModel, VolumeNode};
let volumes = drive.volumes(); // Returns tree structure
```

**Replacement:**
```rust
use storage_models::volume::VolumeInfo;
let volumes = disks_client.get_volumes(device).await?; // Returns Vec<VolumeInfo>
```

**Files:** volumes/state.rs, volumes/helpers.rs, volumes/view.rs, sidebar/view.rs, btrfs/view.rs, volumes/disk_header.rs, volumes/update/partition.rs, dialogs/state.rs

#### DiskManager (2 files)

**Replacement:** `disks_client.subscribe_device_events()`

**Files:** app/subscriptions.rs, app/update/mod.rs

#### BtrfsFilesystem (2 files)

**Replacement:** Use existing `btrfs_client` (already in disks-ui/src/client/btrfs.rs)

**Files:** app/update/btrfs.rs, volumes/update/btrfs.rs

#### SmartInfo (2 files)

**Replacement:** `disks_client.get_smart_info(device)`

**Files:** app/update/smart.rs, app/update/drive.rs, dialogs/message.rs, dialogs/state.rs

---

### CATEGORY 3: OPERATIONS (60+ call sites)

#### Drive Operations (6 sites)

- `DriveModel::get_drives()` → `DisksClient::get_drives()`
- `drive.format_disk()` → `PartitionsClient::create_partition_table()`
- `drive.remove()` → `DisksClient::eject()`
- `drive.power_off()` → `DisksClient::power_off()`
- `drive.smart_info()` → `DisksClient::get_smart_info()`

#### Partition Operations (3 sites)

- `create_partition()` → `PartitionsClient::create_partition()`
- `delete()` → `PartitionsClient::delete_partition()`
- `resize()` → `PartitionsClient::resize_partition()`

#### Filesystem Operations (4 sites)

- `format()` → `FilesystemsClient::format()`
- `edit_filesystem_label()` → `FilesystemsClient::set_label()`
- `check_filesystem()` → `FilesystemsClient::check()`
- `repair_filesystem()` → `FilesystemsClient::repair()`

#### Mount/Unmount (12+ sites)

- `mount()` → `FilesystemsClient::mount()`
- `unmount()` → `FilesystemsClient::unmount()`

**Files:** volumes/update/mount.rs (4 sites), app/update/mod.rs (2 sites), volumes/update/encryption.rs, volumes/update/partition.rs, app/update/image/ops.rs (2 sites)

#### Encryption (3 sites)

- `unlock()` → `LuksClient::unlock()`
- `lock()` → `LuksClient::lock()`

**Files:** volumes/update/encryption.rs (2 sites), volumes/update/partition.rs

#### Btrfs (10+ sites)

- `BtrfsFilesystem::new()` → Use existing `btrfs_client`
- `list_subvolumes()` → `btrfs_client.list_subvolumes()`
- `create_subvolume()` → `btrfs_client.create_subvolume()`
- `create_snapshot()` → `btrfs_client.create_snapshot()`
- `delete_subvolume()` → `btrfs_client.delete_subvolume()`

**Files:** app/update/btrfs.rs (5+ operations), volumes/update/btrfs.rs (2+ operations)

#### Device Events (1 site)

- `DiskManager::new()`, `device_event_stream_signals()` → `DisksClient::subscribe_device_events()`

**File:** app/subscriptions.rs

---

## Implementation Strategy

### Phase 1: Type-Only Replacements (1-2 days, Low Risk)

Replace import statements only:
- Utility functions → `storage_models::`
- Constants/catalogs → `storage_models::`
- Enums/structs → `storage_models::`
- Btrfs types → `storage_models::btrfs::`

**Testing:** Compile check only

**Files:** ~20 files

### Phase 2: Initialize Clients (2-3 days, Medium Risk)

Add client infrastructure:

```rust
pub struct AppState {
    disks_client: Arc<DisksClient>,
    partitions_client: Arc<PartitionsClient>,
    filesystems_client: Arc<FilesystemsClient>,
    luks_client: Arc<LuksClient>,
    btrfs_client: Arc<BtrfsClient>,
    image_client: Arc<ImageClient>,
}
```

Initialize on startup, pass to components.

**Testing:** App starts, clients connect

### Phase 3: Replace Discovery (3-5 days, Medium Risk)

Replace data sources:
- `DriveModel::get_drives()` → `disks_client.get_drives()`
- Update state storage: `Vec<DiskInfo>` instead of `Vec<DriveModel>`
- Volume trees use `VolumeInfo`
- Device event subscription

**Testing:** Drive list loads, hotplug works

**Files:** ~10 files

### Phase 4: Replace Operations (8-10 days, High Risk)

Incrementally replace 60+ operation call sites:

**Sub-phases:**
1. Read-only operations (SMART, check, list)
2. Mount/unmount operations
3. Partition operations (create, delete, resize)
4. Filesystem operations (format, label, repair)
5. Encryption & Btrfs operations

**Testing:** Each operation tested individually

**Files:** 20+ files

### Phase 5: Cleanup (1 day, Low Risk)

- Remove `disks-dbus` from Cargo.toml
- Verify no remaining imports
- Final integration test

---

## Risk Assessment

**Low Risk:**
- Type-only import changes (Phase 1)
- Storage-models types are 1:1 with disks-dbus

**Medium Risk:**
- Client initialization (Phase 2) - new async startup
- Discovery changes (Phase 3) - core data flow

**High Risk:**
- Operation replacements (Phase 4) - D-Bus signature differences
- Error handling changes
- State management without DriveModel references

**Mitigation:**
- Implement one phase at a time
- Extensive per-operation testing
- Feature flag for gradual rollout (optional)

---

## Testing Strategy

### Per-Phase Testing

**Phase 1:** Compile check, no logic changes

**Phase 2:** App starts, clients connect, graceful error if service not running

**Phase 3:** Drive list, sidebar, volume tree, device hotplug events

**Phase 4:** Each operation with valid/invalid inputs, errors, permissions

**Phase 5:** Full app workflow without disks-dbus

### Critical Test Scenarios

1. **Drive Discovery:** Multiple device types (NVMe, SATA, USB)
2. **Device Hotplug:** Insert/remove USB drive
3. **Partition Operations:** Create, delete, resize
4. **Filesystem Operations:** Format, mount, unmount (with process killing)
5. **LUKS:** Unlock/lock with correct/wrong passphrase
6. **Btrfs:** List, create subvolume/snapshot, delete
7. **SMART:** Get status/attributes, NotSupported error
8. **Error Handling:** Service not running, permission denied, device busy

---

## Prerequisites

**Before starting:**

- ✅ storage-models types defined (Phase 3A complete)
- ⏸️ **storage-service D-Bus interface implemented** (Phase 3B)
  - All operation methods exposed
  - Polkit policies configured
  - Service installed and running
- ⏸️ Client wrappers exist in disks-ui/src/client/
  - DisksClient
  - PartitionsClient
  - FilesystemsClient
  - LuksClient
  - BtrfsClient ✅ (already exists)
  - ImageClient

---

## Success Criteria

- [ ] All 20+ `use disks_dbus::` imports replaced
- [ ] All client instances initialized
- [ ] All 60+ operation call sites use clients
- [ ] `disks-dbus` removed from Cargo.toml
- [ ] Zero compilation errors
- [ ] Full test suite passes
- [ ] No regressions in functionality
- [ ] Error handling improved

---

## Timeline Estimate

| Phase | Days | Risk | Dependencies |
|-------|------|------|--------------|
| Phase 1: Type Imports | 1-2 | Low | None |
| Phase 2: Client Init | 2-3 | Medium | Phase 1 |
| Phase 3: Discovery | 3-5 | Medium | Phase 2 |
| Phase 4: Operations | 8-10 | High | Phase 3 |
| Phase 5: Cleanup | 1 | Low | Phase 4 |

**Total: 15-21 days (3-4 weeks)**

**With 25% buffer: 19-26 days (4-5 weeks)**

---

## File Summary

**Type-Only (20 files):**
utils/ui.rs, utils/segments.rs, ui/dialogs/view/partition.rs, ui/dialogs/view/image.rs, ui/volumes/usage_pie.rs, ui/volumes/view.rs, ui/volumes/helpers.rs, ui/app/view.rs, ui/sidebar/view.rs, ui/btrfs/* (3 files), ui/volumes/update/* (3 files), ui/dialogs/state.rs, ui/volumes/state.rs

**Operational Types (15 files):**
ui/app/mod.rs, ui/app/message.rs, ui/app/subscriptions.rs, ui/app/update/* (4 files), ui/sidebar/* (2 files), ui/volumes/* (4 files), ui/volumes/update/btrfs.rs

**Operations (20+ files, 60+ call sites):**
All update/* modules with actual D-Bus method calls

---

**Document Status:** Analysis Complete — Ready for Implementation  
**Last Updated:** 2026-02-14  
**Next Review:** After Phase 3B completion
