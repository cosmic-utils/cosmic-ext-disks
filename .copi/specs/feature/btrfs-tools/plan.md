# BTRFS Management Tools — Plan

**Branch:** `feature/btrfs-mgmt`  
**Source:** User request (implements README V1 goal #3)  
**Status:** Spec created, ready for implementation (uses existing overlay dialogs)

---

## Context

The application currently supports BTRFS as a filesystem type for formatting, but provides no BTRFS-specific management features beyond basic mounting/unmounting. Users with BTRFS filesystems cannot:
- View or manage subvolumes
- Create or manage snapshots
- See BTRFS-specific usage breakdown (data/metadata/system allocation)
- View or configure compression settings

This represents a significant gap compared to GNOME Disks and other disk utilities that offer "first-class BTRFS support." Many Linux users choose BTRFS for advanced features like snapshots and subvolumes, making these capabilities essential for parity with established tools.

**Referenced Documentation:**
- README V1 goal #3: [README.md](../../../README.md#L27) — "1st class BTRFS support - Subvolumes CRUD, and snapshotting maybe"
- Current BTRFS support: Formatonly, via [disks-ui/src/utils/fs_tools.rs](../../../disks-ui/src/utils/fs_tools.rs) (detection) and partition type catalog
- Volume detail view: [disks-ui/src/ui/app/view.rs](../../../disks-ui/src/ui/app/view.rs#L280-L1000) — where BTRFS section will integrate

**Dependency Note:** Originally blocked on modal-dialogs feature. Since that feature is now deferred pending upstream libcosmic support, this implementation will use the existing overlay dialog system (via `Application::dialog()`).

---

## Goals

1. **Detection**: Automatically detect when selected filesystem is BTRFS
2. **UI Section**: Add dedicated "BTRFS Management" section to volume detail view
3. **Subvolume Operations (CRUD)**:
   - List all subvolumes with ID, path, and mount status
   - Create new subvolumes (with name input)
   - Delete subvolumes (with confirmation)
   - Set default subvolume (optional stretch goal)
4. **Snapshot Management**:
   - Create snapshots of subvolumes (read-only or writable)
   - List existing snapshots with creation time
   - Delete snapshots
5. **Usage Breakdown**:
   - Show BTRFS-specific allocation (data, metadata, system)
   - Display current usage vs. total for each category
   - Visual representation (bar chart or text Summary)
6. **Compression Info**:
   - Display current compression algorithm (zlib, lzo, zstd, none)
   - Show compression as read-only info (no on-the-fly changes)

---

## Non-Goals

- **BTRFS advanced features**: No scrub operations, balance, device management, RAID profile changes, or send/receive
- **BTRFS filesystem creation options**: No special BTRFS options in partition creation dialog (use existing type selection)
- **Mount option presets**: No BTRFS-specific mount option templates beyond what's in edit-mount-options
- **Other advanced filesystems**: No ZFS, LVM thin provisioning, or bcachefs support
- **Quota management**: No quota groups or limits (future enhancement if needed)
- **Performance tuning**: No defragmentation, dedupe status, or performance metrics

---

## Proposed Approach

### A) BTRFS Detection

**Method:**
- Check `volume.id_type == "btrfs"` in volume detail view
- Verify mount point exists (most operations require mounted filesystem)

**Where:**
- In `build_partition_info()` and `build_volume_node_info()` functions in [disks-ui/src/ui/app/view.rs](../../../disks-ui/src/ui/app/view.rs)

---

### B) BTRFS CLI Wrapper Module

**Location:** New module `disks-ui/src/utils/btrfs.rs` or `disks-dbus/src/btrfs.rs`

**Functions:**
```rust
pub async fn list_subvolumes(mount_point: &str) -> Result<Vec<Subvolume>>;
pub async fn create_subvolume(mount_point: &str, name: &str) -> Result<()>;
pub async fn delete_subvolume(path: &str) -> Result<()>;
pub async fn get_filesystem_usage(mount_point: &str) -> Result<UsageInfo>;
pub async fn get_compression(mount_point: &str) -> Result<Option<String>>;
pub fn command_exists() -> bool;

pub struct Subvolume {
    pub id: u64,
    pub path: String,
    pub name: String,
}

pub struct UsageInfo {
    pub data_used: u64,
    pub data_total: u64,
    pub metadata_used: u64,
    pub metadata_total: u64,
    pub system_used: u64,
    pub system_total: u64,
}
```

**Commands used:**
- `btrfs subvolume list <mount_point>` — enumerate subvolumes
- `btrfs subvolume create <path>` — create subvolume
- `btrfs subvolume delete <path>` — delete subvolume
- `btrfs subvolume snapshot [-r] <source> <dest>` — create snapshot
- `btrfs filesystem usage <mount_point>` — usage breakdown
- `btrfs property get <mount_point> compression` — compression setting

**Error handling:**
- Command not found: "BTRFS tools not installed. Install btrfs-progs package."
- Permission denied: "Permission denied. Root access required for this operation."
- Parsing failure: "Unable to parse BTRFS output. Please report this issue."

---

### C) UI Integration

**Location:** Add expandable section in volume detail view, below filesystem info and above action buttons

**Components:**
1. **Section header**: "BTRFS Management" (collapsible)
2. **Subvolumes list**:
   - Scrollable table with columns: Name/Path, ID, Actions (delete icon)
   - "Create Subvolume" button
3. **Snapshots section** (within subvolumes, or separate):
   - Similar list with: Name, Source, Created, Read-only status
   - "Create Snapshot" button
4. **Usage breakdown**:
   - Text summary or horizontal bar chart
   - Format: "Data: 45.2 GB / 100 GB (45%) | Metadata: 2.1 GB / 5 GB (42%) | System: 16 MB / 32 MB (50%)"
5. **Compression info**:
   - Simple text: "Compression: zstd" or "Compression: disabled"

**UI module structure:**
- New module: `disks-ui/src/ui/btrfs/` with `mod.rs`, `view.rs`, `state.rs`, `message.rs`
- Integration point: Called from `build_partition_info()` when BTRFS detected

---

### D) Dialogs for BTRFS Operations

**Note:** This spec assumes modal dialog windows have been implemented (feature/modal-dialogs branch). All BTRFS dialogs will be modal windows.

**Dialogs needed:**
1. **Create Subvolume**: Text input for name, Create/Cancel buttons
2. **Delete Subvolume**: Confirmation dialog (use generic `ConfirmAction`)
3. **Create Snapshot**: Dropdown for source subvolume, text input for name, read-only checkbox

**Dialog state enum additions:**
```rust
pub enum ShowDialog {
    // ... existing variants
    BtrfsCreateSubvolume(BtrfsCreateSubvolumeDialog),
    BtrfsCreateSnapshot(BtrfsCreateSnapshotDialog),
}

pub struct BtrfsCreateSubvolumeDialog {
    pub mount_point: String,
    pub name: String,
    pub running: bool,
    pub error: Option<String>,
}

pub struct BtrfsCreateSnapshotDialog {
    pub mount_point: String,
    pub subvolumes: Vec<Subvolume>,
    pub selected_source_index: usize,
    pub snapshot_name: String,
    pub read_only: bool,
    pub running: bool,
    pub error: Option<String>,
}
```

---

### E) Permissions and Polkit

**Challenge:** BTRFS operations typically require root or appropriate capabilities.

**Approaches:**
1. **UDisks2 integration** (preferred): Check if UDisks2 exposes BTRFS D-Bus methods
2. **Direct CLI with polkit** (fallback): Run `btrfs` commands via pkexec or polkit rules
3. **Read-only operations**: Listing subvolumes may work without elevation

**Implementation:**
- If UDisks2 lacks BTRFS support, document required polkit rules for packagers
- Provide clear error messages when operations fail due to permissions

---

## User/System Flows

### Flow 1: User Views BTRFS Subvolumes
1. User selects a BTRFS filesystem volume in sidebar
2. Volume detail view shows standard info + **"BTRFS Management" section**
3. Section loads and displays:
   - List of subvolumes (e.g., @, @home, @snapshots)
   - "Create Subvolume" button
   - Usage breakdown chart
4. User sees subvolume names, IDs, and can identify their setup

### Flow 2: User Creates BTRFS Subvolume
1. From BTRFS Management section, user clicks "Create Subvolume"
2. **Modal dialog** opens asking for subvolume name
3. User enters name (e.g., "mydata"), clicks "Create"
4. System runs `btrfs subvolume create /mnt/btrfs_mount/mydata`
5. If permission needed, polkit prompt appears
6. On success, dialog closes, subvolume list refreshes automatically
7. New subvolume appears in list

### Flow 3: User Deletes BTRFS Subvolume
1. User hovers over subvolume in list, clicks delete icon (trash can)
2. **Modal confirmation dialog** appears: "Delete subvolume 'mydata'? This cannot be undone."
3. User confirms
4. System runs `btrfs subvolume delete /mnt/btrfs_mount/mydata`
5. Operation completes, list refreshes, subvolume removed

### Flow 4: User Creates BTRFS Snapshot
1. User clicks "Create Snapshot" button in BTRFS section
2. **Modal dialog** opens with:
   - Source subvolume dropdown (populated from subvolume list)
   - Snapshot name input
   - "Read-only" checkbox (checked by default)
3. User selects source (e.g., @home), enters name (e.g., "home-backup-2026-02-12"), leaves read-only checked
4. User clicks "Create"
5. System runs `btrfs subvolume snapshot -r /mnt/btrfs_mount/@home /mnt/btrfs_mount/home-backup-2026-02-12`
6. Snapshot appears in subvolume list (snapshots are subvolumes in BTRFS)

### Flow 5: User Views BTRFS Usage Breakdown
1. BTRFS Management section automatically displays:
   - Text or chart: `Data: 45GB / 100GB | Metadata: 2GB / 5GB | System: 16MB / 32MB`
   - Below: "Compression: zstd"
2. Info is read-only, no user action required
3. User understands BTRFS allocation (helps diagnose "disk full" issues where data shows space but metadata is full)

---

## Risks & Mitigations

### Risk 1: BTRFS Command Parsing Fragility
**Risk:** `btrfs` command output format changes between versions, causing parsing errors.

**Mitigation:**
- Use stable command formats (same ones GNOME Disks uses)
- Extensive error handling with fallback to "raw output" display
- Unit tests for parsing known output samples from different btrfs-progs versions
- Graceful degradation: if parsing fails, show "Unable to parse BTRFS info" with raw output in details

### Risk 2: BTRFS Operations Require Root
**Risk:** All BTRFS subvolume operations may require root, leading to constant polkit prompts.

**Mitigation:**
- Research polkit rules for BTRFS operations
- Check if UDisks2 already provides some D-Bus interfaces (unlikely but worth checking)
- Document required polkit rules for users/packagers
- Consider read-only subvolume listing without elevation (test if supported)
- Provide clear error messages when operations fail due to permissions

### Risk 3: Complex BTRFS Setups
**Risk:** Nested subvolumes, multiple devices, RAID profiles could produce unexpected UI states.

**Mitigation:**
- Start with simple single-device BTRFS (most common case)
- Display raw command output for complex edge cases ("Advanced BTRFS setup detected")
- Incremental feature support (ship basic subvolume CRUD first, enhance later)
- Test on various BTRFS configurations

### Risk 4: Dependency on Modal Dialogs Feature
**Risk:** This feature assumes modal dialog windows are implemented. If that feature is delayed or blocked, BTRFS dialogs cannot be created.

**Mitigation:**
- Implement modal dialogs first (feature/modal-dialogs branch)
- If blocked, could temporarily use overlay dialogs for BTRFS (not ideal but allows progress)
- Coordinate implementation: merge modal dialogs before starting BTRFS work

### Risk 5: BTRFS Tools Not Installed
**Risk:** Users may not have `btrfs-progs` package installed, breaking all features.

**Mitigation:**
- Detect `btrfs` command availability at startup
- Show clear message in BTRFS section: "BTRFS tools not installed. Install btrfs-progs package."
- Don't crash or show errors, just gracefully disable features
- Add detection to existing fs_tools.rs module

---

## Acceptance Criteria

### Detection & UI
- [ ] BTRFS filesystems are detected (check `id_type == "btrfs"`)
- [ ] "BTRFS Management" section appears in volume detail view only for BTRFS volumes
- [ ] Section does not appear for non-BTRFS filesystems
- [ ] Section is collapsible/expandable

### Subvolume Management
- [ ] Subvolume list displays with columns: name/path, ID
- [ ] "Create Subvolume" button opens modal dialog
- [ ] Create subvolume dialog has name input field and validation (non-empty, no slashes)
- [ ] Subvolume creation succeeds andlist refreshes
- [ ] Delete subvolume icon shows on each row
- [ ] Delete shows confirmation modal dialog
- [ ] Subvolume deletion succeeds and list refreshes
- [ ] Cannot delete mounted subvolumes (error or disabled button)

### Snapshot Management
- [ ] "Create Snapshot" button opens modal dialog
- [ ] Dialog has source dropdown populated from subvolume list
- [ ] Dialog has snapshot name input field
- [ ] Dialog has "Read-only" checkbox
- [ ] Snapshot creation succeeds for read-only snapshots
- [ ] Snapshot creation succeeds for writable snapshots
- [ ] Snapshots appear in subvolume list after creation

### Usage Breakdown
- [ ] Usage info loads asynchronously when section expands
- [ ] Data/metadata/system allocation displayed clearly (text or chart)
- [ ] Values match `btrfs filesystem usage <mount>` output
- [ ] Compression info displayed (algorithm name or "disabled")

### Error Handling
- [ ] All BTRFS operations show appropriate errors for permission failures
- [ ] Clear error message if `btrfs` command not installed
- [ ] Parsing errors handled gracefully (don't crash, show friendly message)
- [ ] Operations refresh relevant UI sections on completion

### Localization & Quality
- [ ] All BTRFS UI strings localized in `en/cosmic_ext_disks.ftl`
- [ ] Swedish translations added in `sv/cosmic_ext_disks.ftl`
- [ ] No compiler warnings (`cargo clippy --workspace`)
- [ ] All tests pass (`cargo test --workspace`)
- [ ] Code follows repo conventions

### Documentation
- [ ] README updated to show V1 goal #3 as completed (✅)
- [ ] In-code comments explain BTRFS CLI integration
- [ ] Error messages are user-friendly and actionable

---

## Open Questions

### Q1: UDisks2 BTRFS Support
**Question:** Does UDisks2 expose BTRFS-specific D-Bus methods, or do we need direct CLI calls?

**Action:** Check UDisks2 docs, introspect running daemon  
**Decision:** If no D-Bus support, use CLI with polkit rules for elevation  
**Impact:** Determines implementation complexity and permission model

### Q2: BTRFS Subvolume Mount State
**Question:** How to determine if a subvolume is mounted separately (vs. accessed via parent mount)?

**Action:** Test with `/proc/mounts`, `findmnt`, or `btrfs subvolume show`  
**Fallback:** Show path only, let mount info be inferred from parent volume

### Q3: Default Subvolume Setting
**Question:** Should we support setting default subvolume, or is this too advanced?

**Decision:** Nice-to-have, not required for V1. Defer to future enhancement if time permits.

### Q4: Nested Subvolumes Display
**Question:** Should subvolumes be shown as flat list or tree (for nested subvolumes)?

**Decision:** Start with flat list (simpler). Tree view is enhancement for later.

### Q5: Snapshot Organization
**Question:** Should snapshots be in separate list, or mixed with subvolumes (since they're technically subvolumes)?

**Decision:** Display all as subvolumes in one list initially. Can add filtering later if needed.

---

## Related Work

- **README V1 Goal #3**: Feature directly implements "1st class BTRFS support"
- **Feature: modal-dialogs**: All BTRFS dialogs depend on modal dialog windows being implemented first
- **Feature: filesystem-tools-detection**: BTRFS tool detection (`btrfs` command) should integrate with existing detection system
- **GNOME Disks**: Reference implementation for BTRFS subvolume/snapshot UI patterns

---

## Implementation Order

**Prerequisite:** feature/modal-dialogs must be merged to main

**Recommended task order:**
1. Detection and UI section scaffold (Task 1-2)
2. CLI wrapper module (Task 3)
3. Subvolume listing (Task 4)
4. Create/delete subvolume (Task 5-6)
5. Snapshot creation (Task 7)
6. Usage breakdown (Task 8)
7. Polish and localization (Task 9)
