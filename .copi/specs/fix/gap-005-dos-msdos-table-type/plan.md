# Implementation Spec — GAP-005 (MBR/DOS table type mismatch)

Branch: `fix/gap-005-dos-msdos-table-type`

Source:
- Audit: `.copi/audits/2026-01-24T18-03-04Z.md`
- Gap: `GAP-005 — MBR/DOS partition creation likely broken (table type mismatch)`

## Context
The DBus layer’s partition creation path appears to treat `"msdos"` as the only valid MBR/DOS partition-table identifier, while the partition type catalog and helper logic use `"dos"`. If UDisks2 reports `"dos"` (common), the create-partition code likely rejects the table type and/or fails a compatibility check.

This can make MBR/DOS disks effectively unusable for partition creation (or lead to confusing “unsupported table type” errors), even when the UI offers DOS-compatible partition types.

UDisks2’s D-Bus API reports the partition table scheme via `org.freedesktop.UDisks2.PartitionTable.Type`, where known values include `dos` and `gpt`. Partition creation is done via:

- `CreatePartition(offset_bytes, size_bytes, type, name, options) -> created_partition`
- `CreatePartitionAndFormat(offset_bytes, size_bytes, type, name, options, format_type, format_options) -> created_partition`

For DOS/MBR tables, the `partition-type` option controls primary/extended/logical.

## Goals
- Make MBR/DOS partition creation work end-to-end when the underlying disk uses a DOS/MBR partition table.
- Remove `msdos` usage and make table-type handling consistent across UI + DBus layer using the UDisks2-correct value (`dos`).
- Keep behavior for GPT disks unchanged.

## Non-Goals
- Implement the unrelated stubbed partition operations (tracked as GAP-009).
- Add new disk backends beyond UDisks2.

## Proposed Approach
- Standardize on UDisks2-reported partition table type values (`dos`, `gpt`) across the repo.
  - Replace `"msdos"` comparisons/branches with `"dos"`.
  - Ensure any UI logic that branches on table type also expects `dos`.
- Stop relying on `PartitionTable.Partitions` ordering to find the new partition.
  - Prefer `CreatePartitionAndFormat` to avoid “created partition object not yet present” races.
  - If separate steps are retained, capture `created_partition` from `CreatePartition` and format that returned object path (optionally with a short, bounded wait for object readiness).
- For DOS/MBR tables, explicitly set the `partition-type` option (string) to control primary/extended/logical; default to `primary` unless/until UI supports selecting otherwise.
- Add unit tests around the selection/compatibility logic; add a targeted test (or mocked contract test) that asserts we use `dos` and consume the returned `created_partition` path.

Likely touched areas:
- `disks-dbus/src/disks/drive.rs` (create partition branching + validation)
- `disks-dbus/src/partition_type.rs` (valid names helper and/or catalog table_type)
- Potentially UI wiring only if it depends on the raw reported table type

## User/System Flows
### Create partition on an MBR/DOS disk
1. User selects a disk that reports partition table type `dos`.
2. User opens Create Partition dialog and chooses a DOS-compatible partition type.
3. System creates the partition successfully via UDisks2.
4. UI refresh shows the newly created partition.

### Create partition on unsupported/unknown table type
1. If the table type is missing/unknown, UI disables creation or the backend returns a clear error.

## Risks & Mitigations
- Risk: Unexpected table type values (blank/unknown) still occur.
  - Mitigation: error clearly and/or disable create in UI when type is missing/unknown.
- Risk: Partition creation/formatting can race device node/udisks object appearance, causing transient “Object does not exist” failures.
  - Mitigation: use `CreatePartitionAndFormat` where possible; otherwise format the returned `created_partition` object path with a short, bounded wait/retry for readiness.
- Risk: UI offset/size units mismatch (bytes vs sectors) could lead to confusing segmentation anomaly warnings and/or incorrect create requests.
  - Mitigation: verify offset/size are treated as bytes end-to-end (UDisks2 expects bytes) and add assertions/logging in debug builds.

## Acceptance Criteria
- [ ] On DOS/MBR disks, partition creation is either disabled with a clear explanation, or it succeeds end-to-end.
- [ ] No `msdos` string comparisons remain on the create-partition critical path (use `dos`).
- [ ] Partition creation uses the `created_partition` returned by UDisks2 (or `CreatePartitionAndFormat`) and does not depend on `Partitions().last()` ordering.
- [ ] Unit tests cover the create-partition table-type selection logic.
