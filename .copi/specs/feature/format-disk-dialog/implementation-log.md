# Implementation Log — feature/format-disk-dialog

## 2026-01-25

- Added a new “Format Disk” dialog with Erase + Partitioning dropdowns.
- Wired Disk → Format Disk menu action to open the dialog with defaults derived from the selected drive.
- Implemented a best-effort backend call using UDisks2 `Block.Format` on the whole-disk block device, mapping:
  - Partitioning: `dos` / `gpt` / `empty`
  - Erase (slow): `erase=zero`
- Added a preflight before whole-disk format: unmount mounted filesystems (including nested volumes) and lock unlocked encrypted containers to avoid device-busy failures.
- Implemented a consistent “Working…” busy state for pre-action dialogs:
  - Create Partition
  - Unlock Encrypted
  - Delete Partition confirmation
  - (Format Disk already had busy state)
  This disables primary actions to prevent double-submission and keeps dialogs open until the async task completes and the UI refreshes.
- Addressed a `clippy::large_enum_variant` warning by boxing `Message::Dialog` payloads.

Commands (expected to run for validation):
- `cargo fmt --all --check`
- `cargo clippy --workspace --all-features`
- `cargo test --workspace --all-features`

Commands run:
- `cargo fmt --all`
- `cargo clippy --workspace --all-features`
- `cargo test --workspace --all-features`
