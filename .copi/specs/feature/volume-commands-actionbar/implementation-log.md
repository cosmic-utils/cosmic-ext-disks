# Implementation Log — Volume Commands Actionbar

## 2026-01-25
- Implemented volume command actionbar buttons (with tooltips) and dialogs, wired to DBus operations and drive refresh.
- Addressed follow-up compile issues (borrow checker + moved values) and removed accidental UI-layer dependency on `udisks2` / `enumflags2` by pushing flag conversion into `disks-dbus`.
- Fixed actionbar construction borrow conflict by switching from a closure capturing `&mut action_bar` to a helper that returns an element.

### Commands run
- `cargo check --workspace --all-features`
- `cargo fmt --all --check` (and `cargo fmt --all`)
- `cargo clippy --workspace --all-features`
- `cargo test --workspace --all-features`

### Notable files touched
- `disks-ui/src/views/volumes.rs`
- `disks-ui/src/views/dialogs.rs`
- `disks-ui/src/app.rs`
- `disks-ui/i18n/en/cosmic_ext_disks.ftl`
- `disks-ui/i18n/sv/cosmic_ext_disks.ftl`
- `disks-dbus/src/disks/partition.rs`
- `disks-dbus/src/disks/volume.rs`
- `disks-dbus/src/partition_type.rs`

### Follow-ups
- Manual validation on a loop device / non-critical disk for each command (polkit prompts, mounted/unmounted error surfacing).
