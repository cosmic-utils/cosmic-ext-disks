# Implementation Log — feature/image-menu-commands

- 2026-01-25
  - Implemented all Image menu commands (new image, attach, create/restore drive+partition).
  - Added UDisks2 helpers for `LoopSetup`, `OpenForBackup`, `OpenForRestore`, and filesystem mount.
  - Added UI dialogs for path/size inputs and a cancellable streaming copy engine.

- 2026-01-25 (validation follow-ups)
  - Added dialog error logging to console.
  - Fixed `LoopSetup` argument type to use an FD handle.
  - Included loop devices in drive enumeration and showed “Backing File” in the drive info pane.
  - Made enumeration tolerant of missing UDisks2 interfaces.
  - Identified a UX gap: filesystem-on-loop images (no partition table) render as “free space”.
  - Appended follow-up spec items for `VolumeModel` refactor + loop filesystem fallback.

- 2026-01-25 (follow-up implementation)
  - Replaced `PartitionModel` with `VolumeModel` and introduced `VolumeType { Container, Partition, Filesystem }`.
  - Renamed drive’s flat list from `partitions` to `volumes_flat` and updated UI to consume it.
  - Added filesystem-on-block fallback so loop images with ext4-on-`loopX` render as a single filesystem volume (not free space).
  - Verified with formatting, clippy, and tests.

- 2026-01-25 (dialog polish)
  - Show selected partition name/path in partition image dialogs.
  - Move the image path text input below the device/partition info.
  - Add `partition` + `path` i18n keys (EN + SV).

## Commands run

- `cargo build -p cosmic-ext-disks`
- `cargo build -p cosmic-ext-storage-dbus`
- `cargo fmt --all --check`
- `cargo clippy --workspace --all-features -- -D warnings`
- `cargo test --workspace --all-features`

## Follow-up commands run

- `cargo fmt --all`
- `cargo clippy --workspace --all-features -- -D warnings`
- `cargo test --workspace --all-features`

## Dialog polish commands run

- `cargo fmt --all --check`
- `cargo clippy --workspace --all-features`
- `cargo test --workspace --all-features`

## Notable files changed

- storage-dbus/src/disks/image.rs
- storage-dbus/src/disks/drive.rs
- storage-dbus/src/disks/partition.rs
- storage-dbus/src/disks/mod.rs
- storage-ui/src/views/menu.rs
- storage-ui/src/views/dialogs.rs
- storage-ui/src/app.rs
- storage-ui/i18n/en/cosmic_ext_disks.ftl
- storage-ui/i18n/sv/cosmic_ext_disks.ftl
- storage-ui/README.md
