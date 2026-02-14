# Format Disk Dialog (Disk → Format Disk)

Branch: `feature/format-disk-dialog`

Source: N/A (brief)

## Context

The Disk menu currently contains “Format Disk”, but selecting it shows an informational dialog stating the feature is not implemented. We need a proper “Format Disk” dialog that captures the user’s intent for:

- erase behavior (quick vs overwrite)
- partition-table behavior (GPT vs DOS/MBR vs none)

This spec focuses on adding the dialog and state plumbing end-to-end, and defining how it should map to the backend operation(s).

## Goals

- Replace the “not implemented yet” info dialog for Disk → Format Disk with a real dialog.
- Dialog contains two comboboxes:
  - **Erase**: “Don’t Overwrite (Quick)” and “Overwrite (Slow)”
  - **Partitioning**: “Legacy Compatible (DOS/MBR)”, “Modern (GPT)”, and “None” (empty)
- Provide a safe default selection strategy, clear destructive messaging, and a single confirm action.

## Non-Goals

- Implementing a full “format wizard” (filesystem type selection, labels, mount options).
- Implementing imaging/restore/benchmark/SMART features.
- Polkit/privilege model changes (assume existing UDisks privilege handling).

## Proposed Approach

### UI state + messaging

- Add a new `ShowDialog` variant for a format-disk dialog state (e.g. `ShowDialog::FormatDisk(FormatDiskDialog)`), similar to `UnlockEncrypted`.
- Add a small message enum for the dialog interactions (e.g. `FormatDiskMessage`) to track:
  - erase selection changes
  - partitioning selection changes
  - cancel
  - confirm
- Render the dialog in `views/dialogs.rs` with `dropdown` widgets and destructive primary action.

### Default selections

- **Erase** default: “Don’t Overwrite (Quick)”.
- **Partitioning** default:
  - If the selected drive has a known partition table type (`DriveModel.partition_table_type`): default to that (mapping UDisks `dos` → “Legacy Compatible (DOS/MBR)”, `gpt` → “Modern (GPT)”).
  - Otherwise: default to “None”.

### Mapping to backend behavior (when confirm is pressed)

Define a backend request structure that captures the two combobox choices:

- `erase_mode`: quick vs overwrite
- `partitioning`: none vs gpt vs dos

Implementation detail (to be confirmed during implementation):

- If `partitioning` is `gpt` or `dos`, the operation should create/replace the disk’s partition table accordingly.
- If `partitioning` is `none`, the operation should remove any existing partition table (or otherwise leave the disk without a partition table) if the stack supports it.
- If `erase_mode` is overwrite, perform a slow “write zeros” (or UDisks equivalent) operation.

If the repository does not yet have an appropriate storage-dbus API for whole-disk format / partition table creation, add one in `storage-dbus` (DriveModel method), and wire it from the UI.

### Error handling

- Confirm action should transition to a busy/disabled state while running.
- On backend error: show an `Info` dialog with the error message (consistent with other flows).

### i18n

Add new Fluent strings for the new labels/options (and reuse existing ones where appropriate). The existing `format-disk` label in the menu should remain.

## User / System Flows

1. User selects a drive in the left nav.
2. User opens **Disk → Format Disk**.
3. App shows “Format Disk” dialog:
   - Erase combobox
   - Partitioning combobox
   - Destructive “Format”/“Continue” primary action and “Cancel” secondary action
4. User selects options and confirms.
5. App performs disk operation; on success refreshes drive list / volumes view.

## Risks & Mitigations

- **Risk: destructive operation without enough friction.**
  - Mitigation: destructive primary button + clear prompt text; consider a secondary confirmation step if needed.
- **Risk: UDisks limitations for “no partitioning”.**
  - Mitigation: treat `None` as “do not create a partition table” (and document behavior), or implement “wipe signatures” if supported.
- **Risk: long-running overwrite blocks UI.**
  - Mitigation: async task with disabled controls; optional progress indicator.

## Acceptance Criteria

- Disk → Format Disk opens the new dialog (no longer shows “not implemented yet”).
- Dialog contains:
  - Erase combobox with exactly:
    - “Don’t Overwrite (Quick)”
    - “Overwrite (Slow)”
  - Partitioning combobox with exactly:
    - “Legacy Compatible (DOS/MBR)”
    - “Modern (GPT)”
    - “None” (empty)
- Defaults:
  - Erase defaults to “Don’t Overwrite (Quick)”.
  - Partitioning defaults to current drive table type when known, otherwise “None”.
- Cancel closes the dialog without changes.
- Confirm triggers a single async operation based on the selected options and refreshes the UI on success.
- `cargo fmt --all --check`, `cargo clippy --workspace --all-features`, and `cargo test --workspace --all-features` pass.
