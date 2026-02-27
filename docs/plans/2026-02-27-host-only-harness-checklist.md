# Host-Only Harness Migration Checklist

**Date:** 2026-02-27  
**Scope:** Action-level checklist to remove containerized harness execution, run harness on host only, remove CI harness container job, and debug host test stability.

---

## A. Preparation

- [ ] Confirm approved design exists: `docs/plans/2026-02-27-host-only-harness-design.md`.
- [ ] Confirm approved implementation plan exists: `docs/plans/2026-02-27-host-only-harness-implementation-plan.md`.
- [ ] Confirm branch scope is limited to harness host-only migration + related docs/CI updates.
- [ ] Run baseline check and capture timestamp: `cargo check -p storage-testing`.
- [ ] Capture baseline harness status and failure mode:
  - [ ] `STORAGE_TESTING_TEST_TIMEOUT_SECS=10 cargo run -p storage-testing --bin harness -- run`
  - [ ] Record summary (PASS/SKIP/FAIL + setup errors) in section K.

## B. Global Guardrails

- [ ] Do not change storage service runtime behavior beyond what is required for host harness execution/debugging.
- [ ] Preserve test IDs, suite names, and spec names unless a rename is explicitly required.
- [ ] Preserve structured harness result reporting (`group ... summary`, `summary: PASS=... SKIP=... FAIL=...`).
- [ ] Preserve artifact emission from orchestrator (`group-*.log`, `test-*.log`, `run-summary.json`).
- [ ] Keep destructive behavior opt-in/explicit where already present in tests.

## C. CLI Migration to Host-Only

## C1. `harness run` command contract

- [ ] Modify `storage-testing/src/bin/harness.rs` so `Run` executes orchestrator directly.
- [ ] Remove hidden `RunLocal` subcommand.
- [ ] Remove container-only `Run` args:
  - [ ] `--runtime`
  - [ ] `--keep`
- [ ] Keep host-relevant `Run` args:
  - [ ] `--suite`
  - [ ] `--test-id`
  - [ ] `--max-parallel-groups`
  - [ ] `--dry-run`
- [ ] Confirm `--help` output matches new contract.

## C2. Container-only CLI surfaces

- [ ] Remove `Shell` subcommand (container-only).
- [ ] Keep `Cleanup`, but ensure behavior is host lab cleanup (not container removal).

## D. Remove In-Container Routing and Config

## D1. Harness module surface cleanup

- [ ] Remove container planning/execution path from `storage-testing/src/harness/mod.rs`:
  - [ ] `plan_run` container build/run command construction
  - [ ] container shell planning
  - [ ] runtime command orchestration for docker/podman harness run
- [ ] Remove dependencies in this module on container runtime command assembly.
- [ ] Keep exports needed for host orchestration and support helpers.

## D2. Remove container assets from active path

- [ ] Remove container harness assets from repository (or at minimum from all references):
  - [ ] `storage-testing/container/Containerfile`
  - [ ] `storage-testing/container/run-harness.sh`
- [ ] Verify no code path references `storage-testing/container/*`.

## D3. Cleanup command behavior

- [ ] Route `harness cleanup` to host lab cleanup flow (`lab::orchestrator::cleanup_all`).
- [ ] Ensure cleanup prints host commands/outcomes consistently.

## E. CI Workflow Updates

- [ ] Edit `.github/workflows/ci.yml`.
- [ ] Remove `harness` job entirely.
- [ ] Remove harness failure artifact-upload block tied to that job.
- [ ] Keep `build`, `clippy`, and `fmt` jobs intact.
- [ ] Validate CI YAML formatting/syntax.

## F. Documentation and Developer Workflow

- [ ] Update `storage-testing/README.md` to host-only harness commands.
- [ ] Remove container runtime shell guidance from `storage-testing/README.md`.
- [ ] Update root `justfile` harness recipe to host-only run semantics.
- [ ] Ensure command examples align with actual CLI help output.

## G. Host-Only Test Runtime Guardrails

- [ ] Keep per-test global timeout behavior via `STORAGE_TESTING_TEST_TIMEOUT_SECS`.
- [ ] Keep internal short-circuit timeouts where added for long-hanging operations:
  - [ ] usage scan test
  - [ ] LUKS format tests
  - [ ] mdraid create test
- [ ] Keep image tests explicit opt-in guard (`STORAGE_TESTING_ENABLE_IMAGE_TESTS=1`) to avoid known panic path by default.

## H. Host Debug Pass (Targeted)

## H1. Logical suite pass

- [ ] Run targeted logical suite:
  - [ ] `STORAGE_TESTING_TEST_TIMEOUT_SECS=10 cargo run -p storage-testing --bin harness -- run --suite logical`
- [ ] If setup errors occur, capture exact failing command and stderr.
- [ ] Apply one root-cause fix at a time.
- [ ] Re-run same command after each fix.

## H2. Partition/LUKS suite pass

- [ ] Run targeted partition suite:
  - [ ] `STORAGE_TESTING_TEST_TIMEOUT_SECS=10 cargo run -p storage-testing --bin harness -- run --suite partition`
- [ ] Run targeted luks suite:
  - [ ] `STORAGE_TESTING_TEST_TIMEOUT_SECS=10 cargo run -p storage-testing --bin harness -- run --suite luks`
- [ ] Confirm no new FAIL introduced by host-only migration.

## I. Full Host Harness Verification

- [ ] Run full host harness with bounded timeout:
  - [ ] `STORAGE_TESTING_TEST_TIMEOUT_SECS=10 cargo run -p storage-testing --bin harness -- run`
- [ ] Record summary counts in section K.
- [ ] Confirm no container command invocation appears in output.
- [ ] Confirm no setup failure caused by removed container path logic.

## J. Final Quality Gates

- [ ] `cargo check -p storage-testing`
- [ ] `cargo fmt --all --check`
- [ ] Optional: `cargo clippy -p storage-testing --all-targets`
- [ ] Verify only intended files changed:
  - [ ] `storage-testing/src/bin/harness.rs`
  - [ ] `storage-testing/src/harness/mod.rs` (or equivalent removed/reduced surface)
  - [ ] `.github/workflows/ci.yml`
  - [ ] `storage-testing/README.md`
  - [ ] `justfile`
  - [ ] `storage-testing/container/*` removals (if applied)

## K. Verification Evidence (fill during execution)

- [ ] Baseline `cargo check -p storage-testing`:
- [ ] Baseline full harness run summary:
- [ ] Post-migration CLI help snapshot verified:
- [ ] Targeted logical run summary:
- [ ] Targeted partition run summary:
- [ ] Targeted luks run summary:
- [ ] Final full host harness summary:
- [ ] CI workflow diff sanity check:

## L. Done Definition

- [ ] `harness run` is host-only and no longer routes through container execution.
- [ ] Hidden `run-local` path is removed.
- [ ] Container-only CLI surfaces and config are removed from active harness flow.
- [ ] CI no longer contains containerized harness job.
- [ ] Docs and recipes reflect host-only harness usage.
- [ ] Host harness debug run completes with deterministic summary output and no container setup path.
