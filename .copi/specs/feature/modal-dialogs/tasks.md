# Modal Dialog Windows — Tasks

**Branch:** N/A (deferred)  
**Status:** ⛔ **DEFERRED** — Waiting for upstream libcosmic support  
**Target:** N/A (spec deferred)

---

## ⚠️ Implementation Status Update (2026-02-12)

**Task 1:** ✅ **COMPLETE** — Research findings documented in [research-findings.md](research-findings.md)

**Critical Discovery:** libcosmic does NOT support parent-child window relationships. True modal windows are not possible with current framework.

**Tasks 2-11:** ⏸️ **ON HOLD** — Blocked pending product decision (see [plan.md](plan.md))

**Decision Made:** Option A (Wait for Upstream) chosen. This spec is deferred indefinitely until libcosmic supports parent-child window relationships.

**Impact:** All tasks 2-11 are cancelled. No implementation will occur until upstream feature becomes available.

---

## Task Dependency Graph

> **Note:** This dependency graph is for the original plan (true modal windows). If pivoting to Option B/C, the graph will change.

```
Task 1 (Research)
    ↓
Task 2 (PoC)
    ↓
Task 3 (Dialog infrastructure)
    ↓
Task 4-10 (Dialog migration, can be done in parallel)
    ↓
Task 11 (Dialog cleanup)
```

**Recommended order**: Sequential Tasks 1-3, then parallel Tasks 4-10, finish with Task 11.

---

## Task 1: Research COSMIC Window Management APIs ✅ COMPLETE

**Status:** ✅ **COMPLETE** (2026-02-12)  
**Estimated effort:** 2-4 hours → **Actual: 4 hours**  
**Outcome:** Research documented in [research-findings.md](research-findings.md)

**Scope:** Investigate how to spawn modal child windows in COSMIC applications.

**What was researched:**
- libcosmic `Application` trait and window management APIs
- iced `window::open()` and multi-window support
- COSMIC applet popup/subsurface mechanisms
- Wayland xdg_popup protocol (for menus/dropdowns)
- libcosmic multi-window example
- libcosmic source code (`get_window()`, surface management)

**Key Findings:**
1. ✅ COSMIC supports multiple independent windows via `window::open()`
2. ✅ COSMIC has `Application::view_window(id)` for rendering additional windows
3. ❌ COSMIC does NOT support parent-child window relationships (transient_for)
4. ❌ COSMIC does NOT support modal window semantics (automatic z-order, parent blocking)
5. ℹ️ libcosmic source has TODO comment: `None, // TODO parent for window, platform specific option maybe?`
6. ℹ️ Wayland popups exist but are for menus/tooltips, not dialog windows

**Implications:**
- **Original spec goal (true modal windows) is NOT FEASIBLE** with current libcosmic
- Windows created with `window::open()` are independent top-level windows
- No way to enforce modal behavior, parent-child relationship, or relative positioning

**Recommendations documented:**
- **Option A:** Wait for upstream libcosmic support (months timeline)
- **Option B:** Enhanced overlay dialogs (achievable now, 15-25h)
- **Option C:** Hybrid approach (experimental, 25-35h)

**Done When:**
- [x] Investigated libcosmic/iced window management APIs
- [x] Reviewed multi-window example and source code
- [x] Identified limitation (no parent-child support)
- [x] Documented findings in [research-findings.md](research-findings.md)
- [x] Updated [plan.md](plan.md) with research results and recommendations
- [x] Flagged spec as ON HOLD pending product decision

**Next Step:** User decision required on how to proceed (Option A, B, or C).

---

## Task 2: Proof of Concept with Info Dialog ⏸️ ON HOLD

**Files/Areas:**
- COSMIC documentation (online, GitHub repos)
- `libcosmic` source code (`cosmic::Application` trait, windowing APIs)
- Example COSMIC apps (cosmic-settings, cosmic-files, cosmic-store, etc.)
- Community channels (Matrix, Discord)

**Steps:**
1. Search libcosmic docs for "window", "dialog", "modal", "child window", "multi-window"
2. Examine `cosmic::Application` trait methods for window spawning support
3. Check if `cosmic::Core` or `cosmic::app::Command` expose window management APIs
4. Look for existing COSMIC apps with multiple windows:
   - Does cosmic-settings spawn modal dialogs?
   - Does cosmic-files use separate windows for any operations?
5. Review iced documentation (COSMIC's underlying framework) for multi-window support
6. Ask in cosmic-epoch Matrix/Discord: "What's the recommended way to spawn modal child windows from a COSMIC Application?"
7. Document findings in this file (append "Research Results" section)

**Test Plan:**
- n/a (research only)

**Done When:**
- [ ] Documented approach to spawning child windows (with code examples if available)
- [ ] Identified whether native multi-window support exists
- [ ] Determined fallback strategy if native support is insufficient
- [ ] Decision made: native multi-window vs. subprocess vs. enhanced overlay

**Estimated effort:** 2-4 hours

**Research Results:** (append after completion)
```
[Findings to be documented here after Task 1]
```

---

## Task 2: Proof of Concept — Info Dialog as Separate Window

**Scope:** Implement simplest dialog (Info dialog) as a separate window to validate approach from Task 1.

**Files Likely Modified:**
- `disks-ui/src/ui/app/state.rs` (add dialog window tracking)
- `disks-ui/src/ui/app/message.rs` (add window spawn/close messages)
- `disks-ui/src/ui/app/update/mod.rs` (handle window messages)
- `disks-ui/src/ui/app/mod.rs` (multi-window application setup, if needed)
- `disks-ui/src/ui/dialogs/view/common.rs` (adapt `info()` function for window context)

**Steps:**
1. Based on Task 1 findings, implement window spawning for Info dialog
2. Create message type: `Message::OpenDialogWindow(ShowDialog::Info { title, body })`
3. In update handler, spawn new window with dialog content:
   - Set window properties (title, size, modality)
   - Associate with parent window (if API allows)
4. Add window ID to `AppModel` (e.g., `pub info_dialog_window: Option<WindowId>`)
5. Implement window close handler:
   - Removes window ID from tracking
   - Cleans up any state
6. Test: Trigger an info dialog (e.g., force an error condition by unplugging device mid-operation)
7. Verify:
   - Dialog opens in separate window
   - Parent window becomes non-interactive (modal behavior)
   - Close via X button → correct cleanup
   - Close via OK button → correct cleanup

**Test Plan:**
- Manual: Trigger error that shows Info dialog (e.g., unmount failure)
- Verify window appears separately (not overlay)
- Click X button → window closes, no errors in logs
- Click OK button → window closes, same behavior
- Parent window regains focus after close

**Done When:**
- [ ] Info dialog opens in separate OS window
- [ ] Dialog window is modal (parent dims or becomes non-interactive)
- [ ] Both X button and OK button close the dialog correctly
- [ ] No crashes, no state leaks
- [ ] Code compiles without warnings
- [ ] Approach is validated and can be generalized to other dialogs

**Estimated effort:** 4-8 hours

---

## Task 3: Dialog Window Infrastructure

**Scope:** Generalize PoC to support all dialog types with reusable infrastructure.

**Files/Areas:**
- `disks-ui/src/ui/app/state.rs`
- `disks-ui/src/ui/app/message.rs`
- `disks-ui/src/ui/app/update/dialogs.rs` (new file for dialog window lifecycle)
- `disks-ui/src/ui/dialogs/window.rs` (new file for dialog window views)

**Steps:**
1. Create `DialogWindowState` tracking (if not done in Task 2):
   ```rust
   pub struct AppModel {
       pub dialog_windows: HashMap<WindowId, ShowDialog>,
       ...
   }
   ```
2. Create helper module `disks-ui/src/ui/app/update/dialogs.rs`:
   ```rust
   pub(super) fn spawn_dialog_window(
       app: &mut AppModel,
       dialog: ShowDialog,
   ) -> Task<Message> {
       // Spawn window, track by ID
   }
   
   pub(super) fn close_dialog_window(
       app: &mut AppModel,
       window_id: WindowId,
   ) -> Task<Message> {
       // Close window, cleanup state
   }
   
   pub(super) fn dialog_window_view(
       window_id: WindowId,
       dialog: &ShowDialog,
   ) -> Element<Message> {
       // Dispatcher: routes to appropriate dialog view function
   }
   ```
3. Define message variants:
   ```rust
   pub enum Message {
       OpenDialog(ShowDialog),
       CloseDialogWindow(WindowId),
       DialogWindowClosed(WindowId),  // From window close event
       ...
   }
   ```
4. Implement update handlers for all window lifecycle messages
5. Create `disks-ui/src/ui/dialogs/window.rs`:
   - Dispatcher function that calls appropriate view function based on `ShowDialog` variant
6. Update existing dialog view functions to work in window context (if needed)
7. Test: Open Info dialog using new infrastructure, verify it still works

**Test Plan:**
- Unit tests for helper functions (if possible without full app context)
- Manual: Open Info dialog via new infrastructure
- Verify window spawns and tracks correctly in `dialog_windows`
- Close window, verify it's removed from HashMap

**Done When:**
- [ ] Infrastructure functions compile and are callable
- [ ] `AppModel` tracks dialog windows by ID in HashMap
- [ ] Open/close messages route correctly through update loop
- [ ] Window view dispatcher can render all 17 dialog types (even if not migrated yet)
- [ ] PoC Info dialog still works with new infrastructure
- [ ] Code is documented (comments explain message flow)

**Estimated effort:** 3-6 hours

---

## Task 4: Migrate Confirmation Dialogs

**Scope:** Convert simple confirmation dialogs to use window infrastructure.

**Dialogs included:**
- `ShowDialog::DeletePartition`
- `ShowDialog::ConfirmAction` (generic confirmation)

**Files/Areas:**
- `disks-ui/src/ui/dialogs/view/common.rs` (adapt `confirmation()` function)
- `disks-ui/src/ui/app/update/mod.rs` (update button press handlers to spawn windows)
- All call sites that create these dialogs

**Steps:**
1. Find all call sites for `DeletePartition` and `ConfirmAction` dialogs
2. Replace `app.dialog = Some(ShowDialog::DeletePartition(...))` with:
   ```rust
   spawn_dialog_window(app, ShowDialog::DeletePartition(...))
   ```
3. Update `confirmation()` view function in `common.rs`:
   - Ensure it works in window container (may need window-specific styling)
   - Test button message routing
4. Test delete partition flow:
   - Select partition
   - Click Delete button
   - Verify confirmation opens in separate window
   - Click Delete → operation proceeds, window closes
   - Test: Click Cancel → window closes, no operation
   - Test: Click X button → same as Cancel
5. Test generic confirmation flows (e.g., check filesystem, repair filesystem)

**Test Plan:**
- Manual: Select partition, click Delete
- Verify confirmation opens in separate window (not overlay)
- Test all button paths (Delete, Cancel, X)
- Verify async operations (if any) complete correctly
- Check that parent window refocuses after close

**Done When:**
- [ ] Delete and generic confirmations open in windows
- [ ] All button actions work correctly (Delete, Cancel, X)
- [ ] Window closes after action completes
- [ ] No regressions in delete functionality
- [ ] Running indicator shows correctly for async confirmations

**Estimated effort:** 2-3 hours

---

## Task 5: Migrate Partition Dialogs

**Scope:** Convert partition creation and editing dialogs.

**Dialogs:**
- `ShowDialog::AddPartition`
- `ShowDialog::EditPartition`
- `ShowDialog::ResizePartition`

**Files/Areas:**
- `disks-ui/src/ui/dialogs/view/partition.rs`
- `disks-ui/src/ui/app/update/mod.rs`
- `disks-ui/src/ui/volumes/update/create.rs`
- `disks-ui/src/ui/volumes/update/partition.rs`

**Steps:**
1. Update spawn call sites for each dialog type (replace overlay with window spawn)
2. Ensure dialog messages route to correct window:
   - `CreateMessage` variants (size update, type selection, etc.)
   - `EditPartitionMessage` variants
   - `ResizePartitionMessage` variants
3. Test create partition:
   - Open dialog from free space
   - Test form validation (size limits, name length)
   - Test filesystem type selection (dropdown or radio list)
   - Test encryption checkbox + password fields
   - Test unit conversion (MB/GB inputs)
   - Click Apply → verify partition created
4. Test edit partition:
   - Open dialog from existing partition
   - Test partition type changes
   - Test flag toggles (bootable, hidden, system)
   - Click Apply → verify changes saved
5. Test resize:
   - Open dialog
   - Test slider/input synchronization
   - Test min/max bounds enforcement
   - Click Apply → verify resize completes

**Test Plan:**
- Manual: Create partition with various filesystems (ext4, NTFS, BTRFS)
- Manual: Create encrypted partition (test password validation)
- Manual: Edit partition flags
- Manual: Resize partition to different sizes (grow and shrink if possible)
- Verify all operations succeed and windows close properly
- Check for state desync issues (form inputs not updating)

**Done When:**
- [ ] All three partition dialogs open in windows
- [ ] Form inputs update correctly (no state desync between window and model)
- [ ] Validation works (size limits, name constraints, password match)
- [ ] Success/error handling works as before
- [ ] Async operations show progress correctly
- [ ] Window closes on success, stays open on error with error message

**Estimated effort:** 4-6 hours

---

## Task 6: Migrate Format Dialogs

**Scope:** Convert format and filesystem label dialogs.

**Dialogs:**
- `ShowDialog::FormatPartition`
- `ShowDialog::FormatDisk`
- `ShowDialog::EditFilesystemLabel`

**Files/Areas:**
- `disks-ui/src/ui/dialogs/view/partition.rs`
- `disks-ui/src/ui/dialogs/view/disk.rs`
- `disks-ui/src/ui/volumes/update/format.rs`

**Steps:**
1. Update spawn call sites for format dialogs
2. Test format partition:
   - Open dialog from partition
   - Select filesystem type
   - Toggle encryption
   - Test password validation (match, required if encryption enabled)
   - Click Format → verify async operation runs
   - Verify progress indicator during format
   - Verify window closes on completion
3. Test format disk:
   - Open from disk menu
   - Select partitioning scheme (GPT, MBR, None)
   - Test erase options (quick vs. overwrite)
   - Click Format → verify operation
4. Test edit filesystem label:
   - Open from mounted filesystem
   - Enter new label
   - Test special character handling
   - Click Apply → verify label changes

**Test Plan:**
- Manual: Format partition with encryption enabled
- Manual: Format partition without encryption
- Manual: Format entire disk (WARNING: use test disk/image!)
- Manual: Edit filesystem label on ext4 and NTFS
- Verify all operations complete successfully
- Check error handling (e.g., format operation fails)

**Done When:**
- [ ] Format dialogs open in windows
- [ ] Async formatting operations show progress correctly
- [ ] Errors display in dialog without closing it prematurely
- [ ] Password validation works
- [ ] Label editing works for supported filesystems

**Estimated effort:** 3-4 hours

---

## Task 7: Migrate Encryption Dialogs

**Scope:** Convert LUKS-related dialogs.

**Dialogs:**
- `ShowDialog::UnlockEncrypted`
- `ShowDialog::ChangePassphrase`
- `ShowDialog::TakeOwnership`
- `ShowDialog::EditEncryptionOptions`

**Files/Areas:**
- `disks-ui/src/ui/dialogs/view/encryption.rs`
- `disks-ui/src/ui/volumes/update/encryption.rs`

**Steps:**
1. Update spawn call sites for encryption dialogs
2. Test unlock:
   - Select locked LUKS volume
   - Click Unlock
   - Enter password
   - Test show/hide toggle for password field
   - Click Unlock → verify volume unlocks
   - Test incorrect password → error shown
3. Test change passphrase:
   - Open from unlocked LUKS volume
   - Enter current passphrase
   - Enter new passphrase + confirm
   - Test validation (current required, new match confirm)
   - Click Apply → verify passphrase changed
4. Test take ownership:
   - Open from mounted encrypted filesystem
   - Test recursive checkbox
   - Click Apply → verify ownership change async operation
5. Test edit encryption options:
   - Open dialog
   - Test checkboxes (mount at startup, unlock at startup, etc.)
   - Click Save → verify options saved to UDisks2 config

**Test Plan:**
- Manual: Create encrypted partition, lock it, then unlock via dialog
- Manual: Change passphrase on unlocked encrypted volume
- Manual: Take ownership of files in encrypted filesystem
- Manual: Edit encryption mount options
- Verify all operations work as before

**Done When:**
- [ ] All encryption dialogs open in windows
- [ ] Password fields work (toggle visibility)
- [ ] Validation logic unchanged and functional
- [ ] Async operations (unlock, take ownership) show progress
- [ ] Error handling works (wrong password, operation failure)

**Estimated effort:** 3-4 hours

---

## Task 8: Migrate Mount Options Dialog

**Scope:** Convert edit mount options dialog.

**Dialogs:**
- `ShowDialog::EditMountOptions`

**Files/Areas:**
- `disks-ui/src/ui/dialogs/view/mount.rs`
- `disks-ui/src/ui/volumes/update/mount.rs`

**Steps:**
1. Update spawn call site for mount options dialog
2. Test dialog:
   - Open from partition context menu
   - Test all checkbox states:
     - Mount at startup
     - Show in UI
     - Require authorization to mount
   - Test text input fields:
     - Custom mount point
     - Display name
     - Icon name
     - Symbolic icon name
   - Test "Other options" text area (custom mount flags)
3. Click Save:
   - Verify options written to UDisks2 configuration
   - Verify window closes
4. Click Revert/Cancel:
   - Verify no changes applied
   - Verify window closes

**Test Plan:**
- Manual: Edit mount options for a partition
- Toggle various checkboxes
- Enter custom mount point (e.g., `/mnt/mydata`)
- Add custom mount options (e.g., `noatime`)
- Save and verify changes persist (umount/remount if needed)
- Revert changes and verify dialog doesn't save

**Done When:**
- [ ] Mount options dialog opens in window
- [ ] All form controls functional (checkboxes, text inputs)
- [ ] Save writes options via UDisks2 correctly
- [ ] Revert/Cancel doesn't apply changes
- [ ] No regressions in mount options functionality

**Estimated effort:** 2-3 hours

---

## Task 9: Migrate Image Dialogs

**Scope:** Convert disk image creation/management dialogs.

**Dialogs:**
- `ShowDialog::NewDiskImage`
- `ShowDialog::AttachDiskImage`
- `ShowDialog::ImageOperation`

**Files/Areas:**
- `disks-ui/src/ui/dialogs/view/image.rs`
- `disks-ui/src/ui/app/update/image.rs`

**Steps:**
1. Update spawn call sites for image dialogs
2. Test new disk image:
   - Open dialog
   - Enter size (test unit conversions)
   - Click path picker → COSMIC file chooser opens
   - Select save location
   - Click Create → verify image file created
   - Verify progress indicator during creation
3. Test attach disk image:
   - Open dialog
   - Click path picker → COSMIC file chooser opens
   - Select existing image file (.img, .iso, etc.)
   - Test read-only checkbox
   - Click Attach → verify image appears as drive
4. Test image operation progress:
   - Trigger image create or restore operation
   - Verify progress dialog shows
   - Test cancellation (if supported)
   - Verify window closes on completion

**Test Plan:**
- Manual: Create new 10MB disk image
- Manual: Attach existing image file
- Manual: Detach image (from different UI, but verify no issues)
- Verify file picker still works (async COSMIC file dialog within modal window)
- Check that image operations complete successfully or show errors

**Done When:**
- [ ] Image dialogs open in windows
- [ ] Path pickers functional (COSMIC file dialogs work within modal windows)
- [ ] Image operations complete successfully (create, attach, restore)
- [ ] Progress indicators show correctly
- [ ] Cancellation works if implemented

**Estimated effort:** 3-5 hours

---

## Task 10: Migrate SMART and Busy Dialogs

**Scope:** Convert remaining dialogs.

**Dialogs:**
- `ShowDialog::SmartData`
- `ShowDialog::UnmountBusy`

**Files/Areas:**
- `disks-ui/src/ui/dialogs/view/disk.rs`
- `disks-ui/src/ui/dialogs/view/mount.rs`
- `disks-ui/src/ui/app/update/smart.rs`

**Steps:**
1. Update spawn call sites for SMART and unmount busy dialogs
2. Test SMART data:
   - Open from disk with SMART support
   - Verify table displays (scrollable if many attributes)
   - Test window resize behavior
   - Click Close → verify window closes
3. Test unmount busy:
   - Create busy mount (open files, cwd in mount)
   - Attempt unmount → busy dialog should appear
   - Verify process list displays correctly
   - Test kill button for process
   - Verify unmount succeeds after killing process
   - Test Close button (abandon unmount)

**Test Plan:**
- Manual: Open SMART data dialog for a drive (if available)
- Manual: Trigger unmount busy by keeping files open, test process display and killing
- Verify scrollable content works in window
- Check that kill operations work as expected

**Done When:**
- [ ] SMART data displays in window (scrollable if needed)
- [ ] Unmount busy dialog shows processes correctly
- [ ] Kill actions work and update UI
- [ ] Both dialogs close properly

**Estimated effort:** 2-3 hours

---

## Task 11: Clean Up Old Dialog Overlay Code

**Scope:** Remove legacy overlay dialog rendering and finalize migration.

**Files/Areas:**
- `disks-ui/src/ui/app/mod.rs` (remove `fn dialog()` override)
- `disks-ui/src/ui/app/view.rs` (remove `dialog()` function L28-119)
- `disks-ui/src/ui/app/state.rs` (remove or repurpose `pub dialog: Option<ShowDialog>`)

**Steps:**
1. Verify all 17 dialog types are migrated to windows:
   - [ ] Info
   - [ ] DeletePartition
   - [ ] ConfirmAction
   - [ ] AddPartition
   - [ ] EditPartition
   - [ ] ResizePartition
   - [ ] FormatPartition
   - [ ] FormatDisk
   - [ ] EditFilesystemLabel
   - [ ] UnlockEncrypted
   - [ ] ChangePassphrase
   - [ ] TakeOwnership
   - [ ] EditEncryptionOptions
   - [ ] EditMountOptions
   - [ ] NewDiskImage
   - [ ] AttachDiskImage
   - [ ] ImageOperation
   - [ ] SmartData
   - [ ] UnmountBusy
2. Remove `fn dialog()` implementation in `impl Application for AppModel`
3. Remove old `pub(crate) fn dialog(app: &AppModel) -> Option<Element<'_, Message>>` function from view.rs
4. Remove or repurpose `app.dialog: Option<ShowDialog>` field:
   - If still used for internal tracking, keep and document
   - If fully replaced by `dialog_windows`, remove
5. Search codebase for any remaining references to overlay dialog logic:
   ```bash
   grep -r "app.dialog = Some" disks-ui/src/
   ```
6. Run full test suite:
   ```bash
   cargo test --workspace
   cargo clippy --workspace --all-features
   cargo fmt --all --check
   ```
7. Manual smoke test: exercise all 17 dialogs end-to-end
8. Update any integration tests that relied on dialog overlay behavior

**Test Plan:**
- Full manual test of all 17 dialog types (comprehensive checklist)
- Verify no overlay artifacts remain (no ghost dialogs)
- Confirm no compiler warnings
- Run automated tests

**Done When:**
- [ ] Old overlay code removed (functions, state fields)
- [ ] All tests pass
- [ ] No clippy warnings
- [ ] All 17 dialogs functional as separate windows
- [ ] No regressions found in manual testing
- [ ] Code is cleaner (reduced LOC, clearer separation of concerns)

**Estimated effort:** 2-3 hours

---

## Estimated Total Effort

- **Research & PoC (Tasks 1-2):** 6-12 hours
- **Infrastructure (Task 3):** 3-6 hours
- **Migration (Tasks 4-10):** 19-28 hours
- **Cleanup (Task 11):** 2-3 hours
- **Total:** 30-49 hours (4-6 full work days)

---

## Commit Message Examples

- `feat(dialogs): research and document COSMIC window management approach`
- `feat(dialogs): implement proof-of-concept modal window for Info dialog`
- `feat(dialogs): add dialog window infrastructure and lifecycle management`
- `feat(dialogs): migrate confirmation dialogs to modal windows`
- `feat(dialogs): migrate partition creation/edit dialogs to modal windows`
- `feat(dialogs): migrate format dialogs to modal windows`
- `feat(dialogs): migrate encryption dialogs to modal windows`
- `feat(dialogs): migrate mount options dialog to modal window`
- `feat(dialogs): migrate image dialogs to modal windows`
- `feat(dialogs): migrate SMART and unmount-busy dialogs to modal windows`
- `refactor(dialogs): remove legacy overlay dialog rendering system`
- `docs: update README to reflect modal dialog windows (V1 goal #1)`

---

## Testing Checklist (for Task 11 and final validation)

### All Dialog Types
- [ ] Info dialog
- [ ] Delete partition confirmation
- [ ] Generic confirmation (repair, check filesystem, etc.)
- [ ] Create partition
- [ ] Edit partition
- [ ] Resize partition
- [ ] Format partition
- [ ] Format disk
- [ ] Edit filesystem label
- [ ] Unlock encrypted volume
- [ ] Change passphrase
- [ ] Take ownership
- [ ] Edit encryption options
- [ ] Edit mount options
- [ ] New disk image
- [ ] Attach disk image
- [ ] Image operation progress
- [ ] SMART data
- [ ] Unmount busy (with process list)

### Common Test Cases (apply to all dialogs where relevant)
- [ ] Dialog opens in separate window
- [ ] Dialog is centered on parent window
- [ ] Parent window is non-interactive (modal)
- [ ] Dialog can be moved with mouse
- [ ] X button closes dialog (same as Cancel)
- [ ] Escape key closes dialog
- [ ] Enter key confirms (where applicable)
- [ ] Form validation works
- [ ] Async operations show progress
- [ ] Errors display in dialog
- [ ] Dialog closes on success
- [ ] Dialog stays open on error (shows error message)
- [ ] Closing parent closes all child dialogs

---

## Notes for Implementation

1. **Test incrementally**: After each dialog migration task, manually test that specific dialog before moving to next.

2. **State management**: Ensure message routing is clear (which window sent which message). Use window IDs consistently.

3. **Error messages**: All dialog errors should remain in dialog, not close the window and show error elsewhere (unless operation completes).

4. **Edge cases**:
   - Multiple errors triggering multiple Info dialogs (should all appear)
   - Dialog open when parent window closes (cleanup all dialogs)
   - Dialog open when another dialog spawn is requested (handle gracefully)

5. **Performance**: Window spawn time should be <200ms for good UX. If slower, investigate optimization.

6. **Accessibility**: Ensure dialogs work with keyboard navigation, screen readers if COSMIC runtime provides support.

7. **Documentation**: Update in-code comments to explain window-based dialog architecture for future contributors.
