# Storage Service Phase 3: Disk & Partition Operations — Implementation Spec

**Branch:** `feature/storage-service`  
**Phase:** 3 (Breadth Expansion)  
**Type:** Feature Addition (Service Expansion)  
**Estimated Effort:** 4-6 weeks  
**Status:** Planned  
**Breaking Change:** No (additive)  

**Related Specs:**
- Phase 1-2: [storage-service/plan.md](./plan.md) (BTRFS operations + architecture foundation)

---

## Executive Summary

Expand `storage-service` D-Bus daemon to provide comprehensive disk management operations beyond BTRFS, including:
- **Disk operations**: Discovery, information, SMART monitoring
- **Partition operations**: Create, delete, resize, set type/flags
- **Filesystem operations**: Format, mount, unmount (with process killing), check
- **LVM operations**: Volume group and logical volume management
- **Encryption operations**: LUKS setup, unlock, lock

**Strategy:** Refactor `disks-dbus` to use `storage-models` as its return types, eliminating double conversion. The service then exposes these operations via D-Bus with Polkit authorization.

**Critical Architectural Decision:**
- **storage-models** = Single source of truth for ALL domain models (DiskInfo, PartitionInfo, FilesystemInfo, etc.)
- **disks-dbus** = Thin UDisks2 adapter that returns `storage-models` types directly
  - Refactor existing DriveModel, VolumeNode to use/return storage-models types
  - Internal implementation details can stay, but public API must return storage-models
- **storage-service** = Receives storage-models from disks-dbus, serializes to JSON for D-Bus transport
- **disks-ui/VolumeModel** = Optional UI-specific wrapper over storage-models (if needed for app state)
- Process killing: Expose existing `find_processes_using_mount` and `kill_processes` from disks-dbus

**Data Flow (No Double Conversion):**
```
UDisks2 → disks-dbus → storage_models::DiskInfo → storage-service → JSON →
client → storage_models::DiskInfo → (optional) UI wrapper
```

---

## Prerequisites

**CRITICAL: Phase 3A Must Complete Before Phase 3B**

### Phase 3A: Refactor disks-dbus (2-3 weeks)

**This is essential prerequisite work before implementing the D-Bus service.**

1. **Define complete storage-models API:**
   - Analyze current DriveModel, VolumeNode, and all disks-dbus types
   - Extract pure domain data (no Connection, no UI state)
   - Create storage-models types: DiskInfo, PartitionInfo, FilesystemInfo, VolumeInfo, LvmInfo, LuksInfo
   - Add serde derives for D-Bus transport
   - Re-export ProcessInfo and KillResult with serde derives

2. **Refactor disks-dbus public API (breaking change):**
   - Change method signatures to return `storage_models::*`
   - Example: `DiskManager::list_disks() → Result<Vec<storage_models::DiskInfo>>`
   - Keep internal DriveModel/VolumeNode if useful, but convert at API boundary
   - Update all public methods in: manager.rs, volume.rs, drive/, ops.rs, etc.

3. **Update disks-ui to use storage-models:**
   - Import types from storage-models instead of disks-dbus
   - Create UI-specific wrappers if needed (e.g., VolumeModel with selection state)
   - This is a refactor of existing code, not new functionality

4. **Verify everything still works:**
   - Run UI, test all disk operations
   - Ensure no regressions
   - All tests pass

**Expected Result:**
- disks-dbus returns storage-models types
- disks-ui uses storage-models types
- storage-service can now use storage-models without conversion

### Phase 3B: Implement D-Bus Service (After Phase 3A)

Only after Phase 3A is complete can we proceed with the D-Bus service implementation as originally planned.

---

## Goals

### Primary Objectives

1. **Disk Discovery & Information**
   - List all block devices (drives, partitions, loop devices)
   - Get detailed disk information (model, serial, size, connection type)
   - Monitor hotplug events (USB drives, SD cards)
   
2. **SMART Monitoring**
   - Get SMART health status
   - Read SMART attributes (temperature, power-on hours, errors)
   - Trigger SMART self-tests (short, long, conveyance)
   - Monitor test progress

3. **Partition Management**
   - List partitions with detailed metadata
   - Create new partitions (primary, extended, logical)
   - Delete partitions
   - Resize partitions (grow/shrink)
   - Set partition type (GPT type GUID, MBR type code)
   - Set partition flags (bootable, hidden, system)
   - Create/delete partition tables (GPT, MBR)

4. **Filesystem Operations**
   - Format partitions with filesystem (ext4, xfs, btrfs, fat32, ntfs, exfat)
   - Mount/unmount filesystems
   - Check and repair filesystems (fsck)
   - Label filesystems
   - Get filesystem usage statistics

5. **LVM Management**
   - List volume groups (VGs)
   - List logical volumes (LVs)
   - List physical volumes (PVs)
   - Create/delete volume groups
   - Create/delete logical volumes
   - Resize logical volumes
   - Activate/deactivate LVs

6. **Encryption Support**
   - Format partition as LUKS
   - Unlock LUKS volumes (prompt for passphrase via D-Bus signal)
   - Lock LUKS volumes
   - Change LUKS passphrase
   - Add/remove keyslots

### Non-Goals (Deferred)

- **RAID management** (mdadm) — Phase 4
- **Network storage** (iSCSI, NFS) — Phase 4
- **Disk cloning/imaging** — Phase 4
- **Partition recovery** — Phase 4
- **BitLocker support** — Out of scope (Windows-only)

---

## Problem Context

### Current State

**Phase 1-2 Completed:**
- ✅ BTRFS operations via `disks-btrfs` library
- ✅ D-Bus service architecture established
- ✅ `storage-models` crate for shared types
- ✅ Polkit authorization framework
- ✅ BtrfsClient wrapper in UI

**Existing Infrastructure in disks-dbus:**
- ✅ Complete UDisks2 integration (BlockProxy, PartitionProxy, FilesystemProxy, etc.)
- ✅ VolumeNode abstraction (core disk/partition tree structure)
- ✅ Process finder (identify processes blocking unmount)
- ✅ All disk/partition/filesystem operations already implemented
- ✅ SMART data access via DriveModel
- ✅ LVM support via udisks2 crate

**Current Architectural Issues:**
- ❌ disks-dbus has its own model types (DriveModel, VolumeNode) separate from storage-models
- ❌ Would require conversion: DriveModel → DiskInfo → JSON → DiskInfo → DriveModel (circular)
- ❌ UI still uses `disks-dbus` directly (no privilege separation)
- ❌ No centralized authorization policy for disk operations
- ❌ Process killing on busy unmount not exposed via API

**Refactoring Required:**
- Migrate disks-dbus to use storage-models as return types (DriveModel internals can stay, but return DiskInfo)
- This enables clean flow: UDisks2 → storage-models → service → JSON → client → storage-models

### Architecture Strategy

**Phase 3A: Refactor disks-dbus to Use storage-models (Prerequisite)**

Before implementing the D-Bus service, we must refactor disks-dbus:

1. **Move base models to storage-models:**
   - Define core domain types: DiskInfo, PartitionInfo, FilesystemInfo, VolumeInfo, etc.
   - These are pure data structures (no Connection handles, no async methods)
   - Include all fields currently in DriveModel, VolumeNode that are domain data

2. **Refactor disks-dbus public API:**
   - Methods return `storage_models::*` types instead of custom types
   - Example: `list_disks() → Vec<storage_models::DiskInfo>` not `Vec<DriveModel>`
   - Internal implementation can keep DriveModel/VolumeNode, but convert at boundary
   - This is a breaking change for disks-dbus, but necessary for clean architecture

3. **UI-specific state moves to disks-ui:**
   - If current VolumeModel has UI-specific concerns (selection state, UI flags), those stay in disks-ui
   - Create disks-ui wrapper type if needed: `struct VolumeModel { base: storage_models::VolumeInfo, ui_state: ... }`

**Phase 3B: Implement D-Bus Service (After Refactor)**

**Service Layer Responsibilities:**
- Receive storage-models types from disks-dbus (no conversion needed!)
- Polkit authorization before calling disks-dbus
- Serialize storage-models types to JSON for D-Bus transport
- Deserialize JSON back to storage-models types
- Progress signals for long operations
- Process killing integration for busy unmount

**Clean Data Flow:**
```
UDisks2 raw data
    ↓
disks-dbus (extracts data, creates storage_models::DiskInfo)
    ↓
storage-service (receives storage_models::DiskInfo, serializes to JSON)
    ↓
D-Bus (JSON string transport)
    ↓
client (deserializes JSON to storage_models::DiskInfo)
    ↓
disks-ui (uses storage_models::DiskInfo directly, or wraps in VolumeModel for UI state)
```

No circular conversion, storage-models is single source of truth.

---

## Architecture

### D-Bus Interface Structure

```
org.cosmic.ext.StorageService
├── /org/cosmic/ext/StorageService
│   └── Properties: version, supported_features
│
├── /org/cosmic/ext/StorageService/btrfs
│   └── BTRFS operations (existing - Phase 1-2)
│
├── /org/cosmic/ext/StorageService/disks
│   ├── ListDisks() → JSON (Vec<DiskInfo>)
│   ├── GetDiskInfo(device: String) → JSON (DiskInfo)
│   ├── GetSmartStatus(device: String) → JSON (SmartStatus)
│   ├── GetSmartAttributes(device: String) → JSON (Vec<SmartAttribute>)
│   ├── StartSmartTest(device: String, test_type: String)
│   ├── PowerOff(device: String)
│   └── Signals:
│       ├── DiskAdded(device: String, disk_info: JSON)
│       ├── DiskRemoved(device: String)
│       └── SmartTestCompleted(device: String, success: bool)
│
├── /org/cosmic/ext/StorageService/partitions
│   ├── ListPartitions(disk: String) → JSON (Vec<PartitionInfo>)
│   ├── CreatePartitionTable(disk: String, table_type: String)
│   ├── CreatePartition(disk: String, start: u64, size: u64, type_id: String) → String (partition device)
│   ├── DeletePartition(partition: String)
│   ├── ResizePartition(partition: String, size: u64)
│   ├── SetPartitionType(partition: String, type_id: String)
│   ├── SetPartitionFlags(partition: String, flags: u64)
│   ├── SetPartitionName(partition: String, name: String)
│   └── Signals:
│       ├── PartitionCreated(disk: String, partition: String)
│       ├── PartitionDeleted(disk: String, partition: String)
│       └── PartitionModified(partition: String)
│
├── /org/cosmic/ext/StorageService/filesystems
│   ├── ListFilesystems() → JSON (Vec<FilesystemInfo>)
│   ├── Format(device: String, fs_type: String, label: String, options: JSON)
│   ├── Mount(device: String, mount_point: String, options: JSON) → String (actual mount point)
│   ├── Unmount(device_or_mount: String, force: bool, kill_processes: bool) → Result or JSON (UnmountResult)
│   ├── GetBlockingProcesses(device_or_mount: String) → JSON (Vec<ProcessInfo>)
│   ├── KillProcesses(pids: Vec<i32>) → JSON (Vec<KillResult>)
│   ├── Check(device: String, repair: bool) → JSON (CheckResult)
│   ├── SetLabel(device: String, label: String)
│   ├── GetUsage(mount_point: String) → JSON (FilesystemUsage)
│   └── Signals:
│       ├── FormatProgress(device: String, percent: u8)
│       ├── Mounted(device: String, mount_point: String)
│       └── Unmounted(device: String)
│
├── /org/cosmic/ext/StorageService/lvm
│   ├── ListVolumeGroups() → JSON (Vec<VolumeGroupInfo>)
│   ├── ListLogicalVolumes(vg_name: String) → JSON (Vec<LogicalVolumeInfo>)
│   ├── ListPhysicalVolumes() → JSON (Vec<PhysicalVolumeInfo>)
│   ├── CreateVolumeGroup(name: String, devices: Vec<String>)
│   ├── DeleteVolumeGroup(name: String)
│   ├── CreateLogicalVolume(vg_name: String, name: String, size: u64)
│   ├── DeleteLogicalVolume(vg_name: String, lv_name: String)
│   ├── ResizeLogicalVolume(vg_name: String, lv_name: String, size: u64)
│   ├── ActivateLogicalVolume(vg_name: String, lv_name: String)
│   ├── DeactivateLogicalVolume(vg_name: String, lv_name: String)
│   └── Signals:
│       ├── VolumeGroupCreated(name: String)
│       ├── LogicalVolumeCreated(vg: String, lv: String)
│       └── LogicalVolumeResized(vg: String, lv: String, new_size: u64)
│
└── /org/cosmic/ext/StorageService/encryption
    ├── FormatLuks(device: String, passphrase: String, cipher: String)
    ├── Unlock(device: String) → String (cleartext device)
    ├── Lock(cleartext_device: String)
    ├── ChangePassphrase(device: String, old: String, new: String)
    ├── AddKeyslot(device: String, existing: String, new: String)
    ├── RemoveKeyslot(device: String, slot: u8)
    └── Signals:
        ├── LuksUnlocked(device: String, cleartext: String)
        └── LuksLocked(cleartext: String)
```

### Technology Stack

**Architecture Layers:**
```
[disks-ui]
    ↓ uses storage-models types
    ↓ calls D-Bus client
[D-Bus Client (disks-ui/src/client/)]
    ↓ JSON over D-Bus
[storage-service]
    ↓ calls disks-dbus
    ↓ receives storage-models types
[disks-dbus] ← refactored to return storage-models
    ↓ calls UDisks2
[UDisks2 D-Bus API]
```

**Backend Integration:**
- **disks-dbus** (workspace crate): UDisks2 adapter, returns storage-models types
  - Uses udisks2 crate for D-Bus proxies
  - Extracts data from UDisks2 and constructs storage-models types
- **UDisks2 D-Bus API** (org.freedesktop.UDisks2): System service
  - Block device operations
  - Partition table management
  - Filesystem operations
  - LUKS encryption
- **smartmontools** (via UDisks2): SMART monitoring
- **LVM2** (via UDisks2 or direct): LVM operations

**Rust Crates:**
- `zbus 5.x`: D-Bus framework (existing)
- `zbus_polkit 5.x`: Authorization (existing)
- `tokio`: Async runtime (existing)
- `storage-models`: **Domain model types** (existing, will be expanded)
- `disks-dbus`: UDisks2 adapter (existing, will be refactored)
- `udisks2` crate: UDisks2 D-Bus proxies (used by disks-dbus)

**Data Models (storage-models crate):**

*Note: These are THE domain models, not transport-only types. All layers use these as the canonical representation.*

*Phase 3A will define these by extracting pure domain data from current disks-dbus types (DriveModel, VolumeNode).*

```rust
// storage-models/src/disk.rs
pub struct DiskInfo {
    pub device: String,           // /dev/sda
    pub model: String,            // "Samsung SSD 970 EVO"
    pub serial: String,
    pub size: u64,               // bytes
    pub connection_bus: String,  // "nvme", "usb", "ata"
    pub removable: bool,
    pub ejectable: bool,
    pub rotation_rate: Option<u16>, // RPM (None for SSD)
}

pub struct SmartStatus {
    pub device: String,
    pub healthy: bool,
    pub temperature_celsius: Option<i16>,
    pub power_on_hours: Option<u64>,
    pub power_cycle_count: Option<u64>,
    pub test_running: bool,
    pub test_percent_remaining: Option<u8>,
}

pub struct SmartAttribute {
    pub id: u8,
    pub name: String,
    pub current: u8,
    pub worst: u8,
    pub threshold: u8,
    pub raw_value: u64,
    pub failing: bool,
}

// storage-models/src/partition.rs
pub struct PartitionInfo {
    pub device: String,          // /dev/sda1
    pub number: u32,
    pub parent_device: String,   // /dev/sda
    pub size: u64,
    pub offset: u64,
    pub type_id: String,         // GPT GUID or MBR code
    pub type_name: String,       // "Linux filesystem"
    pub flags: u64,
    pub name: String,            // GPT name
    pub uuid: String,            // Partition UUID
}

pub struct PartitionTableInfo {
    pub device: String,
    pub table_type: String,      // "gpt" or "dos"
    pub max_partitions: u32,
}

// storage-models/src/filesystem.rs
pub struct FilesystemInfo {
    pub device: String,
    pub fs_type: String,         // "ext4", "xfs", "btrfs"
    pub label: String,
    pub uuid: String,
    pub mount_points: Vec<String>,
    pub size: u64,
    pub available: u64,
}

pub struct FormatOptions {
    pub label: Option<String>,
    pub force: bool,
    pub discard: bool,            // Enable TRIM for SSDs
    pub fs_specific: HashMap<String, String>, // mkfs.ext4 -O features
}

pub struct CheckResult {
    pub device: String,
    pub clean: bool,
    pub errors_corrected: u32,
    pub errors_uncorrected: u32,
    pub output: String,
}

pub struct UnmountResult {
    pub success: bool,
    pub error: Option<String>,
    pub blocking_processes: Vec<ProcessInfo>,
}

// Re-export from disks-dbus (already exists)
pub struct ProcessInfo {
    pub pid: i32,
    pub command: String,
    pub uid: u32,
    pub username: String,
}

pub struct KillResult {
    pub pid: i32,
    pub success: bool,
    pub error: Option<String>,
}

// storage-models/src/lvm.rs
pub struct VolumeGroupInfo {
    pub name: String,
    pub uuid: String,
    pub size: u64,
    pub free: u64,
    pub pv_count: u32,
    pub lv_count: u32,
}

pub struct LogicalVolumeInfo {
    pub name: String,
    pub vg_name: String,
    pub uuid: String,
    pub size: u64,
    pub device_path: String,     // /dev/vg0/lv0
    pub active: bool,
}

pub struct PhysicalVolumeInfo {
    pub device: String,
    pub vg_name: Option<String>,
    pub size: u64,
    pub free: u64,
}

// storage-models/src/encryption.rs
pub struct LuksInfo {
    pub device: String,
    pub version: String,         // "luks1" or "luks2"
    pub cipher: String,
    pub key_size: u32,
    pub unlocked: bool,
    pub cleartext_device: Option<String>, // /dev/mapper/luks-...
    pub keyslot_count: u8,
}
```

---

## Implementation Phases

### Phase 3B.1: Disk Discovery & SMART (Week 1-2 after Phase 3A)

**Prerequisites:**
- Phase 3A complete: disks-dbus returns `storage_models::DiskInfo`

**Objectives:**
- Expose disks-dbus disk discovery via D-Bus interface
- Add Polkit authorization layer
- Serialize storage-models to JSON for D-Bus transport

**Tasks:**
1. Create `storage-service/src/handlers/disks.rs`
2. Instantiate DiskManager from disks-dbus
3. Implement D-Bus methods that:
   - Call disks-dbus methods
   - Receive `storage_models::DiskInfo` directly (no conversion!)
   - Check Polkit authorization
   - Serialize to JSON for D-Bus response
4. Implement hotplug monitoring (wrap existing DeviceEventStream)
5. Add Polkit actions: `disks-read` (allow_active), `disks-modify` (auth_admin_keep)
6. Create `disks-ui/src/client/disks.rs` D-Bus client wrapper
7. Write integration tests

**Acceptance Criteria:**
- ✅ `ListDisks()` returns all block devices with accurate metadata
- ✅ `GetSmartStatus()` returns SMART health for supported devices
- ✅ `GetSmartAttributes()` returns detailed SMART data
- ✅ Hotplug events trigger `DiskAdded`/`DiskRemoved` signals
- ✅ Non-root users can read disk info without prompts
- ✅ SMART tests can be triggered and monitored

### Phase 3B.2: Partition Management (Week 3-4 after Phase 3A)

**Prerequisites:**
- Phase 3A complete: disks-dbus returns `storage_models::PartitionInfo`

**Objectives:**
- Expose partition operations via D-Bus
- Polkit authorization for destructive operations

**Tasks:**
1. Create `storage-service/src/handlers/partitions.rs`
2. Call disks-dbus partition operations (already return storage-models types)
3. Add Polkit checks before operations
4. Serialize responses to JSON
5. Emit signals for partition changes
6. Create `disks-ui/src/client/partitions.rs`
7. Integration tests

**Acceptance Criteria:**
- ✅ Can create GPT and MBR partition tables
- ✅ Can create partitions with specified size and type
- ✅ Can delete partitions safely (with confirmation)
- ✅ Can resize partitions (grow/shrink)
- ✅ Can set partition types (Linux, EFI, swap, etc.)
- ✅ Can set bootable flag on MBR partitions
- ✅ Modifications trigger appropriate signals

### Phase 3B.3: Filesystem Operations (Week 5-6 after Phase 3A)

**Prerequisites:**
- Phase 3A complete: disks-dbus returns `storage_models::FilesystemInfo`
- ProcessInfo and KillResult in storage-models with serde derives

**Objectives:**
- Expose filesystem operations via D-Bus
- Add process killing for busy unmount
- Mount/unmount with better error recovery

**Tasks:**
1. Create `storage-service/src/handlers/filesystems.rs`
2. Call disks-dbus filesystem operations
3. Implement format operation with progress signals
4. Implement mount/unmount operations
5. **Add GetBlockingProcesses() method** (calls find_processes_using_mount from disks-dbus)
6. **Add KillProcesses() method** (calls kill_processes from disks-dbus)
7. **Extend Unmount() with kill_processes parameter**
8. Implement filesystem check/repair
9. Implement label setting
10. Add Polkit actions: `filesystems-read`, `filesystems-format`, `filesystems-mount`, `filesystems-kill-processes`
11. Create `disks-ui/src/client/filesystems.rs`
12. Integration tests including busy unmount recovery

**Acceptance Criteria:**
- ✅ Can format with: ext4, xfs, btrfs, fat32, ntfs, exfat
- ✅ Format operations show progress (0-100%)
- ✅ Can mount filesystem to specific path
- ✅ Can unmount by device or mount point
- ✅ Force unmount works when filesystem is busy
- ✅ Filesystem check detects and repairs errors
- ✅ Can set filesystem labels

### Phase 3B.4: LVM Operations (Week 7-8 after Phase 3A)

**Prerequisites:**
- Phase 3A complete: disks-dbus returns `storage_models::LvmInfo` types

**Objectives:**
- Expose LVM management via D-Bus

**Tasks:**
1. Create `storage-service/src/handlers/lvm.rs`
2. Call disks-dbus LVM operations
3. Add Polkit checks
4. Create `disks-ui/src/client/lvm.rs`
5. Integration tests

**Acceptance Criteria:**
- ✅ Can list all VGs with size/free space info
- ✅ Can list all LVs in a VG
- ✅ Can create VG from physical devices
- ✅ Can create LV with specified size
- ✅ Can resize LV (grow/shrink)
- ✅ Can delete LV and VG
- ✅ Can activate/deactivate LVs

### Phase 3B.5: Encryption Support (Week 9-10 after Phase 3A)

**Prerequisites:**
- Phase 3A complete: disks-dbus returns `storage_models::LuksInfo`

**Objectives:**
- Expose LUKS encryption operations via D-Bus

**Tasks:**
1. Create `storage-service/src/handlers/encryption.rs`
2. Call disks-dbus encryption operations
3. Add Polkit checks
4. Create `disks-ui/src/client/encryption.rs`
5. Design passphrase prompt mechanism
6. Integration tests

**Acceptance Criteria:**
- ✅ Can format partition as LUKS1 or LUKS2
- ✅ Can unlock LUKS volume (prompts for passphrase via UI)
- ✅ Can lock LUKS volume
- ✅ Can change LUKS passphrase
- ✅ Can add/remove keyslots
- ✅ Unlocked devices are tracked and signals emitted

---

## Error Handling Strategy

### Error Categories

**1. Authorization Errors:**
- User lacks permission
- Polkit auth failed
- Return: `PermissionDenied` with action name

**2. Device Errors:**
- Device not found
- Device busy
- Device removed during operation
- Return: `DeviceError` with device path

**3. Operation Errors:**
- Invalid parameters (size too large, invalid filesystem type)
- Operation not supported (resize NTFS)
- Filesystem full
- Return: `OperationFailed` with context

**4. Backend Errors:**
- UDisks2 service not available
- UDisks2 method call timeout
- Unexpected D-Bus error
- Return: `BackendError` with details

**Rust Error Types (storage-service):**
```rust
// storage-service/src/error.rs
#[derive(Debug, Error)]
pub enum ServiceError {
    #[error("Permission denied: {action}")]
    PermissionDenied { action: String },
    
    #[error("Device error: {device} - {message}")]
    DeviceError { device: String, message: String },
    
    #[error("Operation failed: {message}")]
    OperationFailed { message: String },
    
    #[error("Invalid argument: {message}")]
    InvalidArgument { message: String },
    
    #[error("Backend error: {message}")]
    BackendError { message: String },
    
    #[error("Not supported: {feature}")]
    NotSupported { feature: String },
}

impl From<ServiceError> for zbus::fdo::Error {
    fn from(err: ServiceError) -> Self {
        match err {
            ServiceError::PermissionDenied { .. } => 
                zbus::fdo::Error::AccessDenied(err.to_string()),
            ServiceError::InvalidArgument { .. } => 
                zbus::fdo::Error::InvalidArgs(err.to_string()),
            _ => zbus::fdo::Error::Failed(err.to_string()),
        }
    }
}
```

**Client Error Types (disks-ui):**
```rust
// disks-ui/src/client/error.rs (extend existing)
#[derive(Debug, Error)]
pub enum ClientError {
    // ... existing variants ...
    
    #[error("Device busy: {device}")]
    DeviceBusy { device: String },
    
    #[error("Not supported: {feature}")]
    NotSupported { feature: String },
    
    #[error("Cancelled by user")]
    Cancelled,
}
```

---

## Security & Authorization

### Polkit Actions

**File:** `data/polkit-1/actions/org.cosmic.ext.storage-service.policy`

```xml
<!-- Disk Operations -->
<action id="org.cosmic.ext.storage-service.disks-read">
  <description>View disk information</description>
  <message>Authentication required to view disk details</message>
  <defaults>
    <allow_any>no</allow_any>
    <allow_inactive>no</allow_inactive>
    <allow_active>yes</allow_active> <!-- No prompt for reading -->
  </defaults>
</action>

<action id="org.cosmic.ext.storage-service.disks-modify">
  <description>Modify disk settings</description>
  <message>Authentication required to modify disk settings</message>
  <defaults>
    <allow_any>no</allow_any>
    <allow_inactive>no</allow_inactive>
    <allow_active>auth_admin_keep</allow_active>
  </defaults>
</action>

<!-- Partition Operations -->
<action id="org.cosmic.ext.storage-service.partitions-read">
  <description>View partition information</description>
  <defaults>
    <allow_active>yes</allow_active>
  </defaults>
</action>

<action id="org.cosmic.ext.storage-service.partitions-modify">
  <description>Modify partitions</description>
  <message>Authentication required to create, delete, or resize partitions</message>
  <defaults>
    <allow_active>auth_admin_keep</allow_active>
  </defaults>
</action>

<!-- Filesystem Operations -->
<action id="org.cosmic.ext.storage-service.filesystems-read">
  <description>View filesystem information</description>
  <defaults>
    <allow_active>yes</allow_active>
  </defaults>
</action>

<action id="org.cosmic.ext.storage-service.filesystems-format">
  <description>Format filesystems</description>
  <message>Authentication required to format a device (WARNING: destroys data)</message>
  <defaults>
    <allow_active>auth_admin</allow_active> <!-- Always prompt -->
  </defaults>
</action>

<action id="org.cosmic.ext.storage-service.filesystems-mount">
  <description>Mount and unmount filesystems</description>
  <message>Authentication required to mount or unmount filesystems</message>
  <defaults>
    <allow_active>yes</allow_active> <!-- Allow for removable media -->
  </defaults>
</action>

<action id="org.cosmic.ext.storage-service.filesystems-kill-processes">
  <description>Kill processes blocking unmount</description>
  <message>Authentication required to kill processes using a filesystem</message>
  <defaults>
    <allow_active>auth_admin_keep</allow_active>
  </defaults>
</action>

<!-- LVM Operations -->
<action id="org.cosmic.ext.storage-service.lvm-read">
  <description>View LVM information</description>
  <defaults>
    <allow_active>yes</allow_active>
  </defaults>
</action>

<action id="org.cosmic.ext.storage-service.lvm-modify">
  <description>Modify LVM volumes</description>
  <message>Authentication required to manage logical volumes</message>
  <defaults>
    <allow_active>auth_admin_keep</allow_active>
  </defaults>
</action>

<!-- Encryption Operations -->
<action id="org.cosmic.ext.storage-service.encryption-read">
  <description>View encryption information</description>
  <defaults>
    <allow_active>yes</allow_active>
  </defaults>
</action>

<action id="org.cosmic.ext.storage-service.encryption-modify">
  <description>Modify encryption</description>
  <message>Authentication required to format device with encryption</message>
  <defaults>
    <allow_active>auth_admin_keep</allow_active>
  </defaults>
</action>

<action id="org.cosmic.ext.storage-service.encryption-unlock">
  <description>Unlock encrypted device</description>
  <message>Authentication required to unlock encrypted device</message>
  <defaults>
    <allow_active>yes</allow_active> <!-- User provides passphrase -->
  </defaults>
</action>

<!-- SMART Operations -->
<action id="org.cosmic.ext.storage-service.smart-read">
  <description>Read S.M.A.R.T. data</description>
  <defaults>
    <allow_active>yes</allow_active>
  </defaults>
</action>

<action id="org.cosmic.ext.storage-service.smart-test">
  <description>Run S.M.A.R.T. tests</description>
  <message>Authentication required to run disk self-tests</message>
  <defaults>
    <allow_active>auth_admin_keep</allow_active>
  </defaults>
</action>
```

### D-Bus Policy

**File:** `data/dbus-1/system.d/org.cosmic.ext.StorageService.conf`

```xml
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE busconfig PUBLIC
 "-//freedesktop//DTD D-BUS Bus Configuration 1.0//EN"
 "http://www.freedesktop.org/standards/dbus/1.0/busconfig.dtd">
<busconfig>
  <!-- Only root can own the service -->
  <policy user="root">
    <allow own="org.cosmic.ext.StorageService"/>
  </policy>

  <!-- All users can call methods (Polkit handles auth) -->
  <policy context="default">
    <allow send_destination="org.cosmic.ext.StorageService"/>
    <allow receive_sender="org.cosmic.ext.StorageService"/>
  </policy>
</busconfig>
```

---

## Testing Strategy

### Unit Tests
- Test data model serialization/deserialization
- Test error conversions
- Test UDisks2 proxy generation

### Integration Tests
- Use loopback devices for partition operations
- Create test filesystems in temporary locations
- Mock UDisks2 for service layer tests
- Test signal emissions

### Manual Testing Checklist
- [ ] List disks on system with NVMe, SATA, USB
- [ ] Monitor hotplug (insert USB drive, see DiskAdded signal)
- [ ] Read SMART data from multiple disk types
- [ ] Create GPT partition table
- [ ] Create 3 partitions
- [ ] Format partition as ext4
- [ ] Mount formatted partition
- [ ] Write files, unmount
- [ ] Resize partition
- [ ] Delete partition
- [ ] Create LVM VG from 2 devices
- [ ] Create LV in VG
- [ ] Format LUKS partition
- [ ] Unlock LUKS partition
- [ ] Mount unlocked partition

---

## Risks & Mitigations

### Risk 1: UDisks2 API Complexity
**Severity:** High  
**Impact:** Complex D-Bus introspection, many edge cases  
**Mitigation:**
- Study GNOME Disks source code (proven UDisks2 usage)
- Start with simple operations (list, get info)
- Extensive logging of UDisks2 method calls
- Fallback to direct commands where UDisks2 is problematic

### Risk 2: Data Loss from Bugs
**Severity:** Critical  
**Impact:** User data destroyed by incorrect partition/format operations  
**Mitigation:**
- Extensive testing with loopback devices
- Clear warnings in UI before destructive operations
- Double-confirmation for format operations
- Dry-run mode in service (simulate operations)
- Comprehensive error handling (fail safe)

### Risk 3: Performance Issues
**Severity:** Medium  
**Impact:** Slow operation enumeration, UI lag  
**Mitigation:**
- Async all the things (no blocking D-Bus calls)
- Cache disk/partition metadata (invalidate on signals)
- Batch operations where possible
- Progress signals for long operations (format, resize)

### Risk 4: Authorization Confusion
**Severity:** Medium  
**Impact:** Users get unexpected password prompts  
**Mitigation:**
- Clear Polkit messages explaining what requires auth
- Document which operations require auth
- Use `allow_active` for read-only operations
- UI shows lock icon for operations requiring auth

### Risk 5: Filesystem Support Matrix
**Severity:** Low  
**Impact:** Some filesystems not supported on all systems  
**Mitigation:**
- Detect available mkfs tools on startup
- Return `NotSupported` error with helpful message
- UI shows only supported filesystems in format dialog
- Document required packages (e2fsprogs, xfsprogs, etc.)

---

## Dependencies

### Runtime Dependencies
- **udisks2** (≥2.9.0): Core storage management service (already used by disks-dbus)
- **disks-dbus** (workspace): Existing UDisks2 integration layer
- **smartmontools**: SMART monitoring
- **e2fsprogs**: ext2/ext3/ext4 tools
- **xfsprogs**: XFS tools
- **btrfs-progs**: BTRFS tools (already required)
- **dosfstools**: FAT32 tools
- **ntfs-3g**: NTFS tools
- **exfatprogs** or **exfat-utils**: exFAT tools
- **lvm2**: LVM tools
- **cryptsetup**: LUKS tools

**Note:** Most tools already required by disks-dbus

### Development Dependencies
- **Rust crates:**
  - `zbus` 5.x (existing)
  - `zbus_polkit` 5.x (existing)
  - `tokio` (existing)
  - `serde` (existing)
  - `thiserror` (existing)
  - `storage-models` (existing - will be expanded in Phase 3A)
  - `disks-dbus` (existing - will be refactored in Phase 3A)
  - `udisks2` (existing in disks-dbus)
  
**Phase 3A Deliverable:**
- storage-models contains complete domain model API
- disks-dbus public methods return storage-models types

### System Requirements
- Linux kernel ≥5.10 (modern block layer)
- systemd (for service management)
- Polkit (for authorization)

---

## Success Criteria

### Functional Completeness
- ✅ All disk/partition/filesystem/LVM/encryption operations implemented
- ✅ Full test coverage (unit + integration)
- ✅ All operations accessible via D-Bus client wrapper
- ✅ Error handling comprehensive and informative

### Quality Gates
- ✅ CI passes: tests, clippy, rustfmt
- ✅ No compiler warnings
- ✅ Documentation complete (rustdoc for all public APIs)
- ✅ Integration tests pass on loopback devices

### User Experience
- ✅ Operations complete without unexpected prompts (where allowed)
- ✅ Clear error messages when operations fail
- ✅ Progress indicators for long operations
- ✅ Hotplug events reflected in UI within 2 seconds

### Performance
- ✅ Disk listing completes in <500ms (typical system)
- ✅ Partition listing completes in <100ms per disk
- ✅ D-Bus method calls add <50ms overhead vs direct UDisks2
- ✅ No UI freezing during operations

---

## Future Enhancements (Phase 4+)

**RAID Management:**
- Create/manage RAID arrays (mdadm)
- Monitor RAID health
- Rebuild degraded arrays

**Network Storage:**
- iSCSI initiator
- NFS client configuration
- SMB/CIFS shares

**Disk Imaging:**
- Clone disk to file
- Restore from image
- Sparse image support

**Advanced Recovery:**
- Partition recovery (testdisk integration)
- File recovery from formatted partitions
- Bad block remapping

**Performance Tuning:**
- I/O scheduler configuration
- Filesystem mount option presets
- SSD optimization (TRIM settings)

---

## Appendix A: UDisks2 Object Model

**Core UDisks2 Interfaces:**
```
/org/freedesktop/UDisks2/drives/{drive_id}
├── org.freedesktop.UDisks2.Drive
│   ├── Properties: Model, Serial, Size, ConnectionBus, Removable
│   └── Methods: PowerOff(), Eject()
└── org.freedesktop.UDisks2.Drive.Ata (if ATA)
    ├── Properties: SmartSupported, SmartEnabled
    └── Methods: SmartGetAttributes(), SmartSelftestStart()

/org/freedesktop/UDisks2/block_devices/{device}
├── org.freedesktop.UDisks2.Block
│   ├── Properties: Device, Size, IdType, IdLabel, Drive
│   └── Methods: Format(), Rescan()
├── org.freedesktop.UDisks2.Filesystem (if formatted)
│   └── Methods: Mount(), Unmount(), SetLabel()
├── org.freedesktop.UDisks2.Partition (if partition)
│   ├── Properties: Number, Type, Offset, Size, Name
│   └── Methods: SetType(), SetFlags(), SetName(), Delete()
├── org.freedesktop.UDisks2.PartitionTable (if has partitions)
│   └── Methods: CreatePartition()
└── org.freedesktop.UDisks2.Encrypted (if LUKS)
    └── Methods: Unlock(), Lock(), ChangePassphrase()
```

**Discovery Pattern:**
```rust
// Get ObjectManager
let object_manager = UDisks2ObjectManagerProxy::new(&connection).await?;

// Get all objects
let objects = object_manager.get_managed_objects().await?;

// Filter for drives
for (path, interfaces) in objects {
    if interfaces.contains_key("org.freedesktop.UDisks2.Drive") {
        // Create Drive proxy
        let drive = UDisks2DriveProxy::builder(&connection)
            .path(path)?
            .build()
            .await?;
        
        // Get properties
        let model = drive.model().await?;
        let size = drive.size().await?;
    }
}
```

---

## Appendix B: Example D-Bus Method Calls

**List all disks:**
```bash
busctl call org.cosmic.ext.StorageService \
  /org/cosmic/ext/StorageService/disks \
  org.cosmic.ext.StorageService.Disks \
  ListDisks
```

**Create partition:**
```bash
busctl call org.cosmic.ext.StorageService \
  /org/cosmic/ext/StorageService/partitions \
  org.cosmic.ext.StorageService.Partitions \
  CreatePartition sstt \
  "/dev/sda" "1048576" "10737418240" "0fc63daf-8483-4772-8e79-3d69d8477de4"
  # Last arg is Linux filesystem type (GPT GUID)
```

**Format as ext4:**
```bash
busctl call org.cosmic.ext.StorageService \
  /org/cosmic/ext/StorageService/filesystems \
  org.cosmic.ext.StorageService.Filesystems \
  Format ssss \
  "/dev/sda1" "ext4" "MyData" "{}"
```

**Mount filesystem:**
```bash
busctl call org.cosmic.ext.StorageService \
  /org/cosmic/ext/StorageService/filesystems \
  org.cosmic.ext.StorageService.Filesystems \
  Mount sss \
  "/dev/sda1" "/mnt/mydata" "{}"
```

---

## Appendix C: Workspace Structure After Phase 3

```
cosmic-ext-disks/
├── storage-models/
│   ├── src/
│   │   ├── lib.rs
│   │   ├── btrfs.rs        (existing - Phase 1-2)
│   │   ├── disk.rs         (new - Phase 3.1)
│   │   ├── partition.rs    (new - Phase 3.2)
│   │   ├── filesystem.rs   (new - Phase 3.3)
│   │   ├── lvm.rs          (new - Phase 3.4)
│   │   └── encryption.rs   (new - Phase 3.5)
│   └── Cargo.toml
│
├── storage-service/
│   ├── src/
│   │   ├── main.rs         (existing - entry point)
│   │   ├── service.rs      (existing - main interface)
│   │   ├── auth.rs         (existing - Polkit)
│   │   ├── error.rs        (extend with new error types)
│   │   ├── btrfs.rs        (existing - Phase 1-2)
│   │   ├── udisks2/        (new - UDisks2 integration)
│   │   │   ├── mod.rs
│   │   │   ├── client.rs
│   │   │   └── proxies.rs
│   │   └── handlers/       (new - operation handlers)
│   │       ├── mod.rs
│   │       ├── disks.rs     (Phase 3.1)
│   │       ├── partitions.rs (Phase 3.2)
│   │       ├── filesystems.rs (Phase 3.3)
│   │       ├── lvm.rs       (Phase 3.4)
│   │       └── encryption.rs (Phase 3.5)
│   └── Cargo.toml
│
├── disks-ui/
│   ├── src/
│   │   ├── client/
│   │   │   ├── mod.rs
│   │   │   ├── error.rs    (existing - extend)
│   │   │   ├── btrfs.rs    (existing - Phase 1-2)
│   │   │   ├── disks.rs    (new - Phase 3.1)
│   │   │   ├── partitions.rs (new - Phase 3.2)
│   │   │   ├── filesystems.rs (new - Phase 3.3)
│   │   │   ├── lvm.rs      (new - Phase 3.4)
│   │   │   └── encryption.rs (new - Phase 3.5)
│   │   └── ...
│   └── Cargo.toml
│
└── disks-btrfs/            (existing - Phase 1-2)
```

---

**End of Plan**
