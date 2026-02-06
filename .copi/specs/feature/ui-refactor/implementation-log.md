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

### Commands run

- `cargo check -p cosmic-ext-disks`
- `cargo test --workspace --all-features`
- `cargo fmt --all` and `cargo fmt --all --check`
- `cargo clippy --workspace --all-features -- -D warnings`

### Notable files changed

- disks-ui/src/ui/sidebar/{mod.rs,state.rs,view.rs}
- disks-ui/src/ui/app/{message.rs,mod.rs,state.rs,view.rs}
- disks-ui/src/ui/app/update/{mod.rs,nav.rs,drive.rs}
- disks-ui/i18n/{en,sv}/cosmic_ext_disks.ftl
