# Type Migration Plan: disks-dbus → storage-models

## Executive Summary

This document maps all uses of `DriveModel`, `VolumeModel`, and `VolumeNode` in disks-ui and proposes the migration strategy to storage-models types with minimal UI-specific wrappers.

**Key Principles:**
1. **Service returns flat lists** - UI builds hierarchies from `parent_path` references
2. **Device paths for operations** - No UDisks2 D-Bus paths exposed to UI (internal to service)
3. **UI models own clients** - Each model has its own client instance and refresh method
4. **No UDisks2 concepts in UI** - All D-Bus operations fully encapsulated by storage-service

---

## Current Types Overview

### 1. DriveModel (50+ uses)
**Source:** `disks-dbus/src/disks/drive/model.rs`

**Structure:**
```rust
pub struct DriveModel {
    // Hardware Properties
    pub can_power_off: bool,
    pub ejectable: bool,
    pub media_available: bool,
    pub media_change_detected: bool,
    pub media_removable: bool,
    pub optical: bool,
    pub optical_blank: bool,
    pub removable: bool,
    pub rotation_rate: i32,
    
    // Identity
    pub id: String,
    pub model: String,
    pub revision: String,
    pub serial: String,
    pub vendor: String,
    pub name: String,              // UI display name
    
    // Storage
    pub size: u64,
    pub block_path: String,         // Device path
    pub path: String,               // UDisks2 object path
    
    // Loop Device
    pub is_loop: bool,
    pub backing_file: Option<String>,
    
    // Partitioning
    pub partition_table_type: Option<String>,
    pub gpt_usable_range: Option<ByteRange>,
    
    // UI-Specific Collections (NOT in storage-models)
    pub volumes_flat: Vec<VolumeModel>,   // Flat list of partitions
    pub volumes: Vec<VolumeNode>,         // Hierarchical tree
    
    // Internal
    pub(super) connection: Connection,    // D-Bus connection
}
```

**Key Usage Patterns:**
- `DriveModel::get_drives()` - Initial load and refresh after operations (20+ calls)
- `drive.volumes` - Sidebar tree view, volume lookup
- `drive.volumes_flat` - Partition offset calculation, flat iteration
- `drive.block_path` - Device operations
- `drive.size`, `drive.model`, `drive.vendor` - Display
- `drive.partition_table_type` - Determine GPT vs DOS
- UI stores `Vec<DriveModel>` in multiple places

### 2. VolumeNode (43+ uses)
**Source:** `disks-dbus/src/disks/volume.rs`

**Structure:**
```rust
pub struct VolumeNode {
    pub kind: VolumeKind,              // Partition, CryptoContainer, Filesystem
    pub label: String,
    pub size: u64,
    pub id_type: String,               // "ext4", "crypto_LUKS", etc.
    pub object_path: OwnedObjectPath,  // UDisks2 object path
    pub device_path: Option<String>,   // "/dev/sda1", etc.
    pub has_filesystem: bool,
    pub mount_points: Vec<String>,
    pub usage: Option<Usage>,
    pub locked: bool,
    pub children: Vec<VolumeNode>,     // Recursive tree structure
    
    // Internal
    connection: Option<Connection>,
}
```

**Key Usage Patterns:**
- Tree traversal for sidebar display
- Finding volumes by object_path (recursive search)
- Collecting mounted descendants for unmount operations
- LUKS container/child detection for BTRFS
- Mount/unmount operations (via methods)

### 3. VolumeModel (23+ uses)
**Source:** `disks-dbus/src/disks/volume_model/mod.rs`

**Structure:**
```rust
pub struct VolumeModel {
    pub volume_type: VolumeType,      // Container, Partition, Filesystem
    pub table_path: OwnedObjectPath,  // Parent partition table
    pub name: String,
    pub partition_type_id: String,    // GPT GUID or MBR type
    pub partition_type: String,       // Human-readable
    pub id_type: String,              // Filesystem type
    pub uuid: String,
    pub number: u32,                  // Partition number
    pub flags: BitFlags<PartitionFlags>, // Boot, System, etc.
    pub offset: u64,                  // Byte offset in disk
    pub size: u64,
    pub path: OwnedObjectPath,
    pub device_path: Option<String>,
    pub has_filesystem: bool,
    pub mount_points: Vec<String>,
    pub usage: Option<Usage>,
    
    // Internal
    connection: Option<Connection>,
    pub drive_path: String,
    pub table_type: String,
}
```

**Key Usage Patterns:**
- Partition operations (offset, size, flags, number)
- Dialog state (delete, resize, format)
- Segment creation (visual disk representation)
- Partition type manipulation

---

## Storage-Models Counterparts

### DiskInfo (target for DriveModel)
**Source:** `storage-models/src/disk.rs`

**Provides:**
```rust
pub struct DiskInfo {
    pub device: String,                // ✓ Maps to block_path
    pub id: String,                    // ✓ 
    pub model: String,                 // ✓
    pub serial: String,                // ✓
    pub vendor: String,                // ✓
    pub revision: String,              // ✓
    pub size: u64,                     // ✓
    pub connection_bus: String,        // NEW
    pub rotation_rate: Option<u16>,    // ✓ (was i32)
    pub removable: bool,               // ✓
    pub ejectable: bool,               // ✓
    pub media_removable: bool,         // ✓
    pub media_available: bool,         // ✓
    pub optical: bool,                 // ✓
    pub optical_blank: bool,           // ✓
    pub can_power_off: bool,           // ✓
    pub is_loop: bool,                 // ✓
    pub backing_file: Option<String>,  // ✓
    pub partition_table_type: Option<String>, // ✓
    pub gpt_usable_range: Option<ByteRange>,  // ✓
}
```

**Missing from DiskInfo:**
- ❌ `name` - UI display name (can derive from model/vendor)
- ❌ `path` - UDisks2 object path (UI shouldn't need this)
- ❌ `media_change_detected` - unused in UI
- ❌ `volumes` / `volumes_flat` - Should be queried separately via DisksClient
- ❌ D-Bus connection - Not needed (operations via client)

### VolumeInfo (target for VolumeNode)
**Source:** `storage-models/src/volume.rs`

**Provides:**
```rust
pub struct VolumeInfo {
    pub kind: VolumeKind,              // ✓
    pub label: String,                 // ✓
    pub size: u64,                     // ✓
    pub id_type: String,               // ✓
    pub device_path: Option<String>,   // ✓
    pub has_filesystem: bool,          // ✓
    pub mount_points: Vec<String>,     // ✓
    pub usage: Option<Usage>,          // ✓
    pub locked: bool,                  // ✓
    pub children: Vec<VolumeInfo>,     // ✓ Recursive
}
```

**Missing from VolumeInfo:**
- ❌ `object_path` - UDisks2 path (UI uses for lookups)
- ❌ D-Bus connection - Not needed

### PartitionInfo (potential target for VolumeModel)
**Source:** `storage-models/src/partition.rs`

**Provides:**
```rust
pub struct PartitionInfo {
    pub device: String,                // ✓ Maps to device_path
    pub number: u32,                   // ✓
    pub parent_device: String,         // NEW
    pub size: u64,                     // ✓
    pub offset: u64,                   // ✓
    pub type_id: String,               // ✓ Maps to partition_type_id
    pub type_name: String,             // ✓ Maps to partition_type
    pub flags: u64,                    // ✓ (different from BitFlags)
    pub name: String,                  // ✓
    pub uuid: String,                  // ✓
}
```

**Missing from PartitionInfo:**
- ❌ `volume_type`, `id_type` - Filesystem/container type
- ❌ `has_filesystem`, `mount_points`, `usage` - Runtime state
- ❌ `table_path`, `path` - UDisks2 paths
- ❌ `table_type` - Parent table type
- ❌ D-Bus connection

---

## Proposed UI-Specific Wrappers

### 1. UiDrive (replaces DriveModel in UI)
```rust
use storage_models::{DiskInfo, PartitionInfo};
use crate::client::DisksClient;

/// UI-specific drive representation with owned client
pub struct UiDrive {
    /// Core disk information from storage-service
    pub disk: DiskInfo,
    
    /// Hierarchical volume tree (built by UI, cached)
    pub volumes: Vec<UiVolume>,
    
    /// Flat list of partitions (for offset calculations)
    pub partitions: Vec<PartitionInfo>,
    
    /// Owned client for operations
    client: DisksClient,
}

impl UiDrive {
    /// Create new drive with fresh client
    pub async fn new(disk: DiskInfo) -> Result<Self> {
        let client = DisksClient::new().await?;
        let mut drive = Self {
            disk,
            volumes: Vec::new(),
            partitions: Vec::new(),
            client,
        };
        drive.refresh().await?;
        Ok(drive)
    }
    
    /// Refresh volumes and partitions
    pub async fn refresh(&mut self) -> Result<()> {
        let all_volumes = self.client.list_volumes().await?;
        self.partitions = self.client.list_partitions(&self.disk.device).await?;
        
        // Build tree from flat list
        self.volumes = build_volume_tree(&self.disk.device, all_volumes)?;
        Ok(())
    }
    
    /// Get device path
    pub fn device_path(&self) -> &str {
        &self.disk.device
    }
    
    /// Get display name
    pub fn display_name(&self) -> String {
        if !self.disk.model.is_empty() {
            self.disk.model.clone()
        } else if !self.disk.vendor.is_empty() {
            format!("{} Disk", self.disk.vendor)
        } else {
            self.disk.device.clone()
        }
    }
}

impl Clone for UiDrive {
    fn clone(&self) -> Self {
        // Create new client on clone
        Self {
            disk: self.disk.clone(),
            volumes: self.volumes.clone(),
            partitions: self.partitions.clone(),
            client: DisksClient::new_sync(), // Sync constructor for clone
        }
    }
}

impl UiDrive {
    /// Get display name (model or vendor-based fallback)
    pub fn display_name(&self) -> &str {
        if !self.disk.model.is_empty() {
            &self.disk.model
        } else if !self.disk.vendor.is_empty() {
            // Could format "Vendor Disk" here
            &self.disk.vendor
        } else {
            &self.disk.device
        }
    }
    
    /// Get device path (commonly used)
    pub fn device_path(&self) -> &str {
        &self.disk.device
    }
}
```

### 2. UiVolume (wraps VolumeInfo with children and client)

```rust
use storage_models::VolumeInfo;
use crate::client::FilesystemsClient;

/// UI volume node with children and owned client
#[derive(Clone)]
pub struct UiVolume {
    /// Core volume information from storage-service
    pub volume: VolumeInfo,
    
    /// Child volumes (built by UI)
    pub children: Vec<UiVolume>,
    
    /// Owned client for operations (created on demand)
    #[serde(skip)]
    client: Option<FilesystemsClient>,
}

impl UiVolume {
    pub fn new(volume: VolumeInfo) -> Self {
        Self {
            volume,
            children: Vec::new(),
            client: None,
        }
    }
    
    /// Get or create client
    async fn client(&mut self) -> Result<&FilesystemsClient> {
        if self.client.is_none() {
            self.client = Some(FilesystemsClient::new().await?);
        }
        Ok(self.client.as_ref().unwrap())
    }
    
    /// Mount this volume
    pub async fn mount(&mut self) -> Result<()> {
        let device = self.volume.device_path.as_ref()
            .ok_or_else(|| anyhow!("No device path"))?;
        let client = self.client().await?;
        client.mount(device, "", None).await
    }
    
    /// Unmount this volume
    pub async fn unmount(&mut self) -> Result<()> {
        let device = self.volume.device_path.as_ref()
            .ok_or_else(|| anyhow!("No device path"))?;
        let client = self.client().await?;
        client.unmount(device, HashMap::new()).await
    }
    
    /// Find a volume by device path (recursive)
    pub fn find_by_device<'a>(
        volumes: &'a [UiVolume],
        device_path: &str,
    ) -> Option<&'a UiVolume> {
        for vol in volumes {
            if vol.volume.device_path.as_deref() == Some(device_path) {
                return Some(vol);
            }
            if let Some(found) = Self::find_by_device(&vol.children, device_path) {
                return Some(found);
            }
        }
        None
    }
    
    /// Collect all mounted descendants (for unmount operations)
    pub fn collect_mounted_descendants(&self) -> Vec<&UiVolume> {
        let mut result = Vec::new();
        if !self.volume.mount_points.is_empty() {
            result.push(self);
        }
        for child in &self.children {
            result.extend(child.collect_mounted_descendants());
        }
        result
    }
}
```

### 3. PartitionInfo - Enhanced by Service \u2705

**Decision:** PartitionInfo from storage-models used directly, no wrapper needed.

```rust
// In storage-models/src/partition.rs (enhanced)
pub struct PartitionInfo {
    /// Device path (e.g., "/dev/nvme0n1p2")
    pub device: String,
    
    /// Parent disk device path (e.g., "/dev/nvme0n1")
    pub parent_path: String,
    
    pub number: u32,
    pub size: u64,
    pub offset: u64,
    pub type_id: String,
    pub type_name: String,
    pub flags: u64,
    pub name: String,
    pub uuid: String,
    
    /// Additional filesystem/mount state (from combined queries)
    pub has_filesystem: bool,
    pub filesystem_type: Option<String>,
    pub mount_points: Vec<String>,
    pub usage: Option<Usage>,
}
```

**Usage in UI:**
- Flat list in `UiDrive.partitions` for offset calculations
- Used directly in dialogs and partition operations
- Operations use `device` field (e.g., `partitions_client.delete(&partition.device)`)
- No UDisks2 concepts exposed

---

## Migration Strategy

### Phase 1: Add UI Models with Owned Clients
```
disks-ui/src/models/
├── mod.rs           # Re-export UiDrive, UiVolume
├── drive.rs         # UiDrive with DisksClient
└── volume.rs        # UiVolume with children array, FilesystemsClient
```

**Goals:**
- Create models with owned clients and refresh methods
- Add children arrays to UiVolume
- Implement tree building from parent_path references
- Each model can perform operations independently

### Phase 2: Replace State Storage
**Files to update:**
1. `ui/sidebar/state.rs` - `drives: Vec<DriveModel>` → `drives: Vec<UiDrive>`
2. `ui/volumes/state.rs` - `model: DriveModel` → `drive: UiDrive`
3. `ui/app/message.rs` - `Vec<DriveModel>` → `Vec<UiDrive>`
4. Update all volume/partition references to use device_path

**Key Changes:**
- Replace object_path lookups with device_path lookups
- Use `volume.parent_path` for hierarchy navigation
- Call `drive.refresh()` or `volume.mount()`/`unmount()` directly

### Phase 3: Replace Operations
**Pattern:**
```rust
// OLD: Direct method calls on models
drive.mount().await?;
volume.unmount().await?;

// NEW: Operations via clients (already done!)
let client = FilesystemsClient::new().await?;
client.mount(&device, "", None).await?;
```

This phase is **already complete** from the client migration!

### Phase 4: Replace Data Loading
**OLD:**
```rust
DriveModel::get_drives().await?
```

**NEW:**
```rust
async fn load_drives() -> Result<Vec<UiDrive>> {
    let client = DisksClient::new().await?;
    let disk_infos = client.list_disks().await?;
    
    let mut ui_drives = Vec::new();
    for disk in disk_infos {
        // UiDrive::new() handles client creation and initial refresh
        ui_drives.push(UiDrive::new(disk).await?);
    }
    
    Ok(ui_drives)
}

// After operations - just refresh the affected drive
async fn after_partition_operation(drive: &mut UiDrive) -> Result<()> {
    drive.refresh().await?;  // Fetches fresh data, rebuilds tree
    Ok(())
}
```

**Key simplification:** Each model owns its client and knows how to refresh itself!

### Phase 5: Remove disks-dbus Dependency
- Remove `disks-dbus = { path = "../disks-dbus" }` from `disks-ui/Cargo.toml`
- Remove all `use disks_dbus::` imports
- Keep `disks-dbus` as standalone crate for backward compatibility (if needed)

---

## Implementation Requirements

### Additional Storage-Service APIs Needed

Based on the API analysis and design decisions, storage-service needs:

1. **Flat Volume List with Parent References**:
```rust
// In storage-service D-Bus interface
impl Disks {
    /// Get flat list of all volumes with parent_path
    pub async fn list_volumes(&self) -> Result<Vec<VolumeInfo>> {
        // Returns flat VolumeInfo list with:
        // - parent_path for building hierarchy
        // - device_path for operations
        // - All metadata from partitions, filesystems, encryption combined
    }
}
```

2. **Enhanced VolumeInfo in storage-models**:
```rust
pub struct VolumeInfo {
    pub kind: VolumeKind,
    pub label: String,
    pub size: u64,
    pub id_type: String,
    
    /// Device path for operations (e.g., "/dev/nvme0n1p2")
    pub device_path: Option<String>,
    
    /// Parent device path for hierarchy (e.g., "/dev/nvme0n1")
    pub parent_path: Option<String>,
    
    pub has_filesystem: bool,
    pub mount_points: Vec<String>,
    pub usage: Option<Usage>,
    pub locked: bool,
}
```

3. **Enhanced PartitionInfo in storage-models**:
```rust
pub struct PartitionInfo {
    pub device: String,              // "/dev/nvme0n1p2"
    pub parent_path: String,         // "/dev/nvme0n1"
    pub number: u32,
    pub size: u64,
    pub offset: u64,
    // ... other fields ...
}
```

4. **Client Operations Accept Device Paths**:
```rust
// All operations use device paths (service translates internally)
impl FilesystemsClient {
    pub async fn mount(&self, device: &str, ...) -> Result<()>;
    pub async fn unmount(&self, device: &str, ...) -> Result<()>;
}

impl PartitionsClient {
    pub async fn delete(&self, device: &str) -> Result<()>;
    pub async fn resize(&self, device: &str, size: u64) -> Result<()>;
}
```

### Helper Functions (UI-Side)

```rust
// In disks-ui/src/models/volume.rs

/// Build volume tree from flat list with parent_path references
fn build_volume_tree(
    root_device: &str,
    all_volumes: Vec<VolumeInfo>,
) -> Result<Vec<UiVolume>> {
    // Group by parent_path
    let mut by_parent: HashMap<Option<String>, Vec<VolumeInfo>> = HashMap::new();
    for vol in all_volumes {
        by_parent.entry(vol.parent_path.clone())
            .or_default()
            .push(vol);
    }
    
    // Recursive function to build node with children
    fn build_node(
        volume: VolumeInfo,
        by_parent: &HashMap<Option<String>, Vec<VolumeInfo>>,
    ) -> UiVolume {
        let device = volume.device_path.clone();
        let mut node = UiVolume::new(volume);
        
        if let Some(device) = device {
            if let Some(children) = by_parent.get(&Some(device)) {
                node.children = children.iter()
                    .map(|v| build_node(v.clone(), by_parent))
                    .collect();
            }
        }
        
        node
    }
    
    // Get root volumes (parent_path = root_device)
    let roots = by_parent.get(&Some(root_device.to_string()))
        .cloned()
        .unwrap_or_default();
    
    Ok(roots.into_iter()
        .map(|v| build_node(v, &by_parent))
        .collect())
}
```

**Note:** Tree building happens in UI, using parent_path references from service!

---

## Usage Map by File

### Files Using DriveModel (50+ references)
| File | Usage | Migration Target |
|------|-------|-----------------|
| `ui/app/message.rs` | `UpdateNav(Vec<DriveModel>)` | `UpdateNav(Vec<UiDrive>)` |
| `ui/sidebar/state.rs` | `drives: Vec<DriveModel>` | `drives: Vec<UiDrive>` |
| `ui/volumes/state.rs` | `model: DriveModel` | `drive: UiDrive` |
| `ui/app/update/*.rs` | `DriveModel::get_drives()` | `load_drives()` helper |
| `ui/dialogs/state.rs` | Dialog payload | Use UiDrive |
| `ui/sidebar/view.rs` | Sidebar rendering | Use UiDrive |
| `ui/volumes/disk_header.rs` | Header display | Use UiDrive |

### Files Using VolumeNode (43+ references)
| File | Usage | Migration Target |
|------|-------|-----------------|
| `ui/volumes/helpers.rs` | Tree traversal | UiVolume with children |
| `ui/sidebar/view.rs` | Tree display | UiVolume.children recursion |
| `ui/app/view.rs` | Volume operations | `volume.mount()`, `volume.unmount()` |
| `ui/volumes/view.rs` | Volume list | UiVolume iteration |
| `ui/app/update/mod.rs` | Find by path | `UiVolume::find_by_device()` |
| `ui/volumes/update/partition.rs` | Mounted children | `volume.collect_mounted_descendants()` |
| `ui/volumes/update/encryption.rs` | LUKS operations | UiVolume with owned client |

### Files Using VolumeModel (23+ references)
| File | Usage | Migration Target |
|------|-------|-----------------|
| `ui/volumes/state.rs` | `Segment.volume` | PartitionInfo with device_path |
| `ui/dialogs/state.rs` | Dialog payloads | PartitionInfo |
| `ui/volumes/helpers.rs` | Partition lookup | Search by device_path |
| `ui/volumes/update/partition.rs` | Partition operations | PartitionInfo |
| `ui/app/view.rs` | Volume info display | PartitionInfo metadata |
| `ui/volumes/helpers.rs` | Partition lookup | UiPartition list |
| `ui/volumes/update/partition.rs` | Flags manipulation | UiPartition |
| `ui/app/view.rs` | Volume info display | UiPartition or UiVolume |

---

## Implementation Checklist

### Storage-Service Changes (REQUIRED FIRST!) ✅ **COMPLETE**
- [x] Add `parent_path: Option<String>` field to `VolumeInfo` ✅
- [x] Add `parent_path: String` field to `PartitionInfo` (renamed from parent_device) ✅
- [x] Implement `list_volumes()` method that returns flat list with parent references ✅
- [x] Implement `get_volume_info()` method for atomic updates ✅
- [x] Ensure all clients accept device paths (not D-Bus object paths) ✅
- [x] Internal translation: device path → UDisks2 object path in service ✅
- [ ] Add `Usage` calculation to filesystem queries (deferred - current Usage already works)
- [x] Remove UDisks2-specific fields from public API (keep internal) ✅

**Files Modified in Phase 0:**
- `storage-models/src/volume.rs`: Added `parent_path: Option<String>` field
- `storage-models/src/partition.rs`: Renamed `parent_device` → `parent_path`
- `storage-service/src/disks.rs`: Added `list_volumes()` and `get_volume_info()` methods
- `disks-dbus/src/disks/volume.rs`: Updated VolumeNode→VolumeInfo conversion
- `disks-dbus/src/disks/volume_model/mod.rs`: Updated PartitionInfo conversion to use parent_path

**Key Implementation Details:**
- `list_volumes()` recursively flattens volume trees from all drives, populating parent_path at each level
- `get_volume_info()` supports atomic updates by querying a single volume with parent_path populated
- Both methods use device paths only (e.g., "/dev/sda1") - no UDisks2 paths exposed
- Parent references: partition→disk, unlocked LUKS→locked partition, etc.
- All changes compile successfully with zero new warnings

### Required New Code (disks-ui) ✅ **PHASE 1 COMPLETE**
- [x] Create `disks-ui/src/models/` module ✅
- [x] Implement `UiDrive` with owned `DisksClient` and `refresh()` ✅
- [x] Implement `UiVolume` with `children: Vec<UiVolume>` and owned `FilesystemsClient` ✅
- [x] Implement `build_volume_tree()` helper to construct hierarchy from parent_path ✅
- [x] Add helper methods to `UiVolume` (find_by_device, find_by_device_mut, collect_mounted_descendants) ✅
- [x] Add atomic update methods: `refresh_volume()`, `update_volume()` ✅
- [x] Add tree mutation methods: `add_partition()`, `remove_partition()`, `add_child()`, `remove_child()` ✅
- [x] Add `validate_tree()` for debug assertions ✅
- [ ] Implement `Clone` for UI models - **DEFERRED** (clients can't be cloned, UI should use references/Arc)
- [ ] Remove any UDisks2 direct calls from UI code - **PHASE 2**

**Files Created in Phase 1:**
- `disks-ui/src/models/mod.rs`: Module exports
- `disks-ui/src/models/ui_drive.rs`: UiDrive with owned DisksClient and PartitionsClient
- `disks-ui/src/models/ui_volume.rs`: UiVolume with owned FilesystemsClient
- `disks-ui/src/models/helpers.rs`: build_volume_tree() and validate_tree()
- `disks-ui/src/client/disks.rs`: Added list_volumes() and get_volume_info() methods

**Key Implementation Details:**
- UiDrive owns DisksClient and PartitionsClient for independent data refresh
- UiVolume owns FilesystemsClient for filesystem operations
- Tree building uses tokio::Runtime::block_on for synchronous context
- Atomic updates: refresh_volume() in UiDrive, update_volume() in UiVolume
- Tree mutations: add_partition/remove_partition in UiDrive, add_child/remove_child in UiVolume
- Recursive helpers: find_by_device(), collect_mounted_descendants()
- No Clone implementation (clients contain non-cloneable connections)
- All code compiles successfully with zero new warnings

### State Updates
- [ ] Replace `Sidebar.drives: Vec<DriveModel>` with `Vec<UiDrive>`
- [ ] Replace `VolumesControl.model: DriveModel` with `drive: UiDrive`
- [ ] Update `Message::UpdateNav` signature
- [ ] Update all dialog state structs to use `VolumeInfo` and `PartitionInfo`

### View Updates
- [ ] Update sidebar tree rendering for UiDrive/UiVolume hierarchy
- [ ] Update disk header for UiDrive
- [ ] Update volume list for UiVolume with children
- [ ] Update partition UI for PartitionInfo
- [ ] Use `volume.device_path` instead of object paths everywhere

### Helper Function Updates
- [ ] Add `UiVolume::find_by_device()` helper method (recursive)
- [ ] Add `UiVolume::collect_mounted_descendants()` helper method
- [ ] Implement `build_volume_tree()` using parent_path references
- [ ] Migrate `detect_btrfs_in_node()` to work with UiVolume
- [ ] Update all volume lookup functions to use device_path instead of object_path

### Data Loading
- [ ] Replace all `DriveModel::get_drives()` calls with `UiDrive::new()`
- [ ] Use `UiDrive::refresh()` after operations instead of full reload
- [ ] Load volumes using `client.list_volumes()` (flat)
- [ ] Build trees in UI using `build_volume_tree()`

### Cleanup
- [ ] Remove `use disks_dbus::DriveModel`
- [ ] Remove `use disks_dbus::VolumeModel`
- [ ] Remove `use disks_dbus::VolumeNode`
- [ ] Remove disks-dbus dependency from disks-ui/Cargo.toml
- [ ] Add `storage_models` dependency to disks-ui/Cargo.toml
- [ ] Update imports to use `storage_models::{DiskInfo, VolumeInfo, PartitionInfo}`
- [ ] Remove any direct zbus/UDisks2 calls from UI code

---

## Testing Strategy

1. **Incremental Conversion:** Convert one component at a time
2. **Parallel Implementation:** New UI models coexist with old during transition
3. **Baseline Testing:** Verify full refresh works for all operations
4. **Atomic Update Testing:** Test each atomic update independently
   - Property updates preserve tree structure
   - Tree mutations maintain consistency
   - Validation catches errors in debug mode
5. **Performance Testing:** Measure query counts before/after atomic updates
6. **End-to-End Testing:** Verify each operation after migration:
   - Sidebar display
   - Volume selection preservation across updates
   - Mount/unmount (atomic path)
   - Partition operations (atomic + full refresh paths)
   - LUKS operations (atomic path)
   - BTRFS management
7. **UI State Preservation:** Verify selection/scroll/expansion maintained after atomic updates

---

## Timeline Estimate

- **Phase 0** (storage-service enhancements): ✅ **COMPLETE** (1 day)
  - ✅ Added parent_path fields to VolumeInfo and PartitionInfo
  - ✅ Implemented flat list_volumes() with parent references
  - ✅ Implemented get_volume_info() for atomic updates
  - ✅ Ensured all methods accept device paths (not UDisks2 paths)
  - ✅ Updated disks-dbus conversions to populate parent_path
- **Phase 1** (UI Models): ✅ **COMPLETE** (1 day)
  - ✅ Created models/ module with UiDrive, UiVolume, helpers
  - ✅ Implemented owned clients (DisksClient, FilesystemsClient, PartitionsClient)
  - ✅ Implemented build_volume_tree() helper
  - ✅ Added helper methods (find_by_device, collect_mounted_descendants, etc.)
  - ✅ Implemented atomic update methods (refresh_volume, update_volume)
  - ✅ Implemented tree mutation methods (add/remove partition/child)
  - ✅ All code compiles successfully
- **Phase 2** (State Migration): **IN PROGRESS** (expect 3-4 days)
  - ✅ Updated Message types to use Vec<UiDrive>
  - ✅ Updated SidebarState to use Vec<UiDrive>
  - ✅ Updated update_nav() to work with UiDrive
  - ✅ Restructured VolumesControl (removed .model field)
  - ✅ Updated Segment to use VolumeInfo/PartitionInfo
  - ✅ Replaced DriveModel::get_drives() → load_all_drives()
  - ✅ Updated helper functions (find_volume_*, detect_btrfs_*)
  - ✅ Added Clone impl for UiDrive/UiVolume (creates new clients)
  - ✅ Added Debug impl for client types
  - 🔄 **Current:** Fixing 155 compilation errors
    - Field access migrations (`.path` → `.device_path`, `.model.*` → direct access)
    - Type conversion errors (error handling for new types)
    - Dialog states still using old types
    - Message handlers using obsolete field names
  - ⏳ Remaining: Update all view/render code
- **Phase 3** (Operations): Already complete! ✅
- **Phase 4** (Data Loading with Full Refresh): 1-2 days
  - Use full refresh uniformly (simple baseline)
- **Phase 5** (Atomic Updates - Mount/Unmount): 1 day
  - Update mount/unmount handlers to use refresh_volume()
- **Phase 6** (Atomic Updates - Tree Mutations): 1-2 days
  - Update create/delete/LUKS handlers
- **Phase 7** (Cleanup): 1 day

**Total:** ~10-12 days of focused development

**Progress:** Phase 0 + Phase 1 complete (2 days)
- **Phase 2** (State Migration): 2-3 days
- **Phase 3** (Operations): Already complete! ✅
- **Phase 4** (Data Loading with Full Refresh): 1-2 days
  - Use full refresh uniformly (simple baseline)
- **Phase 5** (Atomic Updates - Mount/Unmount): 1 day
  - Add refresh_self() to UiVolume
  - Implement refresh_volume() in UiDrive
  - Update mount/unmount handlers
- **Phase 6** (Atomic Updates - Tree Mutations): 1-2 days
  - Implement add_partition(), remove_partition()
  - Implement add_child(), remove_child()
  - Update create/delete/LUKS handlers
- **Phase 7** (Cleanup): 1 day

**Total:** ~10-12 days of focused development

**Benefits of This Approach:**
- Simpler service API (flat lists)
- UI controls tree structure
- Cleaner client ownership (each model has its own)
- No UDisks2 concepts in UI layer
- Incremental optimization path (start simple, optimize hot paths)
- Atomic updates provide 3-5x performance gains for common operations

---

## Key Benefits Summary

### 1. Cleaner Separation of Concerns
- **storage-service**: Returns flat lists with `parent_path` references, accepts device paths
- **disks-ui**: Builds trees, owns display logic, manages client lifecycle

### 2. Simplified Client Management
```rust
// Before: Pass clients everywhere
async fn some_operation(client: &FilesystemsClient, device: &str) { ... }
let result = some_operation(&client, device).await?;

// After: Models own their clients
async fn some_operation(volume: &mut UiVolume) { ... }
volume.mount().await?;  // Internally uses owned client
```

### 3. No UDisks2 Leakage
- UI never sees D-Bus object paths
- All operations use device paths (e.g., `/dev/nvme0n1p2`)
- Service handles UDisks2 translation internally

### 4. Flexible Tree Building  
- UI constructs tree structure based on needs
- Parent references enable any hierarchy
- Can optimize for specific views (sidebar vs flat list)

### 5. Independent Model Operations
```rust
// Each model can refresh itself
drive.refresh().await?;

// Each model can perform operations
volume.mount().await?;
volume.unmount().await?;

// Clone creates new client instances
let drive2 = drive.clone();  // Independent client
```

### 6. Simplified Testing
- Mock clients at model level
- Test tree building independently
- No need to mock D-Bus infrastructure

---

## Design Decisions (Resolved)

### 1. Path Strategy ✅
**Decision:** Only device paths exposed to UI. UDisks2 D-Bus paths internal to service.

**Rationale:**
- UI doesn't need to know about D-Bus object paths
- Service translates device paths → object paths internally
- Cleaner separation of concerns
- Removes UDisks2 concepts from UI layer

**Storage-Models:**
```rust
// In storage-models/src/volume.rs
pub struct VolumeInfo {
    pub kind: VolumeKind,
    pub label: String,
    pub size: u64,
    pub id_type: String,
    
    /// Device path (e.g., "/dev/nvme0n1p2") - used for operations
    pub device_path: Option<String>,
    
    /// Parent device path for building hierarchy
    /// - For partition: parent disk device (e.g., "/dev/nvme0n1")
    /// - For unlocked LUKS: parent partition (e.g., "/dev/nvme0n1p2")
    pub parent_path: Option<String>,
    
    pub has_filesystem: bool,
    pub mount_points: Vec<String>,
    pub usage: Option<Usage>,
    pub locked: bool,
}

pub struct PartitionInfo {
    /// Device path (e.g., "/dev/nvme0n1p2")
    pub device: String,
    
    /// Parent disk device path (e.g., "/dev/nvme0n1")
    pub parent_path: String,
    
    pub number: u32,
    pub size: u64,
    pub offset: u64,
    // ... other fields ...
}
```

**Service Operations Accept Device Paths:**
```rust
// UI calls with device paths
filesystems_client.mount("/dev/nvme0n1p2", "", None).await?;
partitions_client.delete("/dev/nvme0n1p2").await?;

// Service translates internally:
// "/dev/nvme0n1p2" → "/org/freedesktop/UDisks2/block_devices/nvme0n1p2"
```

### 2. Volume Hierarchy Strategy ✅
**Decision:** Service returns FLAT lists with `parent_path` references. UI builds trees.

**Rationale:** 
- Keep service APIs simple (flat lists)
- UI controls tree structure based on its needs
- Parent references enable flexible tree building
- Service doesn't need to know UI's tree representation

**Service-Side Implementation:**
```rust
// In storage-service - returns flat lists with parent references
impl Disks {
    pub async fn list_partitions(&self, disk: &str) -> Result<Vec<PartitionInfo>> {
        // Returns flat list, each with parent_path = disk
    }
    
    pub async fn list_volumes(&self) -> Result<Vec<VolumeInfo>> {
        // Returns flat list with parent_path for each:
        // - Partition: parent_path = disk device
        // - Unlocked LUKS: parent_path = partition device
        // - LVM LV: parent_path = VG path
    }
}
    
**UI-Side Tree Building:**
```rust
// In disks-ui - build tree from flat list
async fn build_volume_tree(disk: &str) -> Result<Vec<UiVolume>> {
    let client = DisksClient::new().await?;
    let volumes = client.list_volumes().await?;
    
    // Filter volumes for this disk and build tree
    let disk_volumes: Vec<_> = volumes.into_iter()
        .filter(|v| v.parent_path.as_deref() == Some(disk) || 
                    is_descendant_of(v, disk, &volumes))
        .collect();
    
    // Build tree structure
    let mut tree_map: HashMap<Option<String>, Vec<UiVolume>> = HashMap::new();
    
    for vol in disk_volumes {
        let parent = vol.parent_path.clone();
        tree_map.entry(parent)
            .or_default()
            .push(UiVolume::new(vol));
    }
    
    // Recursively attach children
    fn attach_children(
        node: &mut UiVolume, 
        tree_map: &HashMap<Option<String>, Vec<UiVolume>>
    ) {
        if let Some(device) = &node.volume.device_path {
            if let Some(children) = tree_map.get(&Some(device.clone())) {
                node.children = children.clone();
                for child in &mut node.children {
                    attach_children(child, tree_map);
                }
            }
        }
    }
    
    let mut roots = tree_map.remove(&Some(disk.to_string())).unwrap_or_default();
    for root in &mut roots {
        attach_children(root, &tree_map);
    }
    
    Ok(roots)
}
```

### 3. Cache Strategy ✅
**Decision:** Hybrid - cache with explicit refresh via owned client.

**Implementation:**
```rust
pub struct UiDrive {
    pub disk: DiskInfo,
    pub volumes: Vec<UiVolume>,
    pub partitions: Vec<PartitionInfo>,
    client: DisksClient,  // Owned client!
}

impl UiDrive {
    /// Create with initial data load (client created here)
    pub async fn new(disk: DiskInfo) -> Result<Self> {
        let client = DisksClient::new().await?;
        let mut drive = Self {
            disk,
            volumes: Vec::new(),
            partitions: Vec::new(),
            client,
        };
        drive.refresh().await?;
        Ok(drive)
    }
    
    /// Refresh volumes/partitions using owned client
    pub async fn refresh(&mut self) -> Result<()> {
        let all_volumes = self.client.list_volumes().await?;
        self.partitions = self.client.list_partitions(&self.disk.device).await?;
        
        // Build tree from flat list with parent_path
        self.volumes = build_volume_tree(&self.disk.device, all_volumes)?;
        Ok(())
    }
}

impl Clone for UiDrive {
    fn clone(&self) -> Self {
        Self {
            disk: self.disk.clone(),
            volumes: self.volumes.clone(),
            partitions: self.partitions.clone(),
            client: DisksClient::new_sync(),  // New client on clone!
        }
    }
}

// In update handlers:
async fn after_partition_operation(drive: &mut UiDrive) {
    drive.refresh().await?;  // Uses owned client
}
```

**Benefits:**
- Each model owns its data fetching logic
- No need to pass clients around
- Clone creates new client instance (independent)
- Simple refresh pattern after operations

### 4. Update Strategy ✅
**Decision:** Full refresh on any device change

**Implementation:**
```rust
async fn reload_all_drives() -> Result<Vec<UiDrive>> {
    let client = DisksClient::new().await?;
    let disk_infos = client.list_disks().await?;
    
    let mut drives = Vec::new();
    for disk in disk_infos {
        drives.push(UiDrive::new(disk).await?);
    }
    
    Ok(drives)
}

// Called after: format, partition, mount, unmount, etc.
async fn handle_operation_complete() -> Message {
    match reload_all_drives().await {
        Ok(drives) => Message::UpdateNav(drives),
        Err(e) => Message::Error(e),
    }
}
```

**Rationale:**
- Simple, reliable (no complex diff logic)
- Matches current behavior
- Avoids synchronization bugs
- Fast enough for typical use (1-10 drives)
- Can optimize later if needed

---

## Service-UI Contract

### What storage-service MUST provide:

1. **Flat Lists with Parent References**
   - `list_volumes()` returns `Vec<VolumeInfo>` with `parent_path` field
   - `list_partitions(disk)` returns flat list with `parent_path = disk`
   - Parent references enable UI to build any tree structure it needs

2. **Device Paths Only (No D-Bus Paths)**
   - All models use device paths (e.g., `/dev/nvme0n1p2`)
   - `parent_path` uses device paths (e.g., `/dev/nvme0n1`)
   - UDisks2 D-Bus paths internal to service (not exposed)

3. **Operations Accept Device Paths**
   - All client methods accept device paths
   - Service translates device → D-Bus path internally
   - Example: `mount("/dev/nvme0n1p2", ...)` not `mount("/org/freedesktop/...", ...)`

4. **Complete Metadata**
   - Volume kind (Partition, CryptoContainer, Filesystem)
   - Filesystem type, label, UUID
   - Mount points and usage statistics
   - Lock status for encrypted volumes
   - All partition metadata (number, offset, size, flags, type)

### What the UI provides:

1. **UI Models with Owned Clients**
   - `UiDrive` owns `DisksClient`, has `refresh()` method
   - `UiVolume` owns `FilesystemsClient`, has children array
   - Each model can perform operations independently

2. **Tree Building Logic**
   - Build hierarchy from flat list using `parent_path` references
   - `UiVolume.children: Vec<UiVolume>` constructed by UI
   - Flexible tree structure based on UI needs

3. **Display Logic**
   - Render trees using `UiVolume.children`
   - Format sizes, names, types for display
   - Handle errors and confirmation dialogs

### What the UI does NOT do:

- ❌ Make direct UDisks2 D-Bus calls
- ❌ Know about UDisks2 object paths
- ❌ Query UDisks2 properties directly
- ❌ Handle D-Bus path resolution

---
