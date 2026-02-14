# Implementation Spec — Volume Commands Actionbar

Branch: feature/volume-commands-actionbar
Source: Brief (2026-01-25)

## Context
The Volumes view currently supports a small subset of actions (mount/unmount, unlock/lock for LUKS containers, delete). Several volume-level operations exist as stubs in the DBus layer (`VolumeModel` methods in `storage-dbus/src/disks/partition.rs`) and are not represented in the UI.

This spec adds a consistent “volume commands” actionbar (buttons with tooltips) under the volumes list/tiles, and implements the missing command plumbing end-to-end:
- UI action buttons + dialogs
- Message handling and async tasks
- DBus methods for partition/filesystem/encrypted operations via UDisks2

## Goals
- Add an actionbar section “below volumes” containing buttons for all requested volume commands.
- Show/hide commands based on `VolumeType` as specified.
- Each command button has a tooltip equal to the command name.
- Implement the DBus-layer operations (no `todo!()` / no-op stubs) for:
  - Partition edit (type/name/flags)
  - Resize
  - Filesystem label edit
  - Filesystem check
  - Filesystem repair
  - Take ownership (recursive option)
  - LUKS change passphrase (requires current passphrase + new passphrase)
- “Format Partition” uses `Block.Format` (format-in-place), not delete+recreate.
- Dialog UX rules:
  - Warn dialogs for long-running operations (check/repair) as specified.
  - Resize clamps min/max as specified, with disable-if-range-too-small.

## Non-Goals
- Implement “edit mount options”, “default mount options”, or “edit encryption options” (these exist as TODOs but are out of scope).
- Add progress reporting for long-running UDisks jobs (may be added later; this spec focuses on correctness and basic UX).
- Add new filesystem creation flows beyond what already exists.
- Add new partition-table types beyond existing catalog behavior.

## Proposed Approach
### 1) UI: add Volume Commands actionbar
- In `storage-ui/src/views/volumes.rs`, extend the existing action bar row to:
  - Continue rendering existing mount/unmount, unlock/lock, delete, create (free space) actions.
  - Add a new group/section for “volume commands” buttons based on the *selected target*:
    - If a child `VolumeNode` is selected, volume commands apply to the child where relevant.
    - Otherwise, commands apply to the selected `VolumeModel` segment.
- Each command is a `widget::button::custom(icon::from_name(...))` with `.tooltip(<command name>)` (or the equivalent COSMIC/iced tooltip wrapper used in this codebase).
- Add new `VolumesControlMessage` variants for each command and for dialog field updates.

### 2) Dialogs
Add new dialog states (likely in `storage-ui/src/app.rs` alongside existing dialogs) and view renderers (in `storage-ui/src/views/dialogs.rs`) for:

**2.1 Format Partition (all `VolumeType`s)**
- Opens the existing Create Partition dialog UI but in “format existing” mode:
  - Prefill fields from the selected volume:
    - Name/label: use current filesystem label if available, otherwise volume name.
    - Filesystem type selection: default to current `id_type` mapping if possible; otherwise default ext4.
    - Erase toggle: default off.
  - Disable all size controls (slider + spinners).
  - On confirm: call `VolumeModel::format(label, erase, filesystem_type)`.

**2.2 Edit Partition (only `VolumeType::Partition`)**
- Dialog fields:
  - Type: full exhaustive list of partition types (not the “common” list).
  - Name
  - Flags:
    - Legacy BIOS Bootable
    - System Partition
    - Hide from firmware
- On confirm: call `VolumeModel::edit_partition(type, name, flags)`.

**2.3 Resize (only `VolumeType::Partition`)**
- Dialog uses the size controls from the existing Create Partition dialog (slider + spinners) but with min/max clamps:
  - `min_size_bytes = used_space_bytes`
  - `max_size_bytes = current_size_bytes + free_space_to_the_right_bytes`
  - If `(max - min) < 1024`, disable Resize button.
- On confirm: call `VolumeModel::resize(new_size_bytes)`.

**2.4 Edit Filesystem (only `VolumeType::Partition` and `VolumeType::Filesystem`)**
- Dialog field:
  - Label
- On confirm: call `VolumeModel::edit_filesystem_label(label)`.

**2.5 Check Filesystem (only `VolumeType::Partition` and `VolumeType::Filesystem`)**
- Confirmation warning dialog (continue/cancel): “can take a long time”.
- On continue: call `VolumeModel::check_filesystem()`.

**2.6 Change Passphrase (only `VolumeType::Container` for LUKS)**
- Dialog fields:
  - Current passphrase (secure)
  - New passphrase (secure)
  - Confirm new passphrase (secure)
- Validation:
  - new == confirm
  - new not empty
- On confirm: call `VolumeModel::change_passphrase(current, new)`.

**2.7 Repair Filesystem (only `VolumeType::Filesystem`)**
- Confirmation warning dialog (continue/cancel): “not always successful”, “can cause data loss”, “should back up”, “can take a long time”.
- On continue: call `VolumeModel::repair_filesystem()`.

**2.8 Take Ownership (only `VolumeType::Filesystem`)**
- Dialog:
  - Recursive mode checkbox
  - Warning text: recursive applies ownership to directories and files within.
- On confirm: call `VolumeModel::take_ownership(recursive)`.

### 3) DBus layer (storage-dbus)
Implement the currently-stubbed methods in `storage-dbus/src/disks/partition.rs` using the `udisks2` crate proxies:

- `edit_partition(partition_type, name, flags)`
  - Use `udisks2::partition::PartitionProxy`:
    - `SetType(type)`
    - `SetName(name)`
    - `SetFlags(flags)`
  - Options: empty `a{sv}`.
  - Flags mapping: UI checkbox state -> `PartitionFlags` bits.

- `edit_filesystem_label(label)`
  - Use `udisks2::filesystem::FilesystemProxy::set_label(label, options)`.

- `resize(new_size_bytes)`
  - Partition resize: `PartitionProxy::resize(new_size_bytes, options)`.
  - Filesystem resize is out-of-scope (not requested); only partition resize is needed here.

- `check_filesystem()` / `repair_filesystem()`
  - Use `FilesystemProxy::check(options)` and `FilesystemProxy::repair(options)`.
  - Treat `VolumeType::Partition` and `VolumeType::Filesystem` the same (object path implements Filesystem).
  - Handle “mounted filesystem” errors by surfacing a clear dialog.

- `take_ownership(recursive)`
  - Use `FilesystemProxy::take_ownership(options)` where options includes `recursive: bool`.

- `change_passphrase(current, new)`
  - Use `udisks2::encrypted::EncryptedProxy::change_passphrase(current, new, options)`.

### 4) Partition type list: exhaustive
- Add a helper in `storage-dbus` to expose the full catalog for a given table type:
  - Either return `Vec<(id, display_name)>` or a structured list with grouping.
  - Filter out `CreateOnly` types for “Edit Partition” (optional; if included, ensure UDisks errors are handled gracefully).
- UI dropdown shows the exhaustive list (may include group headings or a flat list).

### 5) Resize clamp computation (free space to the right)
- UI already has segmentation logic via `compute_disk_segments` / `PartitionExtent`.
- For a selected partition segment:
  - Determine the contiguous free-space segment immediately to the right (until the next partition/reserved segment).
  - `free_space_to_the_right_bytes = that_free_segment.size`.
- Used-space clamp:
  - Prefer `VolumeModel.usage.used_bytes` when present (mounted); if missing, fall back to `0` and rely on backend errors.

## User/System Flows
### A) Format Partition (any volume)
1. User selects a volume.
2. User clicks “Format Partition”.
3. Dialog opens with current details, size controls disabled.
4. User confirms.
5. System calls `Block.Format` and refreshes drive list.

### B) Edit Partition
1. User selects a partition.
2. User clicks “Edit Partition”.
3. Dialog shows exhaustive partition type list, name, flags.
4. User confirms.
5. System applies SetType/SetName/SetFlags and refreshes.

### C) Resize
1. User selects a partition.
2. User clicks “Resize”.
3. Dialog shows size controls clamped to `[min_used, max_right_free+current]`.
4. If range < 1KB, Resize is disabled.
5. User confirms.
6. System calls `Partition.Resize` and refreshes.

### D) Filesystem ops
- Edit filesystem label: open dialog, confirm, call `Filesystem.SetLabel`.
- Check filesystem: warning dialog, continue -> `Filesystem.Check`.
- Repair filesystem: warning dialog, continue -> `Filesystem.Repair`.
- Take ownership: dialog with recursive toggle, confirm -> `Filesystem.TakeOwnership(recursive)`.

### E) Change passphrase (LUKS)
1. User selects a container partition (LUKS).
2. User clicks “Change Passphrase”.
3. Dialog requests current passphrase + new + confirm.
4. Confirm triggers `Encrypted.ChangePassphrase`.

## Risks & Mitigations
- **Mounted filesystem check/repair fails:** UDisks2 rejects mounted FS check/repair.
  - Mitigation: Surface error clearly; optionally suggest unmount first.
- **Partition type catalog is large:** exhaustive list may be unwieldy.
  - Mitigation: allow grouping by subtype (linux/microsoft/other) and/or add search later.
- **Usage clamp may be missing:** used space may not be available.
  - Mitigation: clamp min to 0 when unknown; backend will validate; still keep max clamp.
- **Permissions/Polkit prompts:** many operations require authorization.
  - Mitigation: Ensure errors are displayed; do not swallow failures.

## Acceptance Criteria
- [x] Actionbar shows buttons below volumes; each has a tooltip with the command name.
- [x] “Format Partition” appears for all `VolumeType`s, prefills current details, disables size controls, and formats via `Block.Format`.
- [x] `VolumeType::Partition` shows “Edit Partition” and “Resize” dialogs as specified.
- [x] “Resize” enforces min/max clamps and disables if `(max-min) < 1024` bytes.
- [x] `VolumeType::Partition` + `VolumeType::Filesystem` show “Edit filesystem” and “Check Filesystem” actions.
- [x] `VolumeType::Container` (LUKS) shows “Change Passphrase” dialog with current+new+confirm.
- [x] `VolumeType::Filesystem` shows “Repair Filesystem” and “Take Ownership” dialogs/warnings as specified.
- [x] All DBus methods listed in Goals are implemented (no stub `Ok(())` placeholders).
- [x] Errors are surfaced via UI dialogs (no silent failures).
- [ ] Manual validation on a real disk/loop device confirms each command triggers the expected UDisks2 operation and refreshes the view.

## Implementation Notes
- Workspace builds cleanly and passes repo quality gates: `cargo fmt --all --check`, `cargo clippy --workspace --all-features`, `cargo test --workspace --all-features`.

---

## Addendum (2026-01-25): Edit Mount Options + Edit Encryption Options (GNOME Disks parity)

### Research baseline (required)
Mirror GNOME Disks (gnome-disk-utility) behavior and persistence model:
- Mount options: `src/disks/ui/edit-fstab-dialog.ui` + `src/disks/gdufstabdialog.c`
- Encryption options: `src/disks/ui/edit-crypttab-dialog.ui` + `src/disks/gducrypttabdialog.c`

GNOME Disks stores these settings via UDisks2 `org.freedesktop.UDisks2.Block` “configuration items”:
- `fstab` item → mount options
- `crypttab` item → encryption options

### Scope
Add two new volume commands:
- **Edit Encryption Options…**: only on LUKS container partitions.
- **Edit Mount Options…**: on filesystems and partitions (like GNOME Disks).

### UX (GNOME parity)
#### A) User Session Defaults
Both dialogs have a **User Session Defaults** toggle.
- When enabled: disable all subsequent controls.
- When enabled and a config exists: remove the corresponding config item (`fstab`/`crypttab`).

#### B) Encryption Options (crypttab) dialog fields
- User Session Defaults (checkbox)
- Unlock at system startup (checkbox)
  - Unchecked ⇒ add `noauto`
- Require additional authorisation to unlock (checkbox)
  - Checked ⇒ add `x-udisks-auth`
- Other options (textbox)
  - Comma-delimited list merged into the `options` string.
- Name (textbox)
- Passphrase (textbox)
- Show passphrase (checkbox)
  - UI-only; does not affect persistence.

#### C) Mount Options (fstab) dialog fields
GNOME Disks exposes more than the brief lists; mirror it because the DBus stub already anticipates these fields.
- User Session Defaults (checkbox)
- Mount at system startup (checkbox)
  - Unchecked ⇒ add `noauto`
- Require additional authorisation to mount (checkbox)
  - Checked ⇒ add `x-udisks-auth`
- Show in user interface (checkbox)
  - Checked ⇒ add `x-gvfs-show`
- Other options (textbox)
  - Comma-delimited list merged into `opts`.
- Display Name (textbox) → `x-gvfs-name=<value>`
- Icon Name (textbox) → `x-gvfs-icon=<value>`
- Symbolic Icon Name (textbox) → `x-gvfs-symbolic-icon=<value>`
- Mount Point (textbox)
- Identify As (dropdown)
- Filesystem Type (textbox)

### Persistence model

#### A) `fstab` configuration item
UDisks2 configuration item `( "fstab", a{sv} )` with keys:
- `fsname` (bytestring)
- `dir` (bytestring)
- `type` (bytestring)
- `opts` (bytestring)
- `freq` (int32, keep GNOME default `0`)
- `passno` (int32, keep GNOME default `0`)

Token mapping in `opts` (GNOME parity):
- Startup toggle ↔ `noauto` (inverted)
- Require auth ↔ `x-udisks-auth`
- Show in UI ↔ `x-gvfs-show`
- Display name ↔ `x-gvfs-name=...` (set/remove)
- Icon name ↔ `x-gvfs-icon=...` (set/remove)
- Symbolic icon ↔ `x-gvfs-symbolic-icon=...` (set/remove)

Defaults when creating a new `fstab` item (GNOME parity):
- `type = auto`
- `opts = nosuid,nodev,nofail,x-gvfs-show` (+ add `noauto` for removable drives)

Validation (GNOME parity):
- When not using defaults, disable Apply/OK unless `fsname`, `dir`, `type`, and `opts` are all non-empty.

#### B) `crypttab` configuration item
UDisks2 configuration item `( "crypttab", a{sv} )` with keys:
- `device` (bytestring) — forced to `UUID=<block-uuid>`
- `name` (bytestring)
- `options` (bytestring)
- `passphrase-path` (bytestring)
- `passphrase-contents` (bytestring)

Token mapping in `options` (GNOME parity):
- Startup toggle ↔ `noauto` (inverted)
- Require auth ↔ `x-udisks-auth`

Passphrase behavior (GNOME parity):
- If passphrase is non-empty:
  - set `passphrase-contents` to the entered value
  - set `passphrase-path` to an existing non-empty non-`/dev*` path if available; otherwise default to `/etc/luks-keys/<name>`
- If passphrase is empty:
  - set both `passphrase-path` and `passphrase-contents` to empty

### Implementation notes
- Centralize mount/encryption option parsing/formatting: split on `,`, trim whitespace, stable-dedup tokens, preserve non-managed tokens.
- GNOME Disks only considers the first `fstab`/`crypttab` config item if multiple exist; do the same.
- Editing these settings is Polkit-gated; always surface errors via Info dialog.
- Dialogs should pre-fill from existing `fstab`/`crypttab` configuration items when present.
- For safety, do not pre-fill passphrase contents.

### Acceptance Criteria
- [x] “Edit Encryption Options…” appears only for LUKS containers and opens the dialog described above.
- [x] “Edit Mount Options…” appears for filesystems/partitions and opens the dialog described above.
- [x] “User Session Defaults” removes the corresponding `fstab`/`crypttab` configuration item.
- [x] Toggle-to-token mappings match GNOME Disks exactly (`noauto`, `x-udisks-auth`, `x-gvfs-*`).
- [x] Confirm applies settings via UDisks2 configuration item add/update/remove and refreshes the view.
- [x] Dialogs pre-fill from existing `fstab`/`crypttab` configuration items when present (passphrase not prefilled).
