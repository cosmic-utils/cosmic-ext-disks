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

## Acceptance Criteria
- [ ] Sidebar renders a treeview with expand/collapse on nodes that have children.
- [ ] Sidebar rows render as `Expander | Icon | Title | Trailing actions` with single-line title.
- [ ] Sidebar rows include an inline Eject/Unmount action button that triggers without changing selection.
- [ ] Disk-menu-equivalent actions remain available via the existing top-level Disk menu.
- [ ] Sidebar shows section headers and groups items under: Logical / Internal / External / Images.
- [ ] Selecting a drive still activates the correct page and updates window title.
- [ ] Selecting a child node (partition/container/volume) does not change the main view yet (still shows the parent drive view).
- [ ] No regressions in condensed vs non-condensed layouts (manual QA).

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
- [ ] Disk page layout splits into two sections: top 1/3 for disk header, bottom 2/3 for volume content.
- [ ] Disk info header renders: large icon | (name, partitioning, serial) | used/total box.
- [ ] Volumes control:
  - [ ] Reduced to ~50% vertical height.
  - [ ] Shows only name & size per volume row.
  - [ ] "Show Reserved" checkbox removed (logic retained in state).
  - [ ] "Volumes" label/header removed.
  - [ ] Action buttons removed (deferred to volume-specific view).
- [ ] Color-coded usage bar renders below the volumes control:
  - [ ] Horizontal stacked segments proportional to volume usage.
  - [ ] Each volume has a distinct color.
  - [ ] Center-aligned legend shows volume name and usage.
- [ ] Volume selection synchronizes bi-directionally:
  - [ ] Selecting a volume in the volumes control updates treeview selection and shows volume detail view.
  - [ ] Selecting a volume sub-item in the treeview updates volumes control selection and shows volume detail view.
- [ ] No regressions in existing disk selection, navigation, or dialog behavior.
