# Implementation Log — fix/create-partition-password-protection

- 2026-02-04
  - Confirmed UDisks2 encryption options via storaged.org docs:
    - `org.freedesktop.UDisks2.Block.Format` supports `encrypt.passphrase` and `encrypt.type` (luks1/luks2).
  - Implemented DBus wiring:
    - `storage-dbus/src/disks/ops.rs`
      - Adds `encrypt_type` + `encrypt_passphrase` to `CreatePartitionAndFormatArgs`.
      - Uses `encrypt.*` options when calling `CreatePartitionAndFormat`.
      - Introduces `RedactedString` to keep passphrases out of `Debug` output.
      - Adds unit test ensuring passphrase is redacted.
  - Implemented UI validation:
    - `storage-ui/src/ui/volumes/update/create.rs` blocks create when password is empty/mismatched.
    - `storage-ui/src/ui/dialogs/state.rs` adds `error` to `CreatePartitionDialog`.
    - `storage-ui/src/ui/dialogs/view/partition.rs` renders error.
    - Adds i18n keys `password-required` + `password-mismatch` (EN/SV).

Commands to validate:
- `cargo test -p storage-dbus`
- `cargo test --workspace --all-features`
- `cargo clippy --workspace --all-features`
- `cargo fmt --all --check`
