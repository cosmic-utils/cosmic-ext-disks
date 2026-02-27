# Host-Only Harness Design

## Context
The current `storage-testing` harness executes through a privileged container runtime path. In practice this mutates host kernel loop state, creates stale loop attachments, and introduces instability/noise in local debugging. The requested direction is to remove in-container harness routing/config entirely and run only on host.

## Goals
- Make `harness run` host-only.
- Remove containerized harness execution path from CLI and CI.
- Preserve current test orchestration behavior (group setup/teardown, PASS/SKIP/FAIL summaries, artifacts).
- Keep debugging focused on host-visible preconditions and failures.

## Non-Goals
- No new UI/UX features.
- No new test suites.
- No attempt to solve all backend flakiness in this design phase.

## Considered Approaches

### Approach A: Full host-first rewrite (recommended)
- `harness run` directly calls orchestrator logic in-process (current hidden `run-local` behavior).
- Remove container plan/build/run plumbing.
- Keep host cleanup behavior for lab assets only.

**Trade-offs**
- Pros: matches requested behavior exactly, removes dangerous ambiguity, easier debugging.
- Cons: medium refactor touching CLI and helper modules.

### Approach B: Alias-only (`run-local` -> `run`) with dead container code
- Keep container code in tree but no longer used by default.

**Trade-offs**
- Pros: small patch.
- Cons: dead code risk and future accidental reintroduction.

### Approach C: Hard delete container path + CI harness job removal
- Remove container harness command path and CI harness container job.

**Trade-offs**
- Pros: strongest guarantee of host-only behavior.
- Cons: larger diff.

**Selected direction**: Approach A + C.

## CLI and Module Design

### `storage-testing/src/bin/harness.rs`
- `Run` becomes host-only execution path (orchestrator-driven).
- Remove hidden `RunLocal` command.
- Remove container-oriented run args: `--runtime`, `--keep`.
- Remove `Shell` command (container-only semantics).
- Keep `Cleanup`, but map to host lab cleanup, not container removal.

### `storage-testing/src/harness/mod.rs`
- Remove container planning/execution helpers (`plan_run`, container command builders, runtime-dependent shell plan).
- Retain host-relevant API surface only (or reduce to orchestrator/support exports).

### `storage-testing/src/lib.rs`
- Keep exports needed by host harness orchestration and tests.
- Drop dead exports introduced solely for container command flow.

## Documentation and Workflow Changes

### `storage-testing/README.md`
- Rewrite harness usage as host-only (`cargo run -p storage-testing --bin harness -- run ...`).
- Remove container shell/runtime guidance.

### Root `justfile`
- Ensure `harness` recipe runs host-only command path.
- Remove container runtime assumptions in recipe text/comments.

## CI Changes

### `.github/workflows/ci.yml`
- Remove the dedicated `harness` job that runs containerized harness.
- Keep existing `build`, `clippy`, `fmt` jobs unchanged.

## Host-Mode Debugging Design

### Preflight checks in run flow
- Validate essential command availability (`losetup`, `parted`, `mdadm`, `lvm`, `btrfs` tools).
- Validate service presence (`org.cosmic.ext.Storage.Service` owner check).
- Validate required permissions for destructive lab operations.

### Failure model
- Keep per-test timeout guard and test-level SKIP/FAIL semantics.
- Keep structured artifact logs (`group-*.log`, `test-*.log`, `run-summary.json`).
- Emit actionable host hints on setup failures (loop-state/pv metadata guidance).

## Testing Strategy for Migration
1. Compile and run host harness dry-run path.
2. Run focused suites (`--suite logical`, `--suite partition`) to verify CLI + orchestration behavior.
3. Run full host harness and compare PASS/SKIP/FAIL behavior against prior baseline.
4. Validate CI workflow after `harness` job removal.

## Risks and Mitigations
- **Risk**: Host environment variance increases local failures.
  - **Mitigation**: preflight checks and explicit failure hints.
- **Risk**: Removing container path breaks existing developer habits.
  - **Mitigation**: clear README/justfile updates and concise migration note.
- **Risk**: Hidden references to container APIs remain.
  - **Mitigation**: compile checks and symbol search cleanup.

## Success Criteria
- `harness run` has no container execution path.
- CI has no containerized harness invocation.
- Docs/recipes reflect host-only workflow.
- Harness produces valid summaries and artifacts in host mode.
