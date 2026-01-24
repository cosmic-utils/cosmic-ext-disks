# Implementation Log — fix/partition-segmentation-hacks

- Spec: `.copi/specs/fix/partition-segmentation-hacks/`
- Branch: `fix/partition-segmentation-hacks`
- Gap: `GAP-004`
- Audit: `.copi/audits/2026-01-24T00-37-04Z.md`

## 2026-01-24

### Baseline
- Ran: `cargo fmt --all --check`, `cargo clippy --workspace --all-features`, `cargo test --workspace --all-features`
- Result: Pass (existing clippy warnings in `disks-dbus` are pre-existing).

### Changes
- Implemented a pure segmentation helper that produces a full 0..disk_size coverage model (partitions + free space), with anomaly reporting for overlaps and out-of-range extents.
- Wired the volumes view to use the helper, removing:
  - the “1024KB at the start” offset heuristic
  - the “hide weird end portion” trailing-bytes hack
- Preserved existing UI legibility/scaling: `Segment.width` continues to be derived from the existing log-based `FillPortion` scaling and min-width clamp.

### Commands
- `cargo fmt --all`
- `cargo fmt --all --check`
- `cargo clippy --workspace --all-features`
- `cargo test --workspace --all-features`

### Files changed
- `disks-ui/src/utils/segments.rs`
- `disks-ui/src/utils/mod.rs`
- `disks-ui/src/views/volumes.rs`
- `.copi/specs/fix/partition-segmentation-hacks/tasks.md`

### Notes / Follow-ups
- Manual UI validation on real disks is still needed (Task 4 in `tasks.md`).
- The helper clamps overlapping partitions for ordered rendering and emits anomaly logs via `eprintln!()` in the UI.
