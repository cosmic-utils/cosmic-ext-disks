# fix/luks-delete-preflight — Tasks

Branch: `fix/luks-delete-preflight`

## Task 1: Add delete preflight for unlocked LUKS containers
- Scope: When deleting a crypto container, unmount mounted descendants and lock before delete.
- Files/areas:
  - `disks-ui/src/views/volumes.rs`
- Steps:
  - In `VolumesControlMessage::Delete`, detect whether selected partition maps to a `VolumeKind::CryptoContainer`.
  - If container is unlocked:
    - Collect mounted descendants leaf-first (reuse `collect_mounted_descendants_leaf_first`).
    - Unmount descendants.
    - Call `PartitionModel::lock()`.
  - Call `PartitionModel::delete()`.
  - Refresh drives on success.
- Test plan:
  - Manual: unlock a LUKS container, mount a child filesystem, delete the container; confirm it unmounts/locks then deletes.
- Done when:
  - [x] Unlocked container deletion succeeds with mounted child.
  - [x] Unlocked container deletion works with no mounted child.

## Task 2: Surface delete errors via dialog (no stdout-only failures)
- Scope: Replace silent failures with a user-visible dialog.
- Files/areas:
  - `disks-ui/src/views/volumes.rs`
  - `disks-ui/src/app.rs` (only if new dialog title string is needed)
  - `disks-ui/i18n/**/cosmic_ext_disks.ftl` (only if adding `delete-failed`)
- Steps:
  - Replace `println!("{e}")` + `Message::None` in delete handler with `Message::Dialog(ShowDialog::Info { ... })`.
  - Decide title string:
    - Prefer existing strings if there is a suitable one; otherwise add `delete-failed` in i18n.
- Test plan:
  - Manual: provoke an expected failure (e.g. permissions) and confirm an Info dialog appears.
- Done when:
  - [x] Delete errors are visible in the UI.

## Task 3: Hide Delete button when a child volume is selected
- Scope: Prevent destructive action affordance for child volumes inside LUKS containers.
- Files/areas:
  - `disks-ui/src/views/volumes.rs`
- Steps:
  - In the action bar render path, gate the delete button behind `selected_child_volume.is_none()`.
  - Confirm delete remains available when the container is selected.
- Test plan:
  - Manual: select a child filesystem/LV row; confirm delete icon is absent.
- Done when:
  - [x] Delete button is absent for child selection.
  - [x] Delete button remains for container selection.

## Task 4: Sanity checks / QA
- Scope: Ensure flows don’t regress container lock/unlock or mount/unmount.
- Steps:
  - `cargo fmt --all`
  - `cargo clippy --workspace --all-features`
  - `cargo test --workspace --all-features`
  - Manual smoke: lock/unlock a container; mount/unmount a child; delete a non-encrypted partition.
- Done when:
  - [x] CI-equivalent checks pass locally.
  - [ ] Manual smoke passes.
