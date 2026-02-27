# Storage-Testing Integration Suite Implementation Plan (Spec-Only)

**Date:** 2026-02-27  
**Input:** `2026-02-27-storage-testing-integration-suite-design.md`  
**Mode:** Plan only (do not implement from this file directly).

---

## Phase 0 — Preflight and Scope Lock

### Tasks
1. Confirm baseline passes: `just check`.
2. Confirm current smoke tests still exist and identify replacement targets in `storage-testing/tests`.
3. Confirm no remaining references to `storage-app::client` (must all be `storage-contracts::client`).
4. Freeze scope to integration framework and tests only.

### Exit Criteria
- Baseline green and file targets recorded.

---

## Phase 1 — Shared Orchestrator Refactor (No cross-binary subprocess)

### Required outcome
`harness` and `lab` call shared library methods directly; no binary-to-binary command execution.

### Tasks
1. Add `storage-testing/src/lab_orchestrator.rs` with methods:
  - `create(spec, opts)`
  - `prepare(spec, opts)`
  - `attach(spec, opts)`
  - `mount(spec, opts)`
  - `unmount(spec, opts)`
  - `detach(spec, opts)`
  - `destroy(spec, opts)`
  - `cleanup(target, opts)`
2. Move current execution logic from CLI handlers into orchestrator methods.
3. Keep `src/bin/lab.rs` as clap adapter only.
4. Update `src/bin/harness.rs` and `harness` module to call `lab_orchestrator` directly.
5. Add unit tests for orchestrator method invocation contracts using dry-run planning.

### Exit Criteria
- No code path in `harness` executes `lab` via shell/cargo command.

---

## Phase 2 — Test Contract and Registry

### Required outcome
Deterministic, metadata-rich test registration for grouping and scheduling.

### Tasks
1. Add `storage-testing/src/test_registry.rs`.
2. Define `HarnessTest` trait:
  - `id()`
  - `suite()`
  - `required_spec()`
  - `exclusive()`
  - `run(ctx)`
3. Add typed registry and helper APIs:
  - `all_tests()`
  - `by_suite()`
  - `group_by_spec()`
  - `filter(selection)`
4. Add unit tests for deterministic grouping order and selection behavior.

### Exit Criteria
- Every integration test is representable in registry with required spec.

---

## Phase 3 — Harness Scheduler and Result Model

### Required outcome
Parallel-by-spec execution with stable reporting.

### Tasks
1. Add `storage-testing/src/harness_orchestrator.rs`.
2. Implement run flow:
  - discover tests
  - filter tests
  - group by spec
  - execute groups with bounded parallelism
  - run tests sequentially inside each group (unless explicitly safe)
3. Add exclusive lane handling.
4. Add result structures:
  - `TestResultRecord`
  - `GroupResultRecord`
  - `RunSummary`
5. Ensure teardown failures are captured without hiding primary test failures.

### Exit Criteria
- `harness run` default selects and schedules all tests.

---

## Phase 4 — Replace Smoke Tests with Full Integration Catalog

### Required outcome
`storage-testing/tests` covers all service client functionality categories.

### Test folders/files (minimum)
1. `storage-testing/tests/common/mod.rs`
2. `storage-testing/tests/common/assertions.rs`
3. `storage-testing/tests/common/fixtures.rs`
4. `storage-testing/tests/common/registration.rs`
5. `storage-testing/tests/disk/*.rs`
6. `storage-testing/tests/filesystem/*.rs`
7. `storage-testing/tests/partition/*.rs`
8. `storage-testing/tests/luks/*.rs`
9. `storage-testing/tests/btrfs/*.rs`
10. `storage-testing/tests/logical/*.rs`
11. `storage-testing/tests/image/*.rs`
12. `storage-testing/tests/rclone/*.rs` (conditional if prerequisites unavailable)

### Tasks per suite file
1. Register tests with unique IDs from design catalog.
2. Assign required spec (`2disk` or `3disk`).
3. Mark destructive tests `exclusive=true`.
4. Implement assertions using `storage-contracts::client` APIs only.
5. Remove or retire `harness_smoke.rs` and `lab_smoke.rs` from default run path.

### Exit Criteria
- Smoke-only coverage is replaced by real scenario coverage.

---

## Phase 5 — Group Setup/Teardown Integration

### Required outcome
Every spec-group run is wrapped by in-process lab lifecycle calls.

### Tasks
1. In group setup call:
  - `create` → `prepare` → `attach` → `mount`
2. In group teardown call:
  - `unmount` → `detach` → `destroy`
3. Always run teardown in `finally` path.
4. Persist setup/teardown logs and statuses in artifacts.

### Exit Criteria
- Group orchestration is deterministic and independently recoverable.

---

## Phase 6 — CI Hardening

### Required outcome
CI executes run-all and uploads useful failure artifacts.

### Tasks
1. Keep harness CI invocation on default run-all path.
2. Add artifact upload step for:
  - `target/storage-testing/artifacts/**/index.json`
  - `group-*.log`
  - `test-*.log`
  - `service.log`
3. Print compact pass/fail/skip summary in job output.

### Exit Criteria
- Failed CI runs provide enough data for root-cause triage.

---

## Verification Plan

1. `cargo test -p storage-testing -- --nocapture`
2. `cargo clippy -p storage-testing --all-targets`
3. `cargo run -p storage-testing --bin harness -- run --runtime <runtime>`
4. CI PR run with artifact inspection on forced failure case

---

## Risks and Mitigations

- **Risk:** group parallelism causes shared-state collisions.  
  **Mitigation:** strict spec-group resource boundaries + sequential in-group execution.

- **Risk:** test duration grows too large.  
  **Mitigation:** keep default suite real but minimal per function; move long-soak into extended profile.

- **Risk:** rclone environment not consistently available in CI.  
  **Mitigation:** explicit conditional skip with reason and separate optional job when credentials/config are present.

---

## Definition of Done

1. No cross-binary `harness -> lab` subprocess calls remain.
2. All integration tests are trait-registered with required spec metadata.
3. Full functional suites exist for disk/filesystem/partition/luks/btrfs/logical/image (+conditional rclone).
4. Default harness run executes all tests and reports deterministic summary.
5. CI publishes artifacts on failure.
