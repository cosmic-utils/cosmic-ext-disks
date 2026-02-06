# UI Refactor — Spec

Branch: `feature/ui-refactor`
Source: N/A (brief; 2026-02-06)

## Context
We want to refactor/replace the left sidepanel navigation to support a **treeview** with:
- Collapsible hierarchy: drives/images → containers → volumes → partitions (as applicable)
- Row layout: `Expander | Icon | Title | Trailing action buttons` (no multiline text)
- Trailing actions:
  - Primary action button: **Eject** (for removable drives) or **Unmount** (for mounted volumes)
  - Secondary action button: **vertical 3-dots** (kebab) opening a popup menu containing all relevant actions currently available under the top-level **Disk** menu
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
     - Secondary: kebab (vertical 3 dots) opens a popup menu containing Disk actions
3. Popup menu behavior (Disk actions parity):
  - The kebab menu should expose the same actions as the existing “Disk” top-level menu.
  - Initial set (from current menu wiring): Eject, Power Off, Format Disk, SMART Data / Self Tests, Standby Now, Wake Up.
  - The menu should be contextual: hide/disable actions that don’t apply to the selected drive/node.
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
| Drive | Eject/remove when `is_loop || removable || ejectable` | Disk-menu actions targeting that drive |
| VolumeNode | Unmount when mounted | Disk-menu actions targeting the parent drive |

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
- [ ] Sidebar rows include a kebab button that opens a popup menu containing Disk-menu-equivalent actions.
- [ ] Sidebar shows section headers and groups items under: Logical / Internal / External / Images.
- [ ] Selecting a drive still activates the correct page and updates window title.
- [ ] Selecting a child node (partition/container/volume) does not change the main view yet (still shows the parent drive view).
- [ ] No regressions in condensed vs non-condensed layouts (manual QA).
