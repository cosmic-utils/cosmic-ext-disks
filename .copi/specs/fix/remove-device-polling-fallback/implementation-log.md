# Implementation Log — fix/remove-device-polling-fallback

- Date: 2026-01-24

## Summary

- Removed the legacy polling-based device event stream (periodic `GetBlockDevices()` diff loop).
- Updated the UI to rely on UDisks2 ObjectManager signals only; on subscription failure it logs and stops the subscription task.
- Updated READMEs to call out `udisks2` as a runtime dependency and removed references to polling fallback.

## Notable changes

- `disks-dbus/src/disks/manager.rs`
  - Removed `DiskManager::device_event_stream(interval: Duration)`.
  - Simplified `DiskManager` to hold only a DBus connection for signal subscription.
- `disks-ui/src/app.rs`
  - Removed the polling fallback branch and message.
- `README.md`, `disks-ui/README.md`
  - Documented `udisks2` dependency; updated device update behavior.

## Commands run

- `cargo fmt --all --check`
- `cargo clippy --workspace --all-features`
- `cargo test --workspace --all-features`

## Notes

- Clippy reports pre-existing warnings in the workspace; CI gate is the command invocation (not `-D warnings`).
