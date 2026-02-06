# feature/ui-refactor — Implementation Log

## 2026-02-06

- Implemented custom sidebar treeview to replace built-in `widget::nav_bar` rendering.
- Added sidebar state (`SidebarState`) and view module under `disks-ui/src/ui/sidebar/`.
- Wired sidebar selection/expansion/menu state into app update loop.
- Implemented per-row actions:
  - Drive eject/remove button.
  - Volume unmount button.
- Removed the sidebar kebab menu (popover) due to UX concerns; Disk actions remain available via the top menu.
- Ensured row event handling avoids nested-button conflicts by making only the title region clickable for selection.
- Added i18n key `unmount-failed` for sidebar unmount error dialog.
- Adjusted sidebar item styling so each row container paints the same background as `Container::Card` (matching volumes sections) while keeping selection indicated via an accent border.
- Updated sidebar row titles to use drive vendor+model (no serial/path-derived IDs), render titles with semibold typography, and apply accent/highlight foreground color to title/icon/expander/actions when selected.

### Extended Scope Implementation (2026-02-06)

**Task 9 & 10: Split layout + disk header**
- Created `disk_header.rs` module with dedicated disk info header component
- Header layout: large icon (64px) | name/partitioning/serial (left-aligned) | used/total box (right-aligned)
- Refactored main view in `app/view.rs` to use 1/3 : 2/3 split layout
- Top section contains disk header and volumes control
- Bottom section contains volume-specific detail view (extracted to separate function)
- Added i18n keys: `backing-file`, `disk-usage` (English + Swedish)

**Task 11: Compact volumes control**
- Reduced segment button height from 130px to 65px (~50% reduction)
- Simplified segment controls to show only name and size (removed label, partition type)
- Removed "Volumes" header label from the control
- Removed "Show Reserved" checkbox UI (backing `show_reserved` state flag retained for future settings dialog)
- Removed entire action bar with all action buttons (mount, unmount, format, resize, etc.)
- Action buttons will be relocated to volume detail view in Task 13
- Cleaned up unused imports and dead code

**Task 12: Color-coded usage bar**
- Created `usage_bar.rs` module with horizontal stacked segment visualization
- Each segment width proportional to volume size relative to total disk size
- Implemented 10-color distinct palette for visual differentiation
- Legend below bar displays: color swatch | volume name | size for each partition
- Filters to show only actual partitions (excludes free space and reserved)
- Integrated into main view between volumes control and volume detail view
- Added `#[allow(dead_code)]` attributes for code that will be reused in Task 13

### Commands run

- `cargo check -p cosmic-ext-disks`
- `cargo test --workspace --all-features`
- `cargo fmt --all` and `cargo fmt --all --check`
- `cargo clippy --workspace --all-features -- -D warnings`

### Notable files changed

- disks-ui/src/ui/sidebar/{mod.rs,state.rs,view.rs}
- disks-ui/src/ui/app/{message.rs,mod.rs,state.rs,view.rs}
- disks-ui/src/ui/app/update/{mod.rs,nav.rs,drive.rs}
- disks-ui/src/ui/volumes/{mod.rs,view.rs,disk_header.rs,usage_bar.rs}
- disks-ui/src/ui/volumes/update/selection.rs
- disks-ui/src/ui/dialogs/state.rs
- disks-ui/i18n/{en,sv}/cosmic_ext_disks.ftl

**Task 13: Volume detail view with action buttons and bi-directional selection sync** (2026-02-06)
- Relocated all action buttons from volumes control to bottom 2/3 detail view
- Implemented comprehensive action bar with context-sensitive buttons:
  - Container actions: unlock/lock for LUKS
  - Mount/unmount for volumes and child filesystems
  - Format, edit, resize partition actions
  - Filesystem operations: edit label, mount options, check, repair, take ownership
  - Encryption: change passphrase, edit options
  - Delete partition
  - Create partition (for free space)
- Extracted helper functions: build_volume_node_info, build_partition_info, build_free_space_info, build_action_bar
- Added tooltip_icon_button helper directly in app/view.rs
- Implemented bi-directional selection synchronization between sidebar and volumes control:
  - Added Message::SidebarClearChildSelection to clear sidebar child selection
  - segment_selected() clears sidebar child when segment is selected
  - select_volume() selects corresponding volume in sidebar
  - SidebarSelectChild handler selects corresponding volume in volumes control
  - Added find_volume_child_recursive() helper to search volume tree
  - Handles both direct partition selection and child volume selection
  - Uses proper Task wrapping with cosmic::Action for message propagation
- All tests passing, clippy clean

**Task 14: Integration & polish** (2026-02-06)
- Verified complete disk page flow:
  - Split layout working correctly (1/3 top, 2/3 bottom)
  - Disk header displays properly with icon, name/partitioning/serial, and used/total box
  - Compact volumes control renders below header with reduced height
  - Color-coded usage bar with legend appears below control
  - Volume detail view updates in bottom section
- Application runs successfully with no regressions
- All tests passing (36 tests total: 9 disks-ui, 27 disks-dbus)
- Clippy clean with -D warnings
- All extended scope acceptance criteria met
- Total commits: 6 (spec update + 5 implementation commits)

### Final status

All tasks complete (Tasks 1-14). Extended scope fully implemented. Ready for PR review.
