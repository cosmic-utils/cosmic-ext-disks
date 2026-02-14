# Implementation Log — chore/audit-2026-01-25-architecture-cleanup

## 2026-01-26

- Implemented **Task 1** (UI module skeleton).
- Added initial `storage-ui/src/ui/` module tree and wired it into `storage-ui/src/main.rs`.
- Commands run:
  - `cargo fmt --all`
  - `cargo clippy --workspace --all-features`
  - `cargo test --workspace --all-features`

- Started **Task 3** by extracting `VolumesControlMessage` + conversion impls into `storage-ui/src/ui/volumes/message.rs`.
- Kept `storage-ui/src/views/volumes.rs` as a compatibility layer via re-export.
- Commands run:
  - `cargo fmt --all`
  - `cargo clippy --workspace --all-features`
  - `cargo test --workspace --all-features`

- Continued **Task 3** by extracting `VolumesControl`/`Segment`/`ToggleState` into `storage-ui/src/ui/volumes/state.rs`.
- Kept `storage-ui/src/views/volumes.rs` as a compatibility layer via re-export.
- Commands run:
  - `cargo fmt --all`
  - `cargo clippy --workspace --all-features`
  - `cargo test --workspace --all-features`

- Continued **Task 3** by extracting shared volumes helpers into `storage-ui/src/ui/volumes/helpers.rs`.
- Moved partition-type selection helpers + volume tree search helpers out of `storage-ui/src/views/volumes.rs`.
- Commands run:
  - `cargo fmt --all`
  - `cargo clippy --workspace --all-features`
  - `cargo test --workspace --all-features`
- Result:
  - Clippy clean; all tests passing.

- Implemented **Task 2** (dialogs state/messages moved under `ui/dialogs/`).
- Notable changes:
  - Added `storage-ui/src/ui/dialogs/state.rs` and `storage-ui/src/ui/dialogs/message.rs`.
  - Removed dialog type definitions from `storage-ui/src/app.rs` (now re-exported from `ui::dialogs`).
  - Dialog views no longer import message enums from `views/volumes.rs`.
- Commands run:
  - `cargo fmt --all`
  - `cargo clippy --workspace --all-features`
  - `cargo test --workspace --all-features`

- Continued **Task 3** by extracting `VolumesControl::update` into `storage-ui/src/ui/volumes/update.rs`.
- Kept `storage-ui/src/views/volumes.rs` responsible for rendering, but removed the legacy `update()` implementation to avoid duplication.
- Commands run:
  - `cargo fmt --all`
  - `cargo clippy --workspace --all-features -- -D warnings`
  - `cargo test --workspace --all-features`

- Implemented **Task 12** (logging layering cleanup).
- Notable changes:
  - Initialized `tracing_subscriber::fmt().init()` in `storage-ui/src/main.rs`.
  - Replaced UI `println!/eprintln!` calls with `tracing::warn!/error!` in update + subscription paths.
  - Added missing `tracing` dependency to `storage-ui/Cargo.toml`.
- Commands run:
  - `cargo fmt --all`
  - `cargo clippy --workspace --all-features -- -D warnings`
  - `cargo test --workspace --all-features`

## 2026-01-29

- Implemented **Task 13** (move dialog rendering under `ui/dialogs/view/*`).
  - Added `storage-ui/src/ui/dialogs/view/*` modules and re-wired `storage-ui/src/ui/app/view.rs` to import dialogs via `ui::dialogs::view`.
  - Reduced `storage-ui/src/views/dialogs.rs` to a small compatibility shim.

- Implemented **Task 14** (split `VolumesControl::update` into submodules).
  - Added `storage-ui/src/ui/volumes/update/*` domain modules.
  - Reworked `storage-ui/src/ui/volumes/update.rs` into a thin dispatcher delegating all handlers to submodules.

- Commands run:
  - `cargo fmt --all --check`
  - `cargo clippy --workspace --all-features`
  - `cargo test --workspace --all-features`

- Implemented **Task 11** (remove unwrap-based crash paths in UI view).
- Notable changes:
  - Removed unwraps from `storage-ui/src/ui/app/view.rs` when resolving `VolumesControl` and selected segment.
  - Added clamping for stale segment indices in `VolumesControl::update` to avoid out-of-range selection.
  - Added `no-volumes` i18n string (EN/SV) for an empty-volumes fallback view.
- Commands run:
  - `cargo fmt --all`
  - `cargo clippy --workspace --all-features -- -D warnings`
  - `cargo test --workspace --all-features`

- Implemented **Task 10** (partition type catalog split).
- Notable changes:
  - Replaced the giant `partition_type.rs` with `storage-dbus/src/partition_types/{mod.rs,gpt.rs,dos.rs,apm.rs}`.
  - Preserved existing APM entries (13) in addition to GPT (178) and DOS (37).
  - Added unit tests to lock in the total count (228) and a couple of known ID lookups.
- Commands run:
  - `cargo fmt --all`
  - `cargo clippy --workspace --all-features -- -D warnings`
  - `cargo test --workspace --all-features`

- Continued **Task 3** by extracting rendering to `storage-ui/src/ui/volumes/view.rs` and moving non-UI impls into `storage-ui/src/ui/volumes/state.rs`.
- Reduced `storage-ui/src/views/volumes.rs` to a tiny re-export shim.
- Commands run:
  - `cargo fmt --all`
  - `cargo clippy --workspace --all-features -- -D warnings`
  - `cargo test --workspace --all-features`

- Started **Task 4** by extracting `Message` into `storage-ui/src/ui/app/message.rs` and `ContextPage` into `storage-ui/src/ui/app/state.rs`.
- `storage-ui/src/app.rs` now re-exports both types for API stability.
- Commands run:
  - `cargo fmt --all`
  - `cargo clippy --workspace --all-features -- -D warnings`
  - `cargo test --workspace --all-features`

- Continued **Task 4** by moving `AppModel` struct and `AppModel::update_title` into `storage-ui/src/ui/app/state.rs`.
- `storage-ui/src/app.rs` now re-exports `AppModel` from `ui::app::state`.
- Commands run:
  - `cargo fmt --all`
  - `cargo clippy --workspace --all-features`
  - `cargo test --workspace --all-features`

- Continued **Task 4** by splitting oversized `storage-ui/src/ui/app/update.rs` into small, cohesive submodules:
  - `storage-ui/src/ui/app/update/nav.rs`
  - `storage-ui/src/ui/app/update/drive.rs`
  - `storage-ui/src/ui/app/update/smart.rs`
  - `storage-ui/src/ui/app/update/image.rs` and `storage-ui/src/ui/app/update/image/{dialogs.rs,ops.rs}`
- `storage-ui/src/ui/app/update.rs` is now a thin dispatcher.
- Result: all update-related modules are now under ~400 LOC.
- Commands run:
  - `cargo fmt --all`
  - `cargo clippy --workspace --all-features`
  - `cargo test --workspace --all-features`

- Implemented **Task 5** (typo/naming cleanup).
- Renamed:
  - `PasswordProectedUpdate` → `PasswordProtectedUpdate`
  - `selected_partitition_type` → `selected_partition_type_index`
- Commands run:
  - `cargo fmt --all`
  - `cargo clippy --workspace --all-features`
  - `cargo test --workspace --all-features`

- Implemented **Task 6** (deduplicate DBus bytestring/mount-point helpers).
- Notable changes:
  - Added shared helpers in `storage-dbus/src/dbus/bytestring.rs`.
  - Migrated `storage-dbus/src/disks/partition.rs` and `storage-dbus/src/disks/volume.rs` to use the shared module; removed duplicated helper implementations.
- Commands run:
  - `cargo fmt --all`
  - `cargo clippy --workspace --all-features`
  - `cargo test --workspace --all-features`

- Implemented **Task 7** (standardize byte formatting to a single implementation).
- Notable changes:
  - Removed unused UI duplicate `storage-ui/src/utils/format.rs`.
  - Confirmed formatting helpers are sourced from `disks_dbus::format` in UI call sites.
- Commands run:
  - `cargo fmt --all`
  - `cargo clippy --workspace --all-features -- -D warnings`
  - `cargo test --workspace --all-features`

- Implemented **Task 8** (remove `PartitionModel` alias; align `VolumeModel` module naming).
- Notable changes:
  - Renamed `storage-dbus/src/disks/partition.rs` → `storage-dbus/src/disks/volume_model.rs`.
  - Removed `pub type PartitionModel = VolumeModel` and updated exports.
- Commands run:
  - `cargo fmt --all`
  - `cargo clippy --workspace --all-features -- -D warnings`
  - `cargo test --workspace --all-features`

- Implemented **Task 9** (split `DriveModel` by responsibility).
- Notable changes:
  - Converted `storage-dbus/src/disks/drive.rs` into `storage-dbus/src/disks/drive/` module dir.
  - Split into `model.rs`, `discovery.rs`, `volume_tree.rs`, `smart.rs`, and `actions.rs`.
- Commands run:
  - `cargo fmt --all`
  - `cargo clippy --workspace --all-features -- -D warnings`
  - `cargo test --workspace --all-features`
