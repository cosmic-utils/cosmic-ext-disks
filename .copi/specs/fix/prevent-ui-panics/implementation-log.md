# Implementation Log — fix/prevent-ui-panics

- 2026-01-24
  - Implemented GAP-001/002/003 as a single reliability patch.
  - Changes:
    - Create-partition Cancel now closes the dialog (no crash).
    - Invalid dialog state no longer panics; it is ignored with an `eprintln!`.
    - Menu actions that were `todo!()` now open an informational dialog instead of crashing.
    - Fixed `fl!` macro wrapper to correctly forward named Fluent args (`name = value`) to `i18n_embed_fl::fl!`.
    - Implemented disabled style branch in segment button styling to remove a reachable `todo!()`.
  - Commands run:
    - `cargo fmt --all --check`
    - `cargo clippy --workspace --all-features`
    - `cargo test --workspace --all-features`
  - Notes:
    - Per request, no menu items were hidden; unimplemented actions remain visible and show a prompt.
    - Clippy/test emit pre-existing warnings in `storage-dbus`; not addressed in this spec.
