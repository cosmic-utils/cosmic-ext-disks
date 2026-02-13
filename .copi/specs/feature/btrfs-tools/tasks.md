# BTRFS Management Tools — Tasks

**Branch:** `feature/btrfs-mgmt`  
**Prerequisites:** None (will use existing overlay dialogs)  
**Target:** Small, independently testable commits

---

## Task Dependency Graph

```
Prerequisites: None (using existing overlay dialogs)

Task 1 (Detection)
    ↓
Task 2 (UI scaffold)
    ↓
Task 3 (CLI module)
    ↓
Task 4 (Subvolume list)
    ↓
Task 5 (Create subvolume) ─┐
Task 6 (Delete subvolume) ──┤
Task 7 (Snapshot creation) ─┤ Can be parallel
Task 8 (Usage breakdown) ───┘
    ↓
Task 9 (Polish & localization)
```

---

## Task 1: BTRFS Filesystem Detection

**Scope:** Detect when selected volume is BTRFS and show indicator.

**Files/Areas:**
- `disks-ui/src/ui/app/view.rs` (`build_partition_info`, `build_volume_node_info`)

**Steps:**
1. In volume detail view functions, add BTRFS detection:
   ```rust
   let is_btrfs = volume.id_type == "btrfs" || 
                  (volume.has_filesystem && volume.filesystem_type == Some("btrfs"));
   ```
2. Add conditional UI element after action buttons:
   ```rust
   if is_btrfs {
       elements.push(widget::text("BTRFS Management (coming soon)").into());
   }
   ```
3. Test: Format a partition as BTRFS, select it, verify placeholder text appears
4. Test: Select non-BTRFS partition, verify no BTRFS indicator

**Test Plan:**
- Create BTRFS partition via existing format dialog
- Select it in UI
- Verify "BTRFS Management (coming soon)" text appears
- Select ext4 partition, verify no BTRFS text

**Done When:**
- [x] BTRFS volumes detected correctly
- [x] Non-BTRFS volumes show no BTRFS indicator
- [x] Placeholder UI element displays
- [x] Code compiles without warnings

**Estimated effort:** 1-2 hours

---

## Task 2: BTRFS Management UI Section (Scaffold)

**Scope:** Create collapsible "BTRFS Management" section with empty content.

**Files/Areas:**
- New module: `disks-ui/src/ui/btrfs/mod.rs`
- `disks-ui/src/ui/mod.rs` (module declaration)
- `disks-ui/src/ui/app/view.rs`
- `disks-ui/i18n/en/cosmic_ext_disks.ftl`

**Steps:**
1. Create module structure:
   ```
   disks-ui/src/ui/btrfs/
   ├── mod.rs (exports)
   ├── view.rs (rendering)
   ├── state.rs (state types)
   └── message.rs (messages)
   ```
2. In `btrfs/view.rs`, create function:
   ```rust
   pub fn btrfs_management_section<'a>(
       volume: &'a VolumeModel,
   ) -> Element<'a, Message> {
       // Collapsible section with "Loading..." placeholder
   }
   ```
3. Update `build_partition_info()` to call BTRFS section:
   ```rust
   if is_btrfs {
       elements.push(btrfs::btrfs_management_section(volume));
   }
   ```
4. Add i18n keys:
   ```fluent
   btrfs-management = BTRFS Management
   btrfs-subvolumes = Subvolumes
   btrfs-loading = Loading BTRFS information...
   ```
5. Test: Section appears, is collapsible, shows placeholder

**Test Plan:**
- Select BTRFS volume
- Verify section appears below action buttons
- Click to expand/collapse
- Verify section does not appear for non-BTRFS volumes

**Done When:**
- [x] Module structure created
- [x] Section displays for BTRFS volumes only
- [x] Section is collapsible
- [x] Placeholder content shows
- [x] Localized strings used
- [x] Code compiles

**Estimated effort:** 2-3 hours

---

## Task 3: BTRFS CLI Wrapper Module

**Scope:** Create utility module for executing `btrfs` commands and parsing output.

**Files/Areas:**
- New file: `disks-ui/src/utils/btrfs.rs`
- `disks-ui/src/utils/mod.rs` (module declaration and exports)

**Steps:**
1. Create module with structs:
   ```rust
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
2. Implement detection:
   ```rust
   pub fn command_exists() -> bool {
       which::which("btrfs").is_ok()
   }
   ```
3. Implement `list_subvolumes()`:
   ```rust
   pub async fn list_subvolumes(mount_point: &str) -> Result<Vec<Subvolume>> {
       let output = Command::new("btrfs")
           .args(["subvolume", "list", mount_point])
           .output()
           .await?;
       // Parse output: "ID <id> gen <gen> top level <level> path <path>"
       // Return vec of Subvolume structs
   }
   ```
4. Implement `create_subvolume()`:
   ```rust
   pub async fn create_subvolume(mount_point: &str, name: &str) -> Result<()> {
       let path = format!("{}/{}", mount_point, name);
       Command::new("btrfs")
           .args(["subvolume", "create", &path])
           .output()
           .await?;
       Ok(())
   }
   ```
5. Implement `delete_subvolume()`:
   ```rust
   pub async fn delete_subvolume(path: &str) -> Result<()> {
       Command::new("btrfs")
           .args(["subvolume", "delete", path])
           .output()
           .await?;
       Ok(())
   }
   ```
6. Implement `get_filesystem_usage()`:
   ```rust
   pub async fn get_filesystem_usage(mount_point: &str) -> Result<UsageInfo> {
       let output = Command::new("btrfs")
           .args(["filesystem", "usage", mount_point])
           .output()
           .await?;
       // Parse "Data,single:" and "Metadata,single:" lines
       // Extract used/total values
   }
   ```
7. Add comprehensive error handling with context
8. Write unit tests with mock output samples

**Test Plan:**
- Unit tests for parsing (with sample command outputs)
- Manual test: call `list_subvolumes()` on real BTRFS mount, verify output
- Manual test: call on non-BTRFS mount, verify error handling
- Manual test: call with `btrfs` not installed, verify `command_exists()` returns false

**Done When:**
- [x] Module compiles without warnings
- [x] `list_subvolumes()` returns parsed list on real BTRFS FS
- [x] `create_subvolume()` creates subvolume successfully
- [x] `delete_subvolume()` deletes subvolume successfully
- [x] `get_filesystem_usage()` returns usage info
- [x] Errors are descriptive and contextual
- [x] `command_exists()` correctly detects `btrfs` binary
- [x] Unit tests pass

**Estimated effort:** 4-6 hours

---

## Task 4: Subvolume List Display

**Scope:** Populate subvolume list with real data from BTRFS CLI.

**Files/Areas:**
- `disks-ui/src/ui/btrfs/view.rs`
- `disks-ui/src/ui/btrfs/state.rs`
- `disks-ui/src/ui/btrfs/message.rs`
- `disks-ui/src/ui/app/message.rs`
- `disks-ui/src/ui/app/update/mod.rs`

**Steps:**
1. Define BTRFS state in `state.rs`:
   ```rust
   pub struct BtrfsState {
       pub subvolumes: Vec<Subvolume>,
       pub loading: bool,
       pub error: Option<String>,
   }
   ```
2. Add to AppModel or per-volume state (decide based on architecture)
3. Create message for loading subvolumes:
   ```rust
   pub enum Message {
       LoadBtrfsSubvolumes(String),  // mount_point
       BtrfsSubvolumesLoaded(Result<Vec<Subvolume>>),
       ...
   }
   ```
4. When section expands, trigger load:
   - Send `Message::LoadBtrfsSubvolumes(mount_point)`
5. In update handler:
   - Call `btrfs::list_subvolumes(mount_point).await`
   - Send result via `BtrfsSubvolumesLoaded` message
6. Update `btrfs_management_section()` to display list:
   - Scrollable list/table widget
   - Columns: Name/Path, ID
   - Row per subvolume
7. Handle errors: if command fails, show "Unable to load subvolumes" + error details

**Test Plan:**
- Mount BTRFS volume with existing subvolumes (e.g., default @ and @home)
- Expand BTRFS Management section
- Verify subvolumes load and display correctly
- Verify IDs and paths match `btrfs subvolume list` CLI output
- Test error case: unmounted BTRFS volume (should show error)

**Done When:**
- [x] Subvolume list populates from real data
- [x] List displays correctly (name, ID visible)
- [x] List scrolls if many subvolumes
- [x] Loading indicator shows while fetching
- [x] Error states handled gracefully (message shown, no crash)

**Estimated effort:** 3-4 hours

---

## Task 5: Create Subvolume Dialog

**Scope:** Add "Create Subvolume" button and modal dialog.

**Files/Areas:**
- `disks-ui/src/ui/btrfs/view.rs`
- `disks-ui/src/ui/dialogs/state.rs` (add variant)
- `disks-ui/src/ui/dialogs/view/btrfs.rs` (new file)
- `disks-ui/src/ui/dialogs/message.rs`
- `disks-ui/src/ui/app/update/mod.rs`
- `disks-ui/i18n/en/cosmic_ext_disks.ftl`

**Steps:**
1. Add "Create Subvolume" button to BTRFS section
2. Add dialog variant to `ShowDialog`:
   ```rust
   BtrfsCreateSubvolume(BtrfsCreateSubvolumeDialog),
   
   pub struct BtrfsCreateSubvolumeDialog {
       pub mount_point: String,
       pub name: String,
       pub running: bool,
       pub error: Option<String>,
   }
   ```
3. Create dialog view in `dialogs/view/btrfs.rs`:
   ```rust
   pub fn create_subvolume<'a>(state: BtrfsCreateSubvolumeDialog) -> Element<'a, Message> {
       dialog::dialog()
           .title(fl!("btrfs-create-subvolume"))
           .body(/* name input field */)
           .primary_action(button::standard(fl!("create")).on_press(...))
           .secondary_action(button::standard(fl!("cancel")).on_press(...))
           .into()
   }
   ```
4. Add messages for dialog interaction:
   ```rust
   pub enum BtrfsCreateSubvolumeMessage {
       NameUpdate(String),
       Create,
   }
   ```
5. Implement update handler:
   - Validate name (non-empty, no slashes, reasonable length)
   - Call `btrfs::create_subvolume(mount_point, name).await`
   - On success: close dialog, send message to refresh subvolume list
   - On error: show error in dialog, keep open
6. Add i18n keys:
   ```fluent
   btrfs-create-subvolume = Create Subvolume
   btrfs-subvolume-name = Subvolume Name
   btrfs-subvolume-name-required = Subvolume name is required
   btrfs-subvolume-invalid-chars = Subvolume name cannot contain slashes
   btrfs-subvolume-created = Subvolume created successfully
   ```

**Test Plan:**
- Click "Create Subvolume"
- Enter name "test_subvol"
- Click Create
- Verify dialog shows "Running..." indicator
- Verify subvolume appears in list after creation
- Verify on filesystem: `ls /mount_point/test_subvol` exists
- Test validation: empty name shows error
- Test validation: name with slash shows error

**Done When:**
- [x] Dialog opens in separate window (modal, per modal-dialogs feature)
- [x] Name input field works
- [x] Validation prevents invalid names
- [x] Subvolume creation succeeds
- [x] List refreshes after creation
- [x] Error handling works (permission denied, invalid name, etc.)

**Estimated effort:** 3-4 hours

---

## Task 6: Delete Subvolume Confirmation

**Scope:** Add delete icon to subvolume rows with confirmation dialog.

**Files/Areas:**
- `disks-ui/src/ui/btrfs/view.rs`
- `disks-ui/src/ui/dialogs/state.rs` (use generic `ConfirmAction`)
- `disks-ui/src/ui/app/update/mod.rs`
- `disks-ui/i18n/en/cosmic_ext_disks.ftl`

**Steps:**
1. Add delete icon (trash can) to each subvolume row in list
2. On click, spawn confirmation dialog using existing `ConfirmAction`:
   ```rust
   ShowDialog::ConfirmAction(ConfirmActionDialog {
       title: fl!("btrfs-delete-subvolume"),
       body: format!("{} '{}'? {}", 
           fl!("btrfs-delete-confirm-prefix"),
           name,
           fl!("btrfs-delete-warning")),
       ok_message: Message::BtrfsDeleteSubvolume { path: full_path },
       ...
   })
   ```
3. Implement delete handler in update:
   ```rust
   Message::BtrfsDeleteSubvolume { path } => {
       Task::perform(
           btrfs::delete_subvolume(path),
           |result| Message::BtrfsDeleteResult(result)
       )
   }
   ```
4. On success: refresh subvolume list
5. On error: show error dialog (Info dialog with error details)
6. Add i18n keys:
   ```fluent
   btrfs-delete-subvolume = Delete Subvolume
   btrfs-delete-confirm-prefix = Delete subvolume
   btrfs-delete-warning = This action cannot be undone.
   btrfs-subvolume-deleted = Subvolume deleted successfully
   ```

**Test Plan:**
- Create test subvolume (via Task 5)
- Hover over subvolume row, verify delete icon appears
- Click delete icon
- Verify confirmation modal opens
- Confirm deletion
- Verify subvolume removed from list
- Verify subvolume removed from filesystem
- Test error case: try deleting mounted subvolume (should fail with error)

**Done When:**
- [x] Delete icon appears on subvolume rows
- [x] Confirmation modal opens
- [x] Delete succeeds and list refreshes
- [x] Error handling works (mounted subvolume, permission denied)
- [x] Cannot delete root subvolume or currently mounted subvolume (graceful error)

**Estimated effort:** 2-3 hours

---

## Task 7: Snapshot Creation Dialog

**Scope:** Add "Create Snapshot" button and dialog with source selection.

**Files/Areas:**
- `disks-ui/src/ui/btrfs/view.rs`
- `disks-ui/src/ui/dialogs/state.rs` (add variant)
- `disks-ui/src/ui/dialogs/view/btrfs.rs`
- `disks-ui/src/utils/btrfs.rs` (add `create_snapshot()` function)

**Steps:**
1. Add `create_snapshot()` to btrfs module:
   ```rust
   pub async fn create_snapshot(
       source: &str,
       dest: &str,
       readonly: bool
   ) -> Result<()> {
       let mut cmd = Command::new("btrfs");
       cmd.args(["subvolume", "snapshot"]);
       if readonly {
           cmd.arg("-r");
       }
       cmd.args([source, dest]);
       cmd.output().await?;
       Ok(())
   }
   ```
2. Add "Create Snapshot" button to BTRFS section
3. Add dialog variant:
   ```rust
   BtrfsCreateSnapshot(BtrfsCreateSnapshotDialog),
   
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
4. Create dialog view:
   - Source subvolume dropdown (populated from subvolume list)
   - Snapshot name input
   - "Read-only" checkbox (default: checked)
   - Create/Cancel buttons
5. Implement update handler:
   - Validate snapshot name
   - Build source and dest paths
   - Call `create_snapshot(source, dest, read_only)`
   - On success: close dialog, refresh subvolume list
6. Add i18n keys:
   ```fluent
   btrfs-create-snapshot = Create Snapshot
   btrfs-source-subvolume = Source Subvolume
   btrfs-snapshot-name = Snapshot Name
   btrfs-read-only = Read-only
   btrfs-snapshot-created = Snapshot created successfully
   ```

**Test Plan:**
- Click "Create Snapshot"
- Verify dropdown populated with subvolumes
- Select source subvolume (e.g., @home)
- Enter name "home_snapshot_2026-02-12"
- Check "Read-only"
- Click Create
- Verify snapshot appears in subvolume list
- Verify snapshot is read-only (try to write to it, should fail)
- Test writable snapshot: uncheck "Read-only", create, verify it's writable

**Done When:**
- [ ] Snapshot dialog opens
- [ ] Source dropdown populated correctly
- [ ] Snapshot creation succeeds (both read-only and writable)
- [ ] Snapshot listed after creation
- [ ] Read-only status is correct based on checkbox

**Estimated effort:** 3-5 hours

---

## Task 8: Usage Breakdown Display

**Scope:** Show BTRFS data/metadata/system allocation and compression info.

**Files/Areas:**
- `disks-ui/src/ui/btrfs/view.rs`
- `disks-ui/src/utils/btrfs.rs` (`get_filesystem_usage()` from Task 3)
- `disks-ui/i18n/en/cosmic_ext_disks.ftl`

**Steps:**
1. Add `get_compression()` function to btrfs module:
   ```rust
   pub async fn get_compression(mount_point: &str) -> Result<Option<String>> {
       let output = Command::new("btrfs")
           .args(["property", "get", mount_point, "compression"])
           .output()
           .await?;
       // Parse output: "compression=zstd" or "compression="
       Ok(/* parsed value */)
   }
   ```
2. Load usage info when BTRFS section expands:
   - Send message: `LoadBtrfsUsage(mount_point)`
3. In update handler:
   - Call `get_filesystem_usage()` and `get_compression()`
   - Store in state
4. Display usage in section:
   - Option A: Horizontal bar chart (if widget available)
   - Option B: Text summary:
     ```
     Data: 45.2 GB / 100 GB (45%)
     Metadata: 2.1 GB / 5 GB (42%)
     System: 16 MB / 32 MB (50%)
     Compression: zstd
     ```
5. Add i18n keys:
   ```fluent
   btrfs-usage = Usage Breakdown
   btrfs-data = Data
   btrfs-metadata = Metadata
   btrfs-system = System
   btrfs-compression = Compression
   btrfs-compression-disabled = disabled
   ```

**Test Plan:**
- Select BTRFS volume
- Expand BTRFS section
- Verify usage breakdown loads
- Verify values roughly match `btrfs filesystem usage <mount>` CLI output
- Verify compression info shown (test on FS with compression enabled and disabled)

**Done When:**
- [ ] Usage info loads asynchronously
- [ ] Data/metadata/system displayed clearly
- [ ] Compression info shown (algorithm name or "disabled")
- [ ] Loading indicator while fetching
- [ ] Error handling if commands fail

**Estimated effort:** 2-3 hours

---

## Task 9: Polish, Localization, Final Testing

**Scope:** Add Swedish translations, documentation, and comprehensive testing.

**Files/Areas:**
- `disks-ui/i18n/sv/cosmic_ext_disks.ftl` (add Swedish translations)
- `README.md` (update V1 goal #3)
- `.copi/specs/feature/btrfs-tools/implementation-log.md` (optional tracking doc)

**Steps:**
1. Translate all BTRFS i18n keys to Swedish:
   ```fluent
   btrfs-management = BTRFS-hantering
   btrfs-subvolumes = Undervolymer
   btrfs-create-subvolume = Skapa undervolym
   btrfs-subvolume-name = Undervolymnamn
   btrfs-delete-subvolume = Ta bort undervolym
   btrfs-create-snapshot = Skapa ögonblicksbild
   btrfs-source-subvolume = Källundervolym
   btrfs-snapshot-name = Ögonblicksbildnamn
   btrfs-read-only = Skrivskyddad
   btrfs-usage = Användningsöversikt
   btrfs-data = Data
   btrfs-metadata = Metadata
   btrfs-system = System
   btrfs-compression = Komprimering
   btrfs-compression-disabled = inaktiverad
   ```
2. Update README:
   ```markdown
   #### V1 - 99% Feature Parity with Gnome Disks (plus a few extras!)
   ...
   3. ✅ 1st class BTRFS support - Subvolumes CRUD, snapshots, usage breakdown
   ```
3. Full manual test suite:
   - Create BTRFS partition
   - Create/delete subvolumes (multiple times)
   - Create/delete snapshots (read-only and writable)
   - View usage breakdown
   - Test compression info display
   - Test on non-BTRFS volumes (no section)
   - Test with `btrfs` command not installed (graceful error)
4. Run quality gates:
   ```bash
   cargo test --workspace
   cargo clippy --workspace --all-features
   cargo fmt --all --check
   ```
5. Test edge cases:
   - Nested subvolumes
   - Many subvolumes (scroll behavior)
   - Unmounted BTRFS volume (should show error or disabled state)
   - BTRFS with compression enabled/disabled
6. Write test scenarios document (for manual QA)

**Test Plan:**
- Comprehensive manual testing checklist (all BTRFS features)
- All quality gates pass
- Translations complete and correct
- No known bugs

**Done When:**
- [ ] README updated (V1 goal #3 marked complete)
- [ ] All quality gates pass
- [ ] Full manual test suite executed
- [ ] No clippy warnings
- [ ] No crashes or errors in normal use
- [ ] Documentation complete

**Estimated effort:** 3-4 hours

---

## Estimated Total Effort

- **Detection & Scaffold (Tasks 1-2):** 3-5 hours
- **CLI Module (Task 3):** 4-6 hours
- **Subvolume Features (Tasks 4-6):** 8-11 hours
- **Snapshot & Usage (Tasks 7-8):** 5-8 hours
- **Polish (Task 9):** 3-4 hours
- **Total:** 23-34 hours (3-4 full work days)

---

## Commit Message Examples

- `feat(btrfs): add filesystem detection and UI section scaffold`
- `feat(btrfs): implement BTRFS CLI wrapper module`
- `feat(btrfs): display subvolume list with live data`
- `feat(btrfs): add create subvolume dialog and operation`
- `feat(btrfs): add delete subvolume with confirmation`
- `feat(btrfs): implement snapshot creation (read-only and writable)`
- `feat(btrfs): display usage breakdown and compression info`
- `docs: update README to mark BTRFS support complete (V1 goal #3)`

---

## Notes for Implementation

1. **Start after modal-dialogs merge**: Do not begin until feature/modal-dialogs is merged to main.

2. **Test incrementally**: After each task, manually test on real BTRFS filesystem.

3. **BTRFS test setup**: Create test BTRFS partition:
   ```bash
   # Create 1GB file
   dd if=/dev/zero of=btrfs-test.img bs=1M count=1024
   # Make loop device
   sudo losetup -fP btrfs-test.img
   # Format as BTRFS
   sudo mkfs.btrfs /dev/loop0
   # Mount
   sudo mount /dev/loop0 /mnt/btrfs-test
   ```

4. **Error messages**: All BTRFS operations should have user-friendly errors consistent with existing app patterns.

5. **Polkit**: Document required polkit rules if BTRFS commands need elevation.

6. **Compatibility**: Test on BTRFS with different features (compression, RAID profiles, quotas).

7. **Performance**: Subvolume listing can be slow on large filesystems (>100 subvolumes). Consider adding refresh button instead of auto-refresh.
