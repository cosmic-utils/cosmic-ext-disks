# Layered Crate Structure Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Normalize crate structures toward a consistent layer-separated architecture across the workspace while preserving the intentional paradigms in `storage-app` and `storage-service`.

**Architecture:** Apply a wave-based structural migration with strict dependency direction and minimal behavior changes. Use `storage-app` and `storage-service` as preserved reference styles, and align the remaining crates to clear layer boundaries and predictable module exports.

**Execution Constraint:** No temporary compatibility re-exports/shims and no intermediate implementation commits. Apply final target paths directly and keep all migration code changes for one final big-bang commit.

**Tech Stack:** Rust 2024 workspace, Cargo, `just`, clippy (`just check`), multi-crate module exports.

---

### Task 1: Capture baseline verification state

**Files:**
- Modify: `docs/plans/2026-02-26-layered-crate-structure-checklist.md`

**Step 1: Run baseline check**

Run: `just check`
Expected: Command completes and produces current baseline diagnostics.

**Step 2: Record baseline status in checklist doc**

Add a short note under Preparation with timestamp and outcome.

**Step 3: Checkpoint (no commit)**

Update checklist progress and proceed without committing.

### Task 2: Normalize `storage-types` module exports and structure

**Files:**
- Modify: `storage-types/src/lib.rs`
- Move: `storage-types/src/partition_types.rs` → `storage-types/src/partition_types/mod.rs`
- Create: `storage-types/src/partition_types/catalog.rs`
- Create: `storage-types/src/partition_types/query.rs`
- Move: `storage-types/src/rclone.rs` → `storage-types/src/rclone/mod.rs`
- Create: `storage-types/src/rclone/scope.rs`
- Create: `storage-types/src/rclone/remote.rs`
- Create: `storage-types/src/rclone/provider_catalog.rs`
- Create: `storage-types/src/rclone/mount.rs`
- Create: `storage-types/src/rclone/results.rs`
- Test: workspace compile via `just check`

**Step 1: Define target module ordering and names**

Keep public module names unchanged (`partition_types`, `rclone`) and enforce explicit module/export ordering for clarity.

**Step 2: Apply internal module decomposition (no public rename churn)**

Convert `partition_types` and `rclone` to folder modules with focused internals while preserving existing external module paths.

**Step 3: Fix compile/import paths in dependents**

Update dependents only where needed for removed wildcard exports; keep existing crate-root API symbols intact.

**Step 4: Validate wave**

Run: `just check`
Expected: Pass, or only pre-existing unrelated diagnostics.

**Step 5: Checkpoint (no commit)**

Update checklist progress and proceed without committing.

### Task 3: Normalize `storage-contracts` structure and exports

**Files:**
- Modify: `storage-contracts/src/lib.rs`
- Modify: `storage-contracts/src/protocol/mod.rs`
- Modify: `storage-contracts/src/traits/mod.rs`
- Move: `storage-contracts/src/protocol/errors.rs` → `storage-contracts/src/protocol/error.rs`
- Move: `storage-contracts/src/protocol/ids.rs` → `storage-contracts/src/protocol/id.rs`
- Move: `storage-contracts/src/protocol/operations.rs` → `storage-contracts/src/protocol/operation.rs`
- Modify: `storage-contracts/src/traits/discovery.rs`
- Test: workspace compile via `just check`

**Step 1: Reorganize protocol module layout**

Apply singular protocol module renames and update `protocol/mod.rs` declarations/re-exports to `error`, `id`, `operation`.

**Step 2: Reorganize traits module layout**

Rename discovery trait `FilesystemOps` to `FilesystemDiscovery` and update `traits/mod.rs` re-exports.

**Step 3: Preserve stable façade exports**

Replace wildcard root re-exports in `lib.rs` with explicit grouped protocol + trait exports.

**Step 4: Validate wave**

Run: `just check`
Expected: Pass.

**Step 5: Checkpoint (no commit)**

Update checklist progress and proceed without committing.

### Task 4: Refactor `storage-udisks` to clearer layer boundaries

**Files:**
- Modify: `storage-udisks/src/lib.rs`
- Create: `storage-udisks/src/infra/mod.rs`
- Move: `storage-udisks/src/options.rs` → `storage-udisks/src/infra/options.rs`
- Move: `storage-udisks/src/udisks_block_config.rs` → `storage-udisks/src/infra/udisks_block_config.rs`
- Move: `storage-udisks/src/usage.rs` → `storage-udisks/src/infra/usage.rs`
- Move: `storage-udisks/src/util/process.rs` → `storage-udisks/src/infra/process.rs`
- Delete: `storage-udisks/src/util/mod.rs`
- Modify: `storage-udisks/src/filesystem/config.rs`
- Modify: `storage-udisks/src/encryption/config.rs`
- Test: workspace compile via `just check`

**Step 1: Isolate infra-oriented helpers from domain operation code**

Move options parsing, block configuration proxy, usage probe, and process management into `src/infra/*` modules.

**Step 2: Update module wiring and imports**

In `lib.rs`, replace private module declarations (`options`, `udisks_block_config`, `usage`) with `mod infra;` and remove public `util` module exposure.
Rewire internal imports in `filesystem/config.rs` and `encryption/config.rs` to `crate::infra::options` and `crate::infra::udisks_block_config`.

**Step 3: Keep public façade stable/clean**

Re-export existing crate-root helpers from `infra` so external call sites remain unchanged (`join_options`, `stable_dedup`, `Usage`, `usage_for_mount_point`, `find_processes_using_mount`, `kill_processes`).

**Step 4: Validate wave**

Run: `just check`
Expected: Pass.

**Step 5: Checkpoint (no commit)**

Update checklist progress and proceed without committing.

### Task 5: Refactor `storage-sys` to clearer layer boundaries

**Files:**
- Modify: `storage-sys/src/lib.rs`
- Move: `storage-sys/src/rclone.rs` → `storage-sys/src/rclone/mod.rs`
- Create: `storage-sys/src/rclone/systemd.rs`
- Create: `storage-sys/src/rclone/unix_user.rs`
- Create: `storage-sys/src/rclone/mount_state.rs`
- Modify: `storage-sys/src/usage/mod.rs`
- Modify: `storage-sys/src/bin/scan-categories.rs` (if paths change)
- Modify: `storage-service/src/handlers/filesystems.rs` (usage façade imports)
- Test: workspace compile via `just check`

**Step 1: Split oversized `rclone` module only**

Move helper concerns in `rclone` into focused submodules (`systemd`, `unix_user`, `mount_state`) while preserving existing public APIs.

**Step 2: Normalize `usage` façade exports**

Export mount/progress helpers from `usage/mod.rs` and update callers to avoid deep `usage::mounts` / `usage::progress` imports.

**Step 3: Keep crate root API stable**

Retain `lib.rs` module shape (`error`, `image`, `rclone`, `usage`) and top-level re-exports to minimize downstream churn.

**Step 4: Validate wave**

Run: `just check`
Expected: Pass.

**Step 5: Checkpoint (no commit)**

Update checklist progress and proceed without committing.

### Task 6: Refactor `storage-btrfs` for model/ops clarity

**Files:**
- Modify: `storage-btrfs/src/lib.rs`
- Delete: `storage-btrfs/src/types.rs`
- Modify: `storage-btrfs/src/subvolume.rs`
- Modify: `storage-btrfs/src/usage.rs`
- Modify: `storage-btrfs/Cargo.toml`
- Modify: `storage-btrfs/src/bin/cli.rs` (if imports change)
- Test: workspace compile via `just check`

**Step 1: Remove dead duplicate model file**

Delete `src/types.rs` and rely on shared `storage_types::btrfs` models.

**Step 2: Tighten public API surface**

Remove `pub use btrfsutil;` and replace wildcard `storage_types::btrfs::*` re-export with explicit model exports actually required by consumers.

**Step 3: Keep CLI coupling isolated**

Retain optional CLI behavior without leaking CLI-only dependencies into default library path; trim stale core deps from `Cargo.toml` if no longer used.

**Step 4: Validate wave**

Run: `just check`
Expected: Pass.

**Step 5: Checkpoint (no commit)**

Update checklist progress and proceed without committing.

### Task 7: Refactor `storage-macros` internals

**Files:**
- Modify: `storage-macros/src/lib.rs`
- Create: `storage-macros/src/parse.rs`
- Create: `storage-macros/src/transform.rs`
- Create: `storage-macros/src/emit.rs`
- Test: workspace compile via `just check`

**Step 1: Extract parsing logic**

Move macro attribute-argument parsing into `parse.rs` and keep default action behavior unchanged.

**Step 2: Extract transform logic**

Move method transformation code into `transform.rs` while preserving:
- parameter detection behavior (`#[zbus(connection)]` / `#[zbus(header)]` and fallback names `connection|_connection`, `header|_header`),
- compile error strings for async-only and missing required parameters.

**Step 3: Extract emit/wiring logic**

Keep public proc-macro function in `lib.rs` delegating to internals.
Do not expose additional public symbols beyond `authorized_interface`.

**Step 4: Validate wave**

Run: `just check`
Expected: Pass.

**Step 5: Checkpoint (no commit)**

Update checklist progress and proceed without committing.

### Task 8: `storage-service` alignment pass (no paradigm change)

**Files:**
- Modify: `storage-service/src/main.rs`
- Modify: `storage-service/src/handlers/**`
- Modify: `storage-service/src/policies/**`
- Move/Delete/Modify: `storage-service/src/utilities/**`
- Test: workspace compile via `just check`

**Step 1: Align handler/policy naming pairs**

Normalize naming to improve discoverability.

**Step 2: Remove utility-layer inversion for hotplug**

Move hotplug monitoring and disk signal emission from `utilities/udisks.rs` into disk handler-owned module(s), then update `main.rs` call sites accordingly.

**Step 3: Decompose oversized handlers without changing interface contracts**

Split `handlers/filesystem.rs` and `handlers/rclone.rs` into folder modules by concern while keeping DBus interface names/paths and external behavior unchanged.

**Step 4: Collapse filesystems-specific helper drift**

Move filesystem-only utility helpers under filesystem handler support modules and remove now-empty/unused utility files.

**Step 5: Remove duplicated rclone caller/auth plumbing**

Delete local UID lookup helper and rely on macro-injected `caller.uid`; retain `check_authorization` only for true secondary authorization checks.

**Step 6: Validate wave**

Run: `just check`
Expected: Pass.

**Step 7: Checkpoint (no commit)**

Update checklist progress and proceed without committing.

### Task 9: `storage-app` unified cleanup pass (merged C8/C9, no paradigm change)

**Files:**
- Modify: `storage-app/src/main.rs` (if module wiring changes)
- Delete: `storage-app/src/volumes/mod.rs`
- Delete: `storage-app/src/network/mod.rs`
- Move: `storage-app/src/volumes/disk_header.rs` → `storage-app/src/views/disk.rs`
- Move: `storage-app/src/volumes/usage_pie.rs` → `storage-app/src/controls/usage_pie.rs`
- Move/Split: `storage-app/src/volumes/helpers.rs` helpers into `state/volumes.rs`, `state/btrfs.rs`, `update/volumes/helpers.rs`, and `utils/partition_types.rs`
- Move: `storage-app/src/network/icons.rs` → `storage-app/src/controls/icons.rs`
- Move: `storage-app/src/update/image.rs` → `storage-app/src/update/image/mod.rs`
- Modify: `storage-app/src/update/image/dialogs.rs`
- Modify: `storage-app/src/update/image/ops.rs`
- Modify: corresponding files in `storage-app/src/state/**`, `storage-app/src/update/**`, `storage-app/src/views/**`, `storage-app/src/controls/**`, `storage-app/src/utils/**`
- Test: workspace compile via `just check`

**Step 1: Move display modules into layered homes**

Move disk header view and usage pie control to `views` and `controls` respectively, then rewire imports.

**Step 2: Split helper ownership by layer**

Move volume lookup + segment lookup helpers into `state/volumes.rs`, BTRFS detection helpers into `state/btrfs.rs`, update-only helper into `update/volumes/helpers.rs`, and partition type mapping helpers into `utils/partition_types.rs`.
Delete duplicate local helper implementations in `update/mod.rs` once the shared state helper API is used.

**Step 3: Centralize icon mappings**

Move `network/icons` functionality into `controls/icons.rs`, including scope badge helpers currently local to `views/network.rs`, while keeping behavior unchanged.

**Step 4: Normalize update image module**

Convert to folder-module form (`update/image/mod.rs`), remove pass-through wrappers, and deduplicate repeated `ImageOperationDialog` constructors via one local helper.

**Step 5: Remove obsolete modules and fix imports**

Delete `src/volumes` and `src/network` module scaffolding, remove `mod volumes;` and `mod network;` from `main.rs`, and update all use paths.

**Step 6: Validate wave**

Run: `just check`
Expected: Pass.

**Step 7: Checkpoint (no commit)**

Update checklist progress and proceed without committing.
### Task 10: Final workspace verification and docs sync

**Files:**
- Modify: `docs/plans/2026-02-26-layered-crate-structure-checklist.md`
- Modify: `docs/plans/2026-02-26-layered-crate-structure-design.md` (status notes)

**Step 1: Run final verification**

Run: `just check`
Expected: Pass.

**Step 2: Mark checklist completion state**

Update completed items and unresolved follow-ups.

**Step 3: Single final commit**

Create one implementation commit containing all migration code + docs updates:

```bash
git add storage-types storage-contracts storage-udisks storage-sys storage-btrfs storage-macros storage-service storage-app docs/plans/2026-02-26-layered-crate-structure-checklist.md docs/plans/2026-02-26-layered-crate-structure-design.md docs/plans/2026-02-26-layered-crate-structure-implementation-plan.md
git commit -m "refactor: execute layered crate structure migration (single pass)"
```


