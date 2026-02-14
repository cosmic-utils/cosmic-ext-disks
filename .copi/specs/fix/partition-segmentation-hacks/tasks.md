# fix/partition-segmentation-hacks — Tasks

Source:
- GAP: `GAP-004`
- Audit: `.copi/audits/2026-01-24T00-37-04Z.md`

## Task 1: Baseline current segmentation behavior
- Scope: Understand existing segmentation code and inputs.
- Files/areas:
  - `storage-ui/src/views/volumes.rs`
  - Any related UI utils under `storage-ui/src/utils/`
- Steps:
  - Locate the segment computation code paths referenced in the audit.
  - Document what data is available per partition (start/size/fs type) and per drive (size).
  - Identify all heuristics/magic constants and where they influence output.
  - Document the existing segment width scaling/minimum sizing behavior (the log-based `FillPortion` scaling that clamps to a minimum width).
- Test plan:
  - Build/run locally to confirm current rendering path.
- Done when:
  - [x] Current segment computation, heuristics, and inputs are identified.
  - [x] Existing log-based `FillPortion` width scaling (min width clamp) is documented.

## Task 2: Implement a pure segment computation helper
- Scope: Create a deterministic function that turns disk metadata into segments.
- Files/areas:
  - New helper module in `storage-ui/src/utils/` (or the nearest existing utility module for volumes rendering)
- Steps:
  - Define segment types (partition/free/unknown) and the minimal fields the renderer needs.
  - Implement ordering + gap filling from $0$ to `disk_size`.
  - Define overlap/out-of-range handling (clamp/mark unknown) with logging.
  - Preserve existing UI scaling semantics when porting/refactoring (i.e., do not regress the current minimum-visible sizing of tiny partitions).
  - Ensure the last segment absorbs any remainder due to rounding.
- Test plan:
  - Add unit tests for:
    - Single partition + trailing free
    - Multiple partitions with gaps
    - Unsorted partitions
    - Overlapping partitions
    - Partition end beyond disk size
    - Extremely small partitions remain visible under the existing scaling/min-width behavior
- Done when:
  - [x] Helper exists under `storage-ui/src/utils/` and is pure.
  - [x] Unit tests cover gaps, sorting, overlaps, end-past-disk, and tiny partitions.
  - [x] No `todo!()`/`panic!()` for normal invalid-input cases.

## Task 3: Wire helper into volumes view and remove hacks
- Scope: Replace inline heuristics with helper output.
- Files/areas:
  - `storage-ui/src/views/volumes.rs`
- Steps:
  - Replace current offset heuristic and trailing-hide hack with helper output.
  - Ensure the UI renders free-space segments explicitly.
  - Add debug logs (or tracing) for anomaly cases detected by helper.
- Test plan:
  - `cargo test --workspace --all-features`
  - Manual smoke test: open the app; validate at least one disk with partitions and observe full bar coverage.
- Done when:
  - [x] The audit-referenced offset heuristic and trailing-bytes hiding hack are removed.
  - [x] The bar covers $[0, disk\_size)$ via explicit gap segments.
  - [x] Existing width scaling behavior is preserved.

## Task 4: Manual validation + regression checklist
- Scope: Validate on representative layouts.
- Steps:
  - Validate with:
    - Disk with trailing free space
    - Disk with small alignment gaps
    - Removable media (USB) with a single partition
  - Record screenshots/notes of before/after in PR description (no new repo assets required).
- Test plan:
  - `cargo fmt --all --check`
  - `cargo clippy --workspace --all-features`
- Done when:
  - [ ] Rendering is correct across tested layouts and no panics occur.

## Recommended sequence / dependencies
- Do Task 1 → Task 2 → Task 3 → Task 4.
- Task 2 should land before Task 3 (so UI changes stay small and testable).
