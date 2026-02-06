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

### Commands run

- `cargo check -p cosmic-ext-disks`
- `cargo test --workspace --all-features`
- `cargo fmt --all` and `cargo fmt --all --check`
- `cargo clippy --workspace --all-features -- -D warnings`

### Notable files changed

- disks-ui/src/ui/sidebar/{mod.rs,state.rs,view.rs}
- disks-ui/src/ui/app/{message.rs,mod.rs,state.rs,view.rs}
- disks-ui/src/ui/app/update/{mod.rs,nav.rs,drive.rs}
- disks-ui/src/ui/volumes/{mod.rs,view.rs,disk_header.rs}
- disks-ui/i18n/{en,sv}/cosmic_ext_disks.ftl

### Next steps

- Task 12: Add color-coded usage bar below volumes control
- Task 13: Implement volume-specific detail view with action buttons and bi-directional selection sync
- Task 14: Integration and polish
