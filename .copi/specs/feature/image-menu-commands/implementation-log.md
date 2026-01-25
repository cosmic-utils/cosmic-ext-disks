# Implementation Log — feature/image-menu-commands

- 2026-01-25
  - Implemented all Image menu commands (new image, attach, create/restore drive+partition).
  - Added UDisks2 helpers for `LoopSetup`, `OpenForBackup`, `OpenForRestore`, and filesystem mount.
  - Added UI dialogs for path/size inputs and a cancellable streaming copy engine.

## Commands run

- `cargo build -p cosmic-ext-disks`
- `cargo build -p cosmic-ext-disks-dbus`
- `cargo fmt --all --check`
- `cargo clippy --workspace --all-features -- -D warnings`
- `cargo test --workspace --all-features`

## Notable files changed

- disks-dbus/src/disks/image.rs
- disks-dbus/src/disks/drive.rs
- disks-dbus/src/disks/partition.rs
- disks-dbus/src/disks/mod.rs
- disks-ui/src/views/menu.rs
- disks-ui/src/views/dialogs.rs
- disks-ui/src/app.rs
- disks-ui/i18n/en/cosmic_ext_disks.ftl
- disks-ui/i18n/sv/cosmic_ext_disks.ftl
- disks-ui/README.md
