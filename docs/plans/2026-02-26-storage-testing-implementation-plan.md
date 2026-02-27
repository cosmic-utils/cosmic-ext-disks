# Storage Testing Harness + Lab Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add a new `storage-testing` crate with dual binaries (`harness`, `lab`) that share core logic for containerized integration execution and host-side image lab lifecycle workflows.

**Architecture:** Implement a single Rust crate (`storage-testing`) with a reusable library and two binaries. `harness` orchestrates privileged runtime-agnostic container test runs; `lab` orchestrates host image/loop/partition/mount lifecycle using spec names resolved from `resources/lab-specs`. Shared modules handle runtime detection, spec parsing, command planning, ledgered cleanup, and typed errors.

**Tech Stack:** Rust, Cargo workspace, `clap`, `serde`/`toml`, `anyhow`/`thiserror`, `tokio`, host Linux storage tools (`losetup`, `parted`, `mkfs.*`, `mount`, `umount`), container runtimes (`podman`/`docker`).

---

### Task 1: Workspace + crate scaffolding

**Files:**
- Modify: `Cargo.toml`
- Create: `storage-testing/Cargo.toml`
- Create: `storage-testing/src/lib.rs`
- Create: `storage-testing/src/bin/harness.rs`
- Create: `storage-testing/src/bin/lab.rs`

**Step 1: Write the failing test (workspace membership check)**

Create minimal compile check test shell in `storage-testing/src/lib.rs`:

```rust
#[cfg(test)]
mod tests {
    #[test]
    fn crate_compiles() {
        assert_eq!(2 + 2, 4);
    }
}
```

**Step 2: Run test to verify it fails (before wiring)**

Run: `cargo test -p storage-testing --no-run`
Expected: FAIL with "package ID specification `storage-testing` did not match any packages"

**Step 3: Write minimal implementation**

- Add `"storage-testing"` to workspace members.
- Add minimal `storage-testing` crate + two binaries printing placeholder help.

**Step 4: Run test to verify it passes**

Run: `cargo test -p storage-testing --no-run`
Expected: PASS (crate resolves and compiles)

**Step 5: Commit**

```bash
git add Cargo.toml storage-testing/Cargo.toml storage-testing/src/lib.rs storage-testing/src/bin/harness.rs storage-testing/src/bin/lab.rs
git commit -m "feat(testing): scaffold storage-testing crate with harness/lab binaries"
```

---

### Task 2: Shared error + command execution primitives

**Files:**
- Create: `storage-testing/src/errors.rs`
- Create: `storage-testing/src/cmd.rs`
- Modify: `storage-testing/src/lib.rs`
- Test: `storage-testing/src/cmd.rs` (unit tests module)

**Step 1: Write the failing test**

Add tests for command planner behavior and stderr context formatting.

```rust
#[test]
fn formats_command_context() {
    let rendered = crate::cmd::render("losetup", &["--find", "--show", "disk.img"]);
    assert!(rendered.contains("losetup --find --show disk.img"));
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p storage-testing cmd::tests::formats_command_context -v`
Expected: FAIL (module/function missing)

**Step 3: Write minimal implementation**

- Add typed errors (`RuntimeMissing`, `PrivilegeRequired`, `SpecInvalid`, `CommandFailed`, etc).
- Add command helper with dry-run support and shell-safe arg rendering.

**Step 4: Run test to verify it passes**

Run: `cargo test -p storage-testing cmd::tests::formats_command_context -v`
Expected: PASS

**Step 5: Commit**

```bash
git add storage-testing/src/errors.rs storage-testing/src/cmd.rs storage-testing/src/lib.rs
git commit -m "feat(testing): add shared error model and command helpers"
```

---

### Task 3: Runtime adapter (`auto|podman|docker`)

**Files:**
- Create: `storage-testing/src/runtime.rs`
- Modify: `storage-testing/src/lib.rs`
- Test: `storage-testing/src/runtime.rs` (unit tests module)

**Step 1: Write the failing test**

```rust
#[test]
fn explicit_runtime_selection_is_respected() {
    let rt = crate::runtime::resolve(Some("docker"), || true, || true).unwrap();
    assert_eq!(rt.name(), "docker");
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p storage-testing runtime::tests::explicit_runtime_selection_is_respected -v`
Expected: FAIL (runtime module missing)

**Step 3: Write minimal implementation**

- Add runtime enum and resolver:
  - explicit override if provided
  - auto-detect in order: podman then docker
- Add command builders for image build/run/exec/rm wrappers.

**Step 4: Run test to verify it passes**

Run: `cargo test -p storage-testing runtime::tests -v`
Expected: PASS

**Step 5: Commit**

```bash
git add storage-testing/src/runtime.rs storage-testing/src/lib.rs
git commit -m "feat(testing): add runtime-agnostic podman/docker adapter"
```

---

### Task 4: Lab spec model + resolver

**Files:**
- Create: `resources/lab-specs/2disk.toml`
- Create: `resources/lab-specs/3disk.toml`
- Create: `storage-testing/src/spec.rs`
- Modify: `storage-testing/src/lib.rs`
- Test: `storage-testing/src/spec.rs` (unit tests module)

**Step 1: Write the failing test**

```rust
#[test]
fn resolves_spec_name_without_extension() {
    let spec = crate::spec::load_by_name("2disk").unwrap();
    assert_eq!(spec.name, "2disk");
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p storage-testing spec::tests::resolves_spec_name_without_extension -v`
Expected: FAIL (resolver not implemented)

**Step 3: Write minimal implementation**

- Implement `load_by_name(name)` lookup rooted at `resources/lab-specs`.
- Support extension probing order (`.toml` first).
- Validate required fields (image count/sizes, partition plans).

**Step 4: Run test to verify it passes**

Run: `cargo test -p storage-testing spec::tests -v`
Expected: PASS

**Step 5: Commit**

```bash
git add resources/lab-specs storage-testing/src/spec.rs storage-testing/src/lib.rs
git commit -m "feat(testing): add lab spec parsing and name-based resolver"
```

---

### Task 5: Lab ledger + cleanup reconciliation

**Files:**
- Create: `storage-testing/src/ledger.rs`
- Modify: `storage-testing/src/lib.rs`
- Test: `storage-testing/src/ledger.rs` (unit tests module)

**Step 1: Write the failing test**

```rust
#[test]
fn persists_and_loads_spec_state() {
    let state = crate::ledger::SpecState::new("2disk");
    let path = crate::ledger::save(&state).unwrap();
    let loaded = crate::ledger::load("2disk").unwrap();
    assert_eq!(loaded.spec_name, "2disk");
    assert!(path.exists());
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p storage-testing ledger::tests::persists_and_loads_spec_state -v`
Expected: FAIL

**Step 3: Write minimal implementation**

- Ledger path: `target/storage-testing/lab-state/<spec-name>.json`.
- Store loop devices, image paths, mount points, mapper refs, timestamps.
- Add safe load/save/update/remove helpers.

**Step 4: Run test to verify it passes**

Run: `cargo test -p storage-testing ledger::tests -v`
Expected: PASS

**Step 5: Commit**

```bash
git add storage-testing/src/ledger.rs storage-testing/src/lib.rs
git commit -m "feat(testing): add lab state ledger and reconciliation primitives"
```

---

### Task 6: `lab` image lifecycle command planner (destructive default, optional dry-run)

**Files:**
- Create: `storage-testing/src/image_lab.rs`
- Modify: `storage-testing/src/bin/lab.rs`
- Modify: `storage-testing/src/lib.rs`
- Test: `storage-testing/src/image_lab.rs` and `storage-testing/src/bin/lab.rs`

**Step 1: Write the failing test**

```rust
#[test]
fn create_plan_is_destructive_by_default() {
    let plan = crate::image_lab::plan_create("2disk", false).unwrap();
    assert!(!plan.dry_run);
    assert!(!plan.steps.is_empty());
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p storage-testing image_lab::tests::create_plan_is_destructive_by_default -v`
Expected: FAIL

**Step 3: Write minimal implementation**

- Implement subcommands:
  - `lab image create|prepare|attach|mount|unmount|detach|destroy <spec-name> [--dry-run]`
  - `lab cleanup <spec-name|--all> [--dry-run]`
- Ensure default mode executes; `--dry-run` prints command plan only.
- Integrate ledger updates per successful phase.

**Step 4: Run test to verify it passes**

Run: `cargo test -p storage-testing lab -- -v`
Expected: PASS for new planner/parser tests

**Step 5: Commit**

```bash
git add storage-testing/src/image_lab.rs storage-testing/src/bin/lab.rs storage-testing/src/lib.rs
git commit -m "feat(testing): add lab lifecycle commands with optional dry-run"
```

---

### Task 7: `harness` container run/shell/cleanup orchestration

**Files:**
- Create: `storage-testing/src/harness.rs`
- Modify: `storage-testing/src/bin/harness.rs`
- Modify: `storage-testing/src/lib.rs`
- Create: `storage-testing/container/Containerfile`
- Test: `storage-testing/src/harness.rs` and `storage-testing/src/bin/harness.rs`

**Step 1: Write the failing test**

```rust
#[test]
fn harness_run_builds_expected_runtime_plan() {
    let plan = crate::harness::plan_run("logical", "auto", false, false).unwrap();
    assert!(plan.steps.iter().any(|s| s.contains("run --privileged")));
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p storage-testing harness::tests::harness_run_builds_expected_runtime_plan -v`
Expected: FAIL

**Step 3: Write minimal implementation**

- Implement commands:
  - `harness run [--suite <name>] [--runtime auto|podman|docker] [--keep] [--dry-run]`
  - `harness shell [--runtime ...]`
  - `harness cleanup [--runtime ...]`
- Build and run privileged test container.
- Start service in-container, execute suite command, export artifacts.

**Step 4: Run test to verify it passes**

Run: `cargo test -p storage-testing harness -- -v`
Expected: PASS for planner/parser tests

**Step 5: Commit**

```bash
git add storage-testing/src/harness.rs storage-testing/src/bin/harness.rs storage-testing/src/lib.rs storage-testing/container/Containerfile
git commit -m "feat(testing): add harness container orchestration commands"
```

---

### Task 8: Integration smoke test wiring + artifacts

**Files:**
- Create: `storage-testing/tests/harness_smoke.rs`
- Create: `storage-testing/tests/lab_smoke.rs`
- Modify: `storage-testing/src/artifacts.rs`
- Modify: `storage-testing/src/lib.rs`

**Step 1: Write the failing test**

```rust
#[test]
fn writes_artifact_index_on_run() {
    let dir = crate::artifacts::run_dir("smoke").unwrap();
    let idx = dir.join("index.json");
    assert!(idx.exists());
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p storage-testing writes_artifact_index_on_run -v`
Expected: FAIL

**Step 3: Write minimal implementation**

- Implement artifact directory creation and run metadata.
- Add smoke tests for parser/planner and non-destructive path checks.

**Step 4: Run test to verify it passes**

Run: `cargo test -p storage-testing -v`
Expected: PASS

**Step 5: Commit**

```bash
git add storage-testing/tests storage-testing/src/artifacts.rs storage-testing/src/lib.rs
git commit -m "test(testing): add smoke coverage and artifact persistence checks"
```

---

### Task 9: Workspace integration + developer entrypoints

**Files:**
- Modify: `justfile`
- Modify: `README.md`
- Create: `storage-testing/README.md`

**Step 1: Write the failing test/check**

Define command checks in docs and ensure command entrypoints compile.

**Step 2: Run check to verify missing entrypoints (before change)**

Run: `just --list | grep -E "harness|lab"`
Expected: no match

**Step 3: Write minimal implementation**

- Add just targets:
    - `harness`
    - `lab`
- Document usage in root and crate README.

**Step 4: Run check to verify it passes**

Run: `just --list | grep -E "^\s*harness|^\s*lab"`
Expected: shows both entries

**Step 5: Commit**

```bash
git add justfile README.md storage-testing/README.md
git commit -m "docs(testing): add harness/lab usage and just entrypoints"
```

---

### Task 10: Final verification

**Files:**
- Modify: `docs/plans/2026-02-26-storage-testing-implementation-plan.md` (verification notes section)

**Step 1: Run crate-focused checks**

Run: `cargo clippy -p storage-testing --all-targets`
Expected: PASS

Run: `cargo test -p storage-testing`
Expected: PASS

**Step 2: Run workspace checks**

Run: `just check`
Expected: PASS

**Step 3: Update verification notes**

Add timestamped summary of command outcomes.

**Step 4: Commit**

```bash
git add docs/plans/2026-02-26-storage-testing-implementation-plan.md
git commit -m "chore(testing): record verification results"
```
