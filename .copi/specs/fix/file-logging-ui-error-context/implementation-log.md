# Implementation Log — fix/file-logging-ui-error-context

## 2026-02-05

### Summary
- Added daily-rotating file logging for `disks-ui` with 7-day best-effort retention cleanup.
- Ensured every surfaced UI error dialog created from a backend `Err(e)` also logs via `log_error_and_show_dialog`.
- Preserved UDisks2 `MethodError` name/message for key operations by switching to raw `zbus::Proxy` calls in `disks-dbus`.
- Documented log location and environment overrides.

### Notable changes
- `disks-ui/src/logging.rs`: stdout + daily file logs; `RUST_LOG` support; `COSMIC_EXT_DISKS_LOG_DIR`/`COSMIC_EXT_DISKS_LOG_FILE` overrides; retention cleanup.
- `disks-ui/src/ui/error.rs`: `log_error_and_show_dialog` helper (only allowed wrapper around logging).
- UI migrations:
  - `disks-ui/src/ui/volumes/update/{mount_options,create,filesystem,encryption,partition}.rs`
  - `disks-ui/src/ui/app/update/drive.rs`
- `disks-dbus/src/disks/ops.rs`: raw zbus calls + consistent formatting for `MethodError` details for:
  - `Filesystem.Mount` / `Filesystem.Unmount`
  - `Block.Format`
  - `Encrypted.Unlock` / `Encrypted.Lock`
  - `PartitionTable.CreatePartitionAndFormat`
  - and reused helpers for `Partition.Delete`
- Docs:
  - `README.md`
  - `disks-ui/README.md`

### Commands run
- `cargo test --workspace`
- `cargo fmt --all --check`
- `cargo clippy --workspace --all-features -- -D warnings`

### Notes
- Passphrases remain redacted (`RedactedString`) and are never included in logs or error strings.
- Log retention cleanup is best-effort and based on file modification time.

## 2026-02-06

### Follow-up
- Replaced generic `app-title` error dialog titles with operation-specific `*-failed` titles (e.g., power off, eject, format disk, create partition).
- Added corresponding i18n keys in `en` and `sv`.

### Commands run
- `cargo fmt --all`
- `cargo test --workspace --all-features`
- `cargo clippy --workspace --all-features -- -D warnings`
