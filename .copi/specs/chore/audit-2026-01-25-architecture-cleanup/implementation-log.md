# Implementation Log — chore/audit-2026-01-25-architecture-cleanup

## 2026-01-26

- Implemented **Task 1** (UI module skeleton).
- Added initial `disks-ui/src/ui/` module tree and wired it into `disks-ui/src/main.rs`.
- Commands run:
  - `cargo fmt --all`
  - `cargo clippy --workspace --all-features`
  - `cargo test --workspace --all-features`

- Started **Task 3** by extracting `VolumesControlMessage` + conversion impls into `disks-ui/src/ui/volumes/message.rs`.
- Kept `disks-ui/src/views/volumes.rs` as a compatibility layer via re-export.
- Commands run:
  - `cargo fmt --all`
  - `cargo clippy --workspace --all-features`
  - `cargo test --workspace --all-features`

- Continued **Task 3** by extracting `VolumesControl`/`Segment`/`ToggleState` into `disks-ui/src/ui/volumes/state.rs`.
- Kept `disks-ui/src/views/volumes.rs` as a compatibility layer via re-export.
- Commands run:
  - `cargo fmt --all`
  - `cargo clippy --workspace --all-features`
  - `cargo test --workspace --all-features`

- Continued **Task 3** by extracting shared volumes helpers into `disks-ui/src/ui/volumes/helpers.rs`.
- Moved partition-type selection helpers + volume tree search helpers out of `disks-ui/src/views/volumes.rs`.
- Commands run:
  - `cargo fmt --all`
  - `cargo clippy --workspace --all-features`
  - `cargo test --workspace --all-features`
- Result:
  - Clippy clean; all tests passing.

- Implemented **Task 2** (dialogs state/messages moved under `ui/dialogs/`).
- Notable changes:
  - Added `disks-ui/src/ui/dialogs/state.rs` and `disks-ui/src/ui/dialogs/message.rs`.
  - Removed dialog type definitions from `disks-ui/src/app.rs` (now re-exported from `ui::dialogs`).
  - Dialog views no longer import message enums from `views/volumes.rs`.
- Commands run:
  - `cargo fmt --all`
  - `cargo clippy --workspace --all-features`
  - `cargo test --workspace --all-features`
