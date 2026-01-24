# Implementation Log — GAP-005 (fix/gap-005-dos-msdos-table-type)

## 2026-01-24

- Baseline checks:
  - `cargo fmt --all --check`
  - `cargo test --workspace --all-features`
  - `cargo clippy --workspace --all-features`
- Key decisions:
  - Standardize on UDisks2 `PartitionTable.Type` values (`dos`, `gpt`) and remove `msdos` branches.
  - Use UDisks2 `CreatePartitionAndFormat` so the daemon returns the created object path and performs formatting in one job, avoiding races and avoiding reliance on `PartitionTable.Partitions().last()`.
  - For DOS/MBR, set `partition-type=primary` in create options (until UI supports extended/logical selection).
- Files changed:
  - `disks-dbus/src/disks/drive.rs`
  - `.copi/specs/fix/gap-005-dos-msdos-table-type/{plan.md,tasks.md}`
  - `.copi/architecture.md`
  - `.copi/spec-index.md`

## 2026-01-24 (follow-up)

- Implemented DOS/MBR reserved start region + max-size semantics:
  - UI treats DOS usable range as `[1MiB, disk_size)` so the first 1MiB is reserved and not offered as actionable free space.
  - Backend rejects DOS create requests with `offset < 1MiB`.
  - When a “fill / max size” request is detected, backend passes `size=0` to UDisks2 so it can apply alignment/geometry.
- Checks run:
  - `cargo fmt --all --check`
  - `cargo test --workspace --all-features`
  - `cargo clippy --workspace --all-features`
- Files changed:
  - `disks-ui/src/views/volumes.rs`
  - `disks-ui/src/utils/segments.rs`
  - `disks-dbus/src/disks/drive.rs`
  - `.copi/specs/fix/gap-005-dos-msdos-table-type/{plan.md,tasks.md}`

## Manual validation (pending)

- Create NTFS partition on an MBR/DOS disk.
- Confirm no “Unsupported partition table type: dos”.
- Confirm no transient DBus errors like “Object does not exist at path …/block_devices/sdX1”.
- If “partition segmentation anomaly” warnings appear, capture the offsets/sizes to verify unit consistency.
