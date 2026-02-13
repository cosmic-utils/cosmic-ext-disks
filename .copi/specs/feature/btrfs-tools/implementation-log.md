# Implementation Log — BTRFS Management Tools

**Branch:** `feature/btrfs-mgmt`  
**Spec:** `.copi/specs/feature/btrfs-tools/plan.md` + `tasks.md`  
**Started:** 2026-02-13

---

## Task 0: FsTools Integration + Module Enablement ✅
**Completed:** 2024-01-XX  
**Commit:** 3cabd28

**Summary:**
Implemented foundation for BTRFS management by adding udisks2-btrfs package detection and UDisks2 module enablement button in settings.

**Changes:**
- `disks-dbus/src/disks/manager.rs` - Added `enable_modules()` D-Bus method
- `disks-ui/src/utils/fs_tools.rs` - Added udisks2-btrfs to FS_TOOL_REQUIREMENTS
- `disks-ui/src/ui/app/message.rs` - Added EnableUDisksBtrfs messages
- `disks-ui/src/ui/app/update/mod.rs` - Added message handler with Task::perform and info dialogs
- `disks-ui/src/views/settings.rs` - Added "Try Enable UDisks2 BTRFS" button
- `disks-ui/i18n/en/cosmic_ext_disks.ftl` - Added i18n strings for module enablement

**Key Decisions:**
1. Used Info dialog variant for success/failure feedback (consistent with existing error patterns)
2. Required `.into()` wrapper for Message→Action conversion in Task::perform callbacks
3. Separated title and body strings for dialog display

**Testing:**
- Unit tests: PASSED (fs_tools structure + detection tests)
- Clippy: No warnings in modified files
- Compilation: SUCCESS

---

## Task 1: BTRFS Filesystem Detection ✅
**Completed:** 2026-02-13  
**Commit:** abe0979

**Changes:**
- Modified `disks-ui/src/ui/app/view.rs`
  - Added BTRFS detection in `build_partition_info()` and `build_volume_node_info()`
  - Detection checks `id_type.to_lowercase() == "btrfs"`
  - Placeholder text "BTRFS Management (coming soon)" appears when detected

**Testing:**  
- Compiled successfully: `cargo check --workspace`
- Clippy clean: `cargo clippy --workspace --all-features`

---

## Task 2: BTRFS Management UI Section (Scaffold) ✅
**Completed:** 2026-02-13  
**Commit:** e9af60c

**Changes:**
- Created module structure:
  - `disks-ui/src/ui/btrfs/mod.rs`
  - `disks-ui/src/ui/btrfs/view.rs`
  - `disks-ui/src/ui/btrfs/state.rs`
  - `disks-ui/src/ui/btrfs/message.rs`
- Modified `disks-ui/src/ui/mod.rs` to declare btrfs module
- Modified `disks-ui/src/ui/app/view.rs` to integrate section
- Added i18n keys to `disks-ui/i18n/en/cosmic_ext_disks.ftl`:
  - `btrfs-management` = "BTRFS Management"
  - `btrfs-placeholder` = "BTRFS management features coming soon"

**Implementation Notes:**
- Used inline BtrfsState creation to avoid ownership issues
- Kept simple placeholder for VolumeNode (will refactor in Task 4)
- Section displays with header and placeholder when expanded=true

**Testing:**
- Compiled successfully: `cargo check --workspace`
- Clippy clean with no warnings
- Fixed cosmic::Theme vs cosmic::iced::Theme confusion

---

## Task 3: UDisks2 BTRFS D-Bus Module ✅
**Completed:** 2024-01-XX (REFACTORED from CLI to D-Bus)
**Commit:** b23852f (replaces bc06d69 CLI approach)

**Changes:**
- Created `disks-dbus/src/disks/btrfs.rs` (290 lines)
- Implemented `BtrfsSubvolume` struct with id, parent_id, path fields
- Implemented `BtrfsFilesystem<'a>` D-Bus proxy wrapper:
  - `new(connection, block_path)` - Constructor
  - `is_available()` - Check if BTRFS interface exists
  - `get_subvolumes(snapshots_only)` - Get list via GetSubvolumes D-Bus call
  - `create_subvolume(name)` - Create via CreateSubvolume D-Bus method
  - `remove_subvolume(name)` - Remove via RemoveSubvolume D-Bus method
  - `create_snapshot(source, dest, read_only)` - Create via CreateSnapshot D-Bus method
  - `get_used_space()`, `get_label()`, `get_uuid()`, `get_num_devices()` - Property accessors
- Modified `disks-dbus/src/disks/mod.rs` to export BtrfsFilesystem and BtrfsSubvolume
- Added comprehensive error context using anyhow::Context
- Included unit tests for BtrfsSubvolume::name() extraction logic

**Key Decisions:**
1. Fixed import: `zbus::zvariant::OwnedObjectPath` (not `zbus::names::OwnedObjectPath`)
2. Used explicit type annotations `let proxy: zbus::Proxy<'_>` to satisfy edition 2024 type inference
3. Inlined proxy creation in each method (avoids lifetime complexity)
4. Used explicit `: ()` type annotation for void D-Bus method calls
5. Properties return explicit Result types with intermediate variables

**Refactoring Note:**
This completely replaces the CLI subprocess approach (bc06d69) with native D-Bus integration
matching existing patterns (mount, format, partition operations). Old `disks-ui/src/utils/btrfs.rs`
CLI wrapper will be removed once all UI code is migrated to use D-Bus calls.

**Testing:**
- Unit tests: PASSED (subvolume name extraction)
- Compilation: SUCCESS (cargo check --workspace)
- Clippy: No errors, only expected "unused" warnings (code not integrated yet)

---

## Task 4: Subvolume List Display ✅
**Completed:** 2026-02-13  
**Commit:** bc06d69 → cf6f097

**Note:** Task 4 was completed BEFORE the D-Bus refactor and currently uses CLI wrapper.
Refactoring to use D-Bus is required as part of migration from CLI to D-Bus approach.

**Changes:**
  - `create_snapshot()` - Create snapshot (read-only or writable)
  - `get_filesystem_usage()` - Parse `btrfs filesystem usage -b`
  - `get_compression()` - Query compression property
- Added internal parsing helpers:
  - `parse_subvolume_list()` - Parses ID/path/name from CLI output
  - `parse_filesystem_usage()` - Parses Data/Metadata/System allocation
  - `parse_usage_line()` - Extracts Size/Used from usage line
- Data structures:
  - `Subvolume` - id, path, name
  - `UsageInfo` - data/metadata/system used/total
- Modified `disks-ui/src/utils/mod.rs` to export btrfs module

**Testing:**
- Unit tests created and passing:
  - `test_parse_subvolume_list` ✅
  - `test_parse_filesystem_usage` ✅
  - `test_parse_usage_line` ✅
- Compilation clean: `cargo check --workspace`
- Clippy clean after fixes:
  - Fixed empty line after doc comments
  - Fixed collapsible if statements
  - Added #[allow(dead_code)] for unused functions (will be used in Task 4+)

**Key Decisions:**
- Used tokio::process::Command for async execution
- Used anyhow for error handling with context
- Validation: names max 255 chars, no '/' characters
- Parsing robust to whitespace and multi-word paths

---

## Task 4: Subvolume List Display ✅
**Completed:** 2026-02-13  
**Commit:** cf6f097

**Changes:**
- Modified `disks-ui/src/ui/btrfs/state.rs`
  - Added fields: `loading: bool`, `subvolumes: Option<Result<Vec<Subvolume>, String>>`, `mount_point: Option<String>`
  - Constructor `BtrfsState::new(mount_point)` initializes state
- Created `disks-ui/src/ui/app/update/btrfs.rs` (new file, 56 lines)
  - Message handler `handle_btrfs_message()` for BTRFS operations
  - BtrfsLoadSubvolumes: Sets loading=true, spawns async Task::perform
  - BtrfsSubvolumesLoaded: Updates state with result
- Modified `disks-ui/src/ui/app/message.rs`
  - Added `BtrfsLoadSubvolumes { mount_point }` message
  - Added `BtrfsSubvolumesLoaded { mount_point, result }` message
- Modified `disks-ui/src/ui/app/update/mod.rs`
  - Integrated btrfs message handler routing
  - Pattern match routes BtrfsLoad* to handle_btrfs_message()
- Modified `disks-ui/src/ui/app/update/nav.rs`
  - Initialize BtrfsState when mounting BTRFS volumes
  - Auto-trigger subvolume loading on navigation
  - Returns Task to load subvolumes if not already loaded
- Modified `disks-ui/src/ui/btrfs/view.rs`
  - Display loading state with "Loading subvolumes..."
  - Display list of subvolumes with ID and path
  - Display error messages on failure
- Modified `disks-ui/src/ui/app/view.rs`
  - Pass VolumesControl state to BTRFS section
- Modified `disks-ui/src/ui/volumes/state.rs`
  - Added `btrfs_state: Option<BtrfsState>` field to VolumesControl

**Implementation Details:**
- Uses COSMIC framework `Task<Message>` pattern for async operations
- Proper `.into()` conversion from Message to cosmic::app::Action
- Clone mount_point inside closure to satisfy Fn trait requirement
- Collapsed nested if-let chains per clippy suggestions (let-chains syntax)
- Detection checks both `id_type == "btrfs"` and mount_points.first()

**Testing:**
- Compilation successful: `cargo check --workspace`
- Clippy clean: `cargo clippy --workspace --all-features -- -D warnings`
- All 7 clippy::collapsible_if warnings resolved

**Challenges Resolved:**
- Fixed cosmic::app::Action::App usage → use Message.into()
- Fixed mount_point ownership in closure → clone inside closure
- Fixed newline escapes in multi_replace → proper replacement
- Fixed let-chain collapsing for readability

---

## Task 5: Create Subvolume Dialog ✅
**Completed:** 2026-02-13  
**Commit:** ce18906

**Changes:**
- Added `BtrfsCreateSubvolumeMessage` enum to `dialogs/message.rs`
  - NameUpdate, Create, Cancel variants
- Added `BtrfsCreateSubvolumeDialog` state to `dialogs/state.rs`
  - Fields: mount_point, name, running, error
- Created `dialogs/view/btrfs.rs` (new file, 44 lines)
  - Dialog view with name input, validation display
  - Primary action: Apply button (disabled while running)
  - Secondary action: Cancel button
- Added `OpenBtrfsCreateSubvolume` to `VolumesControlMessage`
- Created `volumes/update/btrfs.rs` (new file, 113 lines)
  - `open_create_subvolume()`: Initialize dialog with mount point
  - `btrfs_create_subvolume_message()`: Handle input and creation
- Modified `volumes/update.rs`
  - Added btrfs module integration
  - Routed OpenBtrfsCreateSubvolume and BtrfsCreateSubvolumeMessage
- Modified `volumes/update/create.rs`
  - Added BtrfsCreateSubvolume pattern to match statement
- Modified `btrfs/view.rs`
  - Added "Create Subvolume" button above list
- Modified `app/view.rs`
  - Added BtrfsCreateSubvolume dialog rendering
- Modified `volumes/message.rs`
  - Added BtrfsCreateSubvolumeMessage wrapping
  - Added From trait implementations
- Added i18n strings to `cosmic_ext_disks.ftl`:
  - btrfs-create-subvolume = "Create Subvolume"
  - btrfs-subvolume-name = "Subvolume Name"
  - btrfs-subvolume-name-required = "Subvolume name is required"
  - btrfs-subvolume-invalid-chars = "Subvolume name cannot contain slashes"
  - btrfs-create-subvolume-failed = "Failed to create subvolume"

**Implementation Details:**
- Validation logic matches CLI module constraints (max 255 chars, no '/')
- Running state disables Apply button and shows "working" text
- Error messages displayed inline in dialog
- Success triggers drive list refresh via Message::UpdateNav
- Subvolume list auto-reloads via nav update mechanism
- Used `cosmic::Task` (not `cosmic::app::Task`) for type compatibility
- Returns `Task<cosmic::Action<Message>>` matching volumes API pattern

**Testing:**
- Compilation successful: `cargo check --workspace`
- Clippy clean: `cargo clippy --workspace --all-features -- -D warnings`

**Challenges Resolved:**
- Fixed Task import (cosmic::Task vs cosmic::app::Task causing double-wrap)
- Added BtrfsCreateSubvolume to all dialog pattern matches
- Used text widget directly instead of non-existent caption helper

---

## Task 6: Delete Subvolume Confirmation ✅
**Completed:** 2026-02-13  
**Commit:** cf34d92

**Changes:**
- Added `BtrfsDeleteSubvolume { path }` message to `app/message.rs`
- Added `BtrfsDeleteSubvolumeConfirm { path }` message for actual deletion
- Modified `btrfs/view.rs`
  - Added delete icon button (user-trash-symbolic) to each subvolume row
  - Row structure: ID/path text + spacer + delete button
  - Button triggers BtrfsDeleteSubvolume message
- Modified `app/update/btrfs.rs`
  - `BtrfsDeleteSubvolume`: Shows ConfirmAction dialog with subvolume name
  - `BtrfsDeleteSubvolumeConfirm`: Performs async delete via btrfs::delete_subvolume
  - Dialog set to running state during deletion
  - Success: closes dialog and refreshes drive list (triggers subvolume reload)
  - Error: shows error dialog with details
- Modified `app/update/mod.rs`
  - Added BtrfsDeleteSubvolume and BtrfsDeleteSubvolumeConfirm to message routing
- Added i18n strings to `cosmic_ext_disks.ftl`:
  - btrfs-delete-subvolume = "Delete Subvolume" 
  - btrfs-delete-confirm = "Delete subvolume '{ $name }'? This action cannot be undone."
  - btrfs-delete-subvolume-failed = "Failed to delete subvolume"

**Implementation Details:**
- Reused existing ConfirmAction dialog pattern (requires FilesystemTarget dummy)
- Icon button with padding(4) for compact row display
- Subvolume name extracted from path using rsplit('/')
- Confirmation body uses fl! macro with name parameter
- Delete operation is async Task returning drives for auto-refresh
- No separate "deleted successfully" dialog - implicit via list refresh

**Testing:**
- Compilation successful: `cargo check --workspace`
- Clippy clean: `cargo clippy --workspace --all-features -- -D warnings`

**Challenges Resolved:**
- Fixed dead_code warning by removing unused mount_point field from messages
- Fixed useless_conversion by removing .into() on direct Message assignment

---

## Task 7: Snapshot Creation Dialog 📋
**Status:** Not started  
**Next:** Implement snapshot creation dialog and integration
- Add BtrfsMessage variants (LoadSubvolumes, SubvolumesLoaded)
- Integrate into AppModel message handling
- Update btrfs_management_section() to display list in scrollable widget
- Handle loading states and errors

**Next Steps:**
1. Define comprehensive BtrfsState structure
2. Add to AppModel or per-volume tracking
3. Wire up message handling
4. Implement view with list display

---

## Commands Used
```bash
# Quality gates
cargo check --workspace
cargo clippy --workspace --all-features -- -D warnings
cargo test --bin cosmic-ext-disks btrfs::tests
cargo fmt --all --check

# Git operations
git branch --show-current
git status --porcelain
git add -A && git commit -m "..."
```

---

## Issues Encountered & Solutions

### Issue 1: Type mismatch cosmic::Theme vs cosmic::iced::Theme
**Problem:** Used `cosmic::iced::Element` which has wrong theme type  
**Solution:** Changed to `cosmic::Element` and `cosmic::iced_widget`

### Issue 2: Ownership of BtrfsState in view function
**Problem:** Can't return Element that borrows local variable  
**Solution:** Inline creation `&BtrfsState { expanded: true }` in function call

### Issue 3: Test failures in parse_usage_line
**Problem:** Incorrect parsing logic for "Data,single: Size:X, Used:Y" format  
**Solution:** Used `find("Size:")` and `find("Used:")` instead of split(':')

---

## Files Modified Summary
- `disks-ui/src/ui/app/view.rs` - Detection + section integration
- `disks-ui/src/ui/mod.rs` - Module declaration
- `disks-ui/src/ui/btrfs/` - New module directory (4 files)
- `disks-ui/src/utils/btrfs.rs` - CLI wrapper (new file)
- `disks-ui/src/utils/mod.rs` - Export btrfs module
- `disks-ui/i18n/en/cosmic_ext_disks.ftl` - i18n keys

---

## Coverage Status
- Task 1 ✅ Complete
- Task 2 ✅ Complete
- Task 3 ✅ Complete
- Task 4 🚧 In progress
- Task 5-9 ⏳ Pending
