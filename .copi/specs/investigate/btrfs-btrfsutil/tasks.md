# BTRFS btrfsutil Migration — Tasks

**Branch:** `investigate/btrfs-btrfsutil`  
**Estimated Total:** 4-6 weeks

---

## Phase 1: Foundation & Core Library (Week 1)

### Task 1.1: Add Dependencies
**Estimated:** 0.5h  
**Files:** `disks-dbus/Cargo.toml`, `Cargo.toml` (workspace root)

- [ ] Add `btrfsutil = "0.2.0"` to disks-dbus
- [ ] Add `uuid = { version = "1.10", features = ["serde"] }` to disks-dbus
- [ ] Add `chrono = { version = "0.4", features = ["serde"] }` to disks-dbus
- [ ] Add `disks-btrfs-helper` to workspace members
- [ ] Run `cargo check --workspace` to verify

**Acceptance:** Workspace builds successfully with new dependencies

---

### Task 1.2: Create Helper Binary Crate
**Estimated:** 2h  
**Files:** `disks-btrfs-helper/Cargo.toml`, `disks-btrfs-helper/src/main.rs`

- [ ] Create `disks-btrfs-helper/` directory
- [ ] Create Cargo.toml with dependencies (btrfsutil, clap, serde, serde_json, anyhow)
- [ ] Create skeleton `main.rs` with CLI structure (clap derive)
- [ ] Define `Commands` enum (List, Create, Delete, Snapshot, SetReadonly, SetDefault, GetDefault, ListDeleted)
- [ ] Implement arg parsing and command dispatch
- [ ] Add placeholder implementations (return mock success)
- [ ] Test: `cargo run --package cosmic-ext-disks-btrfs-helper -- --help`

**Acceptance:** Helper binary compiles and shows help text

---

### Task 1.3: Implement Helper List Operation
**Estimated:** 3h  
**Files:** `disks-btrfs-helper/src/main.rs`

- [ ] Implement `list_subvolumes(mount_point: &Path)` function
- [ ] Use `Subvolume::try_from(mount_point)` to get root
- [ ] Use `SubvolumeIterator` to iterate all subvolumes
- [ ] For each subvolume, call `.info()` to get SubvolumeInfo
- [ ] Create serializable output struct matching BtrfsSubvolume fields
- [ ] Output JSON array to stdout
- [ ] Handle errors with context (anyhow)
- [ ] Test: Create temp BTRFS filesystem and run `helper list /mnt/test`

**Acceptance:** Helper outputs valid JSON with subvolume metadata

---

### Task 1.4: Implement Remaining Helper Operations
**Estimated:** 4h  
**Files:** `disks-btrfs-helper/src/main.rs`

- [ ] Implement `create_subvolume(mount_point, name)` using `Subvolume::create()`
- [ ] Implement `delete_subvolume(mount_point, path, recursive)` using `Subvolume::delete()`
- [ ] Implement `create_snapshot(mount_point, source, dest, readonly)` using `Subvolume::snapshot()`
- [ ] Implement `set_readonly(mount_point, path, readonly)` using `Subvolume::set_ro()`
- [ ] Implement `set_default(mount_point, path)` using `Subvolume::set_default()`
- [ ] Implement `get_default(mount_point)` using `Subvolume::get_default()`
- [ ] Implement `list_deleted(mount_point)` using `Subvolume::deleted()`
- [ ] Add proper error handling for each operation
- [ ] Test each operation manually with temp BTRFS filesystem

**Acceptance:** All helper operations work when invoked directly

---

### Task 1.5: Create Polkit Policy
**Estimated:** 1h  
**Files:** `data/com.system76.CosmicExtDisks.Btrfs.policy`

- [ ] Create `data/` directory if not exists
- [ ] Create polkit policy XML file
- [ ] Set action ID: `com.system76.CosmicExtDisks.btrfs.manage`
- [ ] Set description: "Manage BTRFS subvolumes and snapshots"
- [ ] Set auth requirements: `auth_admin` for all contexts
- [ ] Annotate with exec path: `/usr/libexec/cosmic-ext-disks-btrfs-helper`
- [ ] Add installation step to justfile/build scripts
- [ ] Document installation in README

**Acceptance:** Policy file validates with `polkit-1` tools

---

### Task 1.6: Create btrfs_native.rs Skeleton
**Estimated:** 2h  
**Files:** `disks-dbus/src/disks/btrfs_native.rs`

- [ ] Create new file with module structure
- [ ] Define `BtrfsSubvolume` struct with all fields (id, path, parent_id, uuid, parent_uuid, received_uuid, created, modified, generation, flags, is_readonly, is_default, ctransid, otransid)
- [ ] Add `#[derive(Debug, Clone, Serialize, Deserialize)]`
- [ ] Define `BtrfsFilesystem` struct with mount_point and helper fields
- [ ] Define `BtrfsHelper` struct with helper_path
- [ ] Define `Operation` enum matching helper commands
- [ ] Add skeleton method signatures (unimplemented!() bodies)
- [ ] Add module documentation
- [ ] Export from `disks/mod.rs`

**Acceptance:** File compiles, types are defined

---

### Task 1.7: Implement BtrfsHelper Execute
**Estimated:** 3h  
**Files:** `disks-dbus/src/disks/btrfs_native.rs`

- [ ] Implement `BtrfsHelper::new()` - finds helper binary path
- [ ] Implement `BtrfsHelper::execute()` - spawns helper process
- [ ] Convert `Operation` enum to CLI args
- [ ] Use `tokio::process::Command` to spawn helper
- [ ] Check exit code and parse stdout JSON
- [ ] Use `pkexec` wrapper for privilege escalation
- [ ] Add timeout handling (30s default)
- [ ] Add detailed error context
- [ ] Test: Mock helper invocation

**Acceptance:** Helper can be invoked from Rust code with proper privilege escalation

---

### Task 1.8: Implement BtrfsFilesystem::list_subvolumes
**Estimated:** 2h  
**Files:** `disks-dbus/src/disks/btrfs_native.rs`

- [ ] Implement `BtrfsFilesystem::new(mount_point)` constructor
- [ ] Implement `list_subvolumes()` method
- [ ] Build `Operation::ListSubvolumes`
- [ ] Call `helper.execute()` within `tokio::spawn_blocking()`
- [ ] Parse JSON output into `Vec<BtrfsSubvolume>`
- [ ] Handle errors with context
- [ ] Add conversion from helper output format to BtrfsSubvolume
- [ ] Test: Read subvolumes from test filesystem

**Acceptance:** Can list subvolumes with full metadata from Rust API

---

## Phase 2: Core Migration (Week 2)

### Task 2.1: Implement All BtrfsFilesystem Methods
**Estimated:** 6h  
**Files:** `disks-dbus/src/disks/btrfs_native.rs`

- [ ] Implement `create_subvolume(name)` → Operation::CreateSubvolume
- [ ] Implement `delete_subvolume(path)` → Operation::DeleteSubvolume
- [ ] Implement `create_snapshot(source, dest, readonly)` → Operation::CreateSnapshot
- [ ] Implement `get_subvolume_info(path)` (parse from list)
- [ ] Implement `set_readonly(path, readonly)` → Operation::SetReadonly
- [ ] Implement `get_default_subvolume()` → Operation::GetDefault
- [ ] Implement `set_default_subvolume(path)` → Operation::SetDefault
- [ ] Implement `list_deleted_subvolumes()` → Operation::ListDeleted
- [ ] Test each operation individually

**Acceptance:** All operations work via Rust API

---

### Task 2.2: Add SubvolumeInfo Conversion
**Estimated:** 1h  
**Files:** `disks-dbus/src/disks/btrfs_native.rs`

- [x] Implement `From<SubvolumeInfo>` for BtrfsSubvolume (if using btrfsutil directly)
- [x] Handle Option fields properly (parent_id, parent_uuid, received_uuid)
- [x] Convert timestamps to Local timezone
- [x] Compute is_default flag (requires get_default check)
- [x] Add unit tests for conversion

**Status:** ✅ **Complete** - Already implemented in Phase 1 via TryFrom<SubvolumeHelperOutput>

**Acceptance:** SubvolumeInfo converts correctly to BtrfsSubvolume

---

### Task 2.3: Update disks/mod.rs Exports
**Estimated:** 0.5h  
**Files:** `disks-dbus/src/disks/mod.rs`

- [ ] Remove `mod btrfs;` declaration
- [ ] Add `mod btrfs_native;` declaration
- [ ] Update public exports: `pub use btrfs_native::{BtrfsFilesystem, BtrfsSubvolume};`
- [ ] Run `cargo check` to verify no broken imports

**Acceptance:** Module compiles with new exports

---

### Task 2.4: Delete Old BTRFS Module
**Estimated:** 0.5h  
**Files:** `disks-dbus/src/disks/btrfs.rs` (DELETE)

- [ ] Remove `disks-dbus/src/disks/btrfs.rs` file completely (289 lines)
- [ ] Run `cargo check` to identify any remaining references
- [ ] Fix any compile errors from removed types
- [ ] Commit with message: "refactor!: remove UDisks2 BTRFS implementation"

**Acceptance:** Old BTRFS code completely removed, workspace compiles

---

### Task 2.5: Update UI State Structures
**Estimated:** 2h  
**Files:** `disks-ui/src/ui/btrfs/state.rs`

- [ ] Add new fields to `BtrfsState`:
  - `default_subvolume_id: Option<u64>`
  - `deleted_subvolumes: Option<Vec<BtrfsSubvolume>>`
  - `show_deleted: bool`
  - `selected_subvolume: Option<BtrfsSubvolume>`
  - `show_properties_dialog: bool`
- [ ] Update `Default` impl with new fields
- [ ] Update `Clone` if manual impl exists
- [ ] Run `cargo check` to verify

**Acceptance:** State struct includes all new fields

---

### Task 2.6: Update BTRFS Messages
**Estimated:** 2h  
**Files:** `disks-ui/src/ui/btrfs/message.rs`, `disks-ui/src/ui/app/message.rs`

- [ ] Add new messages to `btrfs/message.rs`:
  - `LoadDefaultSubvolume`
  - `DefaultSubvolumeLoaded(Result<BtrfsSubvolume, String>)`
  - `SetDefaultSubvolume { subvolume_id: u64 }`
  - `ToggleReadonly { subvolume_id: u64 }`
  - `ReadonlyToggled(Result<(), String>)`
  - `ShowProperties { subvolume_id: u64 }`
  - `CloseProperties`
  - `LoadDeletedSubvolumes`
  - `DeletedSubvolumesLoaded(Result<Vec<BtrfsSubvolume>, String>)`
  - `ToggleShowDeleted`
  - `RefreshAll`
- [ ] Add corresponding messages to `app/message.rs`
- [ ] Run `cargo check` to verify

**Acceptance:** All new messages defined

---

### Task 2.7: Rewrite BTRFS Update Handlers
**Estimated:** 6h  
**Files:** `disks-ui/src/ui/app/update/btrfs.rs`

- [ ] Update `load_subvolumes()` to use new `BtrfsFilesystem::new()` and `list_subvolumes()`
- [ ] Ensure all operations use `BtrfsFilesystem` instead of D-Bus proxies
- [ ] Update error handling for new error types
- [ ] Add handler for `ToggleReadonly` → calls `set_readonly()`
- [ ] Add handler for `SetDefaultSubvolume` → calls `set_default_subvolume()`
- [ ] Add handler for `LoadDefaultSubvolume` → calls `get_default_subvolume()`
- [ ] Add handler for `LoadDeletedSubvolumes` → calls `list_deleted_subvolumes()`
- [ ] Update `create_subvolume()` and `delete_subvolume()` handlers
- [ ] Update `create_snapshot()` handler
- [ ] Test each message flow manually

**Acceptance:** All BTRFS operations work from UI

---

## Phase 3: UI Enhancements (Week 3)

### Task 3.1: Add Timestamp Columns
**Estimated:** 2h  
**Files:** `disks-ui/src/ui/btrfs/view.rs`

- [ ] Add "Created" column to subvolume grid
- [ ] Add "Modified" column to subvolume grid
- [ ] Implement `format_relative_time(dt: &DateTime<Local>)` helper
- [ ] Show relative time ("2 hours ago") with tooltip showing full datetime
- [ ] Handle localization with fl! macros
- [ ] Adjust column widths
- [ ] Test with various dates

**Acceptance:** Timestamps visible and formatted correctly

---

### Task 3.2: Add Status Badges
**Estimated:** 2h  
**Files:** `disks-ui/src/ui/btrfs/view.rs`

- [ ] Add "Flags" column to grid
- [ ] Show "DEFAULT" badge if `subvol.is_default`
- [ ] Show lock icon if `subvol.is_readonly`
- [ ] Show camera icon if `subvol.parent_uuid.is_some()` (snapshot)
- [ ] Style badges with theme colors
- [ ] Test badge combinations

**Acceptance:** Badges display correctly for each state

---

### Task 3.3: Replace Delete Button with Context Menu
**Estimated:** 3h  
**Files:** `disks-ui/src/ui/btrfs/view.rs`

- [ ] Replace simple delete button with three-dot menu button
- [ ] Create `widget::popover` with menu items:
  - "Properties" → ShowProperties
  - "Make Read-Only" / "Make Writable" → ToggleReadonly
  - "Set as Default" → SetDefaultSubvolume (if not readonly)
  - "Delete" → DeleteSubvolume (destructive style)
- [ ] Add separators between sections
- [ ] Handle message routing
- [ ] Test menu on each subvolume type

**Acceptance:** Context menu functional with all operations

---

### Task 3.4: Create Properties Dialog
**Estimated:** 4h  
**Files:** `disks-ui/src/ui/btrfs/properties.rs` (new), `disks-ui/src/ui/btrfs/mod.rs`

- [ ] Create new `properties.rs` module
- [ ] Implement `properties_dialog(subvol: &BtrfsSubvolume)` function
- [ ] Use `widget::dialog()` with proper structure
- [ ] Add sections:
  - Identity (name, path, ID, UUID)
  - Timestamps (created, modified)
  - Snapshot Info (parent UUID, notice text) - if applicable
  - Properties (generation, readonly, default)
  - Advanced (ctransid, otransid, flags, received UUID)
- [ ] Implement `property_row(label, value)` helper
- [ ] Add "Close" button handling
- [ ] Export from btrfs/mod.rs
- [ ] Integrate into main view render

**Acceptance:** Properties dialog shows all metadata correctly

---

### Task 3.5: Add Deleted Subvolumes Section
**Estimated:** 3h  
**Files:** `disks-ui/src/ui/btrfs/view.rs`

- [ ] Add collapsible section below subvolume list
- [ ] Header: "Deleted Subvolumes" with expand/collapse button
- [ ] When expanded, load deleted subvolumes if not already loaded
- [ ] Show count: "3 subvolumes pending cleanup (~2.4 GB to reclaim)"
- [ ] List deleted subvolumes with ID and path
- [ ] Add "Clean Up Now" button → triggers sync operation
- [ ] Add "Learn More..." button with explanation dialog
- [ ] Test expand/collapse state

**Acceptance:** Deleted subvolumes visible and cleanable

---

### Task 3.6: Implement Automatic Naming Templates
**Estimated:** 3h  
**Files:** `disks-ui/src/ui/btrfs/view.rs`, `disks-ui/src/config.rs`

- [ ] Add snapshot naming template to app config
- [ ] Implement template variables: `{name}`, `{date}`, `{time}`
- [ ] Add template dropdown to create snapshot dialog:
  - "Timestamped" (default): `{name}-{date}-{time}`
  - "Date Only": `{name}-{date}`
  - "Custom": Free text input
- [ ] Implement `apply_template()` function
- [ ] Show preview of generated name
- [ ] Validate generated name (no '/', must be unique)
- [ ] Test template application

**Acceptance:** Snapshots created with template-based names

---

## Phase 4: Localization (Week 3 continued)

### Task 4.1: Add English Translations
**Estimated:** 2h  
**Files:** `disks-ui/i18n/en/cosmic_ext_disks.ftl`

- [ ] Add all new BTRFS strings:
  - `btrfs-properties`, `btrfs-make-readonly`, `btrfs-make-writable`, `btrfs-set-default`, `btrfs-delete`
  - `time-just-now`, `time-minutes-ago`, `time-hours-ago`, `time-days-ago`
  - `btrfs-deleted-subvolumes`, `btrfs-no-deleted`, `btrfs-deleted-count`, `btrfs-cleanup-deleted`
  - `subvolume-properties`, `name`, `path`, `subvolume-id`, `uuid`, `parent-id`, `timestamps`, `created`, `modified`, `snapshot-info`, `parent-uuid`, `snapshot-notice`, `properties`, `generation`, `readonly`, `default`, `yes`, `no`, `advanced`, `received-uuid`, `flags`
  - Confirmation dialogs: `confirm-set-default-title`, `confirm-set-default-body`, `confirm-readonly-title`, `confirm-readonly-body`, `confirm-writable-title`, `confirm-writable-body`
- [ ] Test string substitution with fl! macro
- [ ] Build and verify strings appear in UI

**Acceptance:** All English strings defined and rendered

---

### Task 4.2: Add Swedish Translations (Optional)
**Estimated:** 1h  
**Files:** `disks-ui/i18n/sv/cosmic_ext_disks.ftl`

- [ ] Translate all new strings to Swedish (if translator available)
- [ ] Otherwise, copy English strings as placeholders
- [ ] Test with `LANG=sv_SE.UTF-8`

**Acceptance:** Swedish locale doesn't show fallback warnings

---

## Phase 5: Testing & Polish (Week 4)

### Task 5.1: Create Integration Test Infrastructure
**Estimated:** 4h  
**Files:** `disks-dbus/tests/btrfs_integration.rs`

- [ ] Create tests/ directory in disks-dbus if not exists
- [ ] Implement `setup_btrfs_loop_device()` helper:
  - Create 1GB sparse file with dd
  - Setup loop device with losetup
  - Format as BTRFS with mkfs.btrfs
- [ ] Implement `mount_btrfs()` helper:
  - Create temp mount point
  - Mount with sudo
- [ ] Implement `cleanup_btrfs()` helper:
  - Unmount filesystem
  - Detach loop device
  - Remove files
- [ ] Add `#[ignore]` attribute (requires root)
- [ ] Document how to run: `sudo cargo test --package disks-dbus btrfs_integration -- --ignored`

**Acceptance:** Test infrastructure can create/destroy BTRFS filesystems

---

### Task 5.2: Write Integration Tests
**Estimated:** 4h  
**Files:** `disks-dbus/tests/btrfs_integration.rs`

- [ ] Test: List initial subvolumes (should have root subvolume ID 5)
- [ ] Test: Create subvolume, verify it appears in list
- [ ] Test: Set readonly flag, verify info reflects change
- [ ] Test: Create snapshot, verify parent_uuid matches source
- [ ] Test: Set default subvolume, verify get_default returns correct ID
- [ ] Test: Delete subvolume, verify appears in deleted list
- [ ] Test: Error cases (invalid paths, permission denied without sudo)
- [ ] Run all tests: `sudo cargo test --package disks-dbus btrfs_integration -- --ignored`

**Acceptance:** All integration tests pass

---

### Task 5.3: Add Unit Tests
**Estimated:** 3h  
**Files:** `disks-dbus/src/disks/btrfs_native.rs`

- [ ] Test: SubvolumeInfo conversion to BtrfsSubvolume
- [ ] Test: Operation serialization to CLI args
- [ ] Test: Helper path resolution
- [ ] Test: JSON parsing of helper output
- [ ] Test: Error handling and context
- [ ] Run: `cargo test --package disks-dbus`

**Acceptance:** Unit tests pass

---

### Task 5.4: Manual Testing Checklist
**Estimated:** 4h  

- [ ] Test: List subvolumes on real BTRFS filesystem
- [ ] Test: Create subvolume, verify in file browser
- [ ] Test: Delete subvolume, verify removed from list
- [ ] Test: Create snapshot, verify it's read-only
- [ ] Test: Toggle readonly flag, verify with `btrfs property get`
- [ ] Test: Set default subvolume, verify with `btrfs subvolume get-default`
- [ ] Test: Properties dialog shows all fields correctly
- [ ] Test: Context menu operations work
- [ ] Test: Deleted subvolumes section updates
- [ ] Test: Automatic naming templates generate correct names
- [ ] Test: Error messages are clear and actionable
- [ ] Test: UI doesn't freeze during long operations

**Acceptance:** All manual tests pass

---

### Task 5.5: Performance Testing
**Estimated:** 2h  

- [ ] Create test filesystem with 100 subvolumes
- [ ] Benchmark list operation time (old vs new)
- [ ] Verify 2-5x speedup claim
- [ ] Check memory usage during operations
- [ ] Profile with `cargo flamegraph` if needed
- [ ] Verify no memory leaks (run operations 1000x)

**Acceptance:** Performance meets expectations, no leaks

---

### Task 5.6: Error Handling Audit
**Estimated:** 2h  
**Files:** All BTRFS-related files

- [ ] Review all error paths
- [ ] Ensure proper context added with `.context()`
- [ ] Verify user-facing error messages are clear
- [ ] Test error scenarios:
  - Helper binary not found
  - Permission denied
  - Invalid mount point
  - Subvolume doesn't exist
  - Filesystem full
- [ ] Update error messages for clarity

**Acceptance:** All errors have clear messages and proper context

---

### Task 5.7: Documentation Updates
**Estimated:** 3h  
**Files:** `README.md`, `disks-dbus/README.md` (if exists)

- [ ] Update README with new BTRFS capabilities
- [ ] Add screenshots showing new UI features
- [ ] Document helper binary installation
- [ ] Add troubleshooting section for polkit issues
- [ ] Update build instructions (new dependencies)
- [ ] Document testing procedures
- [ ] Add migration notes for V1 users

**Acceptance:** Documentation is complete and accurate

---

### Task 5.8: Build System Updates
**Estimated:** 2h  
**Files:** `justfile`, packaging scripts

- [ ] Add `install-helper` target to justfile:
  - Build helper in release mode
  - Install to /usr/libexec/
  - Install polkit policy to /usr/share/polkit-1/actions/
- [ ] Update `install` target to depend on `install-helper`
- [ ] Update packaging scripts (PKGBUILD, .deb, .rpm if they exist)
- [ ] Test installation process
- [ ] Document uninstallation

**Acceptance:** Helper installed correctly via build system

---

## Phase 6: CI & Final Polish (Week 4 continued)

### Task 6.1: Verify CI Passes
**Estimated:** 2h  

- [ ] Run `cargo test --workspace --all-features` locally
- [ ] Run `cargo clippy --workspace --all-features` locally
- [ ] Run `cargo fmt --all --check` locally
- [ ] Fix any warnings or errors
- [ ] Commit fixes with proper commit messages
- [ ] Push to branch and verify CI passes

**Acceptance:** All CI checks pass

---

### Task 6.2: Update Spec Index
**Estimated:** 0.5h  
**Files:** `.copi/spec-index.md`

- [ ] Add entry for this migration:
  - Gap ID: Investigation
  - Title: "BTRFS UDisks2 → btrfsutil migration (V2.0)"
  - Spec Path: `.copi/specs/investigate/btrfs-btrfsutil/`
  - Branch: `investigate/btrfs-btrfsutil`
  - Source Audit: "Investigation (2026-02-13)"
  - Status: "Implemented"

**Acceptance:** Spec index updated

---

### Task 6.3: Create Implementation Log
**Estimated:** 1h  
**Files:** `.copi/specs/investigate/btrfs-btrfsutil/implementation-log.md`

- [ ] Create log with timestamped entries
- [ ] Document key decisions made during implementation
- [ ] List all files changed
- [ ] Note any deviations from plan
- [ ] Record blockers encountered and solutions
- [ ] List follow-up items for future

**Acceptance:** Implementation log complete

---

### Task 6.4: Final Review Checklist
**Estimated:** 2h  

- [ ] All acceptance criteria from plan.md verified
- [ ] No UDisks2 BTRFS code remains
- [ ] All new features functional
- [ ] Tests passing (unit + integration)
- [ ] CI passing
- [ ] Documentation updated
- [ ] Commit messages follow conventions
- [ ] No TODOs or FIXMEs without tracking
- [ ] Performance meets expectations
- [ ] Security audit complete (helper binary)

**Acceptance:** Ready for merge

---

## Task Summary

| Phase | Tasks | Estimated Time |
|-------|-------|----------------|
| Phase 1: Foundation | 8 tasks | ~15.5h (Week 1) |
| Phase 2: Core Migration | 7 tasks | ~20h (Week 2) |
| Phase 3: UI Enhancements | 6 tasks | ~17h (Week 3) |
| Phase 4: Localization | 2 tasks | ~3h (Week 3) |
| Phase 5: Testing & Polish | 8 tasks | ~24h (Week 4) |
| Phase 6: CI & Final | 4 tasks | ~5.5h (Week 4) |
| **Total** | **35 tasks** | **~85h (4-6 weeks)** |

---

## Progress Tracking

- [ ] Phase 1 Complete (8/8 tasks)
- [ ] Phase 2 Complete (7/7 tasks)
- [ ] Phase 3 Complete (6/6 tasks)
- [ ] Phase 4 Complete (2/2 tasks)
- [ ] Phase 5 Complete (8/8 tasks)
- [ ] Phase 6 Complete (4/4 tasks)

---

## Notes

- **Root Required:** Integration tests need `sudo` access
- **Test Filesystem:** Use loop devices, don't risk production data
- **Incremental Commits:** Commit after each major task
- **CI Early:** Push to trigger CI checks frequently
- **Helper Binary:** Test privilege escalation early in Phase 1
