# Disk Menu Commands — Implementation Plan

Branch: `feature/disk-menu-commands`
Source: N/A (brief)

## Context
The Disk menu in the UI is already wired to `Message` variants, but several actions currently show “not implemented yet” dialogs in `storage-ui/src/app.rs`. Additionally, image-related actions currently live under the Image menu even when they conceptually operate on the selected drive.

User request:
- Implement remaining Disk commands under Disk menu:
  - Smart Data & Self-Tests
  - Drive Settings — remove (redundant)
  - Standby Now
  - Wake-up
  - Power off

## Goals
- Disk menu contains the requested set of actions and **Drive Settings** is removed.
- Each Disk menu action performs a real operation against the **currently selected drive** (or clearly errors when unsupported / no drive selected).
- Provide user-visible feedback (progress, success, actionable error messages) and avoid UI freezes.

## Non-Goals
- Implement full-blown “GNOME Disks parity” UI for SMART (graphs, history, advanced settings).
- Implement disk image create/restore (explicitly deferred to a separate PR).
- Implement disk benchmarking (explicitly deferred to a separate PR).
- Add complex scheduling/background job management (single-operation-at-a-time is OK initially).

## Proposed Approach
### 1) Menu + message cleanup
- Update `storage-ui/src/views/menu.rs`:
  - Disk menu: keep only disk-oriented actions (SMART/standby/wake-up/power off), plus existing eject/format.
  - Remove “Drive Settings” item.
  - Leave image-related actions for a separate PR.
- Update `Message` handling in `storage-ui/src/app.rs`:
  - Remove or deprecate `DriveSettings` flow.
  - Replace “not implemented yet” dialogs for Disk actions with real async tasks.

### 2) DBus backend surface in `storage-dbus`
Implement missing drive operations in `storage-dbus/src/disks/drive.rs` (or a dedicated module) using UDisks2:
- `power_off()` already exists — UI should call it and then refresh drive list.
- Add:
  - `standby_now()` and `wakeup()` via UDisks2 Drive ATA methods (verify exact method names via introspection; likely `org.freedesktop.UDisks2.Drive.AtaStandbyNow` / `...AtaWakeup`).
  - SMART data + self-tests via `org.freedesktop.UDisks2.Drive.AtaSmart*` APIs.

  Out of scope here:
  - Disk imaging (create/restore) via `org.freedesktop.UDisks2.Block.OpenForBackup` and `OpenForRestore`.

Implementation notes:
- The `udisks2` crate may not expose all ATA calls; use a raw `zbus::Proxy` call when needed (pattern already used in `storage-dbus/src/disks/ops.rs` for better error messages).
- Prefer “preflight” checks:
  - Ensure no mounted children for restore (and require user confirmation).
  - For imaging, clarify whether to image whole disk block device (`DriveModel.block_path`) vs a partition.

### 3) UI flows
- Add dialogs/pages in `storage-ui/src/views/dialogs.rs` or new view modules:
  - **SMART Data & Self-Tests**: show last update timestamp, basic health/status, a table of key attributes, and buttons for short/extended self-tests.

Out of scope here:
- Create/restore disk image UI.
- Disk benchmark UI.

File selection:
- Determine preferred approach for COSMIC apps:
  - If libcosmic provides a portal/file picker API, use that.
  - Otherwise, consider adding a small dependency for file picking (e.g. `rfd`) and gate to Linux.

### 4) Error handling & permissions
- UDisks2 methods may trigger polkit authentication; ensure UI remains responsive.
- When an operation is unsupported (no ATA SMART, no standby capability, etc.), show a user-friendly “Not supported by this drive” dialog rather than a generic error.

## User/System Flows
- **Power off**: user selects drive → Disk → Power Off → app calls `DriveModel::power_off()` → refresh drives.
- **Standby/Wake-up**: user selects drive → Disk → Standby Now/Wake-up → app calls new DBus methods → show success/error.
- **SMART**: user selects drive → Disk → SMART Data & Self-Tests → fetch + display; allow starting tests.

## Risks & Mitigations
- **Unknown UDisks2 method names/availability**: Mitigate by adding an explicit “introspection verification” step using `busctl introspect` and falling back to “unsupported” when missing.
- **Permissions/polkit prompts**: Ensure operations are async; surface clear error messages.
- **Large copy operations**: Implement streaming with bounded buffers and progress reporting; allow cancel.
- **Safety for restore**: Require confirmation and ideally ensure target is unmounted/locked.

## Acceptance Criteria
- [ ] Disk menu shows: SMART Data & Self-Tests, Standby Now, Wake-up From Standby, Power Off (and existing items like Eject/Format remain unchanged).
- [ ] Drive Settings is removed from the Disk menu.
- [ ] Each of the above Disk actions no longer shows the “not implemented yet” info dialog.
- [ ] Operations run asynchronously and keep the UI responsive.
- [ ] On success, user gets confirmation (or updated UI state) and the drive list refreshes where appropriate.
- [ ] On failure or unsupported hardware, user sees an actionable error message.
