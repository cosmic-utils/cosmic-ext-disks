# Phase 3A Models Refactoring Analysis

**Date:** 2026-02-14  
**Goal:** Analyze existing storage-dbus types and plan migration to storage-models as single source of truth

---

## Executive Summary

Current storage-dbus exports three main domain model types:
- `DriveModel` - represents physical/logical drives
- `VolumeNode` - hierarchical representation of partitions, LUKS containers, LVM volumes
- `VolumeModel` - flat representation of partitions and filesystems

**Key Finding:** All three types mix pure domain data with D-Bus connection handles, making them unsuitable for serialization. The refactoring must extract domain data into storage-models types while keeping connection logic internal to storage-dbus.

---

## 1. DriveModel Analysis

**Location:** `storage-dbus/src/disks/drive/model.rs`

### Fields Breakdown

#### Domain Data (→ storage-models::DiskInfo)
```rust
pub id: String,                    // Drive identifier
pub model: String,                 // Drive model name
pub serial: String,                // Serial number
pub vendor: String,                // Vendor/manufacturer
pub revision: String,              // Firmware revision
pub size: u64,                     // Drive size in bytes
pub name: String,                  // Device name (e.g., "/dev/sda")
pub block_path: String,            // Block device path
pub is_loop: bool,                 // Loop device flag
pub backing_file: Option<String>,  // Loop device backing file
pub removable: bool,               // Removable media
pub ejectable: bool,               // Ejectable drive
pub media_removable: bool,         // Removable media
pub media_available: bool,         // Media present
pub media_change_detected: bool,   // Media change flag
pub can_power_off: bool,           // Power off capability
pub optical: bool,                 // Optical drive
pub optical_blank: bool,           // Blank optical media
pub rotation_rate: i32,            // RPM (0=SSD, -1=unknown, >0=HDD)
pub partition_table_type: Option<String>, // "gpt" or "dos"
pub gpt_usable_range: Option<ByteRange>,  // GPT usable space
```

#### Internal Implementation Details (keep in storage-dbus)
```rust
pub volumes_flat: Vec<VolumeModel>,    // Cached flat list (derived)
pub volumes: Vec<VolumeNode>,          // Cached tree (derived)
pub path: String,                      // Internal UDisks2 path
connection: Connection,                 // D-Bus connection (not serializable)
```

### Methods That Need API Changes
- `from_proxy()` - Should return `storage_models::DiskInfo`
- `from_block_only()` - Should return `storage_models::DiskInfo`
- `name()` - Can remain as internal helper or move to DiskInfo impl
- `supports_power_management()` - Can move to DiskInfo impl

### Recommended storage-models Type
```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiskInfo {
    // Identity
    pub device: String,              // e.g., "/dev/sda"
    pub id: String,                  // UDisks2 ID
    pub model: String,
    pub serial: String,
    pub vendor: String,
    pub revision: String,
    
    // Physical properties
    pub size: u64,
    pub connection_bus: String,      // "usb", "ata", "nvme", "loop"
    pub rotation_rate: Option<u16>,  // RPM (None for SSD/unknown)
    
    // Media properties
    pub removable: bool,
    pub ejectable: bool,
    pub media_removable: bool,
    pub media_available: bool,
    pub optical: bool,
    pub optical_blank: bool,
    pub can_power_off: bool,
    
    // Loop device specific
    pub is_loop: bool,
    pub backing_file: Option<String>,
    
    // Partitioning
    pub partition_table_type: Option<String>, // "gpt", "dos", None
    pub gpt_usable_range: Option<ByteRange>,
}
```

---

## 2. VolumeNode Analysis

**Location:** `storage-dbus/src/disks/volume.rs`

### Fields Breakdown

#### Domain Data (→ storage-models::VolumeInfo)
```rust
pub kind: VolumeKind,                // Partition, CryptoContainer, Filesystem, etc.
pub label: String,                   // Volume label
pub size: u64,                       // Size in bytes
pub id_type: String,                 // Filesystem type (ext4, crypto_LUKS, etc.)
pub device_path: Option<String>,     // Device path (e.g., "/dev/sda1")
pub has_filesystem: bool,            // Has filesystem interface
pub mount_points: Vec<String>,       // Current mount points
pub usage: Option<Usage>,            // Filesystem usage stats
pub locked: bool,                    // LUKS locked state
pub children: Vec<VolumeNode>,       // Nested volumes (recursive)
```

#### Internal Implementation Details
```rust
pub object_path: OwnedObjectPath,    // UDisks2 D-Bus path (internal)
connection: Option<Connection>,       // D-Bus connection (not serializable)
```

### VolumeKind Enum
```rust
pub enum VolumeKind {
    Partition,
    CryptoContainer,
    Filesystem,
    LvmPhysicalVolume,
    LvmLogicalVolume,
    Block,
}
```

### Methods That Need API Changes
- `from_block_object()` - Should return `storage_models::VolumeInfo`
- `crypto_container_for_partition()` - Should return `storage_models::VolumeInfo`
- `probe_basic_block()` - Internal, can stay
- `mount()` - Requires connection, can stay in storage-dbus
- `unmount()` - Requires connection, can stay in storage-dbus
- `default_mount_options()` - Requires connection, can stay
- `get_mount_options_settings()` - Requires connection, can stay
- `is_mounted()`, `can_mount()`, `can_unlock()`, `can_lock()` - Move to VolumeInfo

### Recommended storage-models Type
```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VolumeInfo {
    pub kind: VolumeKind,
    pub label: String,
    pub size: u64,
    pub id_type: String,              // Filesystem type
    pub device_path: Option<String>,  // e.g., "/dev/sda1"
    pub has_filesystem: bool,
    pub mount_points: Vec<String>,
    pub usage: Option<Usage>,
    pub locked: bool,                 // For LUKS volumes
    pub children: Vec<VolumeInfo>,    // Recursive structure
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VolumeKind {
    Partition,
    CryptoContainer,
    Filesystem,
    LvmPhysicalVolume,
    LvmLogicalVolume,
    Block,
}
```

---

## 3. VolumeModel Analysis

**Location:** `storage-dbus/src/disks/volume_model/mod.rs`

### Fields Breakdown

#### Domain Data (→ storage-models::PartitionInfo)
```rust
pub volume_type: VolumeType,         // Container, Partition, Filesystem
pub name: String,                    // Partition name (GPT)
pub partition_type_id: String,       // GUID or MBR type code
pub partition_type: String,          // Human-readable type
pub id_type: String,                 // Filesystem type
pub uuid: String,                    // Partition UUID
pub number: u32,                     // Partition number
pub flags: BitFlags<PartitionFlags>, // Bootable, hidden, etc.
pub offset: u64,                     // Partition offset (bytes)
pub size: u64,                       // Partition size (bytes)
pub device_path: Option<String>,     // Device path
pub has_filesystem: bool,
pub mount_points: Vec<String>,
pub usage: Option<Usage>,
pub drive_path: String,              // Parent drive
pub table_type: String,              // "gpt" or "dos"
```

#### Internal Implementation Details
```rust
pub table_path: OwnedObjectPath,     // UDisks2 partition table path
pub path: OwnedObjectPath,           // UDisks2 object path
connection: Option<Connection>,       // D-Bus connection
```

### VolumeType Enum
```rust
pub enum VolumeType {
    Container,    // Extended partition
    Partition,    // Primary/logical partition
    Filesystem,   // Unpartitioned filesystem
}
```

### Methods That Need API Changes
- `from_proxy()` - Should return `storage_models::PartitionInfo`
- `filesystem_from_block()` - Should return `storage_models::FilesystemInfo`
- `partition_info()` - Can stay internal or move to PartitionInfo
- `is_mounted()`, `can_mount()` - Move to PartitionInfo/FilesystemInfo
- `name()` - Move to PartitionInfo impl

### Recommended storage-models Types

```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PartitionInfo {
    pub device: String,              // e.g., "/dev/sda1"
    pub number: u32,                 // Partition number
    pub parent_device: String,       // Parent disk e.g., "/dev/sda"
    pub size: u64,
    pub offset: u64,
    pub type_id: String,             // GUID or MBR type code
    pub type_name: String,           // Human-readable
    pub flags: u64,                  // Bitfield
    pub name: String,                // GPT name
    pub uuid: String,                // Partition UUID
    pub table_type: String,          // "gpt" or "dos"
    pub has_filesystem: bool,
    pub filesystem_type: Option<String>, // If has_filesystem
    pub mount_points: Vec<String>,
    pub usage: Option<Usage>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PartitionTableType {
    Gpt,
    Mbr,
}
```

---

## 4. Shared Types Analysis

### ProcessInfo and KillResult

**Location:** `storage-dbus/src/disks/process_finder.rs`  
**Status:** Already defined with appropriate fields

```rust
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
```

**Action:** Move to storage-models with `#[derive(Serialize, Deserialize)]`

### Usage Type

**Location:** `storage-dbus/src/usage.rs` (presumably)  
**Status:** Referenced but not analyzed

**Action:** Check if already serializable, move to storage-models if not

### LvmLogicalVolumeInfo

**Location:** `storage-dbus/src/disks/lvm.rs`

```rust
pub struct LvmLogicalVolumeInfo {
    pub vg_name: String,
    pub lv_path: String,
    pub size_bytes: u64,
}
```

**Action:** Keep as-is or expand into more comprehensive LVM types:
- `VolumeGroupInfo` - VG name, size, free space, PV count, LV count
- `LogicalVolumeInfo` - extends current
- `PhysicalVolumeInfo` - device, VG membership, size

### SmartInfo

**Location:** `storage-dbus/src/disks/smart.rs`

```rust
pub struct SmartInfo {
    pub device_type: String,          // "NVMe", "ATA"
    pub updated_at: Option<u64>,      // Unix timestamp
    pub temperature_c: Option<u64>,
    pub power_on_hours: Option<u64>,
    pub selftest_status: Option<String>,
    pub attributes: BTreeMap<String, String>,
}
```

**Action:** Move to storage-models with `#[derive(Serialize, Deserialize)]`

### MountOptionsSettings and EncryptionOptionsSettings

**Location:** `storage-dbus/src/disks/mod.rs`

```rust
pub struct MountOptionsSettings {
    pub identify_as: String,
    pub mount_point: String,
    pub filesystem_type: String,
    pub mount_at_startup: bool,
    pub require_auth: bool,
    pub show_in_ui: bool,
    pub other_options: String,
    pub display_name: String,
    pub icon_name: String,
    pub symbolic_icon_name: String,
}

pub struct EncryptionOptionsSettings {
    pub name: String,
    pub unlock_at_startup: bool,
    pub require_auth: bool,
    pub other_options: String,
}
```

**Action:** Move to storage-models with serde derives, or keep in storage-dbus if only used internally

---

## 5. Storage-Models Types to Create

### Priority 1: Core Domain Types

#### DiskInfo
- Represents physical/logical drives
- ~20 fields covering identity, physical properties, media info
- Replaces DriveModel for public API

#### VolumeInfo
- Hierarchical volume representation
- Supports partitions, LUKS, LVM, filesystems
- Replaces VolumeNode for public API

#### PartitionInfo
- Flat partition representation
- Extended metadata (type, flags, GPT name)
- Replaces VolumeModel for public API

#### FilesystemInfo
```rust
pub struct FilesystemInfo {
    pub device: String,
    pub fs_type: String,           // ext4, xfs, btrfs, etc.
    pub label: String,
    pub uuid: String,
    pub mount_points: Vec<String>,
    pub size: u64,
    pub available: u64,
}
```

### Priority 2: Support Types

#### FormatOptions
```rust
pub struct FormatOptions {
    pub label: String,
    pub force: bool,
    pub discard: bool,
    pub fs_specific: HashMap<String, String>,
}
```

#### MountOptions
```rust
pub struct MountOptions {
    pub read_only: bool,
    pub no_exec: bool,
    pub no_suid: bool,
    pub other: Vec<String>,
}
```

#### CheckResult
```rust
pub struct CheckResult {
    pub device: String,
    pub clean: bool,
    pub errors_corrected: u32,
    pub errors_uncorrected: u32,
    pub output: String,
}
```

#### UnmountResult
```rust
pub struct UnmountResult {
    pub success: bool,
    pub error: Option<String>,
    pub blocking_processes: Vec<ProcessInfo>,
}
```

### Priority 3: LVM Types

#### VolumeGroupInfo
```rust
pub struct VolumeGroupInfo {
    pub name: String,
    pub uuid: String,
    pub size: u64,
    pub free: u64,
    pub pv_count: u32,
    pub lv_count: u32,
}
```

#### LogicalVolumeInfo (expand existing)
```rust
pub struct LogicalVolumeInfo {
    pub name: String,
    pub vg_name: String,
    pub uuid: String,
    pub size: u64,
    pub device_path: String,
    pub active: bool,
}
```

#### PhysicalVolumeInfo
```rust
pub struct PhysicalVolumeInfo {
    pub device: String,
    pub vg_name: Option<String>,
    pub size: u64,
    pub free: u64,
}
```

### Priority 4: Encryption Types

#### LuksInfo
```rust
pub struct LuksInfo {
    pub device: String,
    pub version: LuksVersion,
    pub cipher: String,
    pub key_size: u32,
    pub unlocked: bool,
    pub cleartext_device: Option<String>,
    pub keyslot_count: u8,
}

pub enum LuksVersion {
    Luks1,
    Luks2,
}
```

### Priority 5: Utility Types

#### ByteRange (already exists, verify serializability)
```rust
pub struct ByteRange {
    pub start: u64,
    pub end: u64,
}
```

#### Usage (verify exists and serializability)
```rust
pub struct Usage {
    pub total: u64,
    pub used: u64,
    pub free: u64,
}
```

---

## 6. API Breaking Changes Required

### Public Methods Returning Updated Types

#### DiskManager
- Currently: Internal, doesn't expose DriveModel directly
- After: Will need methods to return `Vec<DiskInfo>` if exposed

#### DriveModel (becomes internal)
- All public construction methods now return `storage_models::DiskInfo`
- Internal DriveModel can keep `volumes` field for caching
- Add conversion: `impl From<DriveModel> for DiskInfo`

#### VolumeNode (becomes internal or changes signature)
- Public methods return `storage_models::VolumeInfo`
- Keep VolumeNode internal with `connection` field
- Add conversion: `impl From<VolumeNode> for VolumeInfo`

#### VolumeModel (becomes internal or changes signature)
- Public methods return `storage_models::PartitionInfo` or `FilesystemInfo`
- Add conversions for both types

### New Methods Needed
- `DiskInfo::supports_power_management(&self) -> bool`
- `VolumeInfo::is_mounted(&self) -> bool`
- `VolumeInfo::can_mount(&self) -> bool`
- `PartitionInfo::is_mounted(&self) -> bool`
- `PartitionInfo::can_mount(&self) -> bool`

---

## 7. Implementation Strategy

### Phase 3A Task Mapping

#### Tasks 1-5: Define storage-models Types
1. Analyze current types ✅ (this document)
2. Define DiskInfo
3. Define VolumeInfo and PartitionInfo
4. Define FilesystemInfo and LvmInfo types
5. Define LuksInfo

#### Tasks 6-11: Refactor storage-dbus
6. Refactor DriveModel to return DiskInfo
   - Keep DriveModel internal
   - Add `From<DriveModel> for DiskInfo`
   - Update `from_proxy()` and `from_block_only()`
   
7. Refactor VolumeNode to return VolumeInfo
   - Keep VolumeNode internal for tree operations
   - Add `From<VolumeNode> for VolumeInfo`
   - Update construction methods
   
8. Update partition operations to use PartitionInfo
   - VolumeModel can stay internal
   - Expose PartitionInfo via new methods
   
9. Update filesystem operations to use FilesystemInfo
   - Mount/unmount can still take device paths (strings)
   - Return FilesystemInfo from query methods
   
10. Update LVM operations to use LvmInfo
    - Expand `list_lvs_for_pv()` to return full LvmInfo
    
11. Update LUKS operations to use LuksInfo
    - New methods to query LUKS metadata

#### Tasks 12-15: Update Consumers
12. Update storage-ui to import from storage-models
    - Change all type imports
    - UI can wrap VolumeInfo in VolumeModel if needed for UI state
    
13. Clean up storage-dbus public API
    - Review exports in mod.rs
    - Ensure only storage-models types exported
    
14. Integration testing
    - Verify all operations still work
    - Check data integrity
    
15. Documentation
    - Document new architecture
    - Update examples

---

## 8. Conversion Helpers

### DriveModel → DiskInfo
```rust
impl From<&DriveModel> for DiskInfo {
    fn from(drive: &DriveModel) -> Self {
        Self {
            device: drive.block_path.clone(),
            id: drive.id.clone(),
            model: drive.model.clone(),
            serial: drive.serial.clone(),
            vendor: drive.vendor.clone(),
            revision: drive.revision.clone(),
            size: drive.size,
            connection_bus: infer_bus_from_path(&drive.name),
            rotation_rate: if drive.rotation_rate > 0 {
                Some(drive.rotation_rate as u16)
            } else {
                None
            },
            removable: drive.removable,
            ejectable: drive.ejectable,
            media_removable: drive.media_removable,
            media_available: drive.media_available,
            optical: drive.optical,
            optical_blank: drive.optical_blank,
            can_power_off: drive.can_power_off,
            is_loop: drive.is_loop,
            backing_file: drive.backing_file.clone(),
            partition_table_type: drive.partition_table_type.clone(),
            gpt_usable_range: drive.gpt_usable_range.clone(),
        }
    }
}
```

### VolumeNode → VolumeInfo (recursive)
```rust
impl From<&VolumeNode> for VolumeInfo {
    fn from(node: &VolumeNode) -> Self {
        Self {
            kind: node.kind,
            label: node.label.clone(),
            size: node.size,
            id_type: node.id_type.clone(),
            device_path: node.device_path.clone(),
            has_filesystem: node.has_filesystem,
            mount_points: node.mount_points.clone(),
            usage: node.usage.clone(),
            locked: node.locked,
            children: node.children.iter().map(Into::into).collect(),
        }
    }
}
```

### VolumeModel → PartitionInfo
```rust
impl From<&VolumeModel> for PartitionInfo {
    fn from(vol: &VolumeModel) -> Self {
        Self {
            device: vol.device_path.clone().unwrap_or_default(),
            number: vol.number,
            parent_device: vol.drive_path.clone(),
            size: vol.size,
            offset: vol.offset,
            type_id: vol.partition_type_id.clone(),
            type_name: vol.partition_type.clone(),
            flags: vol.flags.bits(),
            name: vol.name.clone(),
            uuid: vol.uuid.clone(),
            table_type: vol.table_type.clone(),
            has_filesystem: vol.has_filesystem,
            filesystem_type: if vol.has_filesystem {
                Some(vol.id_type.clone())
            } else {
                None
            },
            mount_points: vol.mount_points.clone(),
            usage: vol.usage.clone(),
        }
    }
}
```

---

## 9. Risk Assessment

### High Risk
- **Breaking API changes**: All downstream code using DriveModel/VolumeNode must update
- **Data loss during conversion**: Must preserve all domain data during From implementations
- **UI state management**: storage-ui currently embeds domain types in UI models

### Medium Risk
- **Connection management**: Must ensure D-Bus operations still have access to Connection
- **Async operations**: All methods returning new types must remain async-compatible
- **Recursive conversions**: VolumeNode tree → VolumeInfo tree must not lose data

### Low Risk
- **Serialization**: Adding serde derives is straightforward
- **Performance**: Conversion overhead is minimal (clone operations)

---

## 10. Success Criteria

### Must Have
- ✅ All storage-models types serializable (Serialize/Deserialize)
- ✅ No D-Bus types (Connection, OwnedObjectPath) in storage-models
- ✅ storage-dbus returns storage-models types from public API
- ✅ storage-ui uses storage-models types directly
- ✅ All existing operations still work (mount, unmount, format, etc.)

### Should Have
- ✅ Helper methods moved to domain types (is_mounted, can_mount, etc.)
- ✅ Comprehensive From implementations for smooth migration
- ✅ No data loss during conversions
- ✅ Clear documentation of new architecture

### Nice to Have
- Additional computed fields on domain types
- Validation methods on domain types
- Builder patterns for complex types

---

## 11. Open Questions (RESOLVED)

1. **Usage type location**: ✅ FOUND at `storage-dbus/src/usage.rs`
   ```rust
   pub struct Usage {
       pub filesystem: String,
       pub blocks: u64,
       pub used: u64,
       pub available: u64,
       pub percent: u32,
       pub mount_point: String,
   }
   ```
   - **Status**: Currently does NOT have serde derives
   - **Action**: Add `#[derive(Serialize, Deserialize, PartialEq, Eq)]`
   - **Location**: Can stay in storage-dbus or move to storage-models

2. **ByteRange location**: ✅ FOUND at `storage-dbus/src/disks/gpt.rs`
   ```rust
   #[derive(Clone, Copy, Debug, PartialEq, Eq)]
   pub struct ByteRange {
       pub start: u64,
       pub end: u64,
   }
   ```
   - **Status**: Already has PartialEq, Eq but missing serde derives
   - **Action**: Add `#[derive(Serialize, Deserialize)]`
   - **Already exported**: Via `storage-dbus/src/disks/mod.rs`

3. **UI state management**: Should storage-ui create wrapper types around VolumeInfo for UI state?
   - **Recommendation**: YES, create UI-specific wrappers
   - **Example**: `struct VolumeModel { info: VolumeInfo, selected: bool, expanded: bool }`
   - **Rationale**: Keeps domain models pure, UI concerns separate

4. **Connection access**: How will internal storage-dbus methods access Connection after refactoring?
   - **Strategy**: Keep internal versions of types WITH connection
   - **Approach**: 
     - Internal: `struct VolumeNodeInternal { info: VolumeInfo, connection: Connection }`
     - Or use DiskManager as context holder, pass connection to methods
   - **Preferred**: Keep current VolumeNode internal with connection, expose VolumeInfo publicly

5. **Caching strategy**: DriveModel caches volumes_flat and volumes - should this continue?
   - **Answer**: YES, keep caching internal to storage-dbus
   - **Internal DriveModel continues** to cache derived data
   - **Public API returns** freshly converted DiskInfo without cache fields

---

## 12. Additional Findings

### Usage Type Details
- Located at `storage-dbus/src/usage.rs`
- Currently exposed publicly via `pub use crate::Usage` in lib.rs
- Used by VolumeNode and VolumeModel for filesystem usage stats
- **Required changes**:
  1. Add serde derives
  2. Keep or move to storage-models (recommend move)

### ByteRange Type Details
- Located at `storage-dbus/src/disks/gpt.rs`
- Already has utility methods (`is_valid_for_disk`, `clamp_to_disk`)
- **Required changes**:
  1. Add serde derives: `#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]`

### Updated storage-models Dependencies
```toml
[dependencies]
serde = { version = "1", features = ["derive"] }
serde_json = "1"  # For JSON serialization examples/tests
```

---

## 13. Next Steps

### Immediate Actions (Task 2)
1. ✅ Analysis complete (this document)
2. **Create storage-models/src/disk.rs**:
   - Define DiskInfo struct with all fields from analysis
   - Add serde derives
   - Implement helper methods (supports_power_management, etc.)
   - Add comprehensive documentation

### Task 3
3. **Create storage-models/src/volume.rs and partition.rs**:
   - Define VolumeInfo (recursive structure)
   - Define VolumeKind enum
   - Define PartitionInfo
   - Define PartitionTableType enum
   - Add helper methods (is_mounted, can_mount, etc.)

### Task 4
4. **Create storage-models/src/filesystem.rs and lvm.rs**:
   - Move Usage from storage-dbus, add serde
   - Define FilesystemInfo
   - Define FormatOptions, MountOptions
   - Define CheckResult, UnmountResult
   - Move ProcessInfo and KillResult, add serde
   - Define VolumeGroupInfo, LogicalVolumeInfo, PhysicalVolumeInfo

### Task 5
5. **Create storage-models/src/encryption.rs**:
   - Define LuksInfo
   - Define LuksVersion enum

### Task 6-11
6. **Refactor storage-dbus**:
   - Add serde to ByteRange in gpt.rs
   - Add From implementations
   - Update method signatures to return storage-models types
   - Keep internal types with Connection
   - Update public API exports

### Task 12
7. **Update storage-ui**:
   - Change imports to storage-models
   - Create UI wrapper types if needed (VolumeModel with UI state)
   - Verify all operations still work

### Documentation Updates
8. **Update READMEs**:
   - storage-models API documentation
   - storage-dbus architecture changes
   - Migration guide for downstream consumers

---

**Document Status:** Complete with all open questions resolved  
**Ready for Implementation:** Yes, proceed to Task 2  
**Estimated Effort:** 2-3 weeks (15 tasks)  
**Last Updated:** 2026-02-14
