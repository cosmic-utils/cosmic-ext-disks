# feature/luks-logical-volumes — Tasks

Branch: `feature/luks-logical-volumes`

Source: brief (user request)

## Task 1: Model nested volumes in `storage-dbus`
- Scope: Add a data model that can represent container/child relationships.
- Files/areas:
  - `storage-dbus/src/disks/drive.rs`
  - `storage-dbus/src/disks/partition.rs`
  - New module likely: `storage-dbus/src/disks/volume.rs` (or similar)
- Steps:
  - Define `VolumeNode` (id/path, size, label/type, mount state, kind, children).
  - Add an accessor on `DriveModel` (e.g. `volumes: Vec<VolumeNode>`), without breaking existing callers.
  - Populate `volumes` for at least “plain filesystem partitions” (baseline parity).
- Test plan:
  - Unit tests that `VolumeNode` construction is stable and handles empty children.
- Done when:
  - [x] `DriveModel` exposes a nested volume representation.
  - [x] Existing UI continues to compile against current `DriveModel.partitions`.

## Task 2: Discover LUKS containers + cleartext devices
- Scope: Enumerate LUKS containers and their unlocked cleartext children.
- Files/areas:
  - `storage-dbus/src/disks/drive.rs`
  - `storage-dbus/src/disks/partition.rs`
  - Possibly new zbus proxy module: `storage-dbus/src/disks/encrypted.rs`
- Steps:
  - For each partition block object, detect `org.freedesktop.UDisks2.Encrypted`.
  - If present, record container state and discover `CleartextDevice` if unlocked.
  - Build child node(s) by probing the cleartext object for `Filesystem` interface + mount points.
  - Ensure robust handling when cleartext is missing (locked) or transient.
- Test plan:
  - Add unit tests using a mocked backend/proxy layer where feasible; otherwise add “pure” tests around mapping logic.
- Done when:
  - [x] Locked LUKS partitions are identified as containers.
  - [x] Unlocked LUKS partitions produce a cleartext child volume node.

## Task 3: Add unlock/lock operations to the backend
- Scope: Provide DBus calls for unlocking encrypted volumes.
- Files/areas:
  - `storage-dbus/src/disks/ops.rs`
  - New zbus proxy if required (Encrypted)
- Steps:
  - Extend `DiskBackend` with `crypto_unlock(path, passphrase) -> cleartext_path` (and optionally `crypto_lock`).
  - Implement in `RealDiskBackend`.
  - Add public methods on the model layer (e.g. `PartitionModel::unlock(passphrase)` or a free function) that calls into ops.
  - Ensure passphrase is not logged and is not stored in structs.
- Test plan:
  - Extend the mock backend in `ops.rs` tests to record unlock calls.
- Done when:
  - [x] Unlock can be invoked via `storage-dbus` API surface.

## Task 4: UI dialog for passphrase + unlock/mount flow
- Scope: Add a passphrase prompt and wire it to unlock + refresh (container actions are Unlock/Lock only).
- Files/areas:
  - `storage-ui/src/views/volumes.rs`
  - `storage-ui/src/views/dialogs.rs`
  - `storage-ui/src/app.rs` (dialog plumbing)
- Steps:
  - Add a `ShowDialog::UnlockEncrypted` dialog.
  - Add a secure text input for passphrase and confirm/cancel actions.
  - When user requests Unlock on a locked encrypted container:
    - show dialog → call unlock → refresh drives.
  - Add a Lock action on unlocked containers.
  - Ensure there is no container-level Mount action; mounting is per-child filesystem/LV.
  - Show errors on failure (wrong passphrase) and keep the dialog/input safe.
- Test plan:
  - Manual: try locked LUKS; verify dialog, wrong passphrase, success path.
- Done when:
  - [x] UI prompts for passphrase before unlock.
  - [x] Success unlock updates the model and shows child volumes.
  - [x] Lock action is present and functional for unlocked containers.

## Task 5: Render contained filesystems with correct sizing (LUKS direct filesystem)
- Scope: Display nested filesystem volumes beneath the selected container.
- Files/areas:
  - `storage-ui/src/views/volumes.rs`
  - `storage-ui/src/utils/segments.rs` (if reuse/extract sizing helper)
- Steps:
  - When selecting a container partition, render a “child segments row” for `VolumeNode.children`.
  - Use the same sizing math currently used for segments (portion calculation), scoped to the child list.
  - Add mount/unmount actions for child filesystem nodes.
- Test plan:
  - Manual: unlock LUKS and confirm child filesystem row appears and mounts.
- Done when:
  - [x] Inner filesystem(s) are visible and actionable.
  - [x] Child sizing matches the existing segment sizing behavior.
  - [x] Nested UI uses a split layout (container top half, children bottom half) and child nodes are selectable.

## Task 6: LVM discovery + LV rendering (required)
- Scope: Treat LVM PV as a container and enumerate LVs; mount filesystem LVs.
- Steps:
  - Detect `IdType == "LVM2_member"` (or relevant UDisks2 hints) on cleartext block.
  - Enumerate logical volumes via UDisks2 LVM2 interfaces (may require adding local zbus proxies).
  - For each LV:
    - If it has a filesystem interface, expose mount/unmount.
    - Otherwise, display as a non-mountable child (type + size).
- Test plan:
  - Manual with a test VM or loopback LVM setup.
- Done when:
  - [x] LVM PVs list logical volumes.
  - [x] Filesystem LVs can be mounted/unmounted from the child list.

## Task 7: Update copy/i18n and docs
- Scope: User-facing strings and quick docs.
- Files/areas:
  - `storage-ui/i18n/**/cosmic_ext_disks.ftl`
  - `README.md` / `storage-ui/README.md` (if there’s a troubleshooting section)
- Steps:
  - Add strings: “Unlock”, “Passphrase”, “Wrong passphrase”, “Locked”, “Unlocked”.
  - Document that unlock uses UDisks2 and may trigger system auth.
- Test plan:
  - Verify app launches and strings render.
- Done when:
  - [x] New dialogs/actions are localized (at least `en`).
