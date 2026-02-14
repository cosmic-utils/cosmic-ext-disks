# fix/gap-007-mount-state-detection — Tasks

Source:
- GAP: `GAP-007`
- Audit: `.copi/audits/2026-01-24T18-03-04Z.md`

## Task 1: Define mount-state contract (DBus layer)
- Scope: Decide what the DBus/model layer exposes for mount state.
- Files/areas: `storage-dbus/src/disks/*` (partition model), UDisks2 property accessors.
- Steps:
  - Identify the UDisks2 fields currently available in the partition model (filesystem interface, mountpoints, etc.).
  - Add a small, explicit representation (e.g., `mounted: bool`, `mountpoints: Vec<String>`).
  - Document edge cases: multiple mountpoints, non-filesystem partitions.
- Test plan:
  - Add/adjust unit tests where feasible for the mapping logic (mocked data structures).
- Done when:
  - [x] A single source-of-truth mount state exists in the DBus/model layer.

## Task 2: Decouple usage (`df`) from mount state
## Task 2: Remove `df` entirely (use UDisks2 mountpoints + `statvfs`)
- Scope: Eliminate external `df` parsing and compute usage locally.
- Files/areas: `storage-dbus/src/usage.rs`, `storage-dbus/src/disks/drive.rs`, `storage-dbus/src/disks/partition.rs`.
- Steps:
  - Replace the current `get_usage_data()` implementation (which shells out to `df`) with a helper that:
    - accepts mountpoint path(s) and returns usage via `libc::statvfs`.
  - Derive mountpoints from UDisks2 (`Filesystem.MountPoints`) and treat non-empty as mounted.
  - Derive device path from UDisks2 (`Block.PreferredDevice`/`Block.Device`) instead of relying on `df`’s filesystem column.
  - Ensure failures in `statvfs` do not affect mount state.
- Test plan:
  - Unit test the mountpoints → mounted mapping.
  - Unit test a pure `statvfs` wrapper behind a small adapter (or test the computation function with injected values).
- Done when:
  - [x] No code path executes `df`.
  - [x] Mount state is independent of usage.

## Task 3: Update UI logic to use mount state
- Scope: Use the new mount-state field for mount/unmount button selection.
- Files/areas: `storage-ui/src/views/volumes.rs` (mount/unmount button logic), related view models.
- Steps:
  - Replace mount-state inference from `usage.is_some()` with `partition.mount_state`.
  - Keep displaying usage when available.
  - Verify mount/unmount label and enabled/disabled state matches UDisks2 mountpoints.
- Test plan:
  - Run `cargo test --workspace --all-features`.
  - Manual check: open UI, compare button state to `udisksctl status` / mounted paths.
- Done when:
  - [x] Mount/unmount buttons reflect UDisks2-reported state.

## Task 4: Regression coverage (minimal)
- Scope: Add at least one regression test that would have failed under the old logic.
- Files/areas: whichever crate has the most test-friendly abstraction.
- Steps:
  - Introduce a small pure function that maps mountpoints → mounted bool.
  - Add unit tests for empty/non-empty mountpoints.
  - Add a “no df” guard test (e.g., ensure usage module no longer calls `Command::new("df")` by removing that code path entirely).
- Test plan:
  - Run tests and ensure the new tests exercise the new behavior.
- Done when:
  - [x] A test exists that anchors the mount-state logic to UDisks2-derived inputs.
