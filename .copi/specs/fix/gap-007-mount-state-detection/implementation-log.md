# Implementation Log — GAP-007

Timestamp (UTC): 2026-01-24
Branch: `fix/gap-007-mount-state-detection`

## Summary
Replaced `df` parsing with UDisks2 mountpoints for mount state and `statvfs` for usage enrichment. Updated UI mount/unmount button logic to use UDisks2-derived mount state.

## Notable changes
- DBus/model:
  - Added `mount_points: Vec<String>` to `PartitionModel` and derived `is_mounted()` from it.
  - Device path now derived from UDisks2 `Block.PreferredDevice`/`Block.Device`.
  - Usage is computed via `statvfs` on the mount point (best-effort).
  - Removed `df` execution/parsing.
- UI:
  - Mount/unmount button uses `PartitionModel::is_mounted()`.
  - “Mounted at” display uses `PartitionModel.mount_points` even if usage is missing.

## Commands run
- `cargo fmt --all`
- `cargo test -p cosmic-ext-disks-dbus`
- `cargo test --workspace --all-features`
- `cargo clippy --workspace --all-features`

## Files changed (high level)
- `disks-dbus/src/usage.rs`
- `disks-dbus/src/disks/partition.rs`
- `disks-dbus/src/disks/drive.rs`
- `disks-ui/src/views/volumes.rs`
- `disks-ui/src/app.rs`
- `.copi/specs/fix/gap-007-mount-state-detection/*`
- `.copi/spec-index.md`
