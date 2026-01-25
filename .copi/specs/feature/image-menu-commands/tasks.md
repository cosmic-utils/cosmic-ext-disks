# feature/image-menu-commands — Tasks

Branch: `feature/image-menu-commands`
Spec: `.copi/specs/feature/image-menu-commands/`

## Task 1: Add missing menu items + messages + i18n keys
- Scope: expose all Image menu commands (drive + partition), and ensure routing exists.
- Files/areas:
  - `disks-ui/src/views/menu.rs`
  - `disks-ui/src/app.rs` (Message enum + update handler)
  - `disks-ui/i18n/en/cosmic_ext_disks.ftl`, `disks-ui/i18n/sv/cosmic_ext_disks.ftl`
- Steps:
  - Add menu items for “Create/Restore … Partition” (and adjust labels for Drive if needed).
  - Add corresponding `Message` variants and map them from menu actions.
  - Add i18n keys for any new labels.
- Test plan:
  - Build the UI crate: `cargo build -p disks-ui`.
  - Smoke: open menu and ensure items appear.
- Done when:
  - [x] Image menu shows 6 commands.
  - [x] Clicking each emits a distinct message.

## Task 2: Implement “New Disk Image” dialog + file creation
- Scope: create empty image file from UI.
- Files/areas:
  - `disks-ui/src/app.rs` (message handling)
  - `disks-ui/src/views/dialogs.rs` (or new dialog module)
- Steps:
  - Create a dialog with destination path + size inputs.
  - Implement async file creation: create_new + set_len.
  - Add success/error feedback.
- Test plan:
  - Manual: create a 16 MiB file; verify size with `ls -lh`.
  - `cargo test --workspace --all-features`.
- Done when:
  - [x] Image → New Disk Image creates a file with requested size.
  - [x] Errors cleanly if the path exists or is invalid.

## Task 3: Add `disks-dbus` support for “Attach Disk” (loop setup)
- Scope: set up a loop device via UDisks2 for an image file.
- Files/areas:
  - `disks-dbus/src/disks/manager.rs` (new proxy methods or raw calls)
  - Potential new module `disks-dbus/src/disks/image.rs`
- Steps:
  - Implement a helper calling `org.freedesktop.UDisks2.Manager.LoopSetup`.
  - Define the minimal return type (created block object path) and error mapping.
  - Validate method signature via `busctl introspect org.freedesktop.UDisks2 /org/freedesktop/UDisks2/Manager`.
- Test plan:
  - Manual on a dev machine with UDisks2: attach a known image and observe device add.
  - Ensure no panics and errors are actionable.
- Done when:
  - [x] A loop device is created for a given image file path.

## Task 4: Implement “Attach Disk” UI flow (mount when possible)
- Scope: add dialog and mount behavior after loop setup.
- Files/areas:
  - `disks-ui/src/app.rs`
  - `disks-ui/src/views/dialogs.rs`
  - `disks-ui/src/views/volumes.rs` (if mount helpers need reuse)
- Steps:
  - Dialog to input image file path.
  - Call `disks-dbus` loop setup helper.
  - Attempt to mount:
    - If the resulting block has a filesystem, mount it.
    - Otherwise present guidance (“Attached; select a partition to mount”).
  - Trigger/await device refresh (existing signal-based stream should pick it up).
- Test plan:
  - Manual: attach an image containing a filesystem and confirm mount point appears.
  - Manual: attach an image with partitions; confirm new partitions show in UI.
- Done when:
  - [x] Attach Disk results in an attached device and mounts when applicable.

## Task 5: Add `disks-dbus` support for OpenForBackup/OpenForRestore
- Scope: obtain readable/writable FDs for imaging via UDisks2 Block methods.
- Files/areas:
  - `disks-dbus/src/disks/drive.rs`, `disks-dbus/src/disks/partition.rs` (or new `image.rs`)
  - Potential raw `zbus::Proxy` calls (similar to `disks-dbus/src/disks/ops.rs`).
- Steps:
  - Implement helpers to open selected drive/partition for backup/restore.
  - Validate method names/signatures via `busctl introspect` on a block object.
  - Ensure errors preserve UDisks messages.
- Test plan:
  - Manual: call helpers and ensure FDs are returned (no permission errors beyond polkit).
- Done when:
  - [x] Drive/partition can be opened for backup and restore via UDisks2.

## Task 6: Implement copy/restore engine + drive flows
- Scope: create/restore image for drives.
- Files/areas:
  - `disks-ui/src/app.rs`
  - `disks-ui/src/views/dialogs.rs` (progress + cancel)
- Steps:
  - Implement streaming copy using the FDs from Task 5.
  - Add progress + cancel.
  - Add confirmation/preflight for restore (ensure unmounted).
- Test plan:
  - Manual: create an image from a small removable drive/VM disk; verify image size.
  - Manual: restore into a disposable target and confirm it becomes readable.
- Done when:
  - [x] Drive create/restore completes successfully with cancel support.

## Task 7: Implement partition flows + selection validation
- Scope: create/restore image for partitions; selection rules.
- Files/areas:
  - `disks-ui/src/app.rs` (resolve selected partition)
  - `disks-ui/src/views/volumes.rs` (if helper methods exist)
- Steps:
  - Resolve selected partition from `VolumesControl` selection.
  - Wire partition create/restore dialogs to the copy engine.
  - Add “invalid selection” UX.
- Test plan:
  - Manual: select a partition and create image; restore to a disposable partition.
- Done when:
  - [x] Partition create/restore works and fails gracefully when selection is invalid.

## Task 8: Polish + documentation
- Scope: reduce friction and ensure quality gates.
- Files/areas:
  - `README.md` or `disks-ui/README.md` (if appropriate)
- Steps:
  - Document what “Attach Disk” does and its limitations.
  - Run formatting and clippy.
- Test plan:
  - `cargo fmt --all --check`
  - `cargo clippy --workspace --all-features`
  - `cargo test --workspace --all-features`
- Done when:
  - [x] Docs updated and CI quality gates pass.
