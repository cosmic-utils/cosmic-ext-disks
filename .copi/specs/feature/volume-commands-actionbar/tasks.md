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
