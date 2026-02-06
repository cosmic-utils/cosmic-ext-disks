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
  - Preserve the existing “don’t switch while dialog is open” behavior.  - Note: Volume selection will later need bi-directional sync with volumes control (see Task 13).- Test plan: manual: expand/collapse + select; confirm title updates and volumes view changes.
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

---

## Extended Scope Tasks — Disk Page Split View & Volumes Control Redesign

**Added:** 2026-02-06

## Task 9: Implement split layout for disk page (1/3 header, 2/3 content)
- Scope: refactor the disk page to use a two-section layout.
- Files/areas:
  - `disks-ui/src/views/volumes.rs` (or equivalent disk page view)
  - `disks-ui/src/ui/app/view.rs` (if changes needed for container/routing)
- Steps:
  - Wrap the current disk page in a `Column` with two sections.
  - Top section: disk header component (new; see Task 10); target ~1/3 of available height using flex or fixed constraints.
  - Bottom section: volume-specific content view (placeholder for now; target ~2/3 height).
  - Ensure both sections have minimum heights and scroll independently if needed.
  - Test with varying window sizes to confirm ratios adapt gracefully.
- Test plan: manual UI test on different screen sizes; verify top/bottom sections render at expected proportions.
- Done when:
  - [ ] Disk page renders two distinct sections with appropriate height distribution.
  - [ ] Scrolling behavior is acceptable for both sections.

## Task 10: Redesign disk info header (icon, name/partitioning/serial, used/total box)
- Scope: create a new disk header component with the specified layout.
- Files/areas:
  - New: `disks-ui/src/ui/volumes/disk_header.rs` (or similar)
  - `disks-ui/src/views/volumes.rs` (integrate the new header)
- Steps:
  - Implement header row layout:
    - Left: large icon (e.g., 64px; use existing drive icon logic).
    - Middle: text column (left-aligned):
      - Line 1: Drive name (bold, primary text).
      - Line 2: Partitioning scheme (secondary text).
      - Line 3: Serial number (secondary text, smaller font if appropriate).
    - Right: "Used / Total" box (distinct background/border; right-aligned).
  - Calculate "Used" and "Total" from drive/volume data (leverage existing usage calculation).
  - Style the box with appropriate spacing, padding, and visual distinction (e.g., background color, border).
  - Replace the current disk header in the disk page view with this new component.
- Test plan: manual UI test; verify layout matches spec on different drives (with/without serial, varying name lengths).
- Done when:
  - [ ] Disk header renders with the specified layout.
  - [ ] "Used / Total" box displays correct values.
  - [ ] Visual styling is clean and consistent with the app theme.

## Task 11: Compact the volumes control (reduce height, simplify content)
- Scope: modify the volumes control to reduce vertical space and remove extraneous elements.
- Files/areas:
  - `disks-ui/src/ui/volumes/*` (volumes control components)
  - `disks-ui/src/views/volumes.rs` (if integration changes needed)
- Steps:
  - Reduce per-volume row height by ~50%:
    - Remove multi-line text or extra padding.
    - Show only volume name and size per row.
  - Remove the "Volumes" label/header above the control.
  - Remove the "Show Reserved" checkbox UI:
    - Retain the backing filter logic in state (e.g., `AppModel` or volumes state).
    - Ensure volumes marked as "reserved" can still be filtered out if the flag is set.
    - Document that this will move to a settings dialog in a future iteration.
  - Remove action buttons from the volumes control itself:
    - Action buttons will appear in the volume-specific view (bottom 2/3; see Task 13).
  - Ensure the control still allows volume selection and updates the bottom content area.
- Test plan: manual UI test; verify control is ~50% shorter, displays only name/size, and no checkboxes or action buttons are present.
- Done when:
  - [ ] Volumes control height reduced by ~50%.
  - [ ] Only name and size displayed per volume row.
  - [ ] "Show Reserved" checkbox removed; backing logic retained.
  - [ ] "Volumes" header label removed.
  - [ ] Action buttons removed from the control.

## Task 12: Add color-coded usage bar below volumes control
- Scope: implement a horizontal stacked usage bar with a legend.
- Files/areas:
  - New: `disks-ui/src/ui/volumes/usage_bar.rs` (or similar)
  - `disks-ui/src/views/volumes.rs` (integrate below the volumes control)
  - `disks-ui/src/utils/segments.rs` (if color assignment logic is needed)
- Steps:
  - Design the usage bar widget:
    - Render as a horizontal row of colored segments.
    - Each segment width is proportional to that volume's usage (relative to total disk size).
    - Assign a distinct color per volume (use a palette or hash-based color scheme).
    - Ensure segments are visually distinct (e.g., borders or slight spacing between them).
  - Below the bar, render a center-aligned legend:
    - For each volume: color swatch | volume name | usage amount (e.g., "50 GB").
    - Use a wrapping row or grid layout if many volumes.
  - Wire the bar to the same volume data used by the volumes control.
  - Ensure the bar updates reactively when volumes change or are resized.
  - Handle edge cases:
    - No volumes: show an empty bar or placeholder.
    - Many small volumes: set a minimum segment width or overflow behavior.
- Test plan: manual UI test with drives that have 0, 1, 3, and 10+ volumes; verify proportions and legend are accurate.
- Done when:
  - [ ] Usage bar renders below the volumes control.
  - [ ] Segments are proportional and color-coded.
  - [ ] Legend displays correctly and is center-aligned.
  - [ ] Bar updates when volumes change.

## Task 13: Implement volume-specific view in bottom 2/3 area (with action buttons)
- Scope: add a detail view for the selected volume, including action buttons relocated from the volumes control, with bi-directional selection synchronization.
- Files/areas:
  - New: `disks-ui/src/ui/volumes/volume_detail.rs` (or similar)
  - `disks-ui/src/views/volumes.rs` (integrate into the bottom 2/3 section)
  - `disks-ui/src/ui/app/message.rs` (if new messages are needed)
  - `disks-ui/src/ui/sidebar/*` (for treeview selection sync)
- Steps:
  - When a volume is selected in the volumes control OR in the treeview, render its detail view in the bottom section:
    - Display volume metadata: name, size, filesystem, mount point, usage, etc.
    - Include action buttons previously in the volumes control (e.g., Format, Mount/Unmount, Resize, etc.).
  - Implement bi-directional selection synchronization:
    - Selecting a volume in the volumes control updates the treeview selection (highlights the corresponding sub-item).
    - Selecting a volume sub-item in the treeview updates the volumes control selection (highlights the corresponding volume).
    - Both selections trigger the same detail view update.
  - If no volume is selected, show a placeholder or summary view.
  - Ensure action buttons trigger the same handlers as before (no behavior regressions).
  - Test with different volume types (partition, container, logical volume, etc.).
- Test plan: manual UI test; select volumes from both treeview and volumes control; verify selection syncs and detail view updates correctly.
- Done when:
  - [ ] Bottom 2/3 section displays selected volume details.
  - [ ] Action buttons relocated from volumes control and functional.
  - [ ] Placeholder shown when no volume is selected.
  - [ ] Selection in volumes control updates treeview selection.
  - [ ] Selection in treeview updates volumes control selection.

## Task 14: Integration & polish (split view + usage bar + compact control)
- Scope: ensure all components work together seamlessly and address any visual/layout issues.
- Files/areas:
  - All modified files from Tasks 9–13
- Steps:
  - Test the complete disk page flow:
    - Select a disk in sidebar → top 1/3 header appears.
    - Compact volumes control renders below header.
    - Usage bar appears below control.
    - Select a volume → bottom 2/3 detail view updates.
  - Verify layout adapts to window resizing and different drive configurations (many/few volumes, varied sizes).
  - Address any visual inconsistencies (spacing, alignment, colors).
  - Ensure no regressions in existing functionality (dialogs, navigation, etc.).
  - Run `cargo test --workspace --all-features` and manual smoke tests.
- Test plan: comprehensive manual UI testing across different drives and window sizes.
- Done when:
  - [ ] All extended scope acceptance criteria are met.
  - [ ] No regressions in existing features.
  - [ ] Visual polish is complete.
