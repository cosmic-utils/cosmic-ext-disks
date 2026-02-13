# BTRFS Feature Enhancements — Implementation Tasks

**Branch:** `feature/btrfs-features`  
**Total:** 10 features across V2.1 (6 features) and V2.2 (4 features)

---

## Phase 1: V2.1 Quick Wins (2.6 weeks)

### Task 1.1: Read-Only Protection Toggle ⭐⭐⭐⭐ (0.5 weeks)

**Scope:** Add UI controls to view and toggle read-only flag on subvolumes

**Files:**
- `disks-ui/src/ui/btrfs/view.rs` (add toggle column)
- `disks-ui/src/ui/btrfs/mod.rs` (handle toggle message)
- `disks-btrfs-helper/src/main.rs` (add set_readonly command)
- `disks-ui/i18n/en/cosmic_ext_disks.ftl` (new strings)

**Steps:**
1. Add `set_readonly` subcommand to helper binary
   - Accept `--subvolume-id` and `--readonly true|false` flags
   - Use `Subvolume::set_read_only(bool)` from btrfsutil
   - Return success/error JSON

2. Add read-only column to subvolume list
   - Show 🔒 lock icon for read-only subvolumes
   - Show unlocked icon or empty for writable
   - Use icon button for inline toggle

3. Add confirmation dialog for making read-only
   - Warn: "This will prevent modifications to the subvolume"
   - Checkbox: "Set all new snapshots as read-only by default"
   - OK/Cancel buttons

4. Wire toggle action through pkexec helper
   - Message: `BtrfsMessage::ToggleReadOnly(u64)`
   - Call helper, reload subvolume list on success
   - Show error toast on failure

5. Add localization strings
   - `btrfs-readonly = Read-Only`
   - `btrfs-toggle-readonly-confirm = Make this subvolume read-only?`
   - `btrfs-toggle-readonly-warning = This will prevent any modifications until you make it writable again.`

**Test Plan:**
- Toggle read-only on/off for writable subvolume
- Verify icon changes immediately after reload
- Confirm dialog shows and cancellation works
- Try modifying read-only subvolume (should fail with kernel error)

**Done When:**
- [ ] Helper supports set_readonly command
- [ ] UI shows lock icon for read-only subvolumes
- [ ] Toggle button functional with confirmation
- [ ] Localization complete (English)
- [ ] Manual testing passed

---

### Task 1.2: Creation Timestamps Display ⭐⭐⭐ (0.3 weeks)

**Scope:** Show creation and modification times in subvolume list

**Files:**
- `disks-ui/src/ui/btrfs/view.rs` (add timestamp columns)
- `disks-ui/i18n/en/cosmic_ext_disks.ftl` (new strings)

**Steps:**
1. Add "Created" and "Modified" columns to subvolume table
   - Use `BtrfsSubvolume.created` and `modified` (already populated from btrfsutil)
   - Format as relative time: "2 days ago", "5 minutes ago"
   - Use chrono's humanize or custom formatter

2. Add tooltip with exact timestamp
   - On hover, show: "2026-02-13 14:30:45 PST"
   - Use `DateTime::format()` with locale-aware format

3. Make columns sortable
   - Click column header to sort by date (newest/oldest first)
   - Add sort indicator (▲ ▼)

4. Add localization strings
   - `btrfs-created = Created`
   - `btrfs-modified = Modified`
   - `btrfs-timestamp-now = Just now`
   - `btrfs-timestamp-minutes = {$n} minutes ago`
   - `btrfs-timestamp-hours = {$n} hours ago`
   - `btrfs-timestamp-days = {$n} days ago`
   - `btrfs-timestamp-weeks = {$n} weeks ago`
   - `btrfs-timestamp-months = {$n} months ago`
   - `btrfs-timestamp-years = {$n} years ago`

5. Handle edge cases
   - Very old subvolumes (show date instead of relative time after 1 year)
   - Future timestamps (corrupted metadata - show "Invalid date")

**Test Plan:**
- Verify timestamps match `btrfs subvolume show <path>` output
- Check relative times update correctly
- Hover to see exact timestamp
- Sort by created/modified date

**Done When:**
- [ ] Created and Modified columns visible
- [ ] Relative time formatting working
- [ ] Tooltips show exact timestamps
- [ ] Sorting functional
- [ ] Localization complete

---

### Task 1.3: Automatic Snapshot Naming ⭐⭐⭐ (0.5 weeks)

**Scope:** Template system for automatic snapshot names with live preview

**Files:**
- `disks-ui/src/ui/dialogs/snapshot.rs` (add template UI)
- `disks-ui/src/config.rs` (save template preference)
- `disks-ui/src/utils/naming.rs` (new file - template engine)
- `disks-ui/i18n/en/cosmic_ext_disks.ftl` (new strings)

**Steps:**
1. Create naming template engine `disks-ui/src/utils/naming.rs`
   - Parse template strings: `{name}`, `{date}`, `{time}`, `{action}`
   - Support variables:
     * `{name}` - source subvolume name
     * `{date}` - current date (YYYY-MM-DD)
     * `{time}` - current time (HHMM)
     * `{action}` - optional action description
     * `{n}` - sequential number
   - Function: `fn expand_template(template: &str, vars: &HashMap<&str, &str>) -> String`

2. Add template dropdown to snapshot dialog
   - Preset templates:
     * "Timestamped: `@{name}-{date}-{time}`" → `@home-2026-02-13-1430`
     * "Date Only: `@{name}-{date}`" → `@home-2026-02-13`
     * "Sequential: `@{name}-snapshot-{n}`" → `@home-snapshot-001`
     * "Action: `@{name}-{action}`" → `@root-before-update`
     * "Custom..." → text field
   - Remember last used template in config

3. Add live preview
   - Text below template showing expanded name
   - Update in real-time as user types
   - Validate: no special characters, max length, no duplicates

4. Add action description field (optional)
   - Text input: "What are you snapshotting before?"
   - Examples: "before-update", "before-install", "working-backup"
   - Only shown for Action template

5. Add custom template editor to settings
   - Settings → BTRFS → "Default Snapshot Template"
   - Dropdown with preset + custom option
   - Help text explaining variables

6. Add localization strings
   - `btrfs-template-timestamped = Timestamped`
   - `btrfs-template-date-only = Date Only`
   - `btrfs-template-sequential = Sequential`
   - `btrfs-template-action = Action-based`
   - `btrfs-template-custom = Custom...`
   - `btrfs-template-preview = Preview:`
   - `btrfs-action-description = Action description (optional):`
   - `btrfs-action-placeholder = before-update`

**Test Plan:**
- Create snapshot with each preset template
- Verify names match preview
- Test custom template with all variables
- Check sequential numbers increment correctly
- Verify template preference is saved

**Done When:**
- [ ] Template engine implemented and tested
- [ ] Dropdown in snapshot dialog working
- [ ] Live preview functional
- [ ] Settings page integration complete
- [ ] Localization complete

---

### Task 1.4: Default Subvolume Management ⭐⭐⭐ (0.5 weeks)

**Scope:** Identify and set the default boot subvolume

**Files:**
- `disks-btrfs-helper/src/main.rs` (add get_default and set_default commands)
- `disks-ui/src/ui/btrfs/view.rs` (add DEFAULT badge)
- `disks-ui/src/ui/btrfs/mod.rs` (handle set default message)
- `disks-ui/i18n/en/cosmic_ext_disks.ftl` (new strings)

**Steps:**
1. Add `get_default` command to helper binary
   - Use `btrfsutil::get_default_subvolume(mountpoint)` to get ID
   - Return default subvolume ID as u64
   - Include in subvolume list response

2. Mark default subvolume in BtrfsSubvolume struct
   - Add `is_default: bool` field to struct
   - Populate during list_subvolumes by comparing IDs

3. Add visual indicators in UI
   - Show "DEFAULT" badge next to default subvolume name
   - Use distinct color (semantic::warning or custom gold)
   - Icon: 📌 or ⭐

4. Add "Set as Default" button/menu item
   - Show in context menu and detail panel
   - Disabled for already-default subvolume
   - Opens confirmation dialog

5. Create confirmation dialog
   - Title: "Set Default Boot Subvolume?"
   - Warning: "This will change which subvolume is mounted at boot. Make sure the target subvolume has a bootable system."
   - Show: Current default → New default
   - Checkbox: "I understand this affects boot configuration"
   - OK/Cancel buttons

6. Add `set_default` command to helper binary
   - Accept `--subvolume-id <id>` flag
   - Use `btrfsutil::set_default_subvolume(id, mountpoint)`
   - Return success/error

7. Wire set default action
   - Message: `BtrfsMessage::SetDefaultSubvolume(u64)`
   - Call helper, reload subvolume list on success
   - Show success toast: "Default subvolume updated. Changes take effect after reboot."

8. Add localization strings
   - `btrfs-default-badge = DEFAULT`
   - `btrfs-set-default = Set as Default`
   - `btrfs-set-default-confirm-title = Set Default Boot Subvolume?`
   - `btrfs-set-default-confirm-body = This will change which subvolume is mounted at boot. Make sure the target subvolume has a bootable system.`
   - `btrfs-set-default-confirm-checkbox = I understand this affects boot configuration`
   - `btrfs-set-default-success = Default subvolume updated. Changes take effect after reboot.`

**Test Plan:**
- Verify DEFAULT badge appears on correct subvolume
- Set different subvolume as default
- Reboot and verify new default is mounted at `/`
- Try setting snapshot as default (rollback scenario)
- Cancel dialog and verify no changes

**Done When:**
- [ ] Helper supports get_default and set_default commands
- [ ] UI shows DEFAULT badge correctly
- [ ] Set Default button working with confirmation
- [ ] Tested with actual boot (VM recommended)
- [ ] Localization complete

---

### Task 1.5: Quick Snapshot Context Menu ⭐⭐⭐⭐ (0.5 weeks)

**Scope:** Right-click context menu with common operations and keyboard shortcuts

**Files:**
- `disks-ui/src/ui/btrfs/view.rs` (add context menu on right-click)
- `disks-ui/src/ui/btrfs/mod.rs` (handle keyboard shortcuts)
- `disks-ui/i18n/en/cosmic_ext_disks.ftl` (new strings)

**Steps:**
1. Add right-click detection to subvolume list
   - Use libcosmic's context menu widget
   - Show menu on right-click or context menu key
   - Position near cursor

2. Implement context menu items
   - 📸 Quick Snapshot Now (Ctrl+T)
     * Uses automatic naming template
     * No dialog, instant snapshot
     * Shows success toast with snapshot name
   - 📋 Properties (Ctrl+I)
     * Opens detail panel with full info
   - 🔒 Make Read-Only / Make Writable
     * Toggle read-only flag (Task 1.1)
     * Show confirmation for read-only
   - 📌 Set as Default (for boot subvolumes)
     * Open set default dialog (Task 1.4)
     * Hidden for snapshots (not bootable)
   - 🗑️ Delete (Del)
     * Open delete confirmation dialog
   - ─────────── (separator)
   - ❌ Cancel

3. Add keyboard shortcuts
   - Ctrl+T: Quick snapshot of selected subvolume
   - Ctrl+I: Show properties
   - Del: Delete selected subvolume
   - Escape: Close context menu
   - Up/Down: Navigate menu items
   - Enter: Activate selected item

4. Handle multi-selection in context menu
   - If multiple subvolumes selected:
     * Quick Snapshot All (Ctrl+T) → batch snapshot
     * Delete All (Del) → batch delete with count
     * Set All Read-Only → batch toggle
   - Disable Properties, Set Default (single-item only)

5. Add visual feedback
   - Show spinner in list item during operation
   - Toast notifications for success/errors
   - Update list immediately after operation

6. Add localization strings
   - `btrfs-context-quick-snapshot = Quick Snapshot Now`
   - `btrfs-context-properties = Properties`
   - `btrfs-context-make-readonly = Make Read-Only`
   - `btrfs-context-make-writable = Make Writable`
   - `btrfs-context-set-default = Set as Default`
   - `btrfs-context-delete = Delete`
   - `btrfs-quick-snapshot-success = Created snapshot: {$name}`
   - `btrfs-batch-snapshot-success = Created {$n} snapshots`

**Test Plan:**
- Right-click subvolume, verify menu appears
- Test each menu item individually
- Verify keyboard shortcuts work
- Test multi-selection operations
- Check menu positioning at screen edges

**Done When:**
- [ ] Context menu appears on right-click
- [ ] All menu items functional
- [ ] Keyboard shortcuts working
- [ ] Multi-selection handled correctly
- [ ] Localization complete

---

### Task 1.6: Deleted Subvolume Cleanup ⭐⭐ (0.3 weeks)

**Scope:** Show deleted subvolumes and provide cleanup button

**Files:**
- `disks-btrfs-helper/src/main.rs` (add list_deleted and sync_deleted commands)
- `disks-ui/src/ui/btrfs/view.rs` (add collapsible deleted section)
- `disks-ui/src/ui/btrfs/mod.rs` (handle cleanup message)
- `disks-ui/i18n/en/cosmic_ext_disks.ftl` (new strings)

**Steps:**
1. Add `list_deleted` command to helper binary
   - Use `btrfsutil::deleted_subvolumes(mountpoint)` iterator
   - Return list of deleted subvolume IDs
   - Include generation number for sorting

2. Add `sync_deleted` command to helper binary
   - Accept `--subvolume-id <id>` or `--all` flag
   - Run `btrfs subvolume sync <mountpoint> <id>` command
   - Parse output for completion status
   - Return number of subvolumes cleaned

3. Add collapsible "Deleted Subvolumes" section to UI
   - Below active subvolume list
   - Collapsed by default
   - Header shows count: "Deleted Subvolumes (3)"
   - Expand/collapse chevron icon

4. Show deleted subvolumes in expandable section
   - List with subvolume ID (no path available)
   - Show generation number
   - Display estimated space to reclaim (if possible)

5. Add "Clean Up All" button
   - Calls sync_deleted --all
   - Shows progress indicator
   - Toast on completion: "Cleaned up 3 deleted subvolumes"

6. Auto-refresh after cleanup operations
   - Reload deleted list after sync
   - Show empty state if no deleted subvolumes: "No deleted subvolumes pending cleanup"

7. Add localization strings
   - `btrfs-deleted-subvolumes = Deleted Subvolumes ({$n})`
   - `btrfs-deleted-subvolume-id = Subvolume ID`
   - `btrfs-deleted-generation = Generation`
   - `btrfs-cleanup-all = Clean Up All`
   - `btrfs-cleanup-success = Cleaned up {$n} deleted subvolumes`
   - `btrfs-no-deleted = No deleted subvolumes pending cleanup`

**Test Plan:**
- Delete a subvolume, verify it appears in deleted list
- Click "Clean Up All" and verify it disappears
- Check that space is actually reclaimed (df command)
- Test with no deleted subvolumes (empty state)

**Done When:**
- [ ] Helper supports list_deleted and sync_deleted commands
- [ ] Deleted section appears with correct count
- [ ] Clean Up All button functional
- [ ] Empty state handled
- [ ] Localization complete

---

## Phase 2: V2.2 Advanced Features (4.5 weeks)

### Task 2.1: Snapshot Relationship Visualization ⭐⭐⭐⭐ (1.5 weeks)

**Scope:** Tree view showing parent-child snapshot relationships

**Files:**
- `disks-ui/src/utils/snapshot_graph.rs` (new file - graph builder)
- `disks-ui/src/ui/btrfs/tree_view.rs` (new file - tree widget)
- `disks-ui/src/ui/btrfs/view.rs` (add toggle for list/tree view)
- `disks-ui/src/ui/btrfs/mod.rs` (build graph from subvolumes)
- `disks-ui/i18n/en/cosmic_ext_disks.ftl` (new strings)

**Steps:**
1. Create snapshot graph builder `disks-ui/src/utils/snapshot_graph.rs`
   - Struct: `SnapshotGraph { by_uuid: HashMap<Uuid, BtrfsSubvolume>, children: HashMap<Uuid, Vec<Uuid>> }`
   - Function: `fn build_graph(subvolumes: &[BtrfsSubvolume]) -> SnapshotGraph`
   - Algorithm:
     1. Index all subvolumes by UUID
     2. For each subvolume with parent_uuid, add to parent's children list
     3. Handle orphaned snapshots (parent deleted)
   - Function: `fn get_children(&self, uuid: &Uuid) -> Vec<&BtrfsSubvolume>`
   - Function: `fn get_parent(&self, subvol: &BtrfsSubvolume) -> Option<&BtrfsSubvolume>`

2. Create tree view widget `disks-ui/src/ui/btrfs/tree_view.rs`
   - Hierarchical list with indent levels
   - Expand/collapse buttons for subvolumes with children
   - Visual connection lines (└─, ├─, │)
   - Snapshot count badge on parents: "(3 snapshots)"
   - Click subvolume to select, expand/collapse with arrow

3. Add view toggle in UI
   - Toolbar with two buttons: "List View" | "Tree View"
   - Save preference in config
   - Default to list view (familiar)

4. Implement tree view rendering
   - Recursive render function
   - Track expansion state: `HashMap<Uuid, bool>`
   - Show snapshots nested under source subvolumes
   - Gray out stale snapshots (parent deleted)

5. Add relationship panel in detail view
   - Section: "Relationships"
   - Show: Parent, Children, Siblings
   - Clickable links to navigate to related subvolumes
   - If no relationships: "No snapshot relationships"

6. Add hover highlighting
   - On hover over snapshot, highlight its parent
   - On hover over parent, highlight all children
   - Use subtle background color change

7. Add localization strings
   - `btrfs-list-view = List View`
   - `btrfs-tree-view = Tree View`
   - `btrfs-snapshot-count = ({$n} snapshots)`
   - `btrfs-relationships = Relationships`
   - `btrfs-parent-subvolume = Parent Subvolume`
   - `btrfs-child-snapshots = Child Snapshots`
   - `btrfs-sibling-snapshots = Sibling Snapshots`
   - `btrfs-no-relationships = No snapshot relationships`
   - `btrfs-orphaned-snapshot = (parent deleted)`

**Test Plan:**
- Create snapshot chain: @root → @root-snapshot1 → @root-snapshot2
- Verify tree view shows hierarchy correctly
- Test expand/collapse functionality
- Check relationship panel shows correct parent/children
- Delete parent, verify orphaned snapshot indicator
- Test with flat structure (no snapshots) - should show all at root level

**Done When:**
- [ ] Snapshot graph builder implemented and tested
- [ ] Tree view widget functional
- [ ] List/Tree toggle working
- [ ] Relationship panel shows correct info
- [ ] Hover highlighting working
- [ ] Localization complete

---

### Task 2.2: Batch Operations ⭐⭐⭐ (1.0 weeks)

**Scope:** Multi-select with batch operations toolbar

**Files:**
- `disks-ui/src/ui/btrfs/view.rs` (add selection checkboxes and batch toolbar)
- `disks-ui/src/ui/btrfs/mod.rs` (handle batch operations with progress)
- `disks-ui/i18n/en/cosmic_ext_disks.ftl` (new strings)

**Steps:**
1. Add selection mode to UI state
   - `selection_mode: bool` - toggles checkbox visibility
   - `selected_subvolumes: HashSet<u64>` - tracks selected IDs
   - Button to enter/exit selection mode

2. Add selection checkboxes to list
   - Show checkbox column when selection_mode = true
   - Click checkbox to toggle selection
   - Click row selects (if in selection mode)
   - Show selected count in toolbar: "3 selected"

3. Implement batch operations toolbar
   - Shown when selection_mode = true and selected > 0
   - Buttons:
     * 📸 Snapshot All
     * 🔒 Set All Read-Only
     * 🗑️ Delete All
     * ❌ Cancel Selection
   - Disabled if no items selected

4. Implement batch snapshot operation
   - For each selected subvolume ID:
     1. Generate name using template
     2. Call snapshot helper
     3. Track progress
   - Show progress bar: "Snapshotting 2 of 5..."
   - Collect errors for error summary
   - Toast on completion: "Created 5 snapshots (2 failed)"

5. Implement batch delete operation
   - Confirmation dialog: "Delete 5 subvolumes?"
   - List selected subvolumes by name
   - Warning about irreversibility
   - For each selected subvolume:
     1. Call delete helper
     2. Track progress
   - Progress bar: "Deleting 3 of 5..."
   - Toast: "Deleted 5 subvolumes (1 failed)"

6. Implement batch set read-only operation
   - Confirmation: "Set 5 subvolumes to read-only?"
   - No progress bar (fast operation)
   - For each selected subvolume:
     1. Call set_readonly helper
   - Toast: "Set 5 subvolumes to read-only (1 failed)"

7. Handle errors gracefully
   - Continue processing remaining items on error
   - Collect errors: `Vec<(u64, String)>`
   - Show error summary in expandable dialog
   - Format: "Failed operations: @home (permission denied), @var (busy)"

8. Add "Select All" and "Select None" buttons
   - Above subvolume list
   - Keyboard: Ctrl+A (select all), Escape (clear selection)

9. Add keyboard shortcuts
   - Ctrl+A: Select all
   - Ctrl+D: Deselect all
   - Ctrl+Shift+T: Batch snapshot
   - Delete: Batch delete

10. Add localization strings
    - `btrfs-selection-mode = Selection Mode`
    - `btrfs-select-all = Select All`
    - `btrfs-select-none = Select None`
    - `btrfs-selected-count = {$n} selected`
    - `btrfs-snapshot-all = Snapshot All`
    - `btrfs-set-all-readonly = Set All Read-Only`
    - `btrfs-delete-all = Delete All`
    - `btrfs-batch-confirm-delete = Delete {$n} subvolumes?`
    - `btrfs-batch-confirm-readonly = Set {$n} subvolumes to read-only?`
    - `btrfs-batch-progress = {$operation} {$current} of {$total}...`
    - `btrfs-batch-success = {$operation} completed: {$success} succeeded, {$failed} failed`
    - `btrfs-batch-errors = Failed operations:`

**Test Plan:**
- Enter selection mode, select multiple subvolumes
- Batch snapshot 5 subvolumes, verify all created
- Batch delete 3 subvolumes, verify all deleted
- Batch set read-only, verify flags changed
- Test with errors (permission denied, busy subvolume)
- Verify error summary shows correctly
- Test keyboard shortcuts

**Done When:**
- [ ] Selection mode with checkboxes working
- [ ] Batch toolbar showing with operations
- [ ] Batch snapshot, delete, set readonly functional
- [ ] Progress tracking and error handling working
- [ ] Keyboard shortcuts implemented
- [ ] Localization complete

---

### Task 2.3: Subvolume Usage Breakdown ⭐⭐⭐ (2.0 weeks)

**Scope:** Per-subvolume disk usage with quota groups (complex)

**Files:**
- `disks-btrfs-helper/src/main.rs` (add quota commands)
- `disks-ui/src/ui/btrfs/usage_view.rs` (new file - usage visualization)
- `disks-ui/src/ui/btrfs/view.rs` (add usage column and chart)
- `disks-ui/src/ui/dialogs/enable_quotas.rs` (new file - quota dialog)
- `disks-ui/i18n/en/cosmic_ext_disks.ftl` (new strings)

**Steps:**
1. Add quota management to helper binary
   - Command: `check_quotas --mountpoint <path>` → returns enabled/disabled
   - Command: `enable_quotas --mountpoint <path>` → enables quota groups
   - Command: `get_usage --mountpoint <path>` → returns usage for all subvolumes
   - Parse `btrfs qgroup show --sync <mountpoint>` output
   - Extract: ID, Referenced, Exclusive for each subvolume

2. Create SubvolumeUsage struct
   ```rust
   pub struct SubvolumeUsage {
       pub subvolume_id: u64,
       pub referenced: u64,  // Total bytes (including shared)
       pub exclusive: u64,   // Bytes exclusive to this subvolume
   }
   ```

3. Check quota status on BTRFS view load
   - Call check_quotas helper
   - Store result: `quotas_enabled: Option<bool>`
   - If disabled, show info banner with "Enable Quotas" button

4. Create quota enable dialog `disks-ui/src/ui/dialogs/enable_quotas.rs`
   - Title: "Enable Quota Groups for Usage Tracking?"
   - Body: "BTRFS quotas allow per-subvolume disk usage tracking but may reduce performance by 5-10%."
   - Pros:
     * ✅ See disk usage per subvolume
     * ✅ Identify large snapshots consuming space
     * ✅ Enable disk usage charts
   - Cons:
     * ⚠️ 5-10% performance overhead for writes
     * ⚠️ Slight increase in metadata size
   - Checkbox: "Enable quotas for this filesystem"
   - Buttons: Enable / Cancel

5. Implement enable quotas flow
   - On "Enable" click, call enable_quotas helper
   - Show progress: "Enabling quotas and rescanning filesystem..."
   - Can take 1-5 minutes depending on size
   - On success, reload usage data
   - On error, show error message

6. Fetch usage data (if quotas enabled)
   - Call get_usage helper periodically (every 30s)
   - Join usage data with subvolumes by ID
   - Cache results to avoid expensive rescan

7. Add usage column to subvolume list
   - Show "Referenced" size (total space)
   - Format with human-readable units (KB, MB, GB, TB)
   - Tooltip: "Referenced: 2.5 GB, Exclusive: 1.2 GB, Shared: 1.3 GB"
   - Sort by usage

8. Create usage visualization `disks-ui/src/ui/btrfs/usage_view.rs`
   - Pie chart showing space distribution:
     * Each subvolume as a slice
     * Color-coded by subvolume
     * Label with percentage
   - Bar chart showing exclusive vs referenced
   - Table view with columns: Name, Referenced, Exclusive, Shared, % of Total

9. Add usage section to detail panel
   - Show for selected subvolume
   - Breakdown:
     * Referenced: 2.5 GB (Total space including shared)
     * Exclusive: 1.2 GB (Unique to this subvolume)
     * Shared: 1.3 GB (Shared with other subvolumes)
   - Visual bar showing proportion

10. Handle quota disabled state gracefully
    - Show gray "—" in usage column
    - Banner: "Enable quotas to see usage per subvolume"
    - Hide usage charts

11. Add localization strings
    - `btrfs-enable-quotas = Enable Quotas`
    - `btrfs-quotas-disabled = Quotas are disabled. Usage per subvolume is not available.`
    - `btrfs-enable-quotas-title = Enable Quota Groups?`
    - `btrfs-enable-quotas-body = BTRFS quotas allow per-subvolume disk usage tracking but may reduce performance by 5-10%.`
    - `btrfs-enable-quotas-pros = Benefits:`
    - `btrfs-enable-quotas-cons = Drawbacks:`
    - `btrfs-enable-quotas-progress = Enabling quotas and rescanning filesystem...`
    - `btrfs-enable-quotas-success = Quotas enabled successfully`
    - `btrfs-usage-referenced = Referenced`
    - `btrfs-usage-exclusive = Exclusive`
    - `btrfs-usage-shared = Shared`
    - `btrfs-usage-breakdown = Usage Breakdown`
    - `btrfs-usage-chart = Usage Distribution`
    - `btrfs-usage-unavailable = —`

**Test Plan:**
- Test with quotas disabled: banner shows, usage column shows "—"
- Enable quotas via dialog, verify it succeeds
- Check usage data loads and matches `btrfs qgroup show` output
- Verify pie chart shows correct proportions
- Test with large filesystem (100+ subvolumes) for performance
- Disable quotas via CLI, verify UI handles gracefully

**Done When:**
- [ ] Quota check/enable commands in helper
- [ ] Enable quotas dialog functional
- [ ] Usage data fetching and parsing working
- [ ] Usage column showing in list
- [ ] Usage charts rendering correctly
- [ ] Performance acceptable (quotas enabled overhead is kernel-level)
- [ ] Localization complete

---

### Task 2.4: Search & Filter ⭐⭐ (0.5 weeks)

**Scope:** Real-time search and filter criteria for subvolume list

**Files:**
- `disks-ui/src/ui/btrfs/view.rs` (add search bar and filter UI)
- `disks-ui/src/ui/btrfs/mod.rs` (implement filtering logic)
- `disks-ui/src/ui/dialogs/filter_dialog.rs` (new file - advanced filter)
- `disks-ui/i18n/en/cosmic_ext_disks.ftl` (new strings)

**Steps:**
1. Add search bar to toolbar
   - Text input with magnifying glass icon
   - Placeholder: "Search subvolumes..."
   - Clear button (X) when text entered
   - Real-time filtering as user types

2. Implement basic search filtering
   - Match against subvolume name and path
   - Case-insensitive
   - Substring matching
   - Update list immediately (no debounce needed with small lists)

3. Add filter button next to search
   - Opens advanced filter dialog
   - Icon shows if filters active (blue/orange indicator)
   - Badge shows filter count: "3 filters active"

4. Create advanced filter dialog `disks-ui/src/ui/dialogs/filter_dialog.rs`
   - Title: "Filter Subvolumes"
   - Sections:
     * **By Type**
       - [ ] Show regular subvolumes
       - [ ] Show snapshots
       - (both checked by default)
     * **By Date**
       - Created after: [date picker]
       - Created before: [date picker]
     * **By Attributes**
       - [ ] Read-only only
       - [ ] Default subvolume only
       - [ ] Has children (is a parent)
     * **By Parent**
       - Dropdown: Select parent UUID to show only its snapshots
   - Buttons: Apply / Reset / Cancel

5. Create SubvolumeFilter struct
   ```rust
   pub struct SubvolumeFilter {
       pub name_contains: Option<String>,
       pub created_after: Option<DateTime<Local>>,
       pub created_before: Option<DateTime<Local>>,
       pub show_regular: bool,
       pub show_snapshots: bool,
       pub readonly_only: bool,
       pub default_only: bool,
       pub has_children_only: bool,
       pub parent_uuid: Option<Uuid>,
   }
   ```

6. Implement filter logic
   - Function: `fn apply_filter(subvolumes: &[BtrfsSubvolume], filter: &SubvolumeFilter) -> Vec<&BtrfsSubvolume>`
   - Apply each criterion:
     * Name: `path.to_string_lossy().to_lowercase().contains(name)`
     * Date: `created >= after && created <= before`
     * Type: `parent_uuid.is_some()` for snapshots
     * Attributes: check `is_readonly`, `is_default`
     * Parent: `parent_uuid == filter.parent_uuid`

7. Show filter status in UI
   - Above list: "Showing 12 of 56 subvolumes (3 filters active)"
   - Link to open filter dialog
   - "Clear all filters" button

8. Save filter preferences
   - Store last used filter in config
   - Option: "Remember filters" checkbox

9. Add keyboard shortcuts
   - Ctrl+F: Focus search bar
   - Ctrl+Shift+F: Open advanced filter dialog
   - Escape: Clear search / close filter dialog

10. Add localization strings
    - `btrfs-search-placeholder = Search subvolumes...`
    - `btrfs-filter = Filter`
    - `btrfs-filters-active = {$n} filters active`
    - `btrfs-filter-dialog-title = Filter Subvolumes`
    - `btrfs-filter-by-type = By Type`
    - `btrfs-filter-show-regular = Show regular subvolumes`
    - `btrfs-filter-show-snapshots = Show snapshots`
    - `btrfs-filter-by-date = By Date`
    - `btrfs-filter-created-after = Created after`
    - `btrfs-filter-created-before = Created before`
    - `btrfs-filter-by-attributes = By Attributes`
    - `btrfs-filter-readonly-only = Read-only only`
    - `btrfs-filter-default-only = Default subvolume only`
    - `btrfs-filter-has-children = Has children (is a parent)`
    - `btrfs-filter-by-parent = By Parent`
    - `btrfs-filter-parent-select = Select parent subvolume`
    - `btrfs-filter-apply = Apply`
    - `btrfs-filter-reset = Reset`
    - `btrfs-filter-clear-all = Clear all filters`
    - `btrfs-filter-showing = Showing {$shown} of {$total} subvolumes`

**Test Plan:**
- Type in search bar, verify list filters in real-time
- Test each filter criterion individually
- Test combinations of filters
- Verify "Showing X of Y" count is accurate
- Save and restore filter preferences
- Test with 100+ subvolumes for performance

**Done When:**
- [ ] Search bar functional with real-time filtering
- [ ] Advanced filter dialog implemented
- [ ] All filter criteria working correctly
- [ ] Filter status display accurate
- [ ] Performance acceptable with large lists
- [ ] Localization complete

---

## Final Integration & Testing

### Task 3.1: Integration Testing Checklist

**Scope:** Ensure all features work together without conflicts

**Test Matrix:**
| Feature | Dependencies | Test Scenario |
|---------|--------------|---------------|
| Read-Only Toggle | Context Menu | Toggle via context menu |
| Timestamps | Search/Filter | Filter by date range |
| Auto Naming | Quick Snapshot | Quick snapshot uses template |
| Default Subvolume | Context Menu | Set default via context menu |
| Context Menu | All Features | All menu items functional |
| Deleted Cleanup | (none) | Delete then cleanup |
| Snapshot Relationships | Tree View | Tree shows correct hierarchy |
| Batch Operations | Selection Mode | Batch snapshot with templates |
| Usage Breakdown | Quotas | Enable quotas, view usage |
| Search & Filter | Tree View | Search works in tree view |

**Steps:**
1. Test all features individually
2. Test feature combinations:
   - Quick snapshot → verify appears in tree → set read-only → verify lock icon
   - Search for subvolume → batch select results → batch delete
   - Enable quotas → view usage → filter by usage > 1GB
   - Create snapshot chain → verify tree relationships → set child as default
3. Performance test:
   - Load filesystem with 200+ subvolumes
   - Verify UI responsive (<100ms filter updates)
   - Check memory usage (no leaks)
4. Error handling:
   - Test with permission errors
   - Test with busy subvolumes
   - Test with corrupted metadata
5. Localization check:
   - All strings use fl!() macro
   - No hardcoded English text
   - Pluralization working correctly

---

### Task 3.2: Documentation & Polish

**Scope:** Update README, screenshots, and final polish

**Steps:**
1. Update README.md with feature list
   - Add "BTRFS Management" section
   - List all 10 features with descriptions
   - Add screenshots for each major feature

2. Take new screenshots
   - Main view with subvolume list
   - Tree view showing relationships
   - Context menu
   - Snapshot dialog with templates
   - Usage breakdown chart
   - Filter dialog
   - Save to screenshots/btrfs/

3. Add feature comparison table to README
   - Compare with GNOME Disks, Timeshift, Snapper
   - Highlight unique features (relationship visualization)

4. Write changelog entries
   - V2.1: Quick wins (6 features)
   - V2.2: Advanced features (4 features)

5. Update CONTRIBUTORS.md if needed

---

## Rollout Strategy

### V2.1 Release (6 features)
- **Target:** 3 weeks from start
- **Features:** Tasks 1.1-1.6 (Read-Only, Timestamps, Auto Naming, Default, Context Menu, Deleted Cleanup)
- **Focus:** Quick wins, improve daily UX dramatically
- **Testing:** 1 week internal testing, 1 week beta testing

### V2.2 Release (4 features)
- **Target:** 6 weeks from V2.1 release
- **Features:** Tasks 2.1-2.4 (Relationships, Batch, Usage, Search)
- **Focus:** Advanced power-user features
- **Testing:** 2 weeks internal testing, 2 weeks beta testing

---

## Dependencies

**No new external dependencies** - all features use existing libs:
- btrfsutil (already added)
- chrono (already added)
- uuid (already added)
- libcosmic widgets (already used)

**Internal dependencies** (task ordering):
- Task 1.5 (Context Menu) depends on 1.1, 1.3, 1.4 for menu items
- Task 2.2 (Batch Operations) depends on 1.3 (Auto Naming) for batch snapshot
- Task 2.3 (Usage Breakdown) is independent, can be done in parallel

**Recommended order:**
1. Phase 1 (V2.1): Do in sequence 1.1 → 1.2 → 1.3 → 1.4 → 1.5 → 1.6
2. Phase 2 (V2.2): Can parallelize 2.1 + 2.3, then 2.2, then 2.4

---

## Success Criteria

### V2.1 (Quick Wins)
- ✅ All 6 features implemented and tested
- ✅ No regressions in existing BTRFS functionality
- ✅ User feedback: "Much easier to manage snapshots"
- ✅ Performance: <50ms list render for 50 subvolumes

### V2.2 (Advanced)
- ✅ All 10 features complete
- ✅ Tree view works with 200+ subvolumes
- ✅ Batch operations handle errors gracefully
- ✅ Usage breakdown performs well even with quotas
- ✅ Search filters 100+ items in <100ms

### Overall
- ✅ Zero critical bugs in first month
- ✅ Positive user testimonials
- ✅ Competitive feature parity with Timeshift/Snapper
- ✅ Unique advantage: relationship visualization

---

## Notes

- This is an **additive** feature set - no breaking changes
- All features are optional/non-intrusive (progressive disclosure)
- Can ship incremental releases (V2.1 before V2.2)
- Focus on UX polish - this is what differentiates us from CLI tools
