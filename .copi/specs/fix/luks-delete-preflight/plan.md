# Implementation Spec — LUKS Delete Preflight + Child Action Hygiene

Branch: `fix/luks-delete-preflight`

Source: brief (PR #36 review feedback)

## Context
The Volumes UI supports nested volumes for LUKS containers (cleartext filesystem and/or LVM LVs). Today, the **Delete** action is wired to `PartitionModel::delete()` (UDisks2 `Partition.Delete`) and:

- When the selected partition is a LUKS container that is **unlocked** and has mounted descendants, the delete attempt can fail due to the device being busy.
- The UI currently swallows delete errors (prints to stdout and emits `Message::None`), which reads as a “silent failure”.
- When a child filesystem/LV inside a LUKS container is selected, the UI still renders the container’s delete button. This is confusing and makes destructive actions easier to misapply.

## Goals
- Make deleting a LUKS container robust:
  - If the container is unlocked, unmount any mounted descendants, lock the container, then delete.
- Ensure delete failures are visible to the user with a proper dialog (no stdout-only failures).
- Remove the Delete button from the UI when a child filesystem/LV inside a LUKS container is selected.

## Non-Goals
- Adding support for deleting child volumes/LVs/filesystems (this spec explicitly avoids it).
- New confirmation UI flows beyond reusing the existing delete confirmation dialog.
- Any new privileged workflow (Polkit), error translation, or localization beyond what’s needed for surfacing errors.

## Proposed Approach
### 1) UI: Add “delete preflight” for encrypted containers
In [disks-ui/src/views/volumes.rs](../../../../disks-ui/src/views/volumes.rs), in the `VolumesControlMessage::Delete` handler:

- Determine the selected partition (already available via `Segment.partition`).
- Determine whether the selected partition corresponds to a `VolumeNode` of kind `CryptoContainer`.
- If it is a crypto container and it is **unlocked**:
  1. Collect mounted descendant leaf volumes (reuse existing helpers `find_volume_node_for_partition` and `collect_mounted_descendants_leaf_first`).
  2. Unmount them leaf-first.
  3. Lock the container (`PartitionModel::lock()`), which maps to UDisks2 `Encrypted.Lock`.
  4. Delete the partition (`PartitionModel::delete()`).
- If it is not a crypto container (or is already locked), keep the existing behavior:
  - call `PartitionModel::delete()` (and let it best-effort unmount the outer partition).

This reuses the same “unmount descendants then lock” logic already used by the **Lock** action.

### 2) UI: Surface delete errors (no silent failures)
Still in `VolumesControlMessage::Delete`, replace the `println!("{e}")` + `Message::None` path with:
- `Message::Dialog(ShowDialog::Info { title, body })`
  - Title: use an existing localized string if available, otherwise add a minimal one (e.g. `delete-failed`).
  - Body: `e.to_string()`.

This should apply to all delete failures, not only LUKS.

### 3) UI: Hide Delete button when a child volume is selected
In [disks-ui/src/views/volumes.rs](../../../../disks-ui/src/views/volumes.rs), the action bar currently always renders delete for `DiskSegmentKind::Partition`.

Change the delete-button visibility rules:
- If a child volume is selected (`self.selected_volume.is_some()` or `selected_child_volume.is_some()`), do not render the delete button.
- Keep delete available when the container itself is selected (top-half selection for the segment).

This matches the existing UX rule: mount/unmount changes meaning when a child is selected; delete should not appear for a child.

## User/System Flows
### Flow A — Delete unlocked LUKS container
1. User selects the LUKS container segment (container, not child).
2. User clicks Delete → confirms.
3. App unmounts any mounted children, locks the container, then deletes the partition.
4. Drives refresh and the partition disappears.

### Flow B — Delete locked LUKS container
1. User selects the locked LUKS container segment.
2. User clicks Delete → confirms.
3. App deletes the partition.
4. If it fails, an error dialog is shown.

### Flow C — Child filesystem selected
1. User selects a child filesystem/LV within a LUKS container.
2. Action bar shows mount/unmount (as today) but does not show delete.

## Risks & Mitigations
- **Unmount/lock ordering:** UDisks2 commonly refuses `Encrypted.Lock` if cleartext is mounted; enforce unmount-leaf-first then lock.
- **Stale graph after unlock/lock:** the drive refresh after delete will naturally resync; avoid caching node references across async work except by cloning `VolumeNode` as done elsewhere.
- **Partial failures (one child fails to unmount):** propagate the first error and show it; do not attempt delete in this case.

## Acceptance Criteria
- [x] Deleting an unlocked LUKS container unmounts mounted children (if any), locks the container, then deletes.
- [x] Deleting a locked LUKS container either succeeds or shows a user-visible error dialog; it never fails silently.
- [x] Delete failures (any partition) show an Info dialog with the error string.
- [x] When a child filesystem/LV inside a LUKS container is selected, the Delete button is not present in the action bar.
- [ ] Manual validation: run through unlocked + mounted child, unlocked + unmounted child, and locked container scenarios.

