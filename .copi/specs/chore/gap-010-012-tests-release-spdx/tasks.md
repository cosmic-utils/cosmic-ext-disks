# chore/gap-010-012-tests-release-spdx — Tasks

Source audit: `.copi/audits/2026-01-24T18-03-04Z.md` (GAP-010, GAP-011, GAP-012)

## Task 1: Define a mockable UDisks2 boundary
- Scope: Make disk operation flows testable without a real DBus/UDisks2 daemon.
- Files/areas: `storage-dbus/src/disks/drive.rs`, `storage-dbus/src/disks/partition.rs`, new module(s) under `storage-dbus/src/`.
- Steps:
  - Identify the minimal set of UDisks2 calls used for create/delete/format/mount/unmount.
  - Introduce a small trait (or similar) that captures those calls.
  - Provide a production implementation backed by `udisks2`.
  - Provide a fake/mock implementation for tests.
- Test plan:
  - Unit tests compile and run with the fake backend.
- Done when:
  - [x] Disk ops code can be constructed/executed using a fake backend.

## Task 2: Add contract-style tests for destructive flows (mocked)
- Scope: Tests that exercise the primary operation flows and error propagation.
- Files/areas: new tests under `storage-dbus/` (and/or `storage-ui/` if needed for state handling).
- Steps:
  - Add tests for create partition success + failure.
  - Add tests for delete partition success + failure.
  - Add tests for format partition success + failure.
  - Add tests for mount/unmount success + failure.
  - Ensure tests assert surfaced errors (not just `Ok(())`).
- Test plan:
  - Run `cargo test --workspace --all-features`.
- Done when:
  - [x] CI exercises these flows via the fake backend.
  - [x] At least one test asserts expected failure propagation.

## Task 3: Tighten publish workflow (remove dirty/no-verify)
- Scope: Improve reproducibility and verification of published crates.
- Files/areas: `.github/workflows/main.yml`.
- Steps:
  - Identify why `--allow-dirty` is currently needed (e.g. version file edits during workflow).
  - Adjust workflow so publish runs from a clean state (commit/tag or workspace reset strategy).
  - Remove `--no-verify` and address any verification failures.
  - Optionally add a guard step that fails if `git status --porcelain` is non-empty before publish.
- Test plan:
  - Validate workflow locally where possible (lint with `actionlint` if available) and ensure `cargo package` / `cargo publish --dry-run` succeeds.
- Done when:
  - [x] Workflow publishes without `--allow-dirty --no-verify`.

## Task 4: Replace SPDX placeholder in i18n module
- Scope: Align all license declarations with the repo’s GPL-3.0.
- Files/areas: `storage-ui/src/i18n.rs`, crate `Cargo.toml` files, and any Rust source headers containing SPDX identifiers.
- Steps:
  - Replace `// SPDX-License-Identifier: {{ license }}` with `GPL-3.0-only`.
  - Update `license = "..."` in crate metadata to `GPL-3.0-only` (where present).
  - Add missing license metadata to crates that lack it.
  - Update any existing SPDX headers that currently use a different identifier.
  - Confirm consistency with the root `LICENSE` text (GPLv3).
- Test plan:
  - Run `cargo fmt --all` and `cargo clippy --workspace --all-features`.
- Done when:
  - [x] No template placeholder remains.
  - [x] All crate license metadata matches GPL-3.0 (`GPL-3.0-only`).
  - [x] All SPDX headers match `GPL-3.0-only`.

## Task 5: Update `.copi/spec-index.md` and any `.copi` references
- Scope: Record canonical mapping from GAP IDs to this spec and branch.
- Files/areas: `.copi/spec-index.md` (and any other eligible `.copi` trackers).
- Steps:
  - Add rows for GAP-010/011/012.
  - Ensure audits remain unmodified.
- Test plan:
  - N/A (documentation change).
- Done when:
  - [x] Spec index includes this spec and branch for all three GAPs.

## Recommended Sequence / Dependencies
- Task 1 → Task 2 can proceed in parallel with Task 3/4.
- Task 5 should be last (or done alongside spec creation).
