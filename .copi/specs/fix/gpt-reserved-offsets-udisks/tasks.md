# fix/gpt-reserved-offsets-udisks — Tasks

Source: `.copi/audits/2026-01-24T00-37-04Z.md` (GAP-004 follow-up)

## Task 1: Confirm UDisks capabilities for usable-range metadata
- Scope: Document the confirmed UDisks surface area and lock in the decided data sources.
- Areas:
  - UDisks DBus introspection for `org.freedesktop.UDisks2.Block` / `PartitionTable` / drive objects
  - Local verification commands (manual): `busctl introspect`, `udisksctl info`, `lsblk -o +PHY-SEC,LOG-SEC`
- Steps:
  - Identify which object path corresponds to the drive block device (already tracked as `DriveModel.block_path`).
  - Record that DBus does **not** provide sector sizes nor GPT first/last usable LBAs.
  - Record that DBus **does** provide disk size, partition offsets/sizes (bytes), table type, and `OpenDevice`.
  - Lock decision:
    - logical sector size: ioctl `BLKSSZGET` (fallback sysfs `queue/logical_block_size`)
    - GPT usable range: parse GPT header via `Block.OpenDevice` FD
- Test plan:
  - Manual: run introspection on at least one GPT disk.
- Done when:
  - [x] Plan reflects the decision and cites the verification method (doc + introspection).

## Task 2: Add a GPT usable-range probe (UDisks OpenDevice + ioctl + header parse)
- Scope: Implement a small helper to derive `{writable_start_bytes, writable_end_bytes}` for GPT disks.
- Likely areas:
  - `storage-dbus/src/disks/drive.rs` or a new `storage-dbus/src/disks/gpt.rs` helper module
  - Potentially reuse `BlockProxy::open_device` for authorized FD access
- Steps:
  - Open the block device via `OpenDevice("r", …)` with `auth.no_user_interaction=true`.
  - Query logical sector size via ioctl `BLKSSZGET`.
  - If ioctl is unavailable, fall back to sysfs `queue/logical_block_size`.
  - Read GPT header at LBA 1 and parse first/last usable LBA.
  - Convert to byte offsets and return a half-open usable range.
  - Emit trace logs for parse failures or suspicious values.
- Test plan:
  - Unit: feed known-good GPT header bytes (fixture) and validate parsed LBAs.
  - Integration/manual: run on a GPT disk and print the computed range in debug logs.
- Done when:
  - [x] GPT usable range is available in the model layer for GPT drives.
  - [x] Errors are non-fatal and logged.

## Task 3: Thread usable-range info into the UI model
- Scope: Ensure the UI has access to the GPT usable byte range for segmentation and create-partition.
- Likely areas:
  - `storage-dbus/src/disks/drive.rs` / `DriveModel`
  - `storage-ui/src/views/volumes.rs`
- Steps:
  - Add fields to `DriveModel` (e.g., `usable_range: Option<(u64,u64)>`), populated only for GPT.
  - Update any serialization/messaging path between dbus layer and UI as needed.
- Test plan:
  - `cargo test --workspace --all-features`
- Done when:
  - [x] UI can read usable-range fields for GPT disks.

## Task 4: Update segmentation to mark reserved regions as non-free
- Scope: Ensure reserved GPT regions aren’t shown/treated as free space.
- Likely areas:
  - `storage-ui/src/utils/segments.rs`
  - `storage-ui/src/views/volumes.rs`
- Steps:
  - Add a segment kind for “Reserved/Unwritable” (or equivalent).
  - When usable range exists, split the disk into reserved/usable/reserved.
  - Ensure only free-space segments inside usable are actionable.
  - Add a Volumes UI toggle (“Show reserved”) to hide reserved and sub-alignment free gaps by default.
  - Treat the toggle as a Volumes-level preference; when switching drives via the nav bar, inherit the previous tab’s value.
  - Add anomaly reporting if any partition lies outside usable range.
- Test plan:
  - Unit: segmentation helper with sample partitions + usable range.
  - Manual: verify the UI no longer offers reserved gaps for creation.
- Done when:
  - [x] GPT reserved regions are not actionable free space.
  - [x] Reserved/tiny segments are hidden by default and can be shown via a Volumes toggle.

## Task 5: Create-partition offset handling within usable range (delegate-first)
- Scope: Ensure create-partition never targets reserved GPT areas.
- Likely areas:
  - `storage-ui/src/views/volumes.rs` (offset selection)
  - `storage-dbus/src/disks/drive.rs` (final validation/clamping before DBus call)
- Steps:
  - Decide whether UDisks supports auto placement (offset=0) on target systems.
  - If not, clamp and align the chosen offset within usable range before calling `CreatePartition`.
  - Improve error mapping so any remaining backend constraint failures are presented clearly.
- Test plan:
  - Manual: attempt creating a partition near the start/end of disk; confirm no invalid-offset errors.
  - CI: fmt/clippy/test.
- Done when:
  - [x] No UDisks invalid-offset errors from UI in normal GPT creation flows.

## Task 6: Manual validation matrix
- Scope: Validate behavior on real disks.
- Steps:
  - Test GPT + 512B logical sector.
  - Test GPT + 4K logical sector (if available).
  - Test drives with existing first partition starting at 1 MiB and with unusual layouts.
- Done when:
  - [ ] Recorded observations and any anomalies to follow up.
