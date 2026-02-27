# Host-Only Harness Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Convert `storage-testing` harness execution to host-only mode, remove containerized harness routing and CI usage, then stabilize/debug host harness runs.

**Architecture:** Move `harness run` to call orchestrator directly in-process, delete container command-planning surface, and update docs/CI to remove container assumptions. Keep existing test orchestration and result artifacts intact, then debug host-run failures using targeted suite runs.

**Tech Stack:** Rust (`clap`, `tokio`), cargo workspace, GitHub Actions CI.

---

### Task 1: Convert `harness run` to host-only CLI path

**Files:**
- Modify: `storage-testing/src/bin/harness.rs`
- Test: `cargo run -p storage-testing --bin harness -- run --dry-run`

**Step 1: Write the failing expectation (CLI contract check)**
- Define expected behavior: `run` no longer accepts container-only args (`--runtime`, `--keep`) and executes orchestrator path directly.
- Failing condition to observe first: current CLI still exposes runtime/keep + hidden `run-local`.

**Step 2: Run CLI help to confirm old behavior exists**
Run: `cargo run -p storage-testing --bin harness -- --help`
Expected (pre-change): `run` includes container runtime options and hidden local indirection still exists.

**Step 3: Implement minimal CLI changes**
- Remove `runtime`/`keep` from `Run` command.
- Remove `RunLocal` variant and merge its logic into `Run` handling.
- Remove `Shell` command (container-only).
- Keep summary reporting behavior unchanged.

**Step 4: Verify CLI behavior**
Run: `cargo run -p storage-testing --bin harness -- --help`
Expected: `run` exposes only host-relevant options (`suite`, `test_id`, `max_parallel_groups`, `dry_run`).

**Step 5: Commit**
```bash
git add storage-testing/src/bin/harness.rs
git commit -m "refactor(harness): make run host-only and remove run-local path"
```

### Task 2: Remove container planning/execution surface

**Files:**
- Modify: `storage-testing/src/harness/mod.rs`
- Modify (if needed): `storage-testing/src/lib.rs`
- Delete: `storage-testing/container/Containerfile`
- Delete: `storage-testing/container/run-harness.sh`

**Step 1: Write failing compile expectation**
- Identify container-only functions (`plan_run`, shell/runtime command builders, docker/podman command assembly).
- Failing condition to observe: host-only CLI still depends on these APIs.

**Step 2: Confirm dependency before edits**
Run: `cargo check -p storage-testing`
Expected (pre-change): compiles with container planning APIs still present.

**Step 3: Implement minimal host-only module surface**
- Remove container command planning/execution APIs from harness module.
- Keep `orchestrator` and `support` module exposure.
- Ensure no remaining references from `harness.rs`.

**Step 4: Verify build after removals**
Run: `cargo check -p storage-testing`
Expected: PASS.

**Step 5: Commit**
```bash
git add storage-testing/src/harness/mod.rs storage-testing/src/lib.rs storage-testing/container/Containerfile storage-testing/container/run-harness.sh
git commit -m "chore(harness): remove container execution path"
```

### Task 3: Update host-only cleanup behavior

**Files:**
- Modify: `storage-testing/src/bin/harness.rs`
- Reference: `storage-testing/src/lab/orchestrator.rs`

**Step 1: Write failing behavior expectation**
- `harness cleanup` should clean host lab state rather than remove a container.

**Step 2: Confirm current cleanup behavior**
Run: `cargo run -p storage-testing --bin harness -- cleanup`
Expected (pre-change): attempts container runtime cleanup.

**Step 3: Implement host cleanup mapping**
- Route `Cleanup` command to `lab::orchestrator::cleanup_all(ExecuteOptions { dry_run: false })`.
- Print host cleanup outcomes consistently.

**Step 4: Verify cleanup command**
Run: `cargo run -p storage-testing --bin harness -- cleanup`
Expected: host cleanup steps execute; no container runtime call.

**Step 5: Commit**
```bash
git add storage-testing/src/bin/harness.rs
git commit -m "feat(harness): make cleanup operate on host lab state"
```

### Task 4: Remove CI harness container job

**Files:**
- Modify: `.github/workflows/ci.yml`

**Step 1: Write failing expectation**
- CI should not include containerized harness job.

**Step 2: Confirm current CI has harness job**
Run: `grep -n "^  harness:" -n .github/workflows/ci.yml`
Expected (pre-change): job exists.

**Step 3: Remove only harness job block**
- Delete the `harness` job and related artifact-upload step.
- Leave `build`, `clippy`, `fmt` unchanged.

**Step 4: Validate YAML structure**
Run: `cargo check -p storage-testing` (quick) and optionally `yamllint .github/workflows/ci.yml` if available.
Expected: no syntax issues introduced.

**Step 5: Commit**
```bash
git add .github/workflows/ci.yml
git commit -m "ci: remove containerized harness workflow job"
```

### Task 5: Update docs and local recipes to host-only workflow

**Files:**
- Modify: `storage-testing/README.md`
- Modify: `justfile`

**Step 1: Write failing doc expectation**
- Docs and recipe still mention container runtime options/shell workflow.

**Step 2: Confirm stale instructions exist**
Run: `grep -n "runtime\|container\|shell" storage-testing/README.md justfile`
Expected (pre-change): container references present.

**Step 3: Apply minimal doc/recipe updates**
- Make harness examples host-only.
- Update `just harness` recipe text/command to host-only run path.

**Step 4: Verify docs/readability**
Run: `cargo run -p storage-testing --bin harness -- --help`
Expected: commands shown in docs align with actual CLI.

**Step 5: Commit**
```bash
git add storage-testing/README.md justfile
git commit -m "docs: switch harness docs and recipes to host-only mode"
```

### Task 6: Host-only harness debug pass (targeted then full)

**Files:**
- Modify as needed from failures in:
  - `storage-testing/src/lab/image.rs`
  - `storage-testing/src/harness/orchestrator.rs`
  - failing tests under `storage-testing/tests/**`

**Step 1: Run targeted suite to reproduce root failures**
Run: `STORAGE_TESTING_TEST_TIMEOUT_SECS=10 cargo run -p storage-testing --bin harness -- run --suite logical`
Expected: deterministic PASS/SKIP/FAIL output with concrete failure reasons.

**Step 2: Apply one root-cause fix at a time (TDD style)**
- For each failure class, add/adjust test guard or setup logic minimally.
- Re-run only impacted suite after each fix.

**Step 3: Verify targeted suite outcome**
Run: same command as step 1.
Expected: no new FAIL introduced; runtime remains bounded.

**Step 4: Run full host harness**
Run: `STORAGE_TESTING_TEST_TIMEOUT_SECS=10 cargo run -p storage-testing --bin harness -- run`
Expected: completes with summary and artifacts; no container setup errors.

**Step 5: Commit debug fixes**
```bash
git add storage-testing/src/lab/image.rs storage-testing/src/harness/orchestrator.rs storage-testing/tests
git commit -m "test(harness): stabilize host-only harness execution"
```

### Task 7: Final verification and PR-ready checks

**Files:**
- Modify only if verification reveals regressions.

**Step 1: Run compile and formatting checks**
Run:
- `cargo check -p storage-testing`
- `cargo fmt --all --check`

Expected: PASS.

**Step 2: Re-run full host harness once more**
Run: `STORAGE_TESTING_TEST_TIMEOUT_SECS=10 cargo run -p storage-testing --bin harness -- run`
Expected: stable summary, no container-related failures.

**Step 3: Verify CI file and docs diff sanity**
Run: `git diff -- .github/workflows/ci.yml storage-testing/README.md justfile`
Expected: only intended host-only changes.

**Step 4: Final commit (if needed)**
```bash
git add -A
git commit -m "chore(harness): finalize host-only migration and validation"
```
