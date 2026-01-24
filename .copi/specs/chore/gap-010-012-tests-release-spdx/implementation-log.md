# Implementation Log — GAP-010/011/012

Branch: `chore/gap-010-012-tests-release-spdx`
Date: 2026-01-24

## Summary
- Added a mockable disk-operations boundary in `disks-dbus` and contract tests for create/delete/format/mount/unmount flows.
- Tightened the publish workflow to publish from tag refs and removed `--allow-dirty --no-verify`.
- Canonicalized license metadata and SPDX headers to `GPL-3.0-only` to match the repo `LICENSE`.

## Key Files Changed
- `disks-dbus/src/disks/ops.rs`
- `disks-dbus/src/disks/drive.rs`
- `disks-dbus/src/disks/partition.rs`
- `.github/workflows/main.yml`
- `Cargo.toml`
- `disks-ui/Cargo.toml`
- `disks-dbus/Cargo.toml`
- `disks-ui/src/{main,app,config,i18n}.rs`

## Commands Run
- `cargo fmt --all`
- `cargo test --workspace --all-features`
- `cargo clippy --workspace --all-features`

## Notes / Decisions
- Publish now runs only for tag refs `v*` and verifies that crate versions and the workspace dependency version match the tag.
- Disk operation flow testing is implemented as mocked/contract tests (no requirement for a live system UDisks2 daemon in CI).
