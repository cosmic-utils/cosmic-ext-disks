# Spec — GAP-010/011/012 (Tests + Release Hygiene + SPDX)

Branch: `chore/gap-010-012-tests-release-spdx`

Source audit: `.copi/audits/2026-01-24T18-03-04Z.md` (GAP-010, GAP-011, GAP-012)

## Context
The repo has some unit test coverage for helper logic (e.g. GPT and UI segment computations), but lacks coverage for the disk operation flows that are most likely to regress (create/delete/format/mount/unmount). In addition, the publish workflow currently uses `cargo publish --allow-dirty --no-verify`, reducing reproducibility and skipping checks. Finally, one shipped Rust source file contains an SPDX template placeholder.

## Goals
- Add CI-exercised tests that cover destructive/system-integrated flows at a mocked/contract level.
- Make publish artifacts verifiable and derived from a clean, reproducible source state.
- Remove template SPDX placeholder and align it with the repo’s declared license.

## Non-Goals
- No new user-facing features in the UI.
- No changes to actual on-disk behavior of destructive operations beyond what is required to make them testable.
- No refactors unrelated to testability/release hygiene.

## Proposed Approach
### GAP-010 — Testing for disk flows
- Introduce a thin abstraction layer for the UDisks2/DBus boundary used by `DriveModel`/`PartitionModel` so tests can substitute a fake/mocked backend.
- Add contract-style tests for:
  - create partition
  - delete partition
  - format partition
  - mount/unmount
- Ensure error surfacing is testable: failures from the backend should propagate as `Err(...)` and be asserted in tests.
- Keep existing pure unit tests; add new ones without requiring a real system UDisks2 daemon in CI.

Likely touched areas:
- `storage-dbus/src/disks/drive.rs`
- `storage-dbus/src/disks/partition.rs`
- Any new trait/module under `storage-dbus/src/` to abstract UDisks2 calls
- Potential small adjustments in `storage-ui/` if UI-layer logic needs to consume richer error states

### GAP-011 — Publish workflow hygiene
- Update `.github/workflows/main.yml` to remove or avoid the need for:
  - `--allow-dirty`
  - `--no-verify`
- Prefer a flow that publishes from a tagged commit (or a commit created by the workflow) where:
  - the working tree is clean
  - `cargo publish` runs with verification enabled
  - dependency locking (`--locked`) is used when appropriate

### GAP-012 — SPDX placeholder
- Replace `// SPDX-License-Identifier: {{ license }}` in `storage-ui/src/i18n.rs` with the repo’s intended identifier.
- Canonicalize licensing across the repo to match the root `LICENSE` (GPLv3):
  - Update crate metadata (`license = ...`) to `GPL-3.0-only` where present.
  - Update any existing SPDX headers that currently mention a different license.
  - Ensure the DBus crate includes explicit license metadata as well.
- Document the choice of SPDX identifier (`GPL-3.0-only`) and verify it matches the distributed license text.

## User / System Flows
- **Developer flow:** `cargo test --workspace --all-features` runs locally and in CI without requiring privileged disk access.
- **Release flow:** push/tag to `main` produces a reproducible release whose publish step fails if verification checks fail.

## Risks & Mitigations
- **Risk:** Mocking/abstraction may diverge from real UDisks2 behavior.
  - **Mitigation:** Keep the abstraction minimal; add a small set of “contract” tests for the fake backend that mirror key UDisks2 call semantics.
- **Risk:** Removing `--no-verify` may break current publish due to missing metadata (README, license files, etc.).
  - **Mitigation:** Fix the underlying causes and keep verification enabled.
- **Risk:** License identifiers disagree across repo (`LICENSE` vs crate metadata).
  - **Mitigation:** Decide the canonical license source and align headers/metadata accordingly.

## Acceptance Criteria
- [x] GAP-010: CI runs tests that exercise create/delete/format/mount/unmount logic at least via a mocked/contract backend.
- [x] GAP-010: A failing backend call is asserted in tests and surfaces as an error (no silent success).
- [x] GAP-011: `.github/workflows/main.yml` no longer uses `cargo publish --allow-dirty --no-verify` (or has a documented, justified exception with equivalent guarantees).
- [x] GAP-011: Publish is executed from a clean tree and is verifiable/reproducible from a commit/tag.
- [x] GAP-012: No template placeholders remain in shipped source headers; `storage-ui/src/i18n.rs` uses the intended SPDX identifier.
