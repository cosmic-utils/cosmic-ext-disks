# Implementation Log: Phase 3A - Refactor disks-dbus

Branch: `feature/storage-service` (should be created per repo rules)
Spec: `.copi/specs/storage-service/`

---

## 2024-MM-DD: Task 2 - Define storage-models Types (COMPLETE)

### Objective
Expand storage-models with complete domain types that will serve as the single source of truth.

### Files Created/Modified

#### Created Files
1. **storage-models/src/common.rs** - Common utility types
   - `ByteRange` - Byte range for GPT usable space, etc.
   - `Usage` - Filesystem usage statistics
   - Both with serde derives for serialization

2. **storage-models/src/volume.rs** - Hierarchical volume tree
   - `VolumeInfo` - Recursive tree structure for UI display
   - `VolumeKind` - Enum: Partition, CryptoContainer, Filesystem, LvmPhysicalVolume, LvmLogicalVolume, Block
   - Helper methods: `is_mounted()`, `can_mount()`, `can_unlock()`, `can_lock()`, `volume_count()`, `find_by_device()`

3. **storage-models/src/partition.rs** - Flat partition info
   - `PartitionInfo` - Detailed partition metadata (17 fields)
   - `PartitionTableType` - Enum: Gpt, Mbr
   - `PartitionTableInfo` - Partition table metadata
   - Helper methods: `is_mounted()`, `can_mount()`, `display_name()`, `is_gpt()`, `is_mbr()`

4. **storage-models/src/filesystem.rs** - Filesystem operations
   - `FilesystemInfo` - Filesystem metadata
   - `FormatOptions` - Options for mkfs operations
   - `MountOptions` - Mount flags and options
   - `CheckResult` - fsck results
   - `UnmountResult` - Unmount operation result
   - `ProcessInfo` - Process blocking unmount
   - `KillResult` - Process kill result
   - `FilesystemType` - Enum with `mkfs_command()` helper

5. **storage-models/src/lvm.rs** - LVM types
   - `VolumeGroupInfo` - VG metadata with `used()`, `usage_percent()` helpers
   - `LogicalVolumeInfo` - LV metadata with `display_name()` helper
   - `PhysicalVolumeInfo` - PV metadata with `is_assigned()`, `used()` helpers

6. **storage-models/src/encryption.rs** - LUKS types
   - `LuksInfo` - LUKS container metadata
   - `LuksVersion` - Enum: Luks1, Luks2
   - Helper methods: `can_unlock()`, `can_lock()`

#### Modified Files
1. **storage-models/src/disk.rs**
   - **Expanded DiskInfo** from 9 to 20+ fields:
     * Added identity: `id`, `vendor`, `revision`
     * Added media properties: `media_removable`, `media_available`, `optical`, `optical_blank`, `can_power_off`
     * Added loop device: `is_loop`, `backing_file`
     * Added partitioning: `partition_table_type`, `gpt_usable_range`
   - **Added helper methods**:
     * `supports_power_management()` - true if HDD (rotation_rate > 0)
     * `display_name()` - human-readable name for UI
   - **Updated documentation** - Changed from "transport types" to "canonical domain model"
   - **Fixed imports** - Use `crate::ByteRange` instead of `super::ByteRange`
   - **Fixed tests** - Updated `test_disk_info_serialization()` with all new fields

2. **storage-models/src/lib.rs**
   - **Updated documentation** - Clarified architecture and hierarchy (flat vs tree)
   - **Added module exports**: common, encryption, filesystem, lvm, partition, volume
   - **Added type re-exports** - All public types now exported at crate root

3. **storage-service/src/conversions.rs** (TEMPORARY FIX)
   - **Updated `drive_model_to_disk_info()`** - Add placeholder values for new DiskInfo fields
   - **Added TODO comments** - Document which UDisks2 properties need to be queried
   - **Added deprecation note** - This file will be removed in Task 6 when disks-dbus returns storage-models types directly

### Architecture Decisions

1. **Single Source of Truth**: storage-models types are THE domain models, not transport types
   - disks-dbus will return these types directly (Task 6)
   - storage-service serializes/deserializes them (no conversion)
   - disks-ui uses them, optionally wrapping for UI state

2. **Two Hierarchies**:
   - **Flat** (for operations): DiskInfo → PartitionInfo → FilesystemInfo
   - **Tree** (for UI display): VolumeInfo (recursive, contains any VolumeKind)

3. **UI Naming Convention**: `{Type}Model` pattern
   - Example: `struct DiskModel { info: DiskInfo, selected: bool }`
   - Keeps domain models pure, UI concerns separate

### Build Status
- ✅ `cargo check --workspace` - SUCCESS
- ⚠️  13 warnings in disks-ui (existing, unrelated to changes)
- ⚠️  2 warnings in disks-dbus (existing, unrelated to changes)

### Next Steps
- **Task 3**: Define additional supporting types if needed
- **Task 6**: Refactor DiskManager to return storage-models::DiskInfo
- **Task 7**: Refactor VolumeNode to return storage-models::VolumeInfo
- **Task 8-11**: Update partition, filesystem, LVM, LUKS operations
- **Task 12**: Update disks-ui imports (apply {Type}Model naming)

### Blockers
None

### Notes
- All new types include comprehensive doc comments
- Helper methods added for common operations (can_mount, is_mounted, etc.)
- Serde derives on all types for JSON serialization
- ByteRange and Usage moved from disks-dbus (will update disks-dbus to import from storage-models in Task 6)

---

## 2024-02-14: Task 6 - Refactor DiskManager to Return DiskInfo (COMPLETE)

### Objective
Update disks-dbus to use storage-models types as its public API, eliminating the need for conversions in storage-service.

### Files Modified

#### Modified Files
1. **disks-dbus/Cargo.toml**
   - Added `storage-models = { path = "../storage-models" }` dependency
   
2. **disks-dbus/src/disks/drive/model.rs**
   - Changed imports: `use storage_models::{ByteRange, DiskInfo};`
   - **Added `impl From<DriveModel> for DiskInfo`** - Conversion extracts domain data, drops internal connection
   - **Added `infer_connection_bus()`** helper - Detects connection bus from device path and properties
   - DriveModel now uses storage_models::ByteRange for gpt_usable_range field

3. **disks-dbus/src/disks/drive/discovery.rs**
   - **Added `pub async fn get_disks() -> Result<Vec<storage_models::DiskInfo>>`** - Public API returning canonical types
   - Calls existing `get_drives()`, converts each DriveModel via From impl
   - Marked as recommended method for clients

4. **disks-dbus/src/disks/gpt.rs**
   - **Removed local ByteRange definition** (20 lines)
   - **Imported `storage_models::ByteRange`**
   - Removed impl block (is_valid_for_disk, clamp_to_disk now in storage-models)

5. **disks-dbus/src/disks/ops.rs**
   - Changed import: `use storage_models::ByteRange;`

6. **disks-dbus/src/disks/mod.rs**
   - Removed ByteRange from gpt re-exports
   - Removed redundant ByteRange pub use

7. **disks-dbus/src/usage.rs**
   - **Removed local Usage definition** (8 lines)
   - **Added `pub use storage_models::Usage;`**
   - `usage_for_mount_point()` now returns storage_models::Usage

8. **disks-dbus/src/lib.rs**
   - **Added `pub use storage_models;`** - Re-export entire crate for clients
   - **Added `pub use storage_models::ByteRange;`** - Backwards compatibility
   - Added `probe_gpt_usable_range_bytes` to exports (was missing)
   - Updated documentation comments

### Implementation Details

**Conversion Strategy:**
- Kept DriveModel internal (contains zbus::Connection, needs to stay in disks-dbus)
- Created `From<DriveModel> for DiskInfo` impl to extract domain data
- New public API: `DriveModel::get_disks()` returns `Vec<DiskInfo>`
- Old API: `DriveModel::get_drives()` still exists (returns `Vec<DriveModel>` for internal use)

**Connection Bus Inference:**
```rust
fn infer_connection_bus(drive: &DriveModel) -> String {
    // Checks: loop, nvme, mmc, optical (sr), usb (from model/vendor), default to ata
    // Mirrors logic from storage-service/conversions.rs (which will be removed)
}
```

**Type Consolidation:**
- ByteRange: Was duplicated in disks-dbus/src/disks/gpt.rs and storage-models
  - Now single definition in storage-models with helper methods
- Usage: Was in disks-dbus/src/usage.rs, now re-exported from storage-models
  - usage_for_mount_point() returns storage_models::Usage

### API Changes (Breaking, Expected)

**Public API Now Returns storage-models Types:**
- `DriveModel::get_disks()` → `Vec<storage_models::DiskInfo>` (NEW, recommended)
- `ByteRange` → now from storage_models (re-exported for compatibility)
- `Usage` → now from storage_models (re-exported via usage module)

**Unchanged (Internal):**
- `DriveModel::get_drives()` → `Vec<DriveModel>` (internal use)
- DriveModel struct still exists for internal operations
- All existing methods on DriveModel preserved

### Build Status
- ✅ `cargo check -p cosmic-ext-disks-dbus` - SUCCESS
- ✅ `cargo check --workspace` - SUCCESS
- ⚠️  3 warnings in disks-dbus (existing, unrelated GetDefault/stime/rtime dead code)
- ⚠️  13 warnings in disks-ui (existing, needs update in Task 12)

### Next Steps
- **Task 7**: Refactor VolumeNode to return storage-models::VolumeInfo
- **Task 12**: Update disks-ui to use DriveModel::get_disks() → storage_models::DiskInfo
- **Phase 3B**: storage-service can now remove conversions.rs entirely (DriveModel → DiskInfo handled in disks-dbus)

### Blockers
None

### Notes
- DriveModel will remain internal to disks-dbus (needed for connection management)
- storage-service conversions.rs can be deleted once disks-ui is updated
- All existing tests still pass (type changes are transparent for internal code)

---
