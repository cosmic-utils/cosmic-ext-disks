# feature/volume-commands-actionbar — Tasks

## Task 1: Define UI command set + message enums
- Status: Done
- Scope: Add message types and selection rules for which commands apply to which selected object.
- Files/areas:
  - `disks-ui/src/views/volumes.rs`
  - `disks-ui/src/app.rs` (dialog state types)
- Steps:
  - Add new `VolumesControlMessage` variants for:
    - FormatPartition
    - EditPartition (open + field updates + confirm/cancel)
    - ResizePartition (open + field updates + confirm/cancel)
    - EditFilesystemLabel (open + field updates + confirm/cancel)
    - CheckFilesystem (warn confirm/cancel)
    - RepairFilesystem (warn confirm/cancel)
    - TakeOwnership (open + recursive toggle + confirm/cancel)
    - ChangePassphrase (open + current/new/confirm updates + confirm/cancel)
  - Define a single “selected target” helper:
    - If a child `VolumeNode` is selected and supports filesystem ops, use it.
    - Else use `selected.volume` (`VolumeModel`).
  - Add placeholder wiring (open dialogs only) without DBus calls yet.
- Test plan:
  - Manual: UI shows the right buttons for each `VolumeType` and selection state.
- Done when:
  - [ ] Buttons and message wiring compile; dialogs open from actionbar clicks.

## Task 2: Add dialogs and validation
- Status: Done
- Scope: Implement dialog UIs for each command with required validation and warnings.
- Files/areas:
  - `disks-ui/src/views/dialogs.rs`
  - `disks-ui/src/app.rs` (new dialog structs + `ShowDialog` variants)
  - `disks-ui/i18n/en/cosmic_ext_disks.ftl` (+ sv if required by repo conventions)
- Steps:
  - Implement dialogs:
    - Edit Partition: type dropdown (exhaustive list), name, flags.
    - Resize: size slider/spinners, min/max clamp display.
    - Edit filesystem label: label field.
    - Check filesystem: warning confirmation.
    - Repair filesystem: warning confirmation.
    - Take ownership: recursive checkbox + warning.
    - Change passphrase: current/new/confirm secure inputs + match validation.
    - Format partition: reuse Create Partition layout with size controls disabled.
  - Add i18n strings for titles, button labels, warnings, tooltips as needed.
- Test plan:
  - Manual: each dialog enforces validation and disables confirm when invalid.
- Done when:
  - [ ] All dialogs render correctly and validation matches requirements.

## Task 3: DBus: implement missing `VolumeModel` operations
- Status: Done
- Scope: Replace stub implementations in `disks-dbus/src/disks/partition.rs`.
- Files/areas:
  - `disks-dbus/src/disks/partition.rs`
  - Potentially `disks-dbus/src/disks/ops.rs` (if new backend helpers are useful)
- Steps:
  - Implement:
    - `edit_partition(type, name, flags)` using `PartitionProxy::set_type`, `set_name`, `set_flags`.
    - `edit_filesystem_label(label)` using `FilesystemProxy::set_label`.
    - `resize(new_size_bytes)` using `PartitionProxy::resize`.
    - `check_filesystem()` using `FilesystemProxy::check`.
    - `repair_filesystem()` using `FilesystemProxy::repair`.
    - `take_ownership(recursive)` using `FilesystemProxy::take_ownership` with `recursive` option.
    - `change_passphrase(current, new)` using `EncryptedProxy::change_passphrase`.
  - Ensure all methods use empty options maps except where explicitly required.
  - Ensure errors are propagated (no swallowing).
- Test plan:
  - Unit tests where feasible (non-destructive): validate option maps and proxy calls via a fake backend if the architecture supports it.
  - Manual: run against a test disk/loopback and confirm operations succeed/fail with clear errors.
- Done when:
  - [ ] No stub `Ok(())` remains for these methods.

## Task 4: Partition type exhaustive list API
- Status: Done
- Scope: Provide “full exhaustive list of partition types” for Edit Partition dropdown.
- Files/areas:
  - `disks-dbus/src/partition_type.rs`
  - `disks-dbus/src/lib.rs` exports (if needed)
  - `disks-ui/src/views/dialogs.rs` (dropdown data shape)
- Steps:
  - Add a function returning full catalog entries for a given table type:
    - Prefer returning a stable list of `(id, display_name)`; optionally include group/subtype.
  - Decide whether to filter `CreateOnly` entries for edit.
  - Wire UI dropdown to this list and store selected partition type id.
- Test plan:
  - Manual: Edit Partition dialog shows large list; selection persists and sets correctly.
- Done when:
  - [ ] UI uses full list, not the “common” list.

## Task 5: UI execution + refresh + error surfacing
- Status: Done
- Scope: Hook dialog confirms to DBus calls, refresh nav, and show errors.
- Files/areas:
  - `disks-ui/src/views/volumes.rs`
  - `disks-ui/src/app.rs` (message routing)
- Steps:
  - For each command confirm message, execute a `Task::perform` calling the corresponding `VolumeModel` method.
  - Refresh drive list via `DriveModel::get_drives()` on success.
  - On error, open an Info dialog with actionable details.
  - Ensure long-running actions disable their confirm buttons while running, and Add "working" label like in other dialogs. 
- Test plan:
  - Manual: run each command and ensure UI refresh + errors show as dialogs.
- Done when:
  - [ ] All commands execute end-to-end from actionbar.

## Task 6: Resize clamp logic
- Status: Done
- Scope: Compute min/max for resize exactly as required and gate the resize action.
- Files/areas:
  - `disks-ui/src/views/volumes.rs` (segment/right-free computation)
  - `disks-ui/src/views/dialogs.rs` (clamped control bounds)
- Steps:
  - Compute `free_space_to_the_right_bytes` from the segment model (contiguous free segment after the selected partition).
  - `max = current_size + free_right`.
  - `min = used_space` (from usage if available, else 0).
  - If `max - min < 1024`, disable the resize action.
- Test plan:
  - Manual: pick a nearly-full partition; resize is disabled when < 1KB range.
- Done when:
  - [ ] Resize bounds and disable rule match requirements.

## Recommended sequence
1) Task 1 → 2 → 3 → 4 → 6 → 5

## Overall test plan
- `cargo fmt --all`
- `cargo clippy --workspace --all-features`
- `cargo test --workspace --all-features`
- Manual validation on a non-critical disk/loop device for each command.

---

# Addendum (2026-01-25): Mount + Encryption Options — Tasks

## Task 7: GNOME Disks parity research capture (fstab/crypttab)
- Status: Done
- Scope: Lock down the exact field set + token mappings we will implement (GNOME Disks parity).
- Files/areas:
  - `.copi/specs/feature/volume-commands-actionbar/plan.md` (addendum section)
- Steps:
  - Capture upstream references (GNOME Disks 46.1):
    - `src/disks/gdufstabdialog.c` + `src/disks/ui/edit-fstab-dialog.ui`
    - `src/disks/gducrypttabdialog.c` + `src/disks/ui/edit-crypttab-dialog.ui`
  - Document mapping:
    - fstab: `opts` tokens (`noauto`, `x-udisks-auth`, `x-gvfs-*`)
    - crypttab: `options` tokens (`noauto`, `x-udisks-auth`)
  - Confirm UDisks2 configuration item keys to be used for `fstab`/`crypttab`.
- Test plan:
  - N/A (documentation task)
- Done when:
  - [x] Spec addendum contains field list + key/token mapping.

## Task 8: DBus: add/update/remove UDisks2 Block configuration items
- Status: Done
- Scope: Implement backend plumbing to read and mutate `org.freedesktop.UDisks2.Block` configuration items for `fstab` and `crypttab`.
- Files/areas:
  - `disks-dbus/src/disks/partition.rs` (implement `edit_mount_options` and extend/replace `edit_encrytion_options`)
  - Potentially `disks-dbus/src/disks/ops.rs` or a new module for Block configuration helpers
- Steps:
  - Add helpers to:
    - Read current `Block.Configuration` list
    - Find the first matching item by type (`fstab`/`crypttab`)
    - Add/update/remove configuration item
  - Implement `VolumeModel::edit_mount_options(...)`:
    - If defaults enabled → remove `fstab` item
    - Else add/update with keys: `fsname`, `dir`, `type`, `opts`, `freq`, `passno`
  - Implement encryption options method (rename typo if desired, but keep public API stable if already used):
    - If defaults enabled → remove `crypttab` item
    - Else add/update with keys: `device`, `name`, `options`, `passphrase-path`, `passphrase-contents`
  - Ensure we follow GNOME Disks “first item only” behavior.
- Test plan:
  - If the repo has a fake backend/test harness: unit test option token parsing + dict creation.
  - Manual: validate a config change is reflected via `udisksctl info` and/or system files managed by udisks.
- Done when:
  - [x] DBus layer exposes working APIs for mount/encryption option persistence.

## Task 9: UI: add “Edit Mount Options…” + dialog
- Status: Done
- Scope: Add action button + dialog UI + confirm wiring for fstab config.
- Files/areas:
  - `disks-ui/src/views/volumes.rs` (actionbar button visibility rules)
  - `disks-ui/src/app.rs` (dialog state + ShowDialog variant)
  - `disks-ui/src/views/dialogs.rs` (dialog layout)
  - `disks-ui/i18n/en/cosmic_ext_disks.ftl` (+ `sv` if required)
- Steps:
  - Add actionbar button and message routing.
  - Implement dialog with GNOME fields (including Identify As dropdown).
  - Implement “User Session Defaults” disabling behavior.
  - On confirm: call DBus method, refresh drives, show errors via Info dialog.
- Test plan:
  - Manual: toggle noauto/auth/show in UI; verify applied and persists across refresh.
- Done when:
  - [x] Mount options dialog works end-to-end for a test filesystem.

## Task 10: UI: add “Edit Encryption Options…” + dialog (LUKS only)
- Status: Done
- Scope: Add action button + dialog UI + confirm wiring for crypttab config.
- Files/areas:
  - `disks-ui/src/views/volumes.rs`
  - `disks-ui/src/app.rs`
  - `disks-ui/src/views/dialogs.rs`
  - `disks-ui/i18n/en/cosmic_ext_disks.ftl` (+ `sv` if required)
- Steps:
  - Add actionbar button visible only when selected segment is a LUKS crypto container.
  - Implement dialog fields per spec (defaults toggle, startup/auth toggles, other options, name, passphrase, show passphrase).
  - On confirm: call DBus method, refresh drives, show errors via Info dialog.
- Test plan:
  - Manual: validate `noauto` and `x-udisks-auth` toggles; validate passphrase behavior.
- Done when:
  - [x] Encryption options dialog works end-to-end for a LUKS container.

## Task 11: Option-token parsing/formatting utility
- Status: Done
- Scope: Avoid corrupting user-entered option strings while managing known tokens.
- Files/areas:
  - `disks-ui/src/utils/` and/or `disks-dbus/src/` (shared helper, or duplicated carefully)
- Steps:
  - Implement helpers:
    - Split by `,`, trim, drop empties
    - Stable dedup
    - Set/remove exact tokens
    - Set/remove key-value tokens by prefix (e.g. `x-gvfs-name=`)
  - Ensure “Other options” merges without duplicating managed tokens.
- Test plan:
  - Unit tests for: duplicate tokens, whitespace, key-value replacement.
- Done when:
  - [x] Both dialogs can round-trip options reliably.
