# Image Menu Commands — Implementation Plan

Branch: `feature/image-menu-commands`
Source: N/A (brief)

## Context
The app already has an **Image** menu wired to `Message` variants, but the handlers currently show “not implemented yet” dialogs.

Current menu wiring:
- `disks-ui/src/views/menu.rs` (Image menu)
- `disks-ui/src/app.rs` (`Message::{NewDiskImage,AttachDisk,CreateDiskFrom,RestoreImageTo}`)

User request: implement all Image menu commands (add missing items if needed)
- Create Disk Image From Drive
- Restore Disk Image To Drive
- Create Disk Image From Partition
- Restore Disk Image To Partition
- Attach Disk — mounts an image file
- New Disk Image — creates an empty image file of a given size in a given location

## Goals
- All requested Image menu commands exist and perform real actions (no placeholder dialogs).
- Commands operate on the current selection (drive/partition) or provide a clear, actionable error if selection is invalid.
- Long-running operations run asynchronously, keep the UI responsive, and provide at least a spinner + cancel.
- Operations are safe-by-default (explicit confirmations for destructive restore; preflight checks).

## Non-Goals
- Compression/encryption formats (e.g., qcow2, gzip) or image catalog management.
- Advanced partition inspection of image files (e.g., browsing partitions inside an image before attach).
- Benchmarking or SMART-related work (already covered by other work).

## Proposed Approach
### 1) Menu surface + message model
Update `disks-ui/src/views/menu.rs` and `disks-ui/src/app.rs` to expose the full set of commands.

Proposed `Message` set (names can vary, but keep it explicit in UI):
- `NewDiskImage`
- `AttachDisk`
- `CreateDiskImageFromDrive`
- `RestoreDiskImageToDrive`
- `CreateDiskImageFromPartition`
- `RestoreDiskImageToPartition`

If keeping existing `CreateDiskFrom`/`RestoreImageTo` names for compatibility, add partition variants and (optionally) rename later with deprecation.

Also update i18n strings:
- Add new keys for partition create/restore if missing in `disks-ui/i18n/*/cosmic_ext_disks.ftl`.

### 2) UI dialogs and flows
Add dialogs to collect parameters and run operations:

**New Disk Image**
- Inputs: destination path, size (bytes + unit), optional “preallocate” (future; default off).
- Behavior: create new file (fail if exists), set length, show success.

**Attach Disk (Mount image)**
- Inputs: image file path.
- Behavior:
  1) Use UDisks2 to set up a loop device for the selected file.
  2) Attempt to mount the resulting block device if it exposes a filesystem.
  3) If it contains partitions (no filesystem on top-level), inform the user and rely on the normal UI to show new devices/partitions for manual mounting.

**Create Disk Image From Drive / Partition**
- Inputs: destination path.
- Behavior: stream-read from selected block device into the destination file.

**Restore Disk Image To Drive / Partition**
- Inputs: source image path.
- Behavior:
  - Strong confirmation dialog (destructive): show target device path + size and warn it overwrites the target.
  - Preflight: ensure target is not mounted (and for partitions, ensure filesystem is unmounted).
  - Stream-write the image into the target.

File selection UX:
- First pass: a simple path text input is acceptable if there is no existing COSMIC file picker integration.
- Follow-up improvement (if feasible with existing deps): integrate xdg-desktop-portal / COSMIC picker.

### 3) Backend support in `disks-dbus`
Implement UDisks2-backed helpers so we do not rely on direct `/dev` access (which often requires root).

**Disk imaging**
- Add methods on `DriveModel`/`PartitionModel` (or a dedicated module) for:
  - `open_for_backup()` (read) and `open_for_restore()` (write)
  - Prefer calling UDisks2 `org.freedesktop.UDisks2.Block.OpenForBackup` and `OpenForRestore`.
  - Use a raw `zbus::Proxy` call when the `udisks2` crate does not expose the method (pattern already exists in `disks-dbus/src/disks/ops.rs`).

**Attach image**
- Add a helper using UDisks2 `org.freedesktop.UDisks2.Manager.LoopSetup` to create a loop device from an image file.
- Return the created block object path (or enough info to locate it) so the UI can attempt a mount.

### 4) Copy engine + progress/cancel
Implement a small streaming copier (likely in `disks-ui` since UI owns progress reporting) that:
- Accepts a read FD (from `OpenForBackup`) and a write file path, or vice versa.
- Copies in bounded chunks (e.g., 4–16 MiB) with periodic progress updates.
- Supports cancel (sets a shared flag; closes fds; surfaces “Cancelled” to UI).

### 5) Selection rules
Define how commands determine the target:
- “From/To Drive”: uses currently selected `DriveModel` (nav active).
- “From/To Partition”: uses the selected segment/volume when it resolves to a `PartitionModel` (or `VolumeNode` whose `kind == Partition`).
- If selection is invalid (e.g., free space, reserved, LVM LV when expecting partition), show a clear info dialog.

## User/System Flows
- **New Disk Image**: Image → New Disk Image → choose path/size → file created → confirmation.
- **Attach Disk**: Image → Attach Disk → choose image path → loop setup → mount if possible → device list updates.
- **Create From Drive/Partition**: Image → Create… → choose destination → copy runs → confirmation.
- **Restore To Drive/Partition**: Image → Restore… → choose source → confirm overwrite → preflight unmount → write image → confirmation.

## Risks & Mitigations
- **UDisks2 API availability differences**: verify with `busctl introspect` and degrade gracefully (“Not supported”).
- **Polkit prompts / permissions**: keep operations async; propagate error messages verbatim where safe.
- **Accidental data loss**: require explicit confirmations for restore and show the target device path/size.
- **Mount ambiguity for images with partitions**: attempt mount of top-level filesystem; otherwise guide user to mount inner partitions once discovered.

## Acceptance Criteria
- [ ] Image menu contains all 6 requested commands (drive + partition create/restore, attach, new image).
- [ ] No Image menu command shows a “not implemented yet” placeholder.
- [ ] New Disk Image creates a new file at requested size and errors if it already exists.
- [ ] Attach Disk sets up a loop device and mounts when a filesystem is present.
- [ ] Create Image streams data to a file for both drive and partition selections.
- [ ] Restore Image streams data from a file to the selected drive/partition, with destructive confirmation.
- [ ] UI remains responsive during copy/restore; cancel stops the operation.
- [ ] `cargo fmt --all --check`, `cargo clippy --workspace --all-features`, `cargo test --workspace --all-features` pass.
