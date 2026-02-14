# Implementation Log — fix/gpt-reserved-offsets-udisks

## 2026-01-24
- Implemented GPT usable-range probing via `org.freedesktop.UDisks2.Block.OpenDevice` + GPT header parse.
  - Uses `auth.no_user_interaction=true` so the probe never triggers a polkit prompt; on denial/cancel it cleanly falls back.
  - Sector size: ioctl `BLKSSZGET` first, sysfs `queue/logical_block_size` fallback.
  - Converts parsed `{first_usable_lba,last_usable_lba}` into a half-open byte range.
  - Conservative fallback usable range reserves 1 MiB at start/end when probing fails.
- Threaded `gpt_usable_range` through the model and UI.
  - UI segmentation marks reserved areas as non-free (and non-actionable).
  - DBus layer validates create-partition requests stay within the usable range.
- i18n: added reserved-space labels (en + sv).
- UI: added a "Show reserved" checkbox; when disabled, reserved segments and tiny free-space (< 1 MiB) are hidden and excluded from segment width calculations.
- UI follow-up: made `show_reserved` strictly a VolumesControl setting.
  - App no longer stores/handles the toggle; Volumes owns it.
  - On drive tab switch (nav select), the newly activated VolumesControl inherits the previous tab’s `show_reserved` value.

### Commands run
- `cargo fmt --all --check`
- `cargo clippy --workspace --all-features`
- `cargo test --workspace --all-features`

### Notable files changed
- storage-dbus/src/disks/gpt.rs
- storage-dbus/src/disks/drive.rs
- storage-ui/src/utils/segments.rs
- storage-ui/src/views/volumes.rs
- storage-ui/i18n/en/cosmic_ext_disks.ftl
- storage-ui/i18n/sv/cosmic_ext_disks.ftl
