# Storage Testing Harness + Lab Migration Checklist

**Date:** 2026-02-26  
**Scope:** Deterministic, action-level checklist for implementing `storage-testing` with dual binaries (`harness`, `lab`) and spec-driven host image workflows.

---

## A. Preparation

- [x] Confirm branch scope includes `storage-testing` crate + testing docs only (no unrelated feature work).
- [x] Confirm approved design exists: `docs/plans/2026-02-26-storage-testing-design.md`.
- [x] Confirm approved implementation plan exists: `docs/plans/2026-02-26-storage-testing-implementation-plan.md`.
- [x] Run baseline verification: `just check`.
- [x] Record baseline verification timestamp and summary in this checklist.

## B. Global Guardrails

- [x] Keep existing workspace crates behavior-compatible; do not change runtime logic in `storage-app`, `storage-service`, `storage-udisks`, `storage-sys` beyond test entrypoints/docs.
- [x] Keep runtime contract literal: `auto|podman|docker` only.
- [x] Keep binary names literal: `harness` and `lab`.
- [x] Keep lab-spec location literal: `resources/lab-specs`.
- [x] Keep lab command input literal: `<spec-name>` with no extension.
- [x] Keep mutating command policy literal: destructive by default; optional `--dry-run` preview mode.

## C. Workspace and Crate Scaffolding

## C1. Add Workspace Member

- [x] Update `Cargo.toml` `[workspace].members` to include `storage-testing`.
- [x] Do not remove or rename any existing member.

## C2. Create Crate Layout

- [x] Create `storage-testing/Cargo.toml` with package metadata and dependencies:
  - `clap`
  - `serde`
  - `toml`
  - `thiserror`
  - `anyhow`
  - `tokio`
- [x] Create `storage-testing/src/lib.rs` with module exports for:
  - `errors`
  - `cmd`
  - `runtime`
  - `spec`
  - `ledger`
  - `image_lab`
  - `harness`
  - `artifacts`
- [x] Create `storage-testing/src/bin/harness.rs` with clap root command and subcommands:
  - `run`
  - `shell`
  - `cleanup`
- [x] Create `storage-testing/src/bin/lab.rs` with clap root command and subcommands:
  - `image create`
  - `image prepare`
  - `image attach`
  - `image mount`
  - `image unmount`
  - `image detach`
  - `image destroy`
  - `cleanup`
- [x] Run compile check: `cargo test -p storage-testing --no-run`.

## D. Shared Error and Command Primitives

## D1. Error Model

- [x] Create `storage-testing/src/errors.rs`.
- [x] Add literal error variants:
  - `RuntimeMissing`
  - `PrivilegeRequired`
  - `SpecNotFound`
  - `SpecInvalid`
  - `CommandFailed`
  - `LedgerIo`
  - `ContainerRuntimeFailed`
  - `ServiceStartupFailed`
- [x] Implement `Display` messages with remediation text (tool missing, permission, cleanup hints).

## D2. Command Helper

- [x] Create `storage-testing/src/cmd.rs`.
- [x] Add `render(command, args)` helper for deterministic command text.
- [x] Add execution helper that supports `dry_run: bool`.
- [x] Ensure `dry_run=true` returns planned command without mutating system.
- [x] Add unit test in `storage-testing/src/cmd.rs` verifying `render("losetup", ["--find", "--show", "disk.img"])` output contains exact token order.
- [x] Run targeted test: `cargo test -p storage-testing cmd::tests::formats_command_context -v`.

## E. Runtime Adapter (`auto|podman|docker`)

- [x] Create `storage-testing/src/runtime.rs`.
- [x] Add enum with literal values:
  - `Auto`
  - `Podman`
  - `Docker`
- [x] Add resolver function:
  - explicit selection accepted for `podman` or `docker`
  - `auto` probes `podman` first, then `docker`
  - returns `RuntimeMissing` when neither exists
- [x] Add command builders for:
  - image build
  - container run (`--privileged` literal for harness run)
  - exec
  - rm
- [x] Add unit tests:
  - explicit runtime selection respected
  - auto selection podman priority
  - missing runtime failure path
- [x] Run targeted tests: `cargo test -p storage-testing runtime::tests -v`.

## F. Lab Spec Model and Resolver

## F1. Spec Fixtures

- [x] Create directory `resources/lab-specs`.
- [x] Create `resources/lab-specs/2disk.toml`.
- [x] Create `resources/lab-specs/3disk.toml`.
- [x] Include literal fields in each spec:
  - `name`
  - `artifacts_root`
  - `images[]` (`file_name`, `size_bytes`)
  - `partition_table` (`gpt` or `dos`)
  - `partitions[]` (`index`, `start`, `end`, `type`, optional `fs`)
  - `mounts[]` (`partition_ref`, `mount_point`)

## F2. Resolver Implementation

- [x] Create `storage-testing/src/spec.rs`.
- [x] Add `load_by_name(spec_name: &str)`.
- [x] Resolve from `resources/lab-specs/<spec-name>.toml` (no extension accepted from CLI input).
- [x] Validate required fields listed in F1.
- [x] Return `SpecNotFound` for missing file and `SpecInvalid` for invalid content.
- [x] Add unit test for `load_by_name("2disk")`.
- [x] Run targeted tests: `cargo test -p storage-testing spec::tests -v`.

## G. Lab Ledger and State Reconciliation

- [x] Create `storage-testing/src/ledger.rs`.
- [x] Define persisted state file path literal: `target/storage-testing/lab-state/<spec-name>.json`.
- [x] Persist literal fields:
  - `spec_name`
  - `image_paths[]`
  - `loop_devices[]`
  - `mapped_partitions[]`
  - `mount_points[]`
  - `updated_at`
- [x] Implement literal methods:
  - `load(spec_name)`
  - `save(state)`
  - `remove(spec_name)`
  - `exists(spec_name)`
- [x] Add unit test for save/load roundtrip.
- [x] Run targeted tests: `cargo test -p storage-testing ledger::tests -v`.

## H. `lab` Command Surface and Execution

## H1. Subcommand Contract

- [x] Implement clap parsing in `storage-testing/src/bin/lab.rs` for exact commands:
  - `lab image create <spec-name> [--dry-run]`
  - `lab image prepare <spec-name> [--dry-run]`
  - `lab image attach <spec-name> [--dry-run]`
  - `lab image mount <spec-name> [--dry-run]`
  - `lab image unmount <spec-name> [--dry-run]`
  - `lab image detach <spec-name> [--dry-run]`
  - `lab image destroy <spec-name> [--dry-run]`
  - `lab cleanup <spec-name|--all> [--dry-run]`
- [x] Ensure command defaults to destructive execution when `--dry-run` is absent.

## H2. Image Lifecycle Engine

- [x] Create `storage-testing/src/image_lab.rs`.
- [x] Implement literal planner/executor methods:
  - `plan_create(spec_name, dry_run)`
  - `plan_prepare(spec_name, dry_run)`
  - `plan_attach(spec_name, dry_run)`
  - `plan_mount(spec_name, dry_run)`
  - `plan_unmount(spec_name, dry_run)`
  - `plan_detach(spec_name, dry_run)`
  - `plan_destroy(spec_name, dry_run)`
  - `plan_cleanup(target, dry_run)`
- [x] Implement concrete host tool invocations:
  - image creation (`truncate` or `dd`)
  - partitioning (`parted`)
  - loop attach/detach (`losetup`)
  - filesystem format if configured (`mkfs.*`)
  - mount/unmount (`mount`, `umount`)
- [x] Update ledger after each successful mutating step.
- [x] Implement idempotent reconciliation for repeated `attach`, `mount`, `unmount`, `detach`.
- [x] Add planner/unit tests for destructive-default and dry-run output.
- [x] Run targeted tests: `cargo test -p storage-testing lab -- -v`.

## I. `harness` Container Integration Execution

## I1. Runtime + CLI

- [x] Implement clap parsing in `storage-testing/src/bin/harness.rs` for exact commands:
  - `harness run [--suite <name>] [--runtime auto|podman|docker] [--keep] [--dry-run]`
  - `harness shell [--runtime auto|podman|docker]`
  - `harness cleanup [--runtime auto|podman|docker]`
- [x] Default `--runtime` to `auto`.

## I2. Harness Orchestration

- [x] Create `storage-testing/src/harness.rs`.
- [x] Implement literal methods:
  - `plan_run(suite, runtime, keep, dry_run)`
  - `plan_shell(runtime)`
  - `plan_cleanup(runtime)`
- [x] Ensure run plan includes literal privileged container run (`--privileged`).
- [x] Start `cosmic-ext-storage-service` inside container during `run` flow.
- [x] Execute suite command and capture exit code.
- [x] Honor `--keep` by skipping final container teardown.
- [x] Add planner/unit test asserting `run --privileged` appears in generated steps.
- [x] Run targeted tests: `cargo test -p storage-testing harness -- -v`.

## I3. Container Build Inputs

- [x] Create `storage-testing/container/Containerfile`.
- [x] Install required tools in image:
  - `lvm2`
  - `mdadm`
  - `btrfs-progs`
  - `util-linux` (for loop/mount helpers)
- [x] Include test runner entrypoint used by `harness run`.

## J. Artifacts, Output, and Smoke Coverage

- [x] Create `storage-testing/src/artifacts.rs`.
- [x] Implement deterministic output roots:
  - `target/storage-testing/artifacts/<run-id>/...`
- [x] Persist literal artifact files:
  - `index.json`
  - `commands.log`
  - `service.log`
  - `test.log`
- [x] Create integration smoke tests:
  - `storage-testing/tests/harness_smoke.rs`
  - `storage-testing/tests/lab_smoke.rs`
- [x] Add smoke assertions for artifact index creation and planner execution flow.
- [x] Run crate tests: `cargo test -p storage-testing -v`.

## K. Developer Entry Points and Docs

- [x] Update `justfile` with literal recipes:
  - `harness` → `cargo run -p storage-testing --bin harness -- run`
  - `lab` → `cargo run -p storage-testing --bin lab -- image create <spec>`
- [x] Update root `README.md` with section describing:
  - purpose of `harness`
  - purpose of `lab`
  - spec-name behavior from `resources/lab-specs`
- [x] Create `storage-testing/README.md` with literal usage examples for each subcommand.
- [x] Ensure docs state destructive-default behavior and optional `--dry-run`.

## L. Verification Tasks (Exact Commands)

- [x] Run `cargo fmt --all -- --check`.
- [x] Run `cargo clippy -p storage-testing --all-targets`.
- [x] Run `cargo test -p storage-testing`.
- [x] Run `just check`.
- [x] Record timestamped verification evidence in this checklist with command summary.

## M. Completion Gate (Strict)

- [x] Mark sections A-L complete before marking this section.
- [x] Confirm both binaries build and show help:
  - `cargo run -p storage-testing --bin harness -- --help`
  - `cargo run -p storage-testing --bin lab -- --help`
- [x] Confirm spec-name resolution works with no extension:
  - `cargo run -p storage-testing --bin lab -- image create 2disk --dry-run`
- [x] Confirm final workspace verification passes with no new warnings/errors.
- [x] Record final timestamp and completion note.

## N. Verification Evidence

- [x] Baseline (`just check`): passed (2026-02-26, pre-implementation baseline run in session context).
- [x] Final `cargo fmt --all -- --check`: passed (2026-02-26).
- [x] Final `cargo clippy -p storage-testing --all-targets`: passed (2026-02-26).
- [x] Final `cargo test -p storage-testing`: passed (11 tests total: 9 unit + 2 integration smoke).
- [x] Final `just check`: passed (workspace clippy/fmt/test complete, 2026-02-26).

**Completion Note (2026-02-26):** Sections A-M are complete. `storage-testing` now exists as a workspace crate with dual binaries (`harness`, `lab`), runtime-agnostic container planning, spec-name lab workflows from `resources/lab-specs`, and verified workspace compatibility.
