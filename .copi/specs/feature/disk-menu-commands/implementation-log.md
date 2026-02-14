# Implementation Log — Disk Menu Commands

Branch: `feature/disk-menu-commands`

## 2026-01-25
- Scoped out disk image + benchmarking per user (separate PRs).
- Implemented Disk menu cleanup: removed Drive Settings (and Benchmark) from Disk menu.
- Implemented Disk → Power Off via `DriveModel::power_off()` and refreshes drives.
- Implemented Disk → Standby Now / Wake-up using UDisks2 ATA interface (shows “Not supported by this drive” when not available).
- Implemented Disk → SMART Data & Self-Tests:
  - NVMe SMART via `org.freedesktop.UDisks2.NVMe.Controller` when available.
  - ATA SMART fallback via `org.freedesktop.UDisks2.Drive.Ata`.
  - UI dialog with Refresh + Short/Extended/Abort self-test.

### Commands run
- `busctl tree org.freedesktop.UDisks2`
- `busctl introspect org.freedesktop.UDisks2 ...`
- `cargo fmt --all`
- `cargo clippy --workspace --all-features`
- `cargo test --workspace --all-features`

### Notable files changed
- storage-ui/src/views/menu.rs
- storage-ui/src/app.rs
- storage-ui/src/views/dialogs.rs
- storage-ui/src/views/volumes.rs
- storage-ui/i18n/en/cosmic_ext_disks.ftl
- storage-ui/i18n/sv/cosmic_ext_disks.ftl
- storage-dbus/src/disks/drive.rs
- storage-dbus/src/disks/smart.rs
- storage-dbus/src/disks/mod.rs
