# GAP-004 — Partition segmentation correctness

Branch: `fix/partition-segmentation-hacks`

Source:
- GAP: `GAP-004`
- Audit: `.copi/audits/2026-01-24T00-37-04Z.md`

## Context
The volumes view renders a partition “segmentation”/bar that is used to visually represent disk layout and (implicitly) safe/unsafe areas for operations.

The audit identified that the current segmentation logic in the UI relies on heuristics and hard-coded hiding of trailing bytes ("Hide weird end portion"), plus offset heuristics. This can misrepresent actual disk layout and free space, increasing the risk of user mistakes for destructive operations.

## Goals
- Make the partition segmentation/segment bar represent the full device size accurately.
- Remove hard-coded hiding/magic constants and offset heuristics from the segment computation.
- Ensure gaps/free space are represented explicitly (including trailing free space).
- Handle irregular layouts defensively (overlaps, out-of-order partitions, missing metadata) without panicking.
- Preserve current UI legibility behavior: keep the existing log-based segment scaling and its minimum-visible width guarantee.

## Non-Goals
- Implementing new partition operations (create/resize/move) beyond what exists today.
- Changing the underlying UDisks2 data model/proxy shape (unless required to expose already-available metadata).
- Redesigning the entire volumes UI; focus is segment computation + presentation correctness.

## Proposed Approach
- Identify the current segment computation in `storage-ui/src/views/volumes.rs` (around the audit references) and map inputs used:
  - Total device size used for normalization
  - Partition start offsets and sizes
  - Any special-case offset or "hidden end" handling
  - Existing UI scaling logic for segment widths (log-based `FillPortion`, with a minimum width of 1)
- Define invariants for segment computation:
  - Segments cover $[0, disk_size)$ completely, partition segments plus free-space segments.
  - No segment has negative length; lengths sum to disk size (modulo safe rounding strategy).
  - Partitions are ordered by start; overlaps are detected and handled (e.g., clamp, merge, or show an “unknown/overlap” segment) without panicking.
- Keep “correctness” (offset/size math) separate from “presentation” (segment width scaling), but reuse the repo’s existing scaling approach rather than inventing a new one.
- Implement a pure helper (in UI utils or a small module) to compute segments from a list of partitions:
  - Input: disk size, list of (start, size, kind)
  - Output: ordered list of display segments (partition/free/unknown)
  - Ensure rounding rules are explicit (bytes vs sectors) and consistent.
- Update the volumes view to consume the helper output and render segments without the “hide end portion” hack.
- Add debug-level logging/tracing for anomalous cases (overlap, out-of-range partition end, unsorted input), so issues can be diagnosed without user harm.

## User/System Flows
- User opens the Volumes view → selects a drive → sees a partition bar that matches actual partition table layout.
- User sees free space segments (including at the end of the disk) instead of missing/hidden space.
- If the disk reports odd metadata (overlap, truncated size), UI still renders a best-effort bar and surfaces a non-fatal warning (log + optional UI label if there is an existing pattern for it).

## Risks & Mitigations
- **Risk:** UDisks2 metadata may be incomplete or in sectors while UI assumes bytes.
  - Mitigation: normalize units at the boundary; document assumptions; add asserts/logs for unit mismatches.
- **Risk:** Rounding for UI widths can cause off-by-one visual artifacts.
  - Mitigation: keep computation in bytes; defer rounding to rendering; ensure the last segment absorbs remainder.
- **Risk:** Existing behavior may have been masking a real kernel/driver reporting quirk.
  - Mitigation: replace hiding with explicit “unknown/trailing metadata” segment + logging.

## Acceptance Criteria
- Segment computation represents the entire disk size; no hard-coded hiding of trailing bytes.
- Partition start offsets and sizes are used authoritatively; no offset heuristics.
- Gaps between partitions (including end-of-disk) render as free space segments.
- Odd/invalid inputs (out-of-order partitions, overlaps, out-of-range ends) do not panic and are observable via logs.
- Segment scaling remains legible: the current log-based width scaling behavior is preserved, and tiny partitions continue to render with a non-zero minimum width.
- CI gates remain clean: `cargo fmt --all --check`, `cargo clippy --workspace --all-features`, `cargo test --workspace --all-features`.
