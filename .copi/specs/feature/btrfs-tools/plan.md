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

### B) UDisks2 BTRFS D-Bus Interface (REFACTORED)

**Discovery:** The `udisks2-btrfs` package provides `org.freedesktop.UDisks2.Filesystem.BTRFS` interface on mounted BTRFS filesystems. This provides all needed operations with automatic polkit integration, eliminating the need for CLI subprocess management and pkexec wrappers.

**Location:** `disks-dbus/src/disks/btrfs.rs` (new module)

**Key Findings:**
- Interface appears on block device paths (e.g., `/org/freedesktop/UDisks2/block_devices/sda2`) when mounted
- Module must be explicitly enabled via `Manager.EnableModules(true)` on systems with `modules_load_preference=ondemand`
- Provides complete BTRFS management matching CLI capabilities
- Uses same polkit auth pattern as existing disk operations (mount, format, etc.)

**Available D-Bus Methods:**
```
org.freedesktop.UDisks2.Filesystem.BTRFS:
  - GetSubvolumes(snapshots_only: bool) → array of (id: u64, parent_id: u64, path: string)
  - CreateSubvolume(name: string, options: dict)
  - RemoveSubvolume(name: string, options: dict)
  - CreateSnapshot(source: string, dest: string, read_only: bool, options: dict)
  - GetDefaultSubvolumeID(options: dict) → uint64
  - SetDefaultSubvolumeID(id: uint64, options: dict)
  - SetLabel(label: string, options: dict)
  - Repair(options: dict)
  - Resize(size: uint64, options: dict)
  - AddDevice(device: object_path, options: dict)
  - RemoveDevice(device: object_path, options: dict)

Properties:
  - label: string
  - uuid: string
  - num_devices: uint64
  - used: uint64
```

**Module Enablement:**
On systems with `modules_load_preference=ondemand` (default on many distros), the BTRFS interface must be explicitly enabled via:
```rust
// Call on app startup or via settings button
manager_proxy.call("EnableModules", &(true,)).await?;
```

After enablement, the interface immediately appears on all mounted BTRFS filesystems.

**Error handling:**
- Package not installed: "UDisks2 BTRFS module not installed. Install udisks2-btrfs package."
- Module not enabled: "BTRFS module not enabled. Click 'Try Enable UDisks2 BTRFS' in settings."
- Permission denied: Polkit handles auth automatically (same as mount, format operations)
- D-Bus errors: Propagate error names/messages from UDisks2 for debugging

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

**Note:** This spec uses the existing overlay dialog system (`Application::dialog()`) as the modal-dialogs feature has been deferred pending upstream libcosmic support.

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
    pub block_path: OwnedObjectPath,
    pub name: String,
    pub running: bool,
    pub error: Option<String>,
}

pub struct BtrfsCreateSnapshotDialog {
    pub block_path: OwnedObjectPath,
    pub subvolumes: Vec<BtrfsSubvolume>,
    pub selected_source_index: usize,
    pub snapshot_name: String,
    pub read_only: bool,
    pub running: bool,
    pub error: Option<String>,
}
```

---

### E) UDisks2 D-Bus Integration

**Architecture:** Consistent with existing disk operations (mount, format, partition creation).

**Pattern:**
1. User triggers operation (button click)
2. App calls D-Bus method on `org.freedesktop.UDisks2.Filesystem.BTRFS` interface
3. UDisks2 daemon checks polkit policy
4. If auth needed, polkit prompt appears (handled by system)
5. Operation executes with elevated privileges
6. Result returned via D-Bus response
7. App updates UI based on success/error

**Polkit Integration:**
- Automatic via UDisks2's existing policies
- No custom polkit rules needed
- Auth cached per polkit session (typically 5 minutes)
- Same UX as mounting, formatting, or creating partitions

**Benefits:**
- ✅ No subprocess management (CLI calls, pipes, parsing)
- ✅ No pkexec wrappers or custom polkit rules
- ✅ Structured error handling (D-Bus errors, not stderr parsing)
- ✅ Works in sandboxed environments (Flatpak, Snap)
- ✅ Consistent with existing codebase architecture

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
2. **Overlay dialog** opens asking for subvolume name
3. User enters name (e.g., "mydata"), clicks "Create"
4. App calls `CreateSubvolume` D-Bus method on BTRFS interface
5. Polkit prompt appears for authentication
6. On success, dialog closes, subvolume list refreshes automatically
7. New subvolume appears in list

### Flow 3: User Deletes BTRFS Subvolume
1. User hovers over subvolume in list, clicks delete icon (trash can)
2. **Overlay confirmation dialog** appears: "Delete subvolume 'mydata'? This cannot be undone."
3. User confirms
4. App calls `RemoveSubvolume` D-Bus method on BTRFS interface
5. Operation completes, list refreshes, subvolume removed

### Flow 4: User Creates BTRFS Snapshot
1. User clicks "Create Snapshot" button in BTRFS section
2. **Overlay dialog** opens with:
   - Source subvolume dropdown (populated from subvolume list)
   - Snapshot name input
   - "Read-only" checkbox (checked by default)
3. User selects source (e.g., @home), enters name (e.g., "home-backup-2026-02-12"), leaves read-only checked
4. User clicks "Create"
5. App calls `CreateSnapshot` D-Bus method with parameters (source, dest, read_only=true)
6. Snapshot appears in subvolume list (snapshots are subvolumes in BTRFS)

### Flow 5: User Views BTRFS Usage Breakdown
1. BTRFS Management section automatically displays:
   - Text or chart: `Data: 45GB / 100GB | Metadata: 2GB / 5GB | System: 16MB / 32MB`
   - Below: "Compression: zstd"
2. Info is read-only, no user action required
3. User understands BTRFS allocation (helps diagnose "disk full" issues where data shows space but metadata is full)

---

## Risks & Mitigations

### Risk 1: Schema Changes in UDisks2 BTRFS Interface (**LOW RISK**)
**Risk:** Future versions of udisks2-btrfs might change D-Bus method signatures or property names.

**Mitigation:**
- D-Bus interfaces are more stable than CLI output formats
- UDisks2 follows semantic versioning and deprecation policies
- Interface introspection allows runtime capability detection
- Unit tests verify expected interface structure matches
- Graceful degradation: if method not found, show "BTRFS management unavailable" with version mismatch note

### Risk 2: BTRFS Module Not Enabled (**RESOLVED**)
**Risk:** UDisks2 BTRFS module may not be loaded on user's system, preventing D-Bus interface from appearing.

**Resolution:**
- Detected that `udisks2-btrfs` requires explicit module enablement on systems with `modules_load_preference=ondemand`
- Module can be enabled via `Manager.EnableModules(true)` D-Bus call
- Once enabled, interface appears immediately on mounted BTRFS filesystems
- Polkit handles authentication seamlessly like other disk operations

**Mitigation Strategy:**
- Add `udisks2-btrfs` to FsTools detection module with package warnings
- Provide "Try Enable UDisks2 BTRFS" button in settings that calls `EnableModules(true)`
- Show clear error if `udisks2-btrfs` not installed with package installation instructions
- Gracefully disable BTRFS management UI if interface unavailable

### Risk 3: Complex BTRFS Setups
**Risk:** Nested subvolumes, multiple devices, RAID profiles could produce unexpected UI states.

**Mitigation:**
- Start with simple single-device BTRFS (most common case)
- Display raw command output for complex edge cases ("Advanced BTRFS setup detected")
- Incremental feature support (ship basic subvolume CRUD first, enhance later)
- Test on various BTRFS configurations

### Risk 4: Overlay Dialog Limitations (**MITIGATED**)
**Risk:** Using overlay dialogs instead of true modal windows may allow accidental background interactions.

**Resolution:**
- Overlay system is well-tested in existing codebase (format, delete, mount options dialogs)
- Modal-dialogs feature deferred pending upstream libcosmic support
- BTRFS dialogs follow same patterns as existing overlay dialogs
- Future migration to modal windows will require minimal changes (same state structures)

### Risk 5: UDisks2 BTRFS Package Not Installed (**UPDATED**)
**Risk:** Users may not have `udisks2-btrfs` package installed, preventing D-Bus interface from existing.

**Mitigation:**
- Add `udisks2-btrfs` detection to `disks-ui/src/utils/fs_tools.rs` alongside other filesystem tools
- Show clear warning in settings: "UDisks2 BTRFS module not installed. Install udisks2-btrfs package."
- Gracefully disable BTRFS management section if interface not available
- Provide fallback message in BTRFS section with installation instructions
- Note: `btrfs-progs` still needed (provides `mkfs.btrfs` for formatting), but BTRFS management only requires `udisks2-btrfs`

---

## Acceptance Criteria

### Package & Module Detection
- [ ] `udisks2-btrfs` package detection added to FsTools module
- [ ] Warning shown in settings if `udisks2-btrfs` not installed
- [ ] "Try Enable UDisks2 BTRFS" button appears in settings
- [ ] Button triggers `Manager.EnableModules(true)` D-Bus call
- [ ] Success/error feedback shown after enabling
- [ ] Button disabled or hidden when module already enabled

### Detection & UI
- [ ] BTRFS filesystems are detected (check `id_type == "btrfs"`)
- [ ] "BTRFS Management" section appears in volume detail view only for BTRFS volumes
- [ ] Section does not appear for non-BTRFS filesystems
- [ ] Section gracefully handles missing BTRFS interface (shows install instructions)

### Subvolume Management (via D-Bus)
- [ ] Subvolume list displays with columns: name/path, ID
- [ ] "Create Subvolume" button opens overlay dialog
- [ ] Create subvolume dialog has name input field and validation (non-empty, no slashes)
- [ ] Subvolume creation succeeds via `CreateSubvolume` D-Bus method
- [ ] Subvolume list refreshes after creation
- [ ] Delete subvolume icon shows on each row
- [ ] Delete shows confirmation overlay dialog
- [ ] Subvolume deletion succeeds via `RemoveSubvolume` D-Bus method
- [ ] List refreshes after deletion

### Snapshot Management (via D-Bus)
- [ ] "Create Snapshot" button opens overlay dialog
- [ ] Dialog has source dropdown populated from subvolume list
- [ ] Dialog has snapshot name input field
- [ ] Dialog has "Read-only" checkbox
- [ ] Snapshot creation succeeds via `CreateSnapshot` D-Bus method (both modes)
- [ ] Snapshots appear in subvolume list after creation

### Usage Display
- [ ] Used space property loads from D-Bus (`used` property)
- [ ] Value displayed clearly with size formatting
- [ ] Label and UUID shown
- [ ] Graceful handling if properties unavailable

### Error Handling
- [ ] Polkit auth prompt appears for write operations (create, delete, snapshot)
- [ ] Clear error message if `udisks2-btrfs` package not installed
- [ ] Clear error message if BTRFS module not enabled
- [ ] D-Bus errors handled gracefully (don't crash, show friendly message)
- [ ] Operations refresh relevant UI sections on completion

### Localization & Quality
- [ ] All BTRFS UI strings localized in `en/cosmic_ext_disks.ftl`
- [ ] Swedish translations added in `sv/cosmic_ext_disks.ftl`
- [ ] No compiler warnings (`cargo clippy --workspace`)
- [ ] All tests pass (`cargo test --workspace`)
- [ ] Code follows repo conventions

### Documentation
- [ ] README updated to show V1 goal #3 as completed (✅)
- [ ] In-code comments explain UDisks2 BTRFS D-Bus integration
- [ ] Package requirements documented (udisks2-btrfs)
- [ ] Error messages are user-friendly and actionable

---

## Open Questions

### Q1: UDisks2 BTRFS Support (**RESOLVED**)
**Question:** Does UDisks2 expose BTRFS-specific D-Bus methods, or do we need direct CLI calls?

**Answer:** ✅ YES! The `udisks2-btrfs` package provides complete `org.freedesktop.UDisks2.Filesystem.BTRFS` interface  
**Discovery:** Interface verified on Arch Linux with `busctl introspect` — all subvolume, snapshot, and management operations available  
**Decision:** Use D-Bus interface exclusively (no CLI subprocess calls needed)  
**Impact:** Significantly simpler implementation, automatic polkit integration, consistent with existing architecture

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
- **Feature: filesystem-tools-detection**: `udisks2-btrfs` package detection integrates with existing FsTools system
- **UDisks2 Architecture**: All disk operations (mount, format, partition) use UDisks2 D-Bus; BTRFS operations now follow same pattern
- **GNOME Disks**: Reference implementation for BTRFS subvolume/snapshot UI patterns

---

## Implementation Order

**Prerequisites:** `udisks2-btrfs` package must be installable (available in user's distro repos)

**Recommended task order:**
1. FsTools integration + EnableModules button (Task 0) — prepares environment
2. Detection and UI section scaffold (Task 1-2) — visual framework
3. D-Bus BTRFS module (Task 3) — core functionality
4. Subvolume listing (Task 4) — basic read operations
5. Create/delete subvolume (Task 5-6) — write operations
6. Snapshot creation (Task 7) — advanced write operations
7. Usage property display (Task 8) — read-only info
8. Polish and localization (Task 9) — final touches

**Testing Strategy:**
- Each task includes unit tests where applicable
- Integration testing requires real BTRFS filesystem (loop device or spare partition)
- Manual polkit testing (confirm auth prompts work)
- Test matrix: enabled/disabled modules, installed/missing package
