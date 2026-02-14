# feature/disk-menu-commands — Tasks

Branch: `feature/disk-menu-commands`

## Task 1: Disk menu restructure (add/move/remove items)
- Scope: Update the menu layout to match requested Disk menu contents.
- Files/areas:
  - `storage-ui/src/views/menu.rs`
  - `storage-ui/i18n/**/cosmic_ext_disks.ftl` (only if label keys change)
- Steps:
  - Remove `DriveSettings` from Disk menu.
  - Do not add image create/restore to Disk menu (handled in a separate PR).
  - Ensure keybind map and enum variants still compile (remove variants only if no longer used).
- Test plan:
  - `cargo fmt --all`
  - `cargo clippy --workspace --all-features`
  - Run UI and verify menu shows expected items.
- Done when:
  - [x] Disk menu reflects requested actions.
  - [x] No dangling menu actions.

## Task 2: Implement Power Off (UI → DBus) + refresh
- Scope: Replace placeholder dialog with real drive power-off.
- Files/areas:
  - `storage-ui/src/app.rs`
  - `storage-dbus/src/disks/drive.rs` (already has `power_off()`)
- Steps:
  - In `Message::PowerOff` handler, call `DriveModel::power_off()` for active drive.
  - On completion, refresh drive list via `DriveModel::get_drives()` and update nav.
  - Handle errors with an info/error dialog containing the error string.
- Test plan:
  - Unit: none (system-integrated).
  - Manual: try on a removable drive (if available).
- Done when:
  - [x] Clicking Power Off no longer shows “not implemented”.

## Task 3: Implement Standby Now + Wake-up via UDisks2 ATA calls
- Scope: Add backend methods and wire them from UI.
- Files/areas:
  - `storage-dbus/src/disks/drive.rs` (new async methods)
  - `storage-ui/src/app.rs`
- Steps:
  - Verify UDisks2 API availability on target system:
    - `busctl introspect org.freedesktop.UDisks2 /org/freedesktop/UDisks2/drives/... org.freedesktop.UDisks2.Drive`
    - Confirm method names for standby/wakeup.
  - Implement raw zbus proxy calls for standby/wakeup.
  - In UI handlers, call these methods and show success/error.
- Test plan:
  - Manual only; verify no UI hang.
- Done when:
  - [x] Standby and Wake-up perform real calls when supported.
  - [x] Unsupported drives show “Not supported”.

## Task 4: SMART Data & Self-Tests (basic)
- Scope: Provide a minimal SMART view and ability to start self-tests.
- Files/areas:
  - `storage-ui/src/app.rs` and new view under `storage-ui/src/views/`
  - `storage-dbus` ATA SMART methods
- Steps:
  - Verify UDisks2 SMART API methods and properties via introspection.
  - Add backend calls to fetch SMART attributes/status.
  - Provide UI with:
    - health/status summary
    - attributes table (key fields)
    - buttons for short/extended self-test
  - Handle unsupported drives gracefully.
- Test plan:
  - Manual on an ATA/NVMe drive that exposes SMART via UDisks.
- Done when:
  - [x] SMART page loads and self-test start works where supported.

## Task 5: Remove Drive Settings end-to-end
- Scope: Ensure Drive Settings is not reachable and any redundant code is removed.
- Files/areas:
  - `storage-ui/src/views/menu.rs`
  - `storage-ui/src/app.rs` (remove handler/variant if appropriate)
- Steps:
  - Delete the menu item.
  - If no longer referenced, remove `MenuAction::DriveSettings` + `Message::DriveSettings`.
- Test plan:
  - `cargo test --workspace --all-features`
- Done when:
  - [x] No Drive Settings UI remains.
  - [x] Build passes.
