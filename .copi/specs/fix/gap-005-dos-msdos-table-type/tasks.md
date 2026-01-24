# fix/gap-005-dos-msdos-table-type — Tasks

Source:
- `.copi/audits/2026-01-24T18-03-04Z.md` (GAP-005)

## Task 1: Standardize partition table type to `dos` (remove `msdos`)
- Scope: Use the UDisks2-correct `PartitionTable.Type` value (`dos`) consistently across the repo.
- Files/areas:
  - `disks-dbus/src/disks/drive.rs`
  - Any UI branches on table type
  - `.copi/architecture.md` (docs)
- Steps:
  - Replace `"msdos"` comparisons/branches with `"dos"`.
  - Update docs mentioning `msdos` to `dos`.
  - Add/update unit tests to assert DOS path is keyed off `dos`.
- Test plan:
  - `cargo test -p disks-dbus`
- Done when:
  - [x] No `msdos` comparisons remain on the create-partition critical path.

## Task 2: Fix create-partition call for DOS/MBR (UDisks2 contract)
- Scope: Ensure `create_partition` accepts UDisks2-reported DOS table types and calls UDisks2 with the expected arguments.
- Files/areas:
  - `disks-dbus/src/disks/drive.rs`
- Steps:
  - Match the drive-reported table type against `dos` (the UDisks2 `PartitionTable.Type` value).
  - Ensure `partition_info.table_type` comparisons are consistent with `dos`.
  - Ensure the error message for unsupported/unknown types is clear and includes the raw + normalized type.
  - Ensure offset/size are treated as bytes end-to-end (UDisks2 expects bytes).
  - Stop relying on `Partitions().last()` to find the created partition.
    - Prefer `CreatePartitionAndFormat` to reduce race conditions.
    - If keeping separate steps, capture the `created_partition` object path from `CreatePartition` and format that path.
  - For DOS/MBR, set the `partition-type` option (string) to `primary` (unless/until UI supports extended/logical selection).
- Test plan:
  - `cargo test -p disks-dbus`
  - Add/adjust unit tests around the logic (if feasible via isolated functions).
- Done when:
  - [x] DOS/MBR table types no longer fail due to `dos/msdos` mismatches.
  - [x] No transient failures caused by depending on `Partitions().last()` ordering.

Status: Implemented in code; manual validation pending.

## Task 3: Align partition type catalog and helpers with canonical values
- Scope: Ensure the partition type catalog and helper functions use the canonical table type, so UI/DBus comparisons remain consistent.
- Files/areas:
  - `disks-dbus/src/partition_type.rs`
- Steps:
  - Verify DOS catalog entries and `valid_names` logic are consistent with canonical `dos`.
  - Ensure any `msdos` appearance is intentional or removed.
- Test plan:
  - `cargo test -p disks-dbus`
- Done when:
  - [x] No critical-path comparisons rely on a non-canonical alias.

## Task 4: Manual validation checklist (non-destructive where possible)
- Scope: Confirm the fix works in a real environment.
- Steps:
  - On a test VM or removable drive with an MBR/DOS partition table, attempt to create an NTFS partition.
  - Confirm UI updates and there is no “Unsupported partition table type: dos”.
  - Confirm there are no transient DBus errors like “Object does not exist at path …/block_devices/sdX1”.
  - Note whether any “partition segmentation anomaly” warnings appear; if they do, capture the values to validate byte-units end-to-end.
- Done when:
  - [ ] Manual test on at least one DOS/MBR disk succeeds (or UI disables with a clear message if unsupported by design).
