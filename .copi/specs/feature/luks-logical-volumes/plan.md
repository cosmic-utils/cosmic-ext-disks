# Implementation Spec — LUKS + Logical/Nested Volumes

Branch: `feature/luks-logical-volumes`

Source: brief (user request)

## Context
Today we only enumerate/render *partition table entries* (`DriveModel.partitions`). This means:
- Encrypted containers (LUKS) appear as a single “partition” segment.
- Inner block devices created by unlocking (cleartext dm-crypt) and any filesystems/volumes inside are not represented.

In UDisks2 terms, the “outer” partition is typically:
- `org.freedesktop.UDisks2.Partition` + `org.freedesktop.UDisks2.Block`
- *and* (for LUKS) `org.freedesktop.UDisks2.Encrypted`

After unlock, UDisks2 exposes a *different* block object (the cleartext device) that can have:
- `org.freedesktop.UDisks2.Filesystem` (direct filesystem), or
- other stacking (e.g. LVM2 PV → VG → LVs, or other logical layers)

The UI should show the nested structure similar to GNOME Disks (container with inner filesystem(s)), and mounting a locked encrypted volume should prompt for a passphrase.

## Goals
- Represent “container → contained block volumes/filesystems” in the model.
- For LUKS-encrypted partitions:
  - Show locked/unlocked state.
- On unlock, prompt for passphrase.
  - Provide explicit **Unlock** and **Lock** actions on the container.
  - After unlock, show contained filesystem(s) and allow mounting/unmounting them (mount actions are only on the contained filesystems, not on the container).
- Apply the same sizing logic used for top-level volumes/segments when rendering contained volumes (scaled within the container).
- Support LVM now: if the unlocked cleartext device is an LVM PV, enumerate LVs and allow mounting/unmounting any filesystem LVs.
- Keep changes compatible with existing partition segmentation UI (free space/reserved/partition segments).

## Non-Goals
- Creating LUKS containers, changing passphrases, keyslots, or keyfile management.
- Full LVM management UI (create VG/LV, resize LV, etc.).
- Supporting non-UDisks backends.
- Persisting passphrases (always ephemeral; no storage).

## Proposed Approach
### 1) Add a first-class “volume graph” alongside partitions
Keep the existing `DriveModel.partitions` for partition-table visualization and operations.

Add a new representation in `storage-dbus` that can express stacking:
- A `VolumeNode` (or `BlockVolume`) representing a UDisks “block-ish” object
- `children: Vec<VolumeNode>` for contained/derived block devices

Minimum viable kinds:
- **Partition** (existing partition)
- **CryptoContainer** (LUKS): outer block with `Encrypted` interface
- **Filesystem** (cleartext mountable)

Optional/phase-2 kinds:
- **LvmPhysicalVolume** (PV)
- **LvmLogicalVolume** (LV)

The spec should not require the UI to understand every possible stack upfront; it should render “unknown block children” as generic items with size/type.

### 2) Enumerate nested devices via UDisks2 object graph
In `DriveModel::get_drives()` we currently derive:
- drive list from `Manager.GetBlockDevices`
- partitions from `PartitionTable.partitions`

Extend enumeration to also discover “derived” block devices and relate them to partitions:
- For each partition path:
  - Detect if it has `org.freedesktop.UDisks2.Encrypted`.
  - If encrypted:
    - Read `CleartextDevice` (or equivalent) if already unlocked.
    - If cleartext exists, build a `VolumeNode` for it and then detect:
      - filesystem mount points (`Filesystem` interface)
      - further stacking opportunities (future: LVM)

Implementation detail:
- If `udisks2 = 0.3.1` does not expose an `EncryptedProxy`, create a local zbus proxy definition (similar to [storage-dbus/src/disks/manager.rs](storage-dbus/src/disks/manager.rs)) for:
  - interface `org.freedesktop.UDisks2.Encrypted`
  - properties: `CleartextDevice`
  - methods: `Unlock(passphrase, options) -> cleartext_object_path`, `Lock(options)`

### 3) Add unlock/mount operations in the DBus layer
Add operations to `storage-dbus` that the UI can call:
- `unlock_luks(block_path, passphrase) -> cleartext_path`
- `lock_luks(block_path)`
- `mount_filesystem(block_path)` remains as-is, but should be usable for the cleartext device.

This likely fits by extending the `DiskBackend` trait in [storage-dbus/src/disks/ops.rs](storage-dbus/src/disks/ops.rs) with crypto operations, and implementing them in `RealDiskBackend`.

### 4) UI: nested volume rendering + passphrase prompt
Add a nested rendering section to the volumes view:
- When a partition segment is selected:
  - If it is an encrypted container:
    - show a “Locked” state + an **Unlock** action.
    - on action: open a dialog prompting for passphrase.
    - on confirm: call unlock → refresh drives.
    - show a “Lock” action while unlocked.
  - If it is unlocked and has a cleartext filesystem:
    - render contained filesystem as a child volume entry/mini-segment row.

Sizing rule:
- Reuse the segment width/portion computation already used in [storage-ui/src/views/volumes.rs](storage-ui/src/views/volumes.rs), but apply it to the set of children volumes, with the denominator being the *container size* (or the sum of child sizes, if that’s what current UX expects).

Dialog:
- Introduce a new `ShowDialog` variant such as `UnlockEncrypted { partition_name, device_hint, passphrase }`.
- Never store the passphrase beyond the action; clear the input after the task completes.

### 5) LVM support (required)
LVM is treated as a first-class nested container type in this work.

If the unlocked cleartext device (or any nested block device) indicates an LVM PV (commonly `IdType == "LVM2_member"`), enumerate logical volumes and surface them as children.

Implementation detail:
- Prefer native UDisks2 LVM2 interfaces if available in the `udisks2` crate.
- Otherwise, add local zbus proxies for the needed UDisks2 LVM2 objects (exact interface names/properties to be verified against UDisks2 docs and the `udisks2` crate API surface).

UX requirement:
- Container-level actions are Unlock/Lock only.
- Mount actions exist on mountable children (filesystem, or filesystem-formatted logical volume).

## User/System Flows
### Flow A — View encrypted partition with nested filesystem
1. User selects a disk.
2. UI shows partition segments including a “LUKS” partition.
3. Selecting that segment shows it as a container and, if unlocked, lists contained filesystem(s).

### Flow B — Mount locked encrypted volume
1. User selects LUKS partition and clicks Unlock.
2. UI prompts for passphrase.
3. UI calls unlock; on success refreshes model.
4. UI shows contained filesystem(s) (and/or LVM logical volumes).
5. User mounts a contained filesystem or filesystem-formatted LV.

### Flow C — Wrong passphrase
1. User enters wrong passphrase.
2. Unlock call fails.
3. UI shows a non-destructive error and keeps the volume locked.

## Risks & Mitigations
- **UDisks2 interface coverage in `udisks2` crate:** mitigate by adding a small local zbus proxy for `Encrypted`.
- **Object graph race / refresh timing after unlock:** mitigate by refreshing drives after unlock and tolerating transient missing objects with retries/backoff (bounded).
- **Security:** ensure passphrases aren’t logged, stored, or reused; clear UI state after action.
- **Multiple inner volumes:** design UI to handle 0..N children (don’t assume 1 filesystem).

## Acceptance Criteria
- [x] A LUKS-encrypted partition displays as a container volume, not just a flat partition.
- [x] If locked, clicking Unlock prompts for passphrase.
- [x] On successful unlock, the UI shows contained filesystem(s) beneath the container.
- [x] Contained filesystem(s) can be mounted/unmounted and show mount points/usage like normal.
- [x] The container provides a Lock action while unlocked.
- [x] Child sizing/visualization uses the same sizing logic as top-level volumes, scoped to the container.
- [x] Wrong passphrase surfaces an error and does not change the mounted/unlocked state.
- [x] No passphrase is persisted or written to logs.
- [x] If the unlocked device is an LVM PV, logical volumes are listed; mountable LVs support mount/unmount.

## Implementation Notes
- Verified with `cargo clippy --workspace --all-features -- -D warnings` and `cargo test --workspace --all-features`.
- Manual runtime validation on real LUKS/LVM setups is still recommended (device-path mapping can vary across distros).
- Final UI polish:
  - Volumes bar and nested child rows are ~30% taller for readability.
  - The nested section is split 50/50: container summary on top, contained filesystems/volumes in the bottom half.
  - Child filesystem volumes are selectable; the details panel reflects the selected filesystem.
  - Cleartext filesystem nodes are labeled as “Filesystem” when no label is available.
