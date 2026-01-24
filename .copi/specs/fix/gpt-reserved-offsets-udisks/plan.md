# GAP-004 follow-up — GPT reserved offsets & alignment (UDisks-first)

Branch: `fix/gpt-reserved-offsets-udisks`

Source:
- GAP: `GAP-004`
- Audit: `.copi/audits/2026-01-24T00-37-04Z.md`
- Prior spec (context): `.copi/specs/fix/partition-segmentation-hacks/`

## Context
After the GAP-004 segmentation correctness work, the UI now represents the full disk size and shows gaps as “free space”. On GPT disks, some of those gaps are not actually writable by the partitioning backend (UDisks/libblockdev/parted), because GPT consumes space at both ends (primary header/PEA at the start; backup PEA/header at the end).

Today this can lead to a bad UX:
- The UI offers “free space” regions that look valid.
- Creating a partition in these regions fails at the UDisks layer.

This follow-up focuses specifically on: identifying GPT reserved/unwritable ranges using UDisks-provided information wherever possible, and ensuring those ranges are not presented as actionable free space.

## Goals
- For GPT disks only, compute an authoritative writable byte range (start/end) and treat outside-of-range regions as non-free (reserved/unwritable).
- Prefer UDisks-sourced data; only fall back to minimal self-derived logic when UDisks does not expose the needed facts.
- Ensure create-partition requests never target reserved GPT regions.
- Maintain anomaly reporting/logging for unusual metadata (e.g., partitions outside the writable region).

## Non-Goals
- Supporting MBR/DOS partition tables (explicitly deferred to the next PR).
- Replacing UDisks’ backend behavior or re-implementing full partition layout policy.
- Changing the UI’s overall design; scope is segment/action correctness.

## Proposed Approach
### 1) What we can (and can’t) get from UDisks (UDisks-first)
Based on UDisks2 DBus/API docs and local DBus introspection:

What UDisks gives us directly (use these as the primary source of truth):
- Partition table type via `org.freedesktop.UDisks2.PartitionTable.Type` (e.g. `gpt`).
- The set of partitions via `org.freedesktop.UDisks2.PartitionTable.Partitions`.
- Partition extents in **bytes** via `org.freedesktop.UDisks2.Partition.Offset` and `.Size`.
- Disk size in bytes via `org.freedesktop.UDisks2.Block.Size`.
- An authorized way to read the block device via `org.freedesktop.UDisks2.Block.OpenDevice`.

What UDisks does *not* expose via DBus (so we must source elsewhere):
- Logical / physical sector size.
- GPT “first usable LBA” / “last usable LBA” (or equivalent usable-range boundaries).
- A stable “recommended alignment boundary” value.

Conclusion:
- We can render partitions accurately from UDisks data, but we cannot determine GPT reserved/unwritable regions *purely* from DBus properties.
- We therefore need a minimal, read-only probe for GPT usable LBAs.

### 2) Derive usable range via GPT header read (minimal “own logic”)
UDisks’ `org.freedesktop.UDisks2.Block.OpenDevice` provides an FD with appropriate authorization semantics; we should use it for reads instead of opening `/dev/*` directly.

To avoid an elevation prompt during browsing/segmentation, request `OpenDevice` with no user interaction. If authorization would be required, this read-only probe should fail and we fall back to conservative reserved bands.

Using that FD:
- Determine logical sector size via ioctl (e.g. `BLKSSZGET`).
  - Fallback (if ioctl unavailable): read sysfs `queue/logical_block_size` for the underlying device.
- Read the GPT primary header from LBA 1.
  - Validate signature `"EFI PART"` and header size.
  - Parse (little-endian):
    - `first_usable_lba`
    - `last_usable_lba`
- Convert LBAs → byte range using the logical sector size:
  - `writable_start_bytes = first_usable_lba * logical_sector_size`
  - `writable_end_bytes = (last_usable_lba + 1) * logical_sector_size` (treat as half-open `[start, end)`)

This keeps policy minimal and authoritative: GPT itself declares what is usable; we only translate to bytes.

Failure handling policy:
- If we cannot open/read/parse GPT header for a disk reported as `gpt`, do not crash.
- Prefer a conservative fallback that avoids creating partitions in the typical reserved bands:
  - Mark `[0, 1MiB)` and `[disk_size - 1MiB, disk_size)` as reserved and log an anomaly.
  - This is intentionally conservative and should only apply when GPT parsing is unavailable.

### 3) Represent reserved ranges in segmentation
- Extend the segmentation model to support non-free “reserved/unwritable” segments for GPT.
- For GPT drives, produce a layout model:
  - Reserved: `[0, writable_start_bytes)`
  - Usable: `[writable_start_bytes, writable_end_bytes)` (partition + free-space inside)
  - Reserved: `[writable_end_bytes, disk_size)`
- Ensure only the “free space inside usable range” is actionable for create partition.

### 4) Create-partition offset policy: delegate where possible
- Prefer letting the backend choose alignment when possible.
  - Validate whether `PartitionTable.CreatePartition(offset=0, …)` is supported as “auto place” on target distros; if not, don’t rely on it.
- If explicit offsets remain required:
  - Clamp the user-selected start to `writable_start_bytes`.
  - Apply the existing 1 MiB alignment policy only within the usable range.
  - Ensure `offset + size <= writable_end_bytes`.

## User/System Flows
- User selects a GPT drive in Volumes view → sees “reserved” regions at the start/end (or at minimum, does not see them as “free space”).
- User selects a free-space segment → create-partition uses an offset within GPT usable range and succeeds (subject to standard constraints).
- If drive metadata is inconsistent (partitions outside GPT usable LBAs), the UI still renders best-effort segments and logs an anomaly.

## Risks & Mitigations
- Risk: Parsing GPT requires careful byte-level work.
  - Mitigation: keep parsing small/surgical (just header), add unit tests with fixture bytes.
- Risk: Authorization / sandbox issues reading block devices.
  - Mitigation: use UDisks `OpenDevice` rather than direct `/dev/*` opens.
- Risk: Some devices report odd sector sizes or hybrid tables.
  - Mitigation: treat GPT parsing failure as “no reserved info” and fall back to current behavior + anomaly log; do not block UI.

## Acceptance Criteria
- [x] Applies only when `partition_table_type == "gpt"`.
- [x] The UI does not present GPT reserved regions as actionable “free space”.
- [x] Create-partition never targets offsets outside the GPT writable range; UDisks errors about invalid offsets are eliminated in normal flows.
- [x] If GPT usable range cannot be determined, the UI falls back safely (no crash) and logs a debug/trace anomaly.
- [x] CI gates remain clean: `cargo fmt --all --check`, `cargo clippy --workspace --all-features`, `cargo test --workspace --all-features`.
