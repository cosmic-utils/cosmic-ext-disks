# Implementation Spec — Volume Commands Actionbar

Branch: feature/volume-commands-actionbar
Source: Brief (2026-01-25)

## Context
The Volumes view currently supports a small subset of actions (mount/unmount, unlock/lock for LUKS containers, delete). Several volume-level operations exist as stubs in the DBus layer (`VolumeModel` methods in `disks-dbus/src/disks/partition.rs`) and are not represented in the UI.

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
- In `disks-ui/src/views/volumes.rs`, extend the existing action bar row to:
  - Continue rendering existing mount/unmount, unlock/lock, delete, create (free space) actions.
  - Add a new group/section for “volume commands” buttons based on the *selected target*:
    - If a child `VolumeNode` is selected, volume commands apply to the child where relevant.
    - Otherwise, commands apply to the selected `VolumeModel` segment.
- Each command is a `widget::button::custom(icon::from_name(...))` with `.tooltip(<command name>)` (or the equivalent COSMIC/iced tooltip wrapper used in this codebase).
- Add new `VolumesControlMessage` variants for each command and for dialog field updates.

### 2) Dialogs
Add new dialog states (likely in `disks-ui/src/app.rs` alongside existing dialogs) and view renderers (in `disks-ui/src/views/dialogs.rs`) for:

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

### 3) DBus layer (disks-dbus)
Implement the currently-stubbed methods in `disks-dbus/src/disks/partition.rs` using the `udisks2` crate proxies:

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
- Add a helper in `disks-dbus` to expose the full catalog for a given table type:
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
