# Implementation Log — GAP-008 (fix/device-change-detection-signals)

## 2026-01-24

### Implemented
- Added a signal-based device event stream backed by UDisks2 ObjectManager signals:
  - `InterfacesAdded` / `InterfacesRemoved`, filtered to `org.freedesktop.UDisks2.Block`
  - Kept the existing polling stream for fallback.
- Updated the UI subscription to prefer signals and fall back to slower polling (10s) if signals can’t be subscribed.
- Documented the behavior in the top-level README.

### Notable decisions
- Filtered ObjectManager signals to the `org.freedesktop.UDisks2.Block` interface to mirror the previous polling behavior which tracked `Manager.GetBlockDevices()`.
- Kept task cancellation semantics simple: background task exits when the receiver drops (send fails).

### Build/tooling fix (required to satisfy CI gates)
- Resolved a build-script dependency mismatch between `vergen` and `vergen-git2` that prevented `cargo clippy --all-features` from compiling.
- Switched the build dependency to `vergen` v8 with `git` + `git2` features and updated `disks-ui/build.rs` accordingly.

### Commands run
- `cargo fmt --all --check`
- `cargo clippy --workspace --all-features`
- `cargo test --workspace --all-features`
