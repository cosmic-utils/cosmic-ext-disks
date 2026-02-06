# feature/ui-refactor — Tasks

## Task 1: Confirm tree model + categorization rules
- Scope: make the treeview node types + grouping rules explicit and implementable.
- Files/areas: `disks-ui/src/ui/app/update/nav.rs` (reference), `disks-dbus` model fields (reference).
- Steps:
  - Define node kinds we will show in the sidebar: Drive/Image/Container/Volume/Partition (and what each maps to in current data).
  - Define initial mapping into sections: Logical / Internal / External / Images.
  - Decide how “Logical” behaves today (likely empty or “future” until LVM/Apple support lands).
  - Define row actions per node kind:
    - Drive: Eject (if removable) + kebab (Disk menu actions)
    - Volume: Unmount (if mounted) + kebab (Disk menu actions may still target the parent drive)
- Test plan: manual inspection on a machine with at least one internal and one removable drive.
- Done when:
  - [x] Spec includes a short, concrete mapping table for sections and node kinds.

## Task 2: Add a sidebar tree view module (static prototype)
- Scope: create a UI component that renders section headers + a treeview row layout.
- Files/areas:
  - `disks-ui/src/ui/app/view.rs`
  - New: `disks-ui/src/ui/sidebar/` (or similar)
- Steps:
  - Add new module for sidebar widgets (section headers, tree rows).
  - Implement tree row rendering:
    - Expander control (shown only if node has children)
    - Icon
    - Single-line title
    - Trailing action buttons: Eject/Unmount + kebab
  - Implement indentation by depth.
  - Wire it into `view::nav_bar()` behind a temporary switch.
- Test plan: `cargo build -p cosmic-ext-disks`.
- Done when:
  - [x] Sidebar renders with static sample tree data and correct row layout.

## Task 3: Build the top-level tree from DriveModel list
- Scope: populate the sidebar from the same drive list used by `Message::UpdateNav`.
- Files/areas:
  - `disks-ui/src/ui/app/update/nav.rs`
  - `disks-ui/src/ui/app/state.rs`
- Steps:
  - Introduce sidebar state in `AppModel` (or derive view-model on the fly): sections, nodes, expanded flags.
  - Rebuild top-level nodes (drives/images) on refresh.
  - Ensure stable keys (likely `DriveModel.block_path`).
- Test plan: run app and confirm drives appear under the expected sections.
- Done when:
  - [x] Sidebar shows the real top-level drives/images grouped by section.

## Task 4: Implement expand/collapse + selection semantics
- Scope: tree interactions without breaking existing navigation behavior.
- Files/areas:
  - `disks-ui/src/ui/app/message.rs`
  - `disks-ui/src/ui/app/update/mod.rs`
  - `disks-ui/src/ui/sidebar/*`
- Steps:
  - Add message(s) for toggling expansion per node.
  - Add message(s) for selecting a node.
  - Ensure expander click does not change selection.
  - Preserve the existing “don’t switch while dialog is open” behavior.
- Test plan: manual: expand/collapse + select; confirm title updates and volumes view changes.
- Done when:
  - [x] Selection behaves like current nav selection.
  - [x] Expansion state persists across refreshes where possible.

## Task 5: Add row primary action button (Eject/Unmount)
- Scope: implement the dedicated trailing action button per row.
- Files/areas:
  - `disks-ui/src/ui/app/message.rs`
  - `disks-ui/src/ui/sidebar/*`
  - `disks-ui/src/ui/app/update/drive.rs` and/or volumes update handlers
- Steps:
  - Add messages for Eject and Unmount actions.
  - Render the primary action conditionally:
    - Drives: Eject when removable
    - Volumes: Unmount when mounted
  - Ensure button press does not change selection.
- Test plan: manual test with a removable drive and a mounted volume.
- Done when:
  - [x] Eject/Unmount triggers the correct command.

## Task 6: Add kebab popup menu mirroring “Disk” menu actions
- Scope: kebab opens a contextual popup menu with Disk actions.
- Files/areas:
  - `disks-ui/src/views/menu.rs` (as the source of truth for actions)
  - `disks-ui/src/ui/sidebar/*`
  - `disks-ui/src/ui/app/message.rs`
  - `disks-ui/src/ui/app/update/drive.rs`
- Steps:
  - Define the menu items to match Disk menu: Eject, Power Off, Format Disk, SMART Data/Self Tests, Standby Now, Wake Up.
  - Implement popup menu UI attached to the kebab button.
  - Wire each menu item to the same message handlers used by the top menu.
  - Make the menu contextual (hide/disable non-applicable actions).
- Test plan: manual: open menu, click each item, verify behavior matches the top menu.
- Done when:
  - [ ] (Dropped) Kebab menu is removed; Disk actions remain in the top menu.

## Task 7: Add children under drives (containers/volumes/partitions)
- Scope: populate the tree beneath a drive with the best-available hierarchy.
- Files/areas:
  - `disks-ui/src/ui/volumes/*` (source of current volumes state)
  - `disks-dbus/src/disks/*` (models)
  - `disks-ui/src/ui/sidebar/*`
- Steps:
  - Decide the minimal viable hierarchy we can build from existing UI state (likely drive → volumes first).
  - Add containers/partitions when the data is available without adding excessive DBus roundtrips.
  - Ensure selecting a child node does **not** change the main view yet (still shows the parent drive view). We only need selection state for later UI.
- Test plan: manual on a drive with partitions; verify tree nodes match what the main view shows.
- Done when:
  - [x] Children appear under a drive and are selectable.
  - [x] Clicking a child node keeps showing the drive view.

## Task 8: Remove dependency on built-in nav bar widget (cleanup)
- Scope: stop rendering `widget::nav_bar(...)` and consolidate to the custom sidebar.
- Files/areas:
  - `disks-ui/src/ui/app/view.rs`
  - `disks-ui/src/ui/app/mod.rs` (nav hooks)
- Steps:
  - Remove temporary switch.
  - Ensure condensed mode behavior remains acceptable.
  - Update docs/spec with final architecture.
- Test plan: `cargo test --workspace --all-features` + manual UI smoke test.
- Done when:
  - [x] No remaining usage of built-in nav bar rendering.
