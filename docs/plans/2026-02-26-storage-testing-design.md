# Storage Testing Harness + Lab Design

**Date:** 2026-02-26  
**Scope:** Define a new `storage-testing` crate that provides containerized integration execution and host-side image lab workflows.

---

## 1) Objectives

- Add a new workspace crate: `storage-testing`.
- Provide dual binaries with shared logic:
  - `harness`: run real integration tests in a privileged container.
  - `lab`: create/prepare/attach/mount image-backed disks on host for manual UI CRUD testing.
- Keep runtime support runtime-agnostic (`podman`/`docker`) via a single adapter layer.
- Use lab specs resolved by name from `resources/lab-specs`.

---

## 2) Final Decisions

- **Execution mode:** container mode for actual integration test execution.
- **Container runtime contract:** runtime-agnostic wrapper (auto-detect `podman` or `docker`, with explicit override).
- **Binary naming:** use `harness` and `lab` (no extra branding prefix).
- **Lab spec location:** `resources/lab-specs`.
- **Lab command input:** pass only spec name (no extension); CLI resolves file.
- **Safety policy:** mutating host operations are destructive by default; `--dry-run` is optional preview mode.

---

## 3) Architecture

`storage-testing` will be a library crate with two binaries.

### 3.1 Shared Library Modules

- `runtime`
  - Detect and normalize container runtime commands.
  - Produce a small trait-backed adapter for command construction/execution.
- `spec`
  - Parse and validate lab spec files from `resources/lab-specs/<name>.(toml|yaml)`.
  - Provide typed models for images, partitions, and preparation/mount intent.
- `image_lab`
  - Image creation, partitioning, loop attach/detach, mount/unmount, and destroy workflows.
  - Maintain operation ledger for reliable cleanup and reconciliation.
- `service_client`
  - Helpers to call storage service APIs for integration orchestration/assertions.
- `artifacts`
  - Store logs, junit-like results, and command traces under deterministic output roots.
- `errors`
  - Typed error hierarchy with actionable remediations.

### 3.2 Binaries

- `harness`
  - Focused on container lifecycle + integration test execution.
- `lab`
  - Focused on host-side disk-image lifecycle for manual testing.

---

## 4) CLI Surface

### 4.1 `harness`

- `harness run [--suite <name>] [--runtime auto|podman|docker] [--keep] [--dry-run]`
  - Detect runtime.
  - Build/reuse container image.
  - Start privileged container.
  - Start `cosmic-ext-storage-service` in-container.
  - Execute integration suites.
  - Export artifacts.
  - Teardown unless `--keep`.
- `harness shell [--runtime ...]`
  - Open shell in prepared test container for debugging.
- `harness cleanup [--runtime ...]`
  - Remove stale containers/volumes/artifacts.

### 4.2 `lab`

All commands resolve specs from `resources/lab-specs` by spec name.

- `lab image create <spec-name> [--dry-run]`
- `lab image prepare <spec-name> [--dry-run]`
- `lab image attach <spec-name> [--dry-run]`
- `lab image mount <spec-name> [--dry-run]`
- `lab image unmount <spec-name> [--dry-run]`
- `lab image detach <spec-name> [--dry-run]`
- `lab image destroy <spec-name> [--dry-run]`
- `lab cleanup <spec-name|--all> [--dry-run]`

Default behavior performs real actions; `--dry-run` prints planned operations.

---

## 5) Data Flow

### 5.1 Harness Flow

1. Resolve runtime (`auto` default).
2. Resolve suite + image configuration.
3. Start privileged container with required tools (`lvm2`, `mdadm`, `btrfs-progs`, loop tooling).
4. Start service process inside container.
5. Run integration tests that call real service endpoints.
6. Persist logs/results under `target/storage-testing/artifacts/...`.
7. Teardown resources (unless `--keep`).

### 5.2 Lab Flow

1. Resolve spec by name from `resources/lab-specs`.
2. Validate image/partition schema.
3. Execute subcommand lifecycle action.
4. Persist/reconcile ledger under `target/storage-testing/lab-state/<spec-name>.json`.
5. Emit exact operator-facing state summary (loop ids, devices, mounts, artifact paths).

---

## 6) Error Handling and Operability

- Use typed errors with clear category and remediation:
  - runtime/tool missing
  - permission/privilege failure
  - spec parse/validation failure
  - loop/partition/mount orchestration failure
  - container lifecycle failure
  - service startup/test execution failure
- Include command context in failures (tool, args, step).
- On partial failures, perform best-effort rollback and emit exact manual recovery commands.
- Keep actions idempotent where possible (`attach`, `mount`, `cleanup` should converge to stable state).

---

## 7) Test Strategy

### 7.1 Unit Tests (`storage-testing`)

- Spec parsing/validation.
- Runtime auto-detection and command generation.
- Ledger state transitions and cleanup reconciliation.
- Command plan generation for `--dry-run`.

### 7.2 Integration Tests (`storage-testing`)

- `harness run` smoke path in a privileged container.
- Service-up checks and basic API call assertions.
- Artifact generation assertions.

### 7.3 Manual UI Workflows (`lab`)

- Specs such as:
  - `2disk`
  - `3disk`
- Use `lab` lifecycle commands to prep host loop-backed devices and run CRUD through app UI.

---

## 8) Non-Goals (v1)

- No non-container integration execution mode.
- No GUI for harness/lab orchestration.
- No generic arbitrary host-path destructive tooling outside repo-managed artifact roots.

---

## 9) Success Criteria

- `storage-testing` crate exists and is a workspace member.
- `harness` can run containerized integration suites end-to-end with artifacts.
- `lab` can resolve spec-by-name from `resources/lab-specs` and execute image lifecycle commands on host.
- Runtime adapter supports auto-detect + explicit runtime selection.
- Operator can repeatedly prepare/tear down test disks for manual UI CRUD verification.
