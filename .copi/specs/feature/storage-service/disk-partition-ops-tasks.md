# Storage Service Phase 3: Disk & Partition Operations — Tasks

**Branch:** `feature/storage-service`  
**Phase:** 3 (Breadth Expansion)  
**Related Plan:** [disk-partition-ops-plan.md](./disk-partition-ops-plan.md)

---

## Task Overview

This document breaks down Phase 3 into commit/small-PR sized tasks. Each task should be completeable in 2-4 hours and result in a working, testable increment.

**CRITICAL: Phase 3A Must Complete First**

Phase 3 is split into two major parts:
- **Phase 3A (Tasks 1-15):** Refactor disks-dbus to use storage-models as return types (PREREQUISITE)
- **Phase 3B (Tasks 16-72):** Implement D-Bus service that exposes disks-dbus operations

**Why Phase 3A is Required:**
- Current: disks-dbus has DriveModel/VolumeNode → would need conversion to/from storage-models (circular)
- Correct: storage-models is single source of truth → disks-dbus returns storage-models types directly
- This eliminates double conversion and creates clean architecture

**Estimated Total:** 72 tasks over 10-12 weeks (15 tasks for Phase 3A, 57 for Phase 3B)

---

## Phase 3A: Refactor disks-dbus to Use storage-models (Tasks 1-15)

**Goal:** Make storage-models the single source of truth by having disks-dbus return these types directly.

---

## Phase 3A: Refactor disks-dbus to Use storage-models (Tasks 1-15)

**Goal:** Make storage-models the single source of truth by having disks-dbus return these types directly.

### Task 1: Analyze Current disks-dbus Models

**Scope:** Understand what data current disks-dbus types contain

**Files:**
- `disks-dbus/src/disks/drive/model.rs` (read)
- `disks-dbus/src/disks/volume.rs` (read)
- `disks-dbus/src/disks/volume_model/mod.rs` (read)

**Steps:**
1. Read DriveModel struct - identify all fields
2. Read VolumeNode struct - identify all fields
3. Read VolumeModel struct - identify all fields
4. Categorize fields:
   - Pure domain data (device path, size, type, etc.) → goes to storage-models
   - Connection/proxy handles → internal to disks-dbus
   - UI state (selection, etc.) → should move to disks-ui later
5. Create list of types needed in storage-models:
   - DiskInfo (from DriveModel domain fields)
   - VolumeInfo (from Volume Node domain fields)
   - PartitionInfo, FilesystemInfo, etc.
6. Document which disks-dbus methods need API changes

**Test Plan:**
- N/A (analysis task)

**Done When:**
- [x] Documented all fields in current types
- [x] Created list of needed storage-models types
- [x] Identified which methods return which types
- [x] Clear plan for refactoring
- **Status:** ✅ COMPLETE (see `.copi/specs/storage-service/models-refactor.md`)

---

### Task 2: Define DiskInfo in storage-models

**Scope:** Create canonical disk information type

**Files:**
- `storage-models/src/disk.rs` (create)
- `storage-models/src/lib.rs` (update)

**Steps:**
1. Create `storage-models/src/disk.rs`
2. Define `Disk Info` struct based on DriveModel analysis:
   - device: String (e.g., "/dev/sda")
   - model: String
   - serial: String
   - size: u64
   - connection_bus: String (e.g., "nvme", "usb", "ata")
   - removable: bool
   - ejectable: bool
   - rotation_rate: Option<u16>
   - (add any other fields from DriveModel that are domain data)
3. Define `SmartStatus` struct
4. Define `SmartAttribute` struct
5. Add `#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]`
6. Export from `storage-models/src/lib.rs`
7. Add documentation for all fields

**Test Plan:**
- Compile check
- Unit test JSON serialization round-trip

**Done When:**
- [x] DiskInfo defined with all necessary fields
- [x] Serde derives working
- [x] Exported from storage-models
- [x] Documentation complete
- [x] Additional types created: VolumeInfo, PartitionInfo, FilesystemInfo, LvmInfo, LuksInfo
- [x] Common types: ByteRange, Usage
- **Status:** ✅ COMPLETE (expanded beyond original scope - see implementation-log.md)

---

### Task 3: Define VolumeInfo and PartitionInfo in storage-models

**Scope:** Create canonical volume/partition types

**Files:**
- `storage-models/src/volume.rs` (create)
- `storage-models/src/partition.rs` (create)
- `storage-models/src/lib.rs` (update)

**Steps:**
1. Based on VolumeNode/VolumeModel analysis, create:
   - `storage-models/src/volume.rs` with `VolumeInfo` struct
   - Include: device, size, volume_type (enum), label, mount_points, etc.
2. Create `storage-models/src/partition.rs` with:
   - `PartitionInfo` struct
   - `PartitionTableInfo` struct
   - `PartitionTableType` enum (Gpt, Mbr)
3. Add serde derives to all
4. Export from lib.rs
5. Document all fields

**Test Plan:**
- Compile check
- JSON serialization tests

**Done When:**
- [x] VolumeInfo defined with domain data from VolumeNode
- [x] PartitionInfo defined
- [x] All types serializable
- [x] Exported and documented
- **Status:** ✅ COMPLETE (completed as part of Task 2 expansion)

---

### Task 4: Define FilesystemInfo and LvmInfo in storage-models

**Scope:** Create filesystem and LVM canonical types

**Files:**
- `storage-models/src/filesystem.rs` (create)
- `storage-models/src/lvm.rs` (create)
- `storage-models/src/lib.rs` (update)

**Steps:**
1. Create `Filesystem Info` struct (device, fs_type, label, uuid, mount_points, size, available)
2. Add `FormatOptions`, `MountOptions`, `CheckResult`, `UnmountResult` structs
3. Move ProcessInfo and KillResult from disks-dbus to storage-models:
   - Copy definitions
   - Add `#[derive(Serialize, Deserialize)]`
4. Create `VolumeGroupInfo`, `LogicalVolumeInfo`, `PhysicalVolumeInfo` for LVM
5. Add serde derives to all
6. Export from lib.rs
7. Document all types

**Test Plan:**
- Compile check
- JSON serialization tests

**Done When:**
- [x] FilesystemInfo and related types defined
- [x] ProcessInfo/KillResult moved to storage-models with serde
- [x] LVM types defined
- [x] All exported and documented
- **Status:** ✅ COMPLETE (completed as part of Task 2 expansion)

---

### Task 5: Define LuksInfo in storage-models

**Scope:** Create encryption types

**Files:**
- `storage-models/src/encryption.rs` (create)
- `storage-models/src/lib.rs` (update)

**Steps:**
1. Create `LuksInfo` struct
2. Add `LuksVersion` enum (Luks1, Luks2)
3. Add serde derives
4. Export from lib.rs
5. Document

**Test Plan:**
- Compile check
- JSON serialization

**Done When:**
- [x] LuksInfo defined
- [x] Exported and documented
- **Status:** ✅ COMPLETE (completed as part of Task 2 expansion)

---

### Task 6: Refactor DiskManager to Return DiskInfo

**Scope:** Update disks-dbus DiskManager to return storage-models types

**Files:**
- `disks-dbus/src/disks/manager.rs` (modify)
- `disks-dbus/src/disks/drive/model.rs` (modify or keep internal)
- `disks-dbus/Cargo.toml` (add storage-models dep)

**Steps:**
1. Add `storage-models` workspace dependency to disks-dbus/Cargo.toml
2. In manager.rs, change method signatures:
   - Before: Returns DriveModel or internal types
   - After: Returns `storage_models::DiskInfo`
3. Implementation options:
   a. Keep DriveModel internal, add conversion: `impl From<DriveModel> for storage_models::DiskInfo`
   b. Or remove DriveModel entirely, build DiskInfo directly from UDisks2
4. Update all methods that returned drive information
5. Ensure connection logic stays internal

**Test Plan:**
- Compile disks-dbus
- Existing tests still pass (may need updates)

**Done When:**
- [x] DiskManager methods return storage_models::DiskInfo
- [x] Internal DriveModel (if kept) converts to DiskInfo
- [x] All tests pass
- [x] No public API exposes DriveModel
- [x] Created `DriveModel::get_disks()` public API
- [x] Moved ByteRange and Usage to storage-models
- [x] Added `impl From<DriveModel> for DiskInfo`
- **Status:** ✅ COMPLETE (see implementation-log.md)

---

### Task 7: Refactor VolumeNode to Return VolumeInfo

**Scope:** Update Volume Node to return storage-models types

**Files:**
- `disks-dbus/src/disks/volume.rs` (modify)
- `disks-dbus/src/disks/volume_model/mod.rs` (modify or remove)

**Steps:**
1. Change VolumeNode methods to return `storage_models::VolumeInfo`
2. Options:
   a. Keep VolumeNode internal for tree structure, add conversion to VolumeInfo
   b. Or refactor to use VolumeInfo directly in tree
3. Update all public methods
4. Remove or privatize VolumeModel (if it's just UI state, it should move to disks-ui later)
5. Ensure tree traversal still works

**Test Plan:**
- Compile disks-dbus
- Test volume operations still work

**Done When:**
- [ ] VolumeNode methods return VolumeInfo
- [ ] Public API uses storage-models types
- [ ] Internal implementation still works
- [ ] Tests pass

---

### Task 8: Update Partition Operations to Use PartitionInfo

**Scope:** Refactor partition methods

**Files:**
- `disks-dbus/src/disks/ops.rs` (modify)
- Partition-related modules

**Steps:**
1. Change partition listing methods to return `Vec<storage_models::PartitionInfo>`
2. Update partition creation/deletion to work with PartitionInfo
3. Update any partition metadata methods

**Test Plan:**
- Partition operations work
- Return correct PartitionInfo

**Done When:**
- [ ] Partition methods use storage-models types
- [ ] Tests pass

---

### Task 9: Update Filesystem Operations to Use FilesystemInfo

**Scope:** Refactor filesystem methods

**Files:**
- `disks-dbus/src/disks/` filesystem-related modules

**Steps:**
1. Change filesystem listing to return `Vec<storage_models::FilesystemInfo>`
2. Update mount/unmount to work with FilesystemInfo
3. Update format operations
4. Ensure ProcessInfo/KillResult from storage-models used

**Test Plan:**
- Filesystem operations work
- mount/unmount return correct info

**Done When:**
- [ ] Filesystem methods use storage-models types
- [ ] ProcessInfo/KillResult from storage-models
- [ ] Tests pass

---

### Task 10: Update LVM Operations to Use LvmInfo

**Scope:** Refactor LVM methods

**Files:**
- `disks-dbus/src/disks/lvm.rs` (modify)

**Steps:**
1. Change LVM listing methods to return storage-models LVM types
2. Update VG/LV operations

**Test Plan:**
- LVM operations work (if available)

**Done When:**
- [ ] LVM methods use storage-models types
- [ ] Tests pass

---

### Task 11: Update LUKS Operations to Use LuksInfo

**Scope:** Refactor encryption methods

**Files:**
- Encryption-related modules in disks-dbus

**Steps:**
1. Change LUKS methods to return `storage_models::LuksInfo`
2. Update unlock/lock operations

**Test Plan:**
- LUKS operations work (if available)

**Done When:**
- [ ] Encryption methods use storage-models types
- [ ] Tests pass

---

### Task 12: Update disks-ui to Import from storage-models

**Scope:** Change UI to use storage-models types

**Files:**
- `disks-ui/src/` various files that import from disks-dbus
- `disks-ui/Cargo.toml` (ensure storage-models dep)

**Steps:**
1. Find all imports of disks-dbus types (grep for DriveModel, VolumeNode, etc.)
2. Change to import from storage-models instead
3. If UI has VolumeModel with app state, update it to wrap `storage_models::VolumeInfo`
4. Update all call sites to match new disks-dbus API

**Test Plan:**
- UI compiles
- Can still browse disks
- All operations work

**Done When:**
- [ ] UI uses storage-models types
- [ ] No direct imports of old disks-dbus model types
- [ ] UI compiles and runs
- [ ] All disk operations work

---

### Task 13: Clean Up disks-dbus Public API

**Scope:** Remove or privatize old model types

**Files:**
- `disks-dbus/src/disks/mod.rs` (update exports)
- Various disks-dbus modules

**Steps:**
1. Review what's still exported from disks-dbus
2. Remove exports of DriveModel, VolumeModel if not needed
3. Or mark as deprecated/internal
4. Ensure only storage-models types in public API
5. Update disks-dbus README if exists

**Test Plan:**
- disks-dbus compiles
- Only intended types are public

**Done When:**
- [ ] Old model types not in public API
- [ ] Public API uses only storage-models types
- [ ] Documentation updated

---

### Task 14: Integration Testing of Refactored disks-dbus

**Scope:** Verify refactored disks-dbus works end-to-end

**Files:**
- Test suites in disks-dbus and disks-ui

**Steps:**
1. Run all disks-dbus tests
2. Run UI in test environment
3. Test all major operations:
   - List disks
   - List partitions
   - Mount/unmount
   - Format (on test device)
4. Verify data in VolumeInfo matches what was in VolumeNode
5. Check for any regressions

**Test Plan:**
- All automated tests pass
- Manual testing shows no regressions

**Done When:**
- [ ] All disks-dbus tests pass
- [ ] UI functional testing complete
- [ ] No regressions identified
- [ ] Data integrity verified

---

### Task 15: Document Phase 3A Completion

**Scope:** Update docs and prepare for Phase 3B

**Files:**
- `storage-models/README.md` (create or update)
- `disks-dbus/README.md` (update)
- Implementation log

**Steps:**
1. Document storage-models public API
2. Add examples of each type
3. Update disks-dbus README to reflect new architecture
4. Document that disks-dbus returns storage-models types
5. Update implementation log with Phase 3A completion
6. Create summary of changes for review

**Test Plan:**
- Documentation is clear
- Examples compile

**Done When:**
- [ ] storage-models API documented
- [ ] disks-dbus architecture documented
- [ ] Implementation log updated
- [ ] Ready to start Phase 3B

---

## Phase 3B.1: Disk Discovery D-Bus Service (Tasks 16-23)

**Prerequisites:** Phase 3A complete (disks-dbus returns storage-models types)

**Goal:** Expose disk discovery and SMART operations via storage-service D-Bus interface

### Task 16: Integrate disks-dbus into storage-service

**Scope:** Add disks-dbus dependency and setup

**Files:**
- `storage-service/Cargo.toml` (update)
- `storage-service/src/lib.rs` (if needed)

**Steps:**
1. Add `disks-dbus` to storage-service dependencies
2. Add workspace dependency for `disks-dbus` in root Cargo.toml if not present
3. Verify disks-dbus re-exports needed types (DiskManager, DeviceEventStream, etc.)
4. Create helper module for converting disks-dbus types → storage-models types
5. Test that DiskManager can be instantiated from storage-service

**Test Plan:**
- Compile check
- Instantiate DiskManager in integration test
- Verify conversion helpers work

**Done When:**
- [ ] disks-dbus dependency added
- [ ] Can create DiskManager instance
- [ ] Conversion helpers defined
- [ ] No build errors

---

### Task 17: Implement Disk Listing Handler

**Scope:** Expose disk listing via D-Bus

**Files:**
- `storage-service/src/handlers/disks.rs` (new)
- `storage-service/src/handlers/mod.rs` (new)
- `storage-service/src/main.rs` (register handler)

**Steps:**
1. Create `storage-service/src/handlers/` directory
2. Create `disks.rs` with `DisksHandler` struct
3. Store DiskManager instance in handler (created in main.rs)
4. Add `list_disks()` async method with D-Bus signature
5. Use DiskManager to enumerate drives (existing functionality)
6. For each drive, use existing DriveModel methods
7. Convert DriveModel → `DiskInfo` from storage-models (use conversion helper)
8. Serialize Vec<DiskInfo> to JSON
9. Add `#[zbus::interface(name = "org.cosmic.ext.StorageService.Disks")]`
10. Register DisksHandler at `/org/cosmic/ext/StorageService/disks` in main.rs
11. Pass DiskManager reference to handler

**Test Plan:**
- `busctl call ... ListDisks` returns JSON array
- Verify all system disks appear in output
- Compare with direct disks-dbus usage (should be identical data)
- Test on system with NVMe, SATA, USB devices

**Done When:**
- [ ] ListDisks() D-Bus method callable
- [ ] Returns accurate disk information (same as disks-dbus)
- [ ] JSON format matches DiskInfo schema
- [ ] Works with multiple disk types

---

### Task 18: Implement Get Disk Info

**Scope:** Get detailed information for a specific disk

**Files:**
- `storage-service/src/handlers/disks.rs` (extend)

**Steps:**
1. Add `get_disk_info(device: String)` method to DisksHandler
2. Find UDisks2 drive object by device path
3. Return error if not found: `DeviceError`
4. Extract all properties (same as list_disks)
5. Serialize single DiskInfo to JSON
6. Add Polkit check: `disks-read` (allow_active)

**Test Plan:**
- `busctl call ... GetDiskInfo s "/dev/sda"` returns JSON object
- Invalid device returns error
- Non-root user can call without prompt

**Done When:**
- [ ] GetDiskInfo() method works for valid devices
- [ ] Returns detailed disk information
- [ ] Error handling for invalid device paths
- [ ] Polkit authorization works

---

### Task 19: Implement SMART Status

**Scope:** Get SMART health status for a disk

**Files:**
- `storage-service/src/handlers/disks.rs` (extend)
- `data/polkit-1/actions/org.cosmic.ext.storage-service.policy` (update)

**Steps:**
1. Add `get_smart_status(device: String)` method
2. Find UDisks2 drive object
3. Check if drive has `org.freedesktop.UDisks2.Drive.Ata` interface
4. Return `NotSupported` error if no SMART support
5. Get SmartSupported, SmartEnabled properties
6. Get SmartFailing property for health status
7. Get SmartTemperature, SmartPowerOnSeconds
8. Convert to `SmartStatus` from storage-models
9. Serialize to JSON
10. Add Polkit action: `smart-read` (allow_active)
11. Add Polkit check in method

**Test Plan:**
- Works on SATA/NVMe drives with SMART
- Returns NotSupported for USB drives
- Temperature and power-on hours accurate

**Done When:**
- [ ] GetSmartStatus() returns health data
- [ ] Handles devices without SMART support
- [ ] Temperature in Celsius, hours as u64
- [ ] Polkit policy created

---

### Task 20: Implement SMART Attributes

**Scope:** Get detailed SMART attribute list

**Files:**
- `storage-service/src/handlers/disks.rs` (extend)

**Steps:**
1. Add `get_smart_attributes(device: String)` method
2. Find UDisks2 drive with Ata interface
3. Call `SmartGetAttributes()` method on drive
4. Parse attribute array from D-Bus response
5. For each attribute, extract:
   - id, name, current_value, worst_value, threshold, raw_value, failing
6. Convert to Vec<SmartAttribute>
7. Serialize to JSON
8. Add Polkit check: `smart-read`

**Test Plan:**
- Returns 20-30 attributes for typical disk
- Attribute values match `smartctl -A` output
- Correctly identifies failing attributes

**Done When:**
- [ ] GetSmartAttributes() returns full attribute list
- [ ] Attribute values accurate
- [ ] Failing attributes flagged correctly
- [ ] JSON format matches SmartAttribute schema

---

### Task 21: Implement SMART Self-Test

**Scope:** Trigger SMART short/long/conveyance tests

**Files:**
- `storage-service/src/handlers/disks.rs` (extend)
- `data/polkit-1/actions/org.cosmic.ext.storage-service.policy` (update)

**Steps:**
1. Add `start_smart_test(device: String, test_type: String)` method
2. Validate test_type: "short", "long", "conveyance"
3. Return InvalidArgument for unknown test types
4. Check Polkit: `smart-test` (auth_admin_keep)
5. Find UDisks2 drive with Ata interface
6. Call `SmartSelftestStart(test_type)` method
7. Emit signal: `SmartTestStarted(device, test_type)`
8. Monitor test progress (UDisks2 property changes)
9. Emit signal when complete: `SmartTestCompleted(device, success)`
10. Add Polkit action: `smart-test` (auth_admin_keep)

**Test Plan:**
- Start short test, verify starts successfully
- Test progress updates via signals (if supported)
- Test completion signal emitted
- Requires admin authorization

**Done When:**
- [ ] StartSmartTest() triggers self-test
- [ ] Supports short, long, conveyance tests
- [ ] Signals emitted for start and completion
- [ ] Authorization required

---

### Task 22: Implement Hotplug Monitoring

**Scope:** Emit signals when disks added/removed

**Files:**
- `storage-service/src/handlers/disks.rs` (extend)
- `storage-service/src/main.rs` (spawn monitoring task)

**Steps:**
1. Add `DiskAdded` and `DiskRemoved` signals to DisksHandler interface
2. Create `monitor_hotplug` async function
3. Subscribe to UDisks2 `InterfacesAdded` signal
4. Filter for `org.freedesktop.UDisks2.Drive` interface
5. On new drive: extract DiskInfo, emit DiskAdded signal
6. Subscribe to UDisks2 `InterfacesRemoved` signal
7. On removed drive: emit DiskRemoved signal
8. Spawn monitoring task in main.rs

**Test Plan:**
- Insert USB drive, verify DiskAdded signal emitted
- Remove USB drive, verify DiskRemoved signal emitted
- Signals include accurate device path and info

**Done When:**
- [ ] DiskAdded signal emitted on hotplug
- [ ] DiskRemoved signal emitted on removal
- [ ] Works with USB and other hotpluggable devices
- [ ] No false positives (partition changes don't trigger)

---

### Task 23: Test Disk Discovery Integration

**Scope:** Verify all disk operations work end-to-end

**Files:**
- Test scripts

**Steps:**
1. List all disks, verify DiskInfo accuracy
2. Get info for specific disk
3. Check SMART status on capable disk
4. Get SMART attributes
5. Start SMART self-test (if safe on test system)
6. Test hotplug monitoring with USB drive
7. Verify all Polkit actions work

**Test Plan:**
- All disk operations return expected data
- SMART data matches smartctl output
- Hotplug events detected
- Authorization prompts appear

**Done When:**
- [ ] All disk operations tested
- [ ] No regressions from Phase 3A refactoring
- [ ] Ready for partition management (Phase 3B.2)

---

## Phase 3B.2: Partition Management (Tasks 24-33)

**Prerequisites:** Phase 3A complete (PartitionInfo types defined in storage-models)

**Goal:** Expose partition management operations via storage-service

### Task 24: Setup Partition Handler

**Scope:** Create partition operations handler skeleton

**Files:**
- `storage-service/src/handlers/partitions.rs` (new)
- `storage-service/src/main.rs` (register)

**Steps:**
1. Create `partitions.rs` with `PartitionsHandler` struct
2. Add `#[zbus::interface(name = "org.cosmic.ext.StorageService.Partitions")]`
3. Define signals: PartitionCreated, PartitionDeleted, PartitionModified
4. Register at `/org/cosmic/ext/StorageService/partitions`
5. Add skeleton methods (will implement in later tasks)

**Test Plan:**
- Service starts without errors
- Interface introspectable via busctl

**Done When:**
- [ ] Handler registered on D-Bus
- [ ] Introspection shows methods/signals
- [ ] Compiles clean

---

### Task 25: Implement List Partitions

**Scope:** List all partitions on a disk

**Files:**
- `storage-service/src/handlers/partitions.rs` (extend)
- `data/polkit-1/actions/` (update)

**Steps:**
1. Add `list_partitions(disk: String)` method
2. Query UDisks2 for all block devices
3. Filter for devices with `org.freedesktop.UDisks2.Partition` interface
4. Check each partition's `Table` property matches requested disk
5. Extract properties: Number, Type, Offset, Size, Flags, Name, UUID
6. Convert to Vec<PartitionInfo>
7. Serialize to JSON
8. Add Polkit check: `partitions-read` (allow_active)
9. Add Polkit action definition

**Test Plan:**
- List partitions for GPT disk
- List partitions for MBR disk
- Empty disk returns empty array
- Non-existent disk returns error

**Done When:**
- [ ] ListPartitions() returns accurate partition list
- [ ] Works with GPT and MBR
- [ ] Partition metadata correct (size, offset, type)
- [ ] No authorization prompt for reading

---

### Task 26: Implement Create Partition Table

**Scope:** Create new GPT or MBR partition table (destroys data)

**Files:**
- `storage-service/src/handlers/partitions.rs` (extend)

**Steps:**
1. Add `create_partition_table(disk: String, table_type: String)` method
2. Validate table_type: "gpt" or "dos" (MBR)
3. Check Polkit: `partitions-modify` (auth_admin_keep)
4. Find UDisks2 block device for disk
5. Call `Format("empty", {"erase": "zero", "partition-table-type": table_type})` method
6. Wait for operation completion
7. Emit signal: `PartitionTableCreated(disk, table_type)`
8. Add error handling for busy disk

**Test Plan:**
- Create GPT table on empty disk
- Create MBR table on empty disk
- Busy disk returns error
- Requires admin authorization

**Done When:**
- [ ] CreatePartitionTable() works for GPT and MBR
- [ ] Wipes existing partitions
- [ ] Signal emitted on success
- [ ] Authorization required

---

### Task 27: Implement Create Partition

**Scope:** Create new partition in available space

**Files:**
- `storage-service/src/handlers/partitions.rs` (extend)

**Steps:**
1. Add `create_partition(disk: String, offset: u64, size: u64, type_id: String)` method
2. Check Polkit: `partitions-modify`
3. Find UDisks2 block device with PartitionTable interface
4. Validate offset and size within disk bounds
5. Validate type_id (GPT GUID or MBR type code)
6. Call `CreatePartition(offset, size, type_id, "", options)` on PartitionTable
7. Parse returned partition object path
8. Extract new partition device path (e.g., /dev/sda1)
9. Emit signal: `PartitionCreated(disk, partition)`
10. Return partition device path

**Test Plan:**
- Create partition on GPT disk
- Create partition on MBR disk
- Invalid offset returns error
- Invalid type_id returns error

**Done When:**
- [ ] CreatePartition() creates partition successfully
- [ ] Returns new partition device path
- [ ] Partition appears in lsblk output
- [ ] Signal emitted

---

### Task 28: Implement Delete Partition

**Scope:** Delete existing partition

**Files:**
- `storage-service/src/handlers/partitions.rs` (extend)

**Steps:**
1. Add `delete_partition(partition: String)` method
2. Check Polkit: `partitions-modify`
3. Validate partition exists
4. Check partition is not mounted (return ResourceBusy error)
5. Find UDisks2 partition object
6. Get parent disk device
7. Call `Delete({})` on Partition interface
8. Emit signal: `PartitionDeleted(parent_disk, partition)`

**Test Plan:**
- Delete unmounted partition
- Mounted partition returns error
- Partition disappears from lsblk
- Signal emitted

**Done When:**
- [ ] DeletePartition() removes partition
- [ ] Mounted partitions protected
- [ ] Parent disk updated
- [ ] Signal emitted

---

### Task 29: Implement Resize Partition

**Scope:** Grow or shrink existing partition

**Files:**
- `storage-service/src/handlers/partitions.rs` (extend)

**Steps:**
1. Add `resize_partition(partition: String, new_size: u64)` method
2. Check Polkit: `partitions-modify`
3. Validate partition exists
4. Get current size and offset
5. Validate new_size fits available space
6. Check partition is not mounted (or allow if filesystem supports online resize)
7. Find UDisks2 partition object
8. Call `Resize(new_size, {})` on Partition interface
9. Emit signal: `PartitionModified(partition)`

**Test Plan:**
- Grow partition (increase size)
- Shrink partition (decrease size)
- Invalid size returns error
- Works when space available

**Done When:**
- [ ] ResizePartition() changes partition size
- [ ] Validates available space
- [ ] Handles mounted partitions appropriately
- [ ] Signal emitted

---

### Task 30: Implement Set Partition Type

**Scope:** Change partition type (GPT GUID or MBR code)

**Files:**
- `storage-service/src/handlers/partitions.rs` (extend)

**Steps:**
1. Add `set_partition_type(partition: String, type_id: String)` method
2. Check Polkit: `partitions-modify`
3. Validate type_id format (GUID for GPT, hex for MBR)
4. Find UDisks2 partition object
5. Call `SetType(type_id, {})` on Partition interface
6. Emit signal: `PartitionModified(partition)`

**Test Plan:**
- Change GPT partition type (e.g., Linux → EFI System)
- Change MBR partition type (e.g., Linux → FAT32)
- Invalid type_id returns error

**Done When:**
- [ ] SetPartitionType() changes partition type
- [ ] Works with GPT and MBR
- [ ] Validation for type_id format
- [ ] Signal emitted

---

### Task 31: Implement Set Partition Flags

**Scope:** Set partition flags (bootable, hidden, etc.)

**Files:**
- `storage-service/src/handlers/partitions.rs` (extend)

**Steps:**
1. Add `set_partition_flags(partition: String, flags: u64)` method
2. Check Polkit: `partitions-modify`
3. Find UDisks2 partition object
4. Call `SetFlags(flags, {})` on Partition interface
5. Emit signal: `PartitionModified(partition)`
6. Document flag values in method docs (0x01 = bootable, etc.)

**Test Plan:**
- Set bootable flag on MBR partition
- Clear bootable flag
- Multiple flags at once

**Done When:**
- [ ] SetPartitionFlags() changes flags
- [ ] Bootable flag works for MBR
- [ ] Documentation for flag values
- [ ] Signal emitted

---

### Task 32: Implement Set Partition Name

**Scope:** Set GPT partition name

**Files:**
- `storage-service/src/handlers/partitions.rs` (extend)

**Steps:**
1. Add `set_partition_name(partition: String, name: String)` method
2. Check Polkit: `partitions-modify`
3. Validate name length (GPT allows 36 characters)
4. Find UDisks2 partition object
5. Check table type is GPT (MBR doesn't support names)
6. Call `SetName(name, {})` on Partition interface
7. Emit signal: `PartitionModified(partition)`

**Test Plan:**
- Set name on GPT partition
- MBR partition returns NotSupported error
- Name appears in `lsblk -o NAME,PARTLABEL`

**Done When:**
- [ ] SetPartitionName() works for GPT
- [ ] MBR returns appropriate error
- [ ] Name length validation
- [ ] Signal emitted

---

### Task 33: Test Partition Management Integration

**Scope:** Verify all partition operations work end-to-end

**Files:**
- Test scripts

**Steps:**
1. List partitions on test disk
2. Create partition table (GPT and MBR)
3. Create partitions with various sizes
4. Resize partition
5. Change partition type and flags
6. Set partition name (GPT)
7. Delete partition
8. Verify all Polkit actions work

**Test Plan:**
- All partition operations successful
- No data corruption
- Authorization prompts appear

**Done When:**
- [ ] All partition operations tested
- [ ] Ready for filesystem operations (Phase 3B.3)

---

## Phase 3B.3: Filesystem Operations (Tasks 34-50)

**Prerequisites:** Phase 3A complete (FilesystemInfo types defined in storage-models)

**Goal:** Expose filesystem operations including process killing for busy unmount

### Task 34: Setup Filesystem Handler

**Scope:** Create filesystem operations handler

**Files:**
- `storage-service/src/handlers/filesystems.rs` (new)
- `storage-service/src/main.rs` (register)

**Steps:**
1. Create `filesystems.rs` with `FilesystemsHandler` struct
2. Add `#[zbus::interface(name = "org.cosmic.ext.StorageService.Filesystems")]`
3. Define signals: FormatProgress, Formatted, Mounted, Unmounted
4. Register at `/org/cosmic/ext/StorageService/filesystems`

**Test Plan:**
- Interface introspectable

**Done When:**
- [ ] Handler registered
- [ ] Introspection works
- [ ] Compiles

---

### Task 35: Implement List Filesystems

**Scope:** List all filesystems on system

**Files:**
- `storage-service/src/handlers/filesystems.rs` (extend)
- `data/polkit-1/actions/` (update)

**Steps:**
1. Add `list_filesystems()` method
2. Query UDisks2 for all block devices with Filesystem interface
3. Extract properties: Device, IdType, IdLabel, IdUUID, MountPoints
4. Get Size and Available from filesystem stats (if mounted)
5. Convert to Vec<FilesystemInfo>
6. Serialize to JSON
7. Add Polkit check: `filesystems-read` (allow_active)
8. Add Polkit action definition

**Test Plan:**
- Lists all mounted and unmounted filesystems
- Size/available accurate for mounted
- Multiple filesystem types

**Done When:**
- [ ] ListFilesystems() returns all filesystems
- [ ] Metadata accurate
- [ ] No auth prompt

---

### Task 36: Detect Available Filesystem Tools

**Scope:** Detect which mkfs/fsck tools are installed

**Files:**
- `storage-service/src/handlers/filesystems.rs` (extend)

**Steps:**
1. Add `detect_filesystem_support()` function
2. Check for executables: mkfs.ext4, mkfs.xfs, mkfs.btrfs, mkfs.vfat, mkfs.ntfs, mkfs.exfat
3. Store supported filesystems in handler state
4. Add `get_supported_filesystems()` D-Bus method
5. Return list of supported filesystem types

**Test Plan:**
- Returns ext4, xfs on typical system
- Missing tool doesn't panic
- Can query which types supported

**Done When:**
- [ ] Detects installed tools
- [ ] GetSupportedFilesystems() method works
- [ ] State cached to avoid repeated checks

---

### Task 37: Implement Format Operation

**Scope:** Format partition with filesystem

**Files:**
- `storage-service/src/handlers/filesystems.rs` (extend)
- `data/polkit-1/actions/` (update)

**Steps:**
1. Add `format(device: String, fs_type: String, label: String, options_json: String)` method
2. Check Polkit: `filesystems-format` (auth_admin)
3. Validate fs_type is supported
4. Check device is not mounted
5. Parse options_json into FormatOptions
6. Find UDisks2 block device
7. Build format options dict based on fs_type
8. Call `Format(fs_type, options)` on Block interface
9. Monitor format progress (if available)
10. Emit FormatProgress signals (0-100%)
11. Emit Formatted signal on completion
12. Add Polkit action: `filesystems-format` (auth_admin)

**Test Plan:**
- Format as ext4
- Format as xfs
- Format as fat32
- Mounted device returns error
- Unsupported fs_type returns error
- Progress signals emitted

**Done When:**
- [ ] Format() creates filesystem
- [ ] Supports ext4, xfs, btrfs, fat32
- [ ] Progress reporting works
- [ ] Authorization required (always prompt)

---

### Task 38: Implement Mount Operation

**Scope:** Mount filesystem to path

**Files:**
- `storage-service/src/handlers/filesystems.rs` (extend)
- `data/polkit-1/actions/` (update)

**Steps:**
1. Add `mount(device: String, mount_point: String, options_json: String)` method
2. Check Polkit: `filesystems-mount` (allow_active for removable, auth otherwise)
3. Parse options_json into MountOptions
4. Find UDisks2 filesystem object
5. Build options dict (ro, noexec, nosuid, etc.)
6. Call `Mount(options)` on Filesystem interface
7. UDisks2 returns actual mount point used
8. Emit Mounted signal
9. Return mount point
10. Add Polkit action: `filesystems-mount`

**Test Plan:**
- Mount ext4 partition to /mnt/test
- Mount with read-only option
- Already mounted returns mount point
- Invalid device returns error

**Done When:**
- [ ] Mount() mounts filesystem
- [ ] Mount options respected
- [ ] Returns actual mount point
- [ ] Signal emitted

---

### Task 39: Implement Unmount Operation (with Process Detection)

**Scope:** Unmount filesystem with process discovery

**Files:**
- `storage-service/src/handlers/filesystems.rs` (extend)

**Steps:**
1. Add `unmount(device_or_mount: String, force: bool, kill_processes: bool)` method
2. Check Polkit: `filesystems-mount` (or `filesystems-kill-processes` if kill_processes=true)
3. Get mount point from device (if device path provided)
4. Wrap VolumeNode unmount operation from disks-dbus
5. If unmount fails with EBUSY:
   a. Call `find_processes_using_mount()` from disks-dbus
   b. If kill_processes=true:
      - Check Polkit: `filesystems-kill-processes`
      - Call `kill_processes()` from disks-dbus
      - Retry unmount
   c. If kill_processes=false:
      - Return UnmountResult with success=false and blocking_processes list
6. Emit Unmounted signal on success
7. Handle force unmount separately (lazy unmount)

**Test Plan:**
- Unmount idle filesystem
- Busy filesystem returns error with process list (kill_processes=false)
- Busy filesystem auto-kills processes and unmounts (kill_processes=true)
- Force unmount works when filesystem in use
- Requires auth for killing processes

**Done When:**
- [ ] Unmount() unmounts filesystem
- [ ] Returns blocking process list on EBUSY
- [ ] kill_processes parameter works
- [ ] Force option works
- [ ] Signal emitted
- [ ] Polkit auth for killing processes

---

### Task 40: Implement Get Blocking Processes

**Scope:** Query which processes are blocking unmount

**Files:**
- `storage-service/src/handlers/filesystems.rs` (extend)

**Steps:**
1. Add `get_blocking_processes(device_or_mount: String)` method
2. Check Polkit: `filesystems-read`
3. Get mount point from device (if device path provided)
4. Call `find_processes_using_mount()` from disks-dbus
5. Convert Vec<ProcessInfo> to JSON
6. Return JSON string

**Test Plan:**
- Mount filesystem, cd into it, call method (should return shell process)
- Idle filesystem returns empty array
- Invalid mount point returns error

**Done When:**
- [ ] GetBlockingProcesses() returns process list
- [ ] Works for mounted filesystems
- [ ] Empty array for idle mounts
- [ ] No auth prompt for reading

---

### Task 41: Implement Kill Processes

**Scope:** Kill processes by PID list

**Files:**
- `storage-service/src/handlers/filesystems.rs` (extend)
- `data/polkit-1/actions/` (update)

**Steps:**
1. Add `kill_processes(pids: Vec<i32>)` method
2. Check Polkit: `filesystems-kill-processes` (auth_admin_keep)
3. Call `kill_processes()` from disks-dbus
4. Convert Vec<KillResult> to JSON
5. Return JSON string
6. Add Polkit action: `filesystems-kill-processes`

**Test Plan:**
- Kill single process
- Kill multiple processes
- Invalid PID returns error in KillResult
- Requires admin authorization

**Done When:**
- [ ] KillProcesses() kills processes
- [ ] Returns results for each PID
- [ ] Invalid PIDs handled gracefully
- [ ] Authorization required
- [ ] Polkit action defined

---

### Task 42: Implement Filesystem Check

**Scope:** Check and repair filesystem (fsck)

**Files:**
- `storage-service/src/handlers/filesystems.rs` (extend)

**Steps:**
1. Add `check(device: String, repair: bool)` method
2. Check Polkit: `filesystems-modify`
3. Check device is not mounted
4. Detect filesystem type
5. For ext4: Call `Check({"fsck-flags": "-f" or "-fy"})` on Block
6. Parse fsck output for errors
7. Create CheckResult with results
8. Serialize to JSON
9. Handle different fsck return codes (0=clean, 1=corrected, 4=uncorrected)

**Test Plan:**
- Check clean filesystem
- Check filesystem with errors (repair=true)
- Mounted filesystem returns error

**Done When:**
- [ ] Check() runs fsck
- [ ] Repair option works
- [ ] CheckResult shows errors found/fixed
- [ ] Works for ext4, xfs

---

### Task 43: Implement Set Label

**Scope:** Change filesystem label

**Files:**
- `storage-service/src/handlers/filesystems.rs` (extend)

**Steps:**
1. Add `set_label(device: String, label: String)` method
2. Check Polkit: `partitions-modify`
3. Validate label length for filesystem type
4. Find UDisks2 filesystem object
5. Call `SetLabel(label, {})` on Filesystem interface
6. Some filesystems require unmount, handle appropriately

**Test Plan:**
- Set label on ext4
- Set label on xfs
- Label appears in lsblk -o LABEL
- Mounted ext4 works, mounted xfs may require unmount

**Done When:**
- [ ] SetLabel() changes label
- [ ] Works for common filesystems
- [ ] Label length validation
- [ ] Handles mount state

---

### Task 44: Implement Get Usage

**Scope:** Get filesystem usage statistics

**Files:**
- `storage-service/src/handlers/filesystems.rs` (extend)

**Steps:**
1. Add `get_usage(mount_point: String)` method
2. Check Polkit: `filesystems-read`
3. Validate mount_point is actually mounted
4. Use statvfs or similar to get filesystem stats
5. Create FilesystemUsage with size, used, available, percent
6. Serialize to JSON
7. For BTRFS, use existing btrfs usage logic

**Test Plan:**
- Get usage for ext4 mount
- Get usage for BTRFS mount
- Unmounted path returns error
- Percent calculation correct

**Done When:**
- [ ] GetUsage() returns accurate statistics
- [ ] Works for all filesystem types
- [ ] BTRFS shows actual used (not apparent)
- [ ] Unmounted returns error

---

### Task 45: Test Filesystem Operations Integration

**Scope:** Verify all filesystem operations work end-to-end

**Files:**
- Test scripts

**Steps:**
1. List all filesystems
2. Format test partition with ext4
3. Mount formatted filesystem
4. Create files, test using mount (create blocking processes)
5. Get blocking processes list
6. Kill blocking processes
7. Unmount successfully
8. Check filesystem
9. Set filesystem label
10. Get usage statistics
11. Verify all Polkit actions work

**Test Plan:**
- All filesystem operations successful
- Process killing works correctly
- Authorization prompts appear appropriately

**Done When:**
- [ ] All filesystem operations tested
- [ ] Process killing integration verified
- [ ] Ready for LVM operations (Phase 3B.4)

---

## Phase 3B.4: LVM Operations (Tasks 46-54)

**Prerequisites:** Phase 3A complete (LVM types defined in storage-models)

**Goal:** Expose LVM management operations

### Task 46: Setup LVM Handler

**Scope:** Create LVM operations handler

**Files:**
- `storage-service/src/handlers/lvm.rs` (new)
- `storage-service/src/main.rs` (register)
- `data/polkit-1/actions/` (update)

**Steps:**
1. Create `lvm.rs` with `LvmHandler` struct
2. Add `#[zbus::interface(name = "org.cosmic.ext.StorageService.Lvm")]`
3. Define signals: VolumeGroupCreated, LogicalVolumeCreated, LogicalVolumeResized
4. Register at `/org/cosmic/ext/StorageService/lvm`
5. Add Polkit actions: `lvm-read`, `lvm-modify`

**Test Plan:**
- Interface introspectable

**Done When:**
- [ ] Handler registered
- [ ] Polkit actions defined
- [ ] Compiles

---

### Task 47: Implement List Volume Groups

**Scope:** List all LVM volume groups

**Files:**
- `storage-service/src/handlers/lvm.rs` (extend)

**Steps:**
1. Add `list_volume_groups()` method
2. Check Polkit: `lvm-read`
3. Query UDisks2 for objects with LVM2 VolumeGroup interface
4. Extract properties: Name, UUID, Size, FreeSize
5. Count PVs and LVs in VG
6. Convert to Vec<VolumeGroupInfo>
7. Serialize to JSON

**Test Plan:**
- Lists all VGs on system
- Metadata accurate (size, free)
- Empty system returns empty array

**Done When:**
- [ ] ListVolumeGroups() works
- [ ] Accurate VG information
- [ ] No auth prompt

---

### Task 48: Implement List Logical Volumes

**Scope:** List logical volumes in a volume group

**Files:**
- `storage-service/src/handlers/lvm.rs` (extend)

**Steps:**
1. Add `list_logical_volumes(vg_name: String)` method
2. Check Polkit: `lvm-read`
3. Query UDisks2 for LVM2 LogicalVolume objects in specified VG
4. Extract properties: Name, UUID, Size, Active, BlockDevice
5. Convert to Vec<LogicalVolumeInfo>
6. Serialize to JSON

**Test Plan:**
- Lists all LVs in VG
- Active state correct
- Device path correct (/dev/vg/lv)

**Done When:**
- [ ] ListLogicalVolumes() works for a VG
- [ ] Accurate LV information
- [ ] Device paths correct

---

### Task 49: Implement List Physical Volumes

**Scope:** List physical volumes (PVs)

**Files:**
- `storage-service/src/handlers/lvm.rs` (extend)

**Steps:**
1. Add `list_physical_volumes()` method
2. Check Polkit: `lvm-read`
3. Query UDisks2 for Block devices with LVM2 PhysicalVolume interface
4. Extract properties: Device, VolumeGroup, Size, FreeSize
5. Convert to Vec<PhysicalVolumeInfo>
6. Serialize to JSON

**Test Plan:**
- Lists all PVs
- Shows which VG each belongs to
- Unassigned PVs show None for vg_name

**Done When:**
- [ ] ListPhysicalVolumes() works
- [ ] Shows VG membership
- [ ] Size information accurate

---

### Task 50: Implement Create Volume Group

**Scope:** Create new LVM volume group

**Files:**
- `storage-service/src/handlers/lvm.rs` (extend)

**Steps:**
1. Add `create_volume_group(name: String, devices: Vec<String>)` method
2. Check Polkit: `lvm-modify`
3. Validate name (alphanumeric, no special chars)
4. Validate all devices exist and are not in use
5. Find UDisks2 Manager object
6. Call `VolumeGroupCreate(name, devices, options)` method
7. Emit signal: VolumeGroupCreated(name)

**Test Plan:**
- Create VG from single device
- Create VG from multiple devices
- Duplicate name returns error
- Used device returns error

**Done When:**
- [ ] CreateVolumeGroup() creates VG
- [ ] Works with multiple devices
- [ ] Validation works
- [ ] Signal emitted

---

### Task 51: Implement Create Logical Volume

**Scope:** Create new logical volume in VG

**Files:**
- `storage-service/src/handlers/lvm.rs` (extend)

**Steps:**
1. Add `create_logical_volume(vg_name: String, lv_name: String, size: u64)` method
2. Check Polkit: `lvm-modify`
3. Validate lv_name (alphanumeric)
4. Validate size <= VG free space
5. Find UDisks2 VolumeGroup object
6. Call `CreatePlainVolume(lv_name, size, options)` method
7. Get returned LV object path
8. Emit signal: LogicalVolumeCreated(vg_name, lv_name)
9. Return device path (/dev/vg_name/lv_name)

**Test Plan:**
- Create LV with specific size
- Size larger than VG returns error
- Duplicate name returns error
- LV appears in lvdisplay

**Done When:**
- [ ] CreateLogicalVolume() creates LV
- [ ] Size validation works
- [ ] Device path returned
- [ ] Signal emitted

---

### Task 52: Implement Resize Logical Volume

**Scope:** Grow or shrink logical volume

**Files:**
- `storage-service/src/handlers/lvm.rs` (extend)

**Steps:**
1. Add `resize_logical_volume(vg_name: String, lv_name: String, new_size: u64)` method
2. Check Polkit: `lvm-modify`
3. Find UDisks2 LogicalVolume object
4. Get current size
5. Validate new_size (grow: enough VG space, shrink: warn data loss risk)
6. Call `Resize(new_size, options)` method
7. Emit signal: LogicalVolumeResized(vg_name, lv_name, new_size)
8. Note: filesystem resize is separate operation

**Test Plan:**
- Grow LV
- Shrink LV (careful!)
- New size reflects in lvdisplay

**Done When:**
- [ ] ResizeLogicalVolume() changes LV size
- [ ] Validation works
- [ ] Signal emitted
- [ ] Documentation warns about filesystem resize

---

### Task 53: Implement Delete LVM Operations

**Scope:** Delete LVs and VGs

**Files:**
- `storage-service/src/handlers/lvm.rs` (extend)

**Steps:**
1. Add `delete_logical_volume(vg_name: String, lv_name: String)` method
2. Check Polkit: `lvm-modify`
3. Check LV is not mounted/active
4. Find UDisks2 LogicalVolume object
5. Call `Delete(options)` method
6. Emit signal: LogicalVolumeDeleted(vg_name, lv_name)
7. Add `delete_volume_group(vg_name: String)` method
8. Check VG has no LVs (or force delete all)
9. Call `Delete(options)` on VolumeGroup
10. Emit signal: VolumeGroupDeleted(vg_name)

**Test Plan:**
- Delete LV
- Delete empty VG
- VG with LVs returns error (or deletes all if forced)
- Active LV returns error

**Done When:**
- [ ] DeleteLogicalVolume() removes LV
- [ ] DeleteVolumeGroup() removes VG
- [ ] Safety checks work
- [ ] Signals emitted

---

### Task 54: Test LVM Integration

**Scope:** Verify all LVM operations work end-to-end

**Files:**
- Test scripts

**Steps:**
1. List volume groups, logical volumes, physical volumes
2. Create volume group from test device
3. Create logical volume in VG
4. Resize logical volume
5. Delete logical volume
6. Delete volume group
7. Verify all Polkit actions work

**Test Plan:**
- All LVM operations successful
- Size calculations correct
- Authorization prompts appear

**Done When:**
- [ ] All LVM operations tested
- [ ] Ready for encryption support (Phase 3B.5)

---

## Phase 3B.5: Encryption Support (Tasks 55-62)

**Prerequisites:** Phase 3A complete (LUKS types defined in storage-models)

**Goal:** Expose LUKS encryption operations

### Task 55: Setup Encryption Handler

**Scope:** Create encryption operations handler

**Files:**
- `storage-service/src/handlers/encryption.rs` (new)
- `storage-service/src/main.rs` (register)
- `data/polkit-1/actions/` (update)

**Steps:**
1. Create `encryption.rs` with `EncryptionHandler` struct
2. Add `#[zbus::interface(name = "org.cosmic.ext.StorageService.Encryption")]`
3. Define signals: LuksFormatted, LuksUnlocked, LuksLocked
4. Register at `/org/cosmic/ext/StorageService/encryption`
5. Add Polkit actions: `encryption-read`, `encryption-modify`, `encryption-unlock`

**Test Plan:**
- Interface introspectable

**Done When:**
- [ ] Handler registered
- [ ] Polkit actions defined
- [ ] Compiles

---

### Task 56: Implement Format LUKS

**Scope:** Format partition with LUKS encryption

**Files:**
- `storage-service/src/handlers/encryption.rs` (extend)

**Steps:**
1. Add `format_luks(device: String, passphrase: String, luks_version: String, cipher: String)` method
2. Check Polkit: `encryption-modify`
3. Validate luks_version: "luks1" or "luks2"
4. Validate device is not mounted
5. Find UDisks2 block device
6. Call `Format("empty", {"encrypt.passphrase": passphrase, "encrypt.type": luks_version, ...})`
7. Wait for format completion
8. Emit signal: LuksFormatted(device)

**Test Plan:**
- Format as LUKS2
- Format as LUKS1
- Mounted device returns error
- Device encrypted (check with cryptsetup luksDump)

**Done When:**
- [ ] FormatLuks() creates LUKS volume
- [ ] Supports LUKS1 and LUKS2
- [ ] Passphrase required
- [ ] Signal emitted

---

### Task 57: Implement Get LUKS Info

**Scope:** Get information about LUKS volume

**Files:**
- `storage-service/src/handlers/encryption.rs` (extend)

**Steps:**
1. Add `get_luks_info(device: String)` method
2. Check Polkit: `encryption-read`
3. Find UDisks2 block device with Encrypted interface
4. Extract properties: Version, CipherType, KeySize
5. Check if unlocked (CleartextDevice property)
6. Create LuksInfo
7. Serialize to JSON

**Test Plan:**
- Get info for LUKS2 volume
- Shows correct version, cipher
- Unlocked state accurate

**Done When:**
- [ ] GetLuksInfo() returns LUKS metadata
- [ ] Works for LUKS1 and LUKS2
- [ ] Unlocked state correct
- [ ] No auth prompt

---

### Task 58: Implement Unlock LUKS

**Scope:** Unlock LUKS volume with passphrase

**Files:**
- `storage-service/src/handlers/encryption.rs` (extend)

**Steps:**
1. Add `unlock(device: String, passphrase: String)` method
2. Check Polkit: `encryption-unlock` (allow_active)
3. Find UDisks2 Encrypted object
4. Call `Unlock(passphrase, options)` method
5. Get returned cleartext device path (/dev/mapper/luks-...)
6. Emit signal: LuksUnlocked(device, cleartext_device)
7. Return cleartext device path

**Test Plan:**
- Unlock with correct passphrase
- Wrong passphrase returns error
- Cleartext device appears
- Signal emitted

**Done When:**
- [ ] Unlock() unlocks LUKS volume
- [ ] Returns cleartext device
- [ ] Wrong passphrase handled
- [ ] Signal emitted

---

### Task 59: Implement Lock LUKS

**Scope:** Lock (close) LUKS volume

**Files:**
- `storage-service/src/handlers/encryption.rs` (extend)

**Steps:**
1. Add `lock(cleartext_device: String)` method
2. Check Polkit: `encryption-unlock`
3. Find UDisks2 Encrypted object by cleartext device
4. Check cleartext device is not mounted
5. Call `Lock(options)` method
6. Emit signal: LuksLocked(cleartext_device)

**Test Plan:**
- Lock unlocked volume
- Mounted cleartext returns error
- Cleartext device disappears

**Done When:**
- [ ] Lock() locks LUKS volume
- [ ] Mounted check works
- [ ] Signal emitted

---

### Task 60: Implement Change Passphrase

**Scope:** Change LUKS passphrase

**Files:**
- `storage-service/src/handlers/encryption.rs` (extend)

**Steps:**
1. Add `change_passphrase(device: String, old_passphrase: String, new_passphrase: String)` method
2. Check Polkit: `encryption-modify`
3. Find UDisks2 Encrypted object
4. Call `ChangePassphrase(old_passphrase, new_passphrase, options)` method
5. Handle wrong old passphrase error

**Test Plan:**
- Change passphrase successfully
- Old passphrase wrong returns error
- New passphrase works to unlock

**Done When:**
- [ ] ChangePassphrase() changes LUKS passphrase
- [ ] Old passphrase validated
- [ ] New passphrase effective immediately

---

### Task 61: Implement Keyslot Management

**Scope:** Add/remove LUKS keyslots

**Files:**
- `storage-service/src/handlers/encryption.rs` (extend)

**Steps:**
1. Add `add_keyslot(device: String, existing_passphrase: String, new_passphrase: String)` method
2. Check Polkit: `encryption-modify`
3. Use cryptsetup luksAddKey (may need to call via UDisks2 or direct command)
4. Add `remove_keyslot(device: String, passphrase: String, slot: u8)` method
5. Use cryptsetup luksKillSlot
6. Handle keyslot full error
7. Handle last keyslot protection (don't remove last keyslot)

**Test Plan:**
- Add second keyslot
- Remove keyslot
- Can't remove last keyslot
- Multiple passphrases work to unlock

**Done When:**
- [ ] AddKeyslot() adds new passphrase
- [ ] RemoveKeyslot() removes passphrase
- [ ] Last keyslot protected
- [ ] Keyslot count accurate

---

### Task 62: Test Encryption Integration

**Scope:** Verify all encryption operations work end-to-end

**Files:**
- Test scripts

**Steps:**
1. Format partition with LUKS2
2. Get LUKS info
3. Unlock with passphrase
4. Lock LUKS volume
5. Change passphrase
6. Add and remove keyslots
7. Verify all Polkit actions work

**Test Plan:**
- All encryption operations successful
- Multiple keyslots work
- Authorization prompts appear

**Done When:**
- [ ] All encryption operations tested
- [ ] Ready for client implementation (Phase 3B.6)

---

## Phase 3B.6: D-Bus Client Wrappers (Tasks 63-67)

**Goal:** Create client wrappers for UI to communicate with service

### Task 63: Create Disks Client

**Scope:** D-Bus client wrapper for disk operations

**Files:**
- `disks-ui/src/client/disks.rs` (new)
- `disks-ui/src/client/mod.rs` (update)

**Steps:**
1. Create `disks.rs` with `DisksClient` struct
2. Add `#[proxy]` annotated trait for DisksInterface
3. Implement async methods: list_disks, get_disk_info, get_smart_status, get_smart_attributes, start_smart_test
4. Deserialize JSON responses to storage-models types
5. Add signal subscriptions: disk_added, disk_removed, smart_test_completed
6. Export from mod.rs

**Test Plan:**
- Call list_disks, verify returns Vec<DiskInfo>
- Subscribe to disk_added signal

**Done When:**
- [ ] DisksClient mirrors service interface
- [ ] All methods working
- [ ] Signal subscriptions work
- [ ] Error handling

---

### Task 64: Create Partitions Client

**Scope:** D-Bus client for partition operations

**Files:**
- `disks-ui/src/client/partitions.rs` (new)
- `disks-ui/src/client/mod.rs` (update)

**Steps:**
1. Create `partitions.rs` with `PartitionsClient` struct
2. Add proxy trait for PartitionsInterface
3. Implement async methods: list_partitions, create_partition_table, create_partition, delete_partition, resize_partition, set_partition_type, set_partition_flags, set_partition_name
4. Deserialize JSON responses
5. Add signal subscriptions
6. Export from mod.rs

**Test Plan:**
- Call list_partitions, verify returns Vec<PartitionInfo>
- Create partition, verify succeeds

**Done When:**
- [ ] PartitionsClient complete
- [ ] All methods working
- [ ] Type safety with storage-models

---

### Task 65: Create Filesystems Client

**Scope:** D-Bus client for filesystem operations

**Files:**
- `disks-ui/src/client/filesystems.rs` (new)
- `disks-ui/src/client/mod.rs` (update)

**Steps:**
1. Create `filesystems.rs` with `FilesystemsClient` struct
2. Add proxy trait
3. Implement async methods: list_filesystems, format, mount, unmount, check, set_label, get_usage
4. Deserialize JSON responses
5. Add signal subscriptions (especially format_progress)
6. Export from mod.rs

**Test Plan:**
- Call format, verify progress signals
- Call mount, verify filesystem mounted

**Done When:**
- [ ] FilesystemsClient complete
- [ ] Progress signals handled
- [ ] All methods working

---

### Task 66: Create LVM Client

**Scope:** D-Bus client for LVM operations

**Files:**
- `disks-ui/src/client/lvm.rs` (new)
- `disks-ui/src/client/mod.rs` (update)

**Steps:**
1. Create `lvm.rs` with `LvmClient` struct
2. Add proxy trait
3. Implement async methods: list_volume_groups, list_logical_volumes, list_physical_volumes, create_volume_group, delete_volume_group, create_logical_volume, delete_logical_volume, resize_logical_volume
4. Deserialize JSON responses
5. Add signal subscriptions
6. Export from mod.rs

**Test Plan:**
- Call list_volume_groups
- Create LV, verify appears

**Done When:**
- [ ] LvmClient complete
- [ ] All methods working

---

### Task 67: Create Encryption Client

**Scope:** D-Bus client for encryption operations

**Files:**
- `disks-ui/src/client/encryption.rs` (new)
- `disks-ui/src/client/mod.rs` (update)

**Steps:**
1. Create `encryption.rs` with `EncryptionClient` struct
2. Add proxy trait
3. Implement async methods: get_luks_info, format_luks, unlock, lock, change_passphrase, add_keyslot, remove_keyslot
4. Deserialize JSON responses
5. Add signal subscriptions
6. Export from mod.rs
7. Handle passphrase prompts (emit request signal → UI shows dialog → return passphrase)

**Test Plan:**
- Call unlock with passphrase
- Format LUKS volume

**Done When:**
- [ ] EncryptionClient complete
- [ ] Passphrase flow designed
- [ ] All methods working

---

## Integration & Testing (Tasks 68-72)

### Task 68: Integration Test Suite

**Scope:** Automated integration tests

**Files:**
- `storage-service/tests/integration_tests.rs` (new)

**Steps:**
1. Create integration test file
2. Setup: create loopback device
3. Test: partition operations (create table, partitions)
4. Test: format partition
5. Test: mount/unmount
6. Teardown: cleanup loopback device
7. Add to CI workflow

**Test Plan:**
- All tests pass on clean system
- CI runs tests

**Done When:**
- [ ] Integration tests cover major flows
- [ ] Tests reliable (not flaky)
- [ ] CI integration

---

### Task 69: Update justfile

**Scope:** Add development recipes for new features

**Files:**
- `justfile` (update)

**Steps:**
1. Add `test-disks-list` recipe
2. Add `test-partition-create` recipe
3. Add `test-format` recipe
4. Add `test-lvm-list` recipe
5. Add loopback device setup helper recipes

**Test Plan:**
- Each recipe works

**Done When:**
- [ ] Recipes added for all major operations
- [ ] Documentation updated

---

### Task 70: Update Documentation

**Scope:** Rustdoc and guides

**Files:**
- `storage-service/README.md` (update)
- Various source files (rustdoc)

**Steps:**
1. Add rustdoc for all public types
2. Add examples to method docs
3. Update README with all operations
4. Document authorization model
5. Document error handling
6. Add D-Bus API reference

**Test Plan:**
- `cargo doc --open` looks good
- README accurate

**Done When:**
- [ ] All public APIs documented
- [ ] Examples provided
- [ ] README comprehensive

---

### Task 71: Performance Testing

**Scope:** Verify performance targets

**Files:**
- `storage-service/benches/` (new)

**Steps:**
1. Create benchmark suite
2. Benchmark disk listing (target: <500ms)
3. Benchmark partition listing (target: <100ms per disk)
4. Benchmark D-Bus overhead (target: <50ms)
5. Optimize if needed

**Test Plan:**
- Benchmarks run successfully
- Targets met

**Done When:**
- [ ] All benchmarks pass targets
- [ ] Bottlenecks identified and fixed

---

### Task 72: Final Review & Cleanup

**Scope:** Code quality pass

**Steps:**
1. Run clippy with --all-features
2. Fix all warnings
3. Run rustfmt
4. Update CHANGELOG
5. Review error messages (user-friendly?)
6. Test on different systems (Ubuntu, Fedora, Arch)
7. Security review of Polkit policies
8. Review logging (no sensitive data)

**Test Plan:**
- CI passes
- Manual testing on 3 distros

**Done When:**
- [ ] No compiler warnings
- [ ] No clippy warnings
- [ ] Formatted
- [ ] Tested on multiple distros
- [ ] Ready to merge

---

## Success Metrics

**Phase 3 Complete When:**
- ✅ All 72 tasks completed (15 Phase 3A refactoring + 57 Phase 3B implementation)
- ✅ Phase 3A: disks-dbus refactored to return storage-models types
- ✅ Phase 3B: All D-Bus methods callable and working
- ✅ All client wrappers implemented
- ✅ Integration tests passing
- ✅ Performance targets met
- ✅ Documentation complete
- ✅ Manual testing passed on 3+ distros

**User Value Delivered:**
- Clean architecture: storage-models as single source of truth
- Full disk and partition management via D-Bus
- Consistent authorization model
- Progress reporting for long operations
- Type-safe Rust client API
- Foundation for Phase 4 (UI integration)

---

**End of Tasks**
