# UI Refactor — Spec

Branch: `feature/ui-refactor`
Source: N/A (brief; 2026-02-06)

## Context
We want to refactor/replace the left sidepanel navigation to support a **treeview** with:
- Collapsible hierarchy: drives/images → containers → volumes → partitions (as applicable)
- Row layout: `Expander | Icon | Title | Trailing action buttons` (no multiline text)
- Trailing actions:
  - Primary action button: **Eject** (for removable drives) or **Unmount** (for mounted volumes)
  - No kebab/menu button (removed due to UX concerns)
- Optional section headers / grouping: Logical (future: LVM + Apple support) / Internal / External / Images

### Investigation (current code)
Current navigation is built on COSMIC’s built-in nav bar widget and model:
- Model: `cosmic::widget::nav_bar::Model` stored as `app.nav`
- Build/update: the model is rebuilt in `disks-ui/src/ui/app/update/nav.rs` via `app.nav.insert().text(...).icon(...).data(...).activate()`
- View: `disks-ui/src/ui/app/view.rs` renders `widget::nav_bar(nav_model, ...)`.

**Finding:** In this codebase, nav items are defined only by `text`, `icon`, and attached `data` (for page state). There’s no current support for:
- Custom per-row widget content (e.g., embedded buttons)
- A tree/hierarchy (expanders, child rows)
- Section/group headers within the built-in nav

**Conclusion:** To reliably support a treeview, embedded action buttons, and section headers, we should replace the built-in nav bar widget with a custom sidebar implementation (while either reusing `nav_bar::Model` for selection/state, or introducing a dedicated sidebar model).

## Goals
- Replace the current sidepanel nav rendering with a custom **treeview sidebar** that supports:
  - Expand/collapse per node
  - Row layout: `Expander | Icon | Title | Trailing action buttons`
  - Trailing actions: Eject/Unmount + kebab popup menu (Disk actions)
  - Section headers and grouped lists (Logical/Internal/External/Images)
- Preserve the current “active drive” semantics: selecting an item should activate the corresponding `DriveModel` page and update the window title.
- Keep behavior consistent across condensed vs non-condensed layouts.
- For now, selecting a child node (partition/container/volume) should still show the **parent drive’s** view; dedicated child views will be added later.
- Add COSMIC-native Open File dialogs for image create/save/load paths (research libcosmic/DE conventions before implementation).

## Non-Goals
- Redesign of the main content area pages (volumes view, dialogs) beyond what’s needed to keep navigation integration working.
- New backend functionality for LVM/Apple; only ensure the sidebar grouping scheme can accommodate future categories.

## Proposed Approach
1. Introduce a dedicated sidebar tree model describing grouped nodes:
  - Sections: Logical / Internal / External / Images
  - Node kinds: Drive / Image / Container / Volume / Partition (final set depends on what data is readily available)
  - Each node carries: id/key, title, icon, kind, expanded state, and supported actions.
2. Render the treeview using standard COSMIC/iced widgets (e.g., `Column`, `Container`, `Row`, `Button`, `Toggler`/custom expander control):
  - Section header rows (non-interactive)
  - Tree node rows with indentation based on depth
  - Row layout: `Expander | Icon | Title | Trailing actions`
    - Expander only shown for nodes with children
    - Title is single-line (truncate/ellipsize if needed)
    - Trailing actions:
     - Primary: Eject (removable drives) or Unmount (mounted volumes)
3. Disk actions parity:
  - Disk-menu actions remain available via the existing top-level “Disk” menu.
4. Selection + routing:
  - Clicking the node row selects/navigates.
  - Selecting a **drive** navigates to that drive as it does today.
  - Selecting a **child node** (partition/container/volume) keeps the current behavior of showing the drive page (no new UI yet).
  - Clicking expander toggles expansion without changing selection (unless we decide it should).
  - Clicking trailing action buttons triggers their message without changing selection.
  - Preserve existing “don’t switch while a dialog is open” behavior.
5. Categorization:
  - Centralize mapping from `DriveModel`/device attributes into the 4 sections.
  - Keep rules extensible for future LVM/Apple support.
6. Image operations UX:
  - Use COSMIC-standard Open File dialogs for image save/load.
  - Research libcosmic for existing file dialog components and mirror current COSMIC apps’ usage patterns.

## Concrete Mapping (current implementation)

### Sections

| Section | Rule |
|---|---|
| Images | `DriveModel.is_loop == true` OR `DriveModel.backing_file.is_some()` |
| External | `DriveModel.removable == true` |
| Internal | default |
| Logical | reserved for future (currently empty) |

### Node kinds

| Node kind | Data source | Children |
|---|---|---|
| Drive | `DriveModel` | `DriveModel.volumes` (tree of `VolumeNode`) |
| Volume/Container/Partition | `VolumeNode` | `VolumeNode.children` |

### Row actions

| Node | Primary action | Kebab menu |
|---|---|---|
| Drive | Eject/remove when `is_loop || removable || ejectable` | N/A |
| VolumeNode | Unmount when mounted | N/A |

## User/System Flows
- Startup: drives loaded → sidebar sections populated → first eligible drive selected.
- User selects drive: main view updates to that drive’s volumes; title updates.
- User expands a drive: containers/volumes/partitions appear beneath it.
- User selects a child node: drive view remains shown (child selection is UI-only for now).
- User presses Eject/Unmount: action runs without navigating.
- User opens kebab menu: chooses any Disk-menu-equivalent action.

## Risks & Mitigations
- **Condensed mode integration:** COSMIC’s built-in nav may behave differently in condensed layouts.
  - Mitigation: keep the custom sidebar behind `app.core.nav_bar_active()` and test in both condensed and normal modes; ensure width constraints match existing `max_width(280)` behavior.
- **Event handling conflicts:** embedded buttons inside a clickable row can steal/cascade events.
  - Mitigation: structure row so button clicks map to distinct messages; avoid wrapping the entire row in a single button when action buttons are present.
- **State duplication:** maintaining both `nav_bar::Model` and a new sidebar model can drift.
  - Mitigation: make the sidebar derived from the same source list and keep selection in one place.
- **Data availability for children:** building a partition/container tree may require more data than the current drive list refresh provides.
  - Mitigation: phase the implementation: start with drives + volumes, then add containers/partitions once the required models are available in UI state.

## Research Notes — COSMIC file dialogs
- libcosmic provides `cosmic::dialog::file_chooser` with `open` and `save` dialogs.
- Use `file_chooser::open::Dialog::new().title(...).open_file().await` for Open.
- Use `file_chooser::save::Dialog::new().title(...).save_file().await` for Save.
- Dialogs are async and should be wired through `Task::perform` and app messages.

## Acceptance Criteria
- [x] Sidebar renders a treeview with expand/collapse on nodes that have children.
- [x] Sidebar rows render as `Expander | Icon | Title | Trailing actions` with single-line title.
- [x] Sidebar rows include an inline Eject/Unmount action button that triggers without changing selection.
- [x] Disk-menu-equivalent actions remain available via the existing top-level Disk menu.
- [x] Sidebar shows section headers and groups items under: Logical / Internal / External / Images.
- [x] Selecting a drive still activates the correct page and updates window title.
- [x] Selecting a child node (partition/container/volume) does not change the main view yet (still shows the parent drive view).
- [x] No regressions in condensed vs non-condensed layouts (manual QA).

---

## Extended Scope — Disk Page Split View & Volumes Control Redesign

**Added:** 2026-02-06

### Context
After establishing the treeview sidebar, the next phase refines the main content area when a disk is selected. Currently, the entire view is consumed by the volumes control. This extended scope introduces:
1. A **split view** for disk pages: top 1/3 for disk-level info, bottom 2/3 for volume/container content.
2. A **redesigned disk info header** with improved layout and styling.
3. A **compact volumes control** that reduces vertical space and removes extraneous elements.
4. A **color-coded usage bar** below the volumes control showing stacked usage per volume with a legend.

### Goals
1. **Split disk page layout:**
   - Disk-level information (header) takes the top 1/3 of the content view.
   - Volume/container/partition content takes the bottom 2/3.
2. **Disk info header redesign:**
   - Layout (left to right):
     - Large icon
     - Name only (no serial), partitioning scheme beneath, serial beneath, all left-aligned
     - Right-aligned box displaying Used / Total size
   - Below this header: the volumes control.
3. **Volumes control compaction:**
   - Reduce vertical height by ~50%.
   - Show only name & size per volume section.
   - Remove the "Show Reserved" checkbox (keep backing logic for future settings dialog).
   - Move action buttons from the volumes control into the volume-specific view (bottom 2/3 area).
   - Remove the "Volumes" header label above the control.
4. **Color-coded usage bar:**
   - Add a horizontally stacked usage bar under the volumes control.
   - Each segment represents a volume's usage (like a multi-segment progress bar).
   - Center-aligned legend beneath the bar showing volume name and usage amount.

### Non-Goals
- Redesign of dialogs or other views beyond the disk page.
- Backend changes to usage calculation logic.
- Full settings dialog implementation (only reserve logic for "Show Reserved").

### Proposed Approach
1. **Split layout implementation:**
   - Refactor `disks-ui/src/views/volumes.rs` (or equivalent disk page view) to render two sections:
     - Top section: disk header component (~1/3 height).
     - Bottom section: placeholder for volume-specific content view (~2/3 height).
   - Use a `Column` with appropriate spacing and fixed/flex ratios.
2. **Disk info header redesign:**
   - Create a dedicated header component:
     - Row layout: `icon | (name, partitioning, serial) | used/total box`.
     - Icon: large size (e.g., 64px or appropriate for header).
     - Text block: left-aligned; name bold, partitioning and serial in secondary text style.
     - Size box: right-aligned, distinct background/border, shows "Used / Total".
   - Replace the current header rendering in the disk page view.
3. **Volumes control compaction:**
   - Modify `disks-ui/src/ui/volumes/` components:
     - Reduce per-volume row height (target ~50% of current).
     - Display only name and size per row (remove other metadata).
     - Remove "Show Reserved" checkbox UI; retain the backing filter logic in state.
     - Remove "Volumes" label/header.
     - Remove action buttons from the control itself (defer to volume-specific view).
   - Ensure the control still provides segment selection for the usage bar and volume-specific view.
4. **Color-coded usage bar:**
   - Implement a new widget under the volumes control:
     - Render as a horizontal row of colored segments, each proportional to volume usage.
     - Assign distinct colors per volume (use a palette or hash-based scheme).
     - Below the bar: render a legend (center-aligned) showing each volume's name and usage.
   - Wire the bar to the same volume data used by the volumes control.
   - Ensure the bar updates when volumes change or are resized.

### User/System Flows
- User selects a disk in sidebar: disk page opens with the new split layout.
- Top 1/3 shows disk header: large icon, name/partitioning/serial, and used/total box.
- Below that: compact volumes control (no header, no checkboxes, no action buttons).
- Below the control: color-coded usage bar with legend.
- User clicks a volume in the control OR in the treeview sidebar: 
  - Bottom 2/3 view updates to show that volume's details (with action buttons).
  - Selection state synchronizes in both places (volumes control highlights the volume, treeview highlights the sub-item).
- User resizes or modifies volumes: usage bar updates to reflect new layout.

### Risks & Mitigations
- **Fixed height ratios may not work on small screens:**
  - Mitigation: use flex ratios that adapt; ensure both sections have minimum heights and scrollability where needed.
- **Usage bar complexity with many small volumes:**
  - Mitigation: set a minimum segment width; if too many volumes, consider a scrollable legend or overflow behavior.
- **Removing "Show Reserved" checkbox may confuse users:**
  - Mitigation: document the change; plan for a settings dialog in a future iteration.
- **Action button relocation may disrupt muscle memory:**
  - Mitigation: ensure the new location is intuitive and consistent with volume selection.

### Acceptance Criteria
- [x] Disk page layout splits into two sections: top 1/3 for disk header, bottom 2/3 for volume content.
- [x] Disk info header renders: large icon | (name, partitioning, serial) | used/total box.
- [x] Volumes control:
  - [x] Reduced to ~50% vertical height.
  - [x] Shows only name & size per volume row.
  - [x] "Show Reserved" checkbox removed (logic retained in state).
  - [x] "Volumes" label/header removed.
  - [x] Action buttons removed (deferred to volume-specific view).
- [x] Color-coded usage bar renders below the volumes control:
  - [x] Horizontal stacked segments proportional to volume usage.
  - [x] Each volume has a distinct color.
  - [x] Center-aligned legend shows volume name and usage.
- [x] Volume selection synchronizes bi-directionally:
  - [x] Selecting a volume in the volumes control updates treeview selection and shows volume detail view.
  - [x] Selecting a volume sub-item in the treeview updates volumes control selection and shows volume detail view.
- [x] No regressions in existing disk selection, navigation, or dialog behavior.
---

## Extended Scope Phase 2 — Refinements & Menu Redesign

**Added:** 2026-02-06

### Context
After implementing the split view and usage visualization, several refinements and a menu reorganization are needed:
1. **Layout refinement:** Fixed height ratios should be replaced with shrink-to-fit for the disk header.
2. **Volume detail header redesign:** Match the disk header layout with a pie chart for usage visualization.
3. **Usage bar compaction:** Further reduce usage bar height for a more compact layout.
4. **Bug fixes:** Correct usage metrics calculation, treeview ordering, and LUKS container selection sync.
5. **Menu reorganization:** Replace menubar items with inline action buttons; reorganize image operations.

### Goals
1. **Shrink-to-fit disk header:**
   - Disk header should consume only the space needed for its content.
   - Volume detail view should fill all remaining vertical space.
2. **Volume detail header matching disk header:**
   - Volume detail header should mirror disk header layout structure.
   - Replace icon with a thin pie chart showing usage proportion.
   - Display "Used / Total" inside the pie chart.
3. **Compact usage bar:**
   - Reduce usage bar height to ~1/4 of current.
4. **Fix usage metrics:**
   - Usage bar currently sums total partition sizes instead of actual used space.
   - Correct to show actual disk usage.
5. **Fix treeview ordering:**
   - Treeview subitems should appear in disk offset order (matching volumes control).
6. **Fix LUKS selection sync:**
   - Selecting a LUKS container in volumes control should update treeview selection.
7. **Inline disk operations:**
   - Move disk operations from menubar to inline buttons below disk header.
   - Move image operations to appropriate contexts:
     - Disk image ops: with disk action buttons.
     - Partition image ops: with partition action buttons.
     - Create/Attach image: segmented button at bottom of sidebar.

### Non-Goals
- Redesign of dialogs or other views.
- Changes to backend operation logic.
- Full menubar removal (only disk/partition operations; app-level actions remain).

### Proposed Approach
1. **Layout sizing fix:**
   - Change disk header container from `Length::FillPortion(1)` to `Length::Shrink`.
   - Ensure volume detail view uses `Length::Fill`.
2. **Volume detail header redesign:**
   - Create a volume detail header component mirroring disk header layout.
   - Implement pie chart widget showing used vs. free space.
   - Display "Used / Total" text inside pie chart.
3. **Usage bar compaction:**
   - Reduce bar height in `usage_bar.rs` to ~1/4.
4. **Usage metrics fix:**
   - Investigate volume model for actual usage fields.
   - Replace `volume.size` summation with actual used space calculation.
   - Verify against system tools (df, etc.).
5. **Treeview ordering fix:**
   - Sort volume children by partition offset before rendering in sidebar.
6. **LUKS selection sync fix:**
   - Debug why LUKS containers don't trigger treeview selection.
   - Ensure object_path matching works for encrypted volumes.
7. **Menu reorganization:**
   - Add disk action button row below disk header.
   - Move partition image ops to volume detail action buttons.
   - Add segmented button ("Create Image" | "Attach Image") at bottom of sidebar.
   - Remove disk/partition operations from menubar.

### User/System Flows
- User selects disk: disk header appears, shrunk to contents; volume detail view fills remaining space.
- Volume detail header displays with pie chart showing usage and matching disk header layout.
- Usage bar is more compact, showing actual disk usage (not total partition sizes).
- Treeview subitems appear in the same order as volumes control segments.
- User selects LUKS container in volumes control: treeview node highlights.
- User clicks disk operation buttons below disk header: operations execute without menubar.
- User clicks "Create Image" in sidebar: new disk image dialog opens.

### Risks & Mitigations
- **Shrink-to-fit may cause layout jumpiness:**
  - Mitigation: ensure header has minimum height; test with various disk configurations.
- **Pie chart complexity:**
  - Mitigation: keep pie chart simple (two segments: used/free); reuse COSMIC theme colors.
- **Usage calculation may require backend changes:**
  - Mitigation: investigate volume model first; if data unavailable, may need to query filesystem usage separately.
- **Menu reorganization may confuse existing users:**
  - Mitigation: keep button labels consistent with menu items; consider adding tooltips.

### Acceptance Criteria
- [x] Disk header shrinks to fit contents; volume detail view fills remaining space.
- [x] Volume detail header matches disk header layout with pie chart replacing icon.
- [x] Pie chart displays usage proportion with "Used / Total" text inside.
- [x] Usage bar height reduced to ~1/4 of current.
- [x] Usage metrics display actual used space, not total partition sizes.
- [x] Treeview subitems appear in disk offset order (matching volumes control).
- [x] Selecting LUKS container in volumes control updates treeview selection.
- [x] Disk operation buttons appear below disk header (including disk image ops).
- [x] Partition image operations appear in volume detail action buttons.
- [x] "Create Image" / "Attach Image" segmented button at bottom of sidebar.
- [x] Menubar no longer contains disk/partition operations.
- [x] All operations functional with no regressions.

---

## Remaining Issues

**Added:** 2026-02-06

### Missing action buttons on filesystem node under LUKS
When a filesystem is selected under an unlocked LUKS container (child node), the following action buttons are missing from the volume detail view:
- Edit Filesystem/Label
- Format
- Check Filesystem
- Repair Filesystem
- Take Ownership
- Edit Mount Options
- Create Partition Image
- Restore Partition Image

**Issue:** Child filesystem nodes under LUKS containers are not showing the full set of filesystem-specific action buttons. These actions should be available since the child is a regular filesystem that can be mounted, formatted, checked, etc.

### Missing action buttons on standard partitions
Standard partition types (like ext4 filesystems) are missing the following action buttons:
- Take Ownership
- Create Partition Image
- Restore Partition Image

**Issue:** Image operations and ownership management should be available for all partition types, not just specific cases.

### Incorrect drive power management capability detection
`can_power_off` is not an appropriate check for whether a disk can standby/wake. This field indicates whether the drive can be safely powered off (removed from the system), not whether it supports power management features like spinning down.

**Issue:** NVMe drives don't support traditional spindown/standby commands, but the current code uses `can_power_off` to determine whether to show Standby/Wake buttons. This causes these buttons to appear for drives that don't support them (or not appear for drives that do). A more appropriate check would be:
- Check for `rotation_rate` > 0 (indicates spinning disk that can spin down)
- Or check drive connection type (exclude NVMe, include SATA/SAS)
- Or query UDisks2 for actual power management capabilities

**Impact:** Users see non-functional Standby/Wake buttons on NVMe drives, or don't see them on drives that do support power management.

**Status:** ✅ Resolved in Task 39

---

## Remaining Issues — Phase 6

**Added:** 2026-02-06

### Treeview node alignment inconsistency
Nodes with and without expanders are not horizontally aligned in the sidebar treeview. Nodes without expanders (regular partitions) appear misaligned compared to nodes with expanders (LUKS containers).

**Issue:** The expander control doesn't have a fixed width, and nodes without expanders don't reserve space for one. This creates visual misalignment at the same hierarchy depth.

**Proposed Fix:** 
- Set fixed width for expander control (e.g., 24px)
- Always indent by expander_width × 2 for child nodes:
  - Nodes with expander: expander widget + (width × 1) additional indent
  - Nodes without expander: (width × 2) empty indent
- Formula: `indent = base + (expander_width × 2 × depth)`

**Impact:** Poor visual hierarchy and difficult to scan tree structure.

**Status:** 📋 Planned in Task 40

### GPT reserved space calculation failures
GPT usable range parsing is failing for multiple drives, falling back to conservative 1MiB bands:

```
WARN: Could not parse GPT usable range for /org/freedesktop/UDisks2/block_devices/sda; falling back to conservative 1MiB bands
WARN: Could not parse GPT usable range for /org/freedesktop/UDisks2/block_devices/nvme0n1; falling back to conservative 1MiB bands
```

**Issue:** The code is unable to read FirstUsableLBA/LastUsableLBA from the UDisks2 PartitionTable interface. This may be due to:
- Incorrect property name or interface query
- Properties not exposed by UDisks2
- Need to read GPT header directly from block device

**Impact:** Reserved space segments show as free space, misleading users about available disk space. Conservative fallback may not accurately represent actual GPT layout.

**Status:** 📋 Planned in Task 41

### No Settings page for user preferences
The application only has an "About" page with version information. User preferences (like "Show Reserved Space") need a dedicated Settings page.

**Issue:** COSMIC provides a built-in config serialization manager (`cosmic::config`), but the app doesn't use it. The "Show Reserved" checkbox was previously in the volumes control but was removed. Users need a way to configure this preference.

**Proposed Implementation:**
- Rename "About" page to "Settings"
- Keep About section, add Settings section above it
- Add "Show Reserved Space" checkbox (default: false)
- Use `cosmic::config::Config` trait for automatic persistence
- Update segment calculations to respect this setting

**Impact:** Users cannot control whether reserved space is shown, reducing flexibility.

**Status:** 📋 Planned in Task 42

### Edit partition icon not showing
The edit partition button's icon is either unset or using an invalid icon name, resulting in no visible icon.

**Issue:** Icon name may be incorrect or missing from COSMIC icon theme.

**Proposed Fix:** Use `document-edit-symbolic` or `edit-symbolic`.

**Impact:** Button is less discoverable without visual icon.

**Status:** 📋 Planned in Task 43

### Missing application icon
The application doesn't have a proper icon set up in the installation/desktop entry.

**Issue:** No application icon configured in `resources/app.desktop` and `resources/icons/hicolor/`.

**Proposed Fix (Temporary):** Use the same icon currently used for drive tree nodes in the sidebar as the application icon until a proper custom icon is designed.

**Impact:** Application is harder to identify in launcher and taskbar.

**Status:** 📋 Planned in Task 44

### Inconsistent format icons
The format partition icon doesn't match the format disk icon, creating visual inconsistency.

**Issue:** Different operations with similar purposes should use consistent iconography.

**Proposed Fix:** Ensure both use the same icon (e.g., `edit-clear-symbolic`).

**Impact:** Minor UX inconsistency; users may not recognize operations as related.

**Status:** 📋 Planned in Task 45

### Volume selection resets after state changes
When mounting, unmounting, locking, or unlocking a volume, the selection resets to the first item in the list instead of staying on the operated volume.

**Issue:** State-changing operations trigger a full drive reload, and the selection state is either cleared or not preserved across the reload.

**Proposed Fix:**
- Save selected volume's object_path before operation
- After drive reload, find and reselect the same volume
- Handle edge cases (volume deleted, object_path changed, etc.)

**Impact:** Poor UX; user loses context after common operations.

**Status:** 📋 Planned in Task 46

### Missing Create/Restore Partition Image buttons
Despite backend infrastructure existing in `disks_dbus::disks::image`, the UI buttons for creating and restoring partition images are still missing.

**Issue:** Image operations were noted as TODO in Task 38 but require significant implementation work:
- File picker dialogs for save/load
- Progress reporting during image operations
- Error handling for various failure cases
- Confirmation dialogs for destructive restore operation

**Impact:** Users cannot create backup images of partitions or restore from images, a key feature for disk management.

**Status:** 📋 Planned in Task 47

### Removable drives show Power Off instead of Eject
Removable drives (USB, SD cards, external drives) show a "Power Off" button when they should show a universal "Eject" button.

**Issue:** The button logic doesn't distinguish between removable and non-removable drives. "Eject" is the standard action for safely removing external media, while "Power Off" is for shutting down internal drives (rare capability).

**Proposed Fix:**
- If `drive.removable` or `drive.ejectable`: show Eject button only
- If non-removable and `drive.can_power_off`: show Power Off button
- Never show both buttons for the same drive

**Impact:** Confusing terminology for removable drives; "Power Off" suggests shutting down the device rather than safe removal.

**Status:** 📋 Planned in Task 48
